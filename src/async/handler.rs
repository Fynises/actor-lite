use crate::error_handling::{Result, Error};

/// an alternative handler which makes use of async_trait to allow the handler to perform async functions
#[async_trait::async_trait]
pub trait Handler<T> {
    async fn handle_message(&mut self, message: T) -> Result<()>;
    async fn on_error(&mut self, error: Error);
}
