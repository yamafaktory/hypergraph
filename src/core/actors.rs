use std::marker::PhantomData;

use futures::future::BoxFuture;
use tokio::sync::{mpsc, oneshot};

use crate::errors::HypergraphError;

/// S -> State
/// P -> Payload
/// R -> Response
type Handler<'a, S, P, R> = dyn Fn(S, P) -> BoxFuture<'a, Result<R, HypergraphError>> + Send + Sync;

struct Actor<'a, S, P, R>
where
    S: Send + Sync,
    P: Send + Sync,
    R: Send + Sync,
{
    handler: &'a Handler<'a, S, P, R>,
    receiver: mpsc::Receiver<ActorMessage<P, R>>,
    state: S,
}

struct ActorMessage<P, R>(P, oneshot::Sender<R>);

impl<'a, S, P, R> Actor<'a, S, P, R>
where
    S: Clone + Send + Sync,
    P: Send + Sync,
    R: Send + Sync,
{
    fn new(
        state: S,
        handler: &'a Handler<'a, S, P, R>,
        receiver: mpsc::Receiver<ActorMessage<P, R>>,
    ) -> Self {
        Actor {
            state,
            handler,
            receiver,
        }
    }

    async fn handle_message(&self, ActorMessage(payload, sender): ActorMessage<P, R>) {
        let response = (self.handler)(self.state.clone(), payload).await.unwrap();

        // Ignore send errors.
        let _ = sender.send(response);
    }
}

async fn runner<S, P, R>(mut actor: Actor<'_, S, P, R>)
where
    S: Clone + Send + Sync,
    P: Send + Sync,
    R: Send + Sync,
{
    while let Some(msg) = actor.receiver.recv().await {
        actor.handle_message(msg).await;
    }
}

/// S -> State
/// P -> Payload
/// R -> Response
#[derive(Clone, Debug)]
pub(crate) struct ActorHandle<S, P, R> {
    sender: mpsc::Sender<ActorMessage<P, R>>,
    state: PhantomData<S>,
}

impl<S, P, R> ActorHandle<S, P, R>
where
    S: Clone + Send + Sync,
    P: Send + Sync,
    R: Send + Sync,
{
    pub(crate) fn new(state: S, handler: &'static Handler<S, P, R>) -> Self {
        let (sender, receiver) = mpsc::channel(8);
        let actor = Actor::new(state.clone(), handler, receiver);

        tokio::spawn(runner(actor));

        Self {
            sender,
            state: PhantomData,
        }
    }

    pub(crate) async fn process(&self, payload: P) -> Result<R, HypergraphError> {
        let (send, recv) = oneshot::channel();
        let message = ActorMessage(payload, send);

        // Ignore send errors. If this send fails, so does the recv.await below.
        let _ = self.sender.send(message).await;
        recv.await.map_err(|_| HypergraphError::PathCreation)
    }
}
