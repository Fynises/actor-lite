use tokio::{sync::mpsc, task::{JoinHandle, JoinError}};
use crate::sync::{actor::Actor as SyncActor, handler::Handler as SyncHandler};
use crate::r#async::{actor::Actor as AsyncActor, handler::Handler as AsyncHandler};
use crate::error::{ActorHandleSendError, ShutdownError};


pub struct ActorHandle<T: Send> {
    sender: Option<mpsc::UnboundedSender<T>>,
    join_handle: JoinHandle<()>,
}

impl <T: Send> ActorHandle<T> {
    /// constructs a new actor handle using a synchronous handler
    pub fn new<U: SyncHandler<T>>(message_handler: U) -> Self
        where
        T: Send + 'static,
        U: SyncHandler<T> + Send + 'static {
        let (tx, rx) = mpsc::unbounded_channel::<T>();
        let mut actor = SyncActor::new(rx, message_handler);
        let join_handle = tokio::spawn(async move { actor.run().await; });
        Self {
            sender: Some(tx),
            join_handle,
        }
    }

    /// constructs a new actor handle using an asynchronous handler
    pub fn new_async<U: AsyncHandler<T>>(message_handler: U) -> Self
        where
        T: Send + 'static,
        U: AsyncHandler<T> + Send + 'static {
        let (tx, rx) = mpsc::unbounded_channel::<T>();
        let mut actor = AsyncActor::new(rx, message_handler);
        let join_handle = tokio::spawn(async move { actor.run().await; });
        Self {
            sender: Some(tx),
            join_handle,
        }
    }

    /// sends a message to this actor
    pub fn send(&self, message: T) -> Result<(), ActorHandleSendError<T>> {
        if let Some(tx) = &self.sender {
            tx.send(message)?;
        } else {
            return Err(ActorHandleSendError::NoneSender);
        }
        Ok(())
    }

    /// orders this join_handle to await\
    /// warning: calling this can cause a deadlock\
    /// if actor hasn't already stopped
    pub async fn abort(self) -> Result<(), JoinError> {
        self.join_handle.await?;
        Ok(())
    }

    /// orders a graceful shutdown of this actor\
    /// allowing all uncompleted messages to complete before awaiting the join_handle
    pub async fn shutdown(mut self) -> Result<(), ShutdownError> {
        let send = match self.sender.take() {
            Some(tx) => tx,
            None => return Err(ShutdownError::NoneSender),
        };
        drop(send);
        self.join_handle.await?;
        Ok(())
    }
}
