use std::{marker::PhantomData, sync::Arc};

use futures::{future::BoxFuture, FutureExt};
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, info, instrument};

use crate::errors::HypergraphError;

type Handler<'a, A, R> = dyn Fn(A) -> BoxFuture<'a, Result<R, HypergraphError>> + Send + Sync;

struct Actor<'a, S, A, R>
where
    A: Send + Sync + 'static,
    R: Send + Sync + 'static,
{
    handler: &'a Handler<'a, A, R>,
    receiver: mpsc::Receiver<ActorMessage<A, R>>,
    state: S,
}

struct ActorMessage<A, R>(A, oneshot::Sender<R>);

impl<'a, A, R, S> Actor<'a, S, A, R>
where
    A: Send + Sync,
    R: Send + Sync,
{
    fn new(
        state: S,
        handler: &'a Handler<'a, A, R>,
        receiver: mpsc::Receiver<ActorMessage<A, R>>,
    ) -> Self {
        Actor {
            state,
            handler,
            receiver,
        }
    }

    async fn handle_message(&self, ActorMessage(argument, sender): ActorMessage<A, R>) {
        let response = (self.handler)(argument).await.unwrap();

        // Ignore send errors.
        let _ = sender.send(response);
    }
}

async fn runner<S, A, R>(mut actor: Actor<'_, S, A, R>)
where
    A: Send + Sync,
    R: Send + Sync,
{
    while let Some(msg) = actor.receiver.recv().await {
        actor.handle_message(msg).await;
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ActorHandle<S, A, R> {
    sender: mpsc::Sender<ActorMessage<A, R>>,
    state: PhantomData<S>,
}

impl<S, A, R> ActorHandle<S, A, R>
where
    S: Send + Sync + 'static,
    A: Send + Sync + 'static,
    R: Send + Sync + 'static,
{
    pub(crate) fn new(state: S, handler: &'static Handler<A, R>) -> Self {
        let (sender, receiver) = mpsc::channel(8);
        let actor = Actor::new(state, handler, receiver);

        tokio::spawn(runner(actor));

        Self {
            sender,
            state: PhantomData,
        }
    }

    pub(crate) async fn process(&self, argument: A) -> Result<R, HypergraphError> {
        let (send, recv) = oneshot::channel();
        let message = ActorMessage(argument, send);

        // Ignore send errors. If this send fails, so does the recv.await below.
        let _ = self.sender.send(message).await;
        let response = recv.await.map_err(|_| HypergraphError::PathCreation)?;

        Ok(response)
    }
}
