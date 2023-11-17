use actor_lite::error::Error;
use actor_lite::{sync::handler::Handler, handle::ActorHandle};
use tokio::sync::oneshot;
use anyhow::anyhow;

#[tokio::test]
async fn test() {
    println!("starting tests");

    let (handler, err_rx) = TestHandler::new();
    let actor_handle = ActorHandle::new(|_tx| handler);

    { // test getting value
        let (tx, rx) = oneshot::channel::<usize>();
        actor_handle.send(TestMessage::GetCount(tx)).unwrap();
        assert_eq!(0, rx.await.unwrap())
    }

    { // test increment then getting value
        let (tx, rx) = oneshot::channel::<usize>();
        actor_handle.send(TestMessage::Increment).unwrap();
        actor_handle.send(TestMessage::GetCount(tx)).unwrap();
        assert_eq!(1, rx.await.unwrap())
    }

    { // test intentional error causing message, then check if handle is closed afterwards
        actor_handle.send(TestMessage::Error).unwrap();
        let err_recv = err_rx.await.is_err();
        assert_eq!(false, err_recv);
        let is_abort_err = actor_handle.abort().await.is_err();
        assert_eq!(false, is_abort_err);
    }
}

#[tokio::test]
async fn test_shutdown() {
    let (handler, _error_tx) = TestHandler::new();
    let actor_handle = ActorHandle::new(|_tx| handler);
    let is_shutdown_error = actor_handle.shutdown().await.is_err();
    assert_eq!(false, is_shutdown_error)
}

#[derive(Debug)]
enum TestMessage {
    Increment,
    GetCount(oneshot::Sender<usize>),
    Error, // intentional error causing message
}

struct TestHandler {
    count: usize,
    error_signaller: Option<oneshot::Sender<()>>,
}

impl TestHandler {
    fn new() -> (Self, oneshot::Receiver<()>) {
        let (tx, rx) = oneshot::channel::<>();
        let this = Self { 
            count: 0,
            error_signaller: Some(tx) 
        };
        (this, rx)
    }

    fn on_increment(&mut self) {
        self.count += 1;
    }
    
    fn on_get_count(&self, tx: oneshot::Sender<usize>) -> Result<(), Error> {
        let _ = tx.send(self.count).map_err(|e| anyhow!("error sending number {}", e));
        Ok(())
    }
}

impl Handler<TestMessage> for TestHandler {
    fn handle_message(&mut self, message: TestMessage) -> Result<(), Error> {
        let _ = match message {
            TestMessage::Increment => self.on_increment(),
            TestMessage::GetCount(tx) => self.on_get_count(tx)?,
            TestMessage::Error => return Err(anyhow!("test error").into()),
        };
        Ok(())
    }

    fn on_error(&mut self, error: Error) {
        println!("error occurred: {error:#?}");
        self.error_signaller.take().unwrap().send(()).unwrap();
    }
}
