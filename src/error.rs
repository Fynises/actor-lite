use tokio::sync::mpsc::error::SendError;

#[derive(Debug, thiserror::Error)]
pub enum ActorError {
    #[error("unknown error occurred")]
    UnknownError(#[from] Box<(dyn std::error::Error + Send + Sync + 'static)>),
    
    #[error("anyhow error encountered")]
    Anyhow(#[from] anyhow::Error)
}

#[derive(Debug, thiserror::Error)]
pub enum ActorHandleError<T> {
    #[error("error occurred sending message")]
    SendError(#[from] SendError<T>),
}
