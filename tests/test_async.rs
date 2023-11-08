use actor_lite::{r#async::handler::Handler, handle::ActorHandle};
use tokio::sync::oneshot;
use actor_lite::error_handling::{Error, Result};
use anyhow::anyhow;

#[tokio::test]
async fn test() {
    let (handler, error_rx) = TestHandler::new();
    let actor_handle = ActorHandle::new_async(handler);

    {
        let (tx, rx) = oneshot::channel::<usize>();
        actor_handle.send(TestMessage::GetCount(tx)).unwrap();
        assert_eq!(0, rx.await.unwrap())
    }

    {
        let (tx, rx) = oneshot::channel::<usize>();
        actor_handle.send(TestMessage::Increment).unwrap();
        actor_handle.send(TestMessage::GetCount(tx)).unwrap();
        assert_eq!(1, rx.await.unwrap())
    }

    {
        let (tx_1, rx_1) = oneshot::channel::<()>();
        let (tx_2, rx_2) = oneshot::channel::<()>();
        let (tx_val, rx_val) = oneshot::channel::<usize>();
        actor_handle.send(TestMessage::TestAsync(tx_1, rx_2)).unwrap();
        actor_handle.send(TestMessage::GetCount(tx_val)).unwrap();
        rx_1.await.unwrap();
        tx_2.send(()).unwrap();
        assert_eq!(2, rx_val.await.unwrap())
    }

    {
        actor_handle.send(TestMessage::TestError).unwrap();
        let err_recv = error_rx.await.is_err();
        assert_eq!(false, err_recv);
        let is_abort_err = actor_handle.abort().await.is_err();
        assert_eq!(false, is_abort_err);
    }
}

#[tokio::test]
async fn test_shutdown() {
    let (handler, _error_tx) = TestHandler::new();
    let actor_handle = ActorHandle::new_async(handler);
    let is_shutdown_error = actor_handle.shutdown().await.is_err();
    assert_eq!(false, is_shutdown_error);
}

#[derive(Debug)]
enum TestMessage {
    Increment,
    GetCount(oneshot::Sender<usize>),
    TestAsync(oneshot::Sender<()>, oneshot::Receiver<()>),
    TestError, // intentional error causing message
}

struct TestHandler {
    count: usize,
    error_tx: Option<oneshot::Sender<()>>,
}

impl TestHandler {
    fn new() -> (Self, oneshot::Receiver<()>) {
        let (tx, rx) = oneshot::channel::<()>();
        let this = Self { 
            count: 0,
            error_tx: Some(tx),
        };
        (this, rx)
    }

    async fn handle_increment(&mut self) {
        self.count += 1;
    }

    async fn handle_get_count(&self) -> usize {
        self.count
    }

    async fn on_test_async(
        &mut self, 
        tx: oneshot::Sender<()>, 
        rx: oneshot::Receiver<()>
    ) {
        tx.send(()).unwrap();
        rx.await.unwrap();
        self.handle_increment().await;   
    }
}

#[async_trait::async_trait]
impl Handler<TestMessage> for TestHandler {
    async fn handle_message(&mut self, message: TestMessage) -> Result<()> {
        let _ = match message {
            TestMessage::Increment => self.handle_increment().await,
            TestMessage::GetCount(tx) => tx.send(self.handle_get_count().await).map_err(|e| {
                anyhow!("error sending count value {e:#?}")
            })?,
            TestMessage::TestAsync(tx, rx) => self.on_test_async(tx, rx).await,
            TestMessage::TestError => return Err(anyhow!("intentional error caused"))
        };
        Ok(())
    }

    async fn on_error(&mut self, error: Error) {
        println!("error occurred: {error:#}");
        self.error_tx.take().unwrap().send(()).unwrap();
    }
}
