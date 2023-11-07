use actor_lite::{handler::Handler, error::ActorError, handle::ActorHandle};
use tokio::sync::oneshot;
use anyhow::anyhow;

#[tokio::test]
async fn test() {
    println!("starting tests");

    let handler = TestHandler::new();
    let actor_handle = ActorHandle::new(handler);

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
}

#[derive(Debug)]
enum TestMessage {
    Increment,
    GetCount(oneshot::Sender<usize>),
}

struct TestHandler {
    count: usize,
}

impl TestHandler {
    fn new() -> Self {
        Self { count: 0 }
    }

    fn on_increment(&mut self) {
        self.count += 1;
    }
    
    fn on_get_count(&self, tx: oneshot::Sender<usize>) -> Result<(), ActorError> {
        let _ = tx.send(self.count).map_err(|e| anyhow!("error sending number {}", e));
        Ok(())
    }
}

impl Handler<TestMessage> for TestHandler {
    fn handle_message(&mut self, message: TestMessage) -> Result<(), ActorError> {
        let _ = match message {
            TestMessage::Increment => self.on_increment(),
            TestMessage::GetCount(tx) => self.on_get_count(tx)?,
        };
        Ok(())
    }

    fn on_error(&mut self, error: ActorError) {
        println!("error occurred: {error:#?}")
    }
}
