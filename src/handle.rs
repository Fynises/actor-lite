use tokio::{sync::mpsc, task::JoinHandle};

use crate::{handler::Handler, actor::Actor, error::ActorHandleError};

pub struct ActorHandle<T: Send> {
    sender: mpsc::UnboundedSender<T>,
    join_handle: JoinHandle<()>,
}

impl <T: Send> ActorHandle<T> {
    pub fn new<U: Handler<T>>(message_handler: U) -> Self
        where
        T: Send + 'static,
        U: Handler<T> + Send + 'static {
        let (tx, rx) = mpsc::unbounded_channel::<T>();
        let mut actor = Actor::new(rx, message_handler);
        let join_handle = tokio::spawn(async move { actor.run().await; });
        Self {
            sender: tx,
            join_handle,
        }
    }

    pub fn send(&self, message: T) -> Result<(), ActorHandleError<T>> {
        self.sender.send(message)?;
        Ok(())
    }

    pub fn abort(&mut self) {
        self.join_handle.abort();
    }
}
