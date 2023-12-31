use tokio::{sync::mpsc::{self, UnboundedSender}, task::{JoinHandle, JoinError}};
use crate::sync::{actor::Actor as SyncActor, handler::Handler as SyncHandler};
use crate::r#async::{actor::Actor as AsyncActor, handler::Handler as AsyncHandler};
use crate::error::{ActorHandleSendError, ShutdownError};


pub struct ActorHandle<T: Send + Sync> {
    sender: Option<mpsc::UnboundedSender<T>>,
    join_handle: JoinHandle<()>,
}

impl <T: Send + Sync> ActorHandle<T> {
    /// constructs a new actor handle using a synchronous handler
    pub fn new<U: SyncHandler<T>>(
        func: impl FnOnce(&UnboundedSender<T>) -> U,
    ) -> Self
        where
        T: Send + Sync + 'static,
        U: SyncHandler<T> + Send + Sync + 'static
    {
        let (tx, rx) = mpsc::unbounded_channel::<T>();
        let mut actor = SyncActor::new(rx, func(&tx));
        let join_handle = tokio::spawn(async move { actor.run().await; });
        Self {
            sender: Some(tx),
            join_handle,
        }
    }

    /// constructs a new actor handle using an asynchronous handler
    pub fn new_async<U: AsyncHandler<T>>(
        func: impl FnOnce(&UnboundedSender<T>) -> U,
    ) -> Self
        where
        T: Send + Sync + 'static,
        U: AsyncHandler<T> + Send + Sync + 'static
    {
        let (tx, rx) = mpsc::unbounded_channel::<T>();
        let mut actor = AsyncActor::new(rx, func(&tx));
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
