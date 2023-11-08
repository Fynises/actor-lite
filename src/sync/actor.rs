use tokio::sync::mpsc;
use super::handler::Handler;

pub(crate) struct Actor<T, U: Handler<T>> {
    receiver: mpsc::UnboundedReceiver<T>,
    handler: U,
}

impl <T, U: Handler<T>> Actor<T, U> {
    pub(crate) fn new(rx: mpsc::UnboundedReceiver<T>, handler: U) -> Self {
        Self {
            receiver: rx,
            handler,
        }
    }

    pub(crate) async fn run(&mut self) {
        while let Some(msg) = self.receiver.recv().await {
            if let Err(e) = self.handler.handle_message(msg) {
                self.receiver.close();
                self.handler.on_error(e);
                return;
            };
        }
    }
}
