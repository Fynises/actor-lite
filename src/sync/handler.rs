use crate::error_handling::{Result, Error};

pub trait Handler<T> {
    fn handle_message(&mut self, message: T) -> Result<()>;
    fn on_error(&mut self, error: Error);
}
