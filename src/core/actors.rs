use futures::{future::BoxFuture, FutureExt};
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, info, instrument};

use crate::errors::HypergraphError;

type Handler<'a, A, R> = dyn Fn(A) -> BoxFuture<'a, Result<R, HypergraphError>> + Send + Sync;

struct Actor<'a, A, R>
where
    A: Send + Sync + 'static,
    R: Send + Sync + 'static,
{
    handler: &'a Handler<'a, A, R>,
    receiver: mpsc::Receiver<ActorMessage<A, R>>,
}

struct ActorMessage<A, R>(A, oneshot::Sender<R>);

impl<'a, A, R> Actor<'a, A, R>
where
    A: Send + Sync,
    R: Send + Sync,
{
    fn new(handler: &'a Handler<'a, A, R>, receiver: mpsc::Receiver<ActorMessage<A, R>>) -> Self {
        Actor { handler, receiver }
    }

    async fn handle_message(&self, ActorMessage(argument, sender): ActorMessage<A, R>) {
        let response = (self.handler)(argument).await.unwrap();

        // Ignore send errors.
        let _ = sender.send(response);
    }
}

async fn runner<A, R>(mut actor: Actor<'_, A, R>)
where
    A: Send + Sync,
    R: Send + Sync,
{
    while let Some(msg) = actor.receiver.recv().await {
        actor.handle_message(msg).await;
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ActorHandle<A, R> {
    sender: mpsc::Sender<ActorMessage<A, R>>,
}

impl<A, R> ActorHandle<A, R>
where
    A: Send + Sync + 'static,
    R: Send + Sync + 'static,
{
    pub(crate) fn new(handler: &'static Handler<A, R>) -> Self {
        let (sender, receiver) = mpsc::channel(8);
        let actor = Actor::new(handler, receiver);

        tokio::spawn(runner(actor));

        Self { sender }
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
