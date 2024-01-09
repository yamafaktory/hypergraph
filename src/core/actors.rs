use std::{future::Future, pin::Pin};

use tokio::sync::{mpsc, oneshot};
use tracing::{debug, info, instrument};

use crate::errors::HypergraphError;

type Handler<A, R> =
    dyn Fn(A) -> Pin<Box<dyn Future<Output = Result<R, HypergraphError>> + Send + 'static>> + Sync;

struct Actor<A, R>
where
    A: Send + Sync + 'static,
    R: Send + Sync + 'static,
{
    handler: &'static Handler<A, R>,
    receiver: mpsc::Receiver<ActorMessage<A, R>>,
}

struct ActorMessage<A, R>(A, oneshot::Sender<R>);

impl<A, R> Actor<A, R>
where
    A: Send + Sync + 'static,
    R: Send + Sync + 'static,
{
    fn new(handler: &'static Handler<A, R>, receiver: mpsc::Receiver<ActorMessage<A, R>>) -> Self {
        Actor { handler, receiver }
    }

    async fn handle_message(&mut self, ActorMessage(argument, sender): ActorMessage<A, R>) {
        let response = (self.handler)(argument).await.unwrap();

        // Ignore send errors.
        let _ = sender.send(response);
    }
}

async fn runner<A, R>(mut actor: Actor<A, R>)
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
