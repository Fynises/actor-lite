type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

pub trait Handler<T> {
    fn handle_message(&mut self, message: T) -> Result<(), Error>;
    fn on_error(&mut self, error: Error);
}
