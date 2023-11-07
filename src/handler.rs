use crate::error::ActorError;

pub trait Handler<T> {
    fn handle_message(&mut self, message: T) -> Result<(), ActorError>;
    fn on_error(&mut self, error: ActorError);
}
