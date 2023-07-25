use cancellable::{Cancellable, CancellationResult, SenderHandle};
use tokio::sync::mpsc::{error::SendError, unbounded_channel, UnboundedReceiver, UnboundedSender};

#[derive(Debug)]
pub(crate) struct Feeder {
    inner: UnboundedSender<i32>,
}

impl Feeder {
    pub(self) fn new(inner: UnboundedSender<i32>) -> Self {
        Self { inner }
    }
}

#[async_trait::async_trait]
impl SenderHandle for Feeder {
    type Item = i32;

    async fn send(&mut self, item: Self::Item) -> Result<(), Self::Item> {
        match self.inner.send(item) {
            Ok(()) => Ok(()),
            Err(SendError(item)) => Err(item),
        }
    }
}

pub(crate) struct MockCancellable {
    receiver: UnboundedReceiver<i32>,
    sender: Option<UnboundedSender<i32>>,
}

impl MockCancellable {
    pub(crate) fn new() -> Self {
        let (sender, receiver) = unbounded_channel();
        Self {
            sender: Some(sender),
            receiver,
        }
    }
}

#[async_trait::async_trait]
impl Cancellable for MockCancellable {
    type Result = i32;
    type Handle = Feeder;
    type Error = anyhow::Error;

    async fn new_handle(&mut self) -> Self::Handle {
        self.sender
            .take()
            .map(Feeder::new)
            .expect("MockCancellable's sender to be present.")
    }

    async fn run(&mut self) -> Result<CancellationResult<Self::Result>, Self::Error> {
        match self.receiver.recv().await {
            Some(item) if item < 0 => Ok(CancellationResult::Continue),
            Some(0) => Err(anyhow::anyhow!("Received zero")),
            Some(item) => Ok(CancellationResult::Item(item * 2)),
            None => Ok(CancellationResult::Break),
        }
    }
}
