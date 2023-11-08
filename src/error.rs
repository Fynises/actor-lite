use tokio::{sync::mpsc::error::SendError, task::JoinError};

#[derive(Debug, thiserror::Error)]
pub enum ActorHandleError {
    #[error("error occurred shutting down actor")]
    ShutdownError(#[from] ShutdownError),
}

#[derive(Debug, thiserror::Error)]
pub enum ShutdownError {
    #[error("sender associated with this actor was already dropped")]
    NoneSender,

    #[error("JoinError occurred while shutting down")]
    JoinError(#[from] JoinError),
}

#[derive(Debug, thiserror::Error)]
#[error("error occurred sending message")]
pub enum ActorHandleSendError<T> {
    #[error("sender associated with this actor is dropped")]
    NoneSender,

    #[error("send error occurred")]
    SendError(#[from] SendError<T>)
}
