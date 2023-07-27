use std::{collections::VecDeque, error::Error};

use cancellable::{async_trait, Cancellable, CancellationResult, CancellationToken};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

#[derive(Debug)]
struct NumberSender {
    inner: UnboundedSender<i32>,
}

impl NumberSender {
    async fn send(&mut self, item: i32) -> Result<(), i32> {
        self.inner.send(item).map_err(|_| item)?;
        Ok(())
    }
}

struct Multiplier {
    number_receiver: UnboundedReceiver<i32>,
    number_sender: Option<UnboundedSender<i32>>,
}

impl Multiplier {
    fn new() -> Self {
        let (sender, receiver) = unbounded_channel();

        Self {
            number_receiver: receiver,
            number_sender: Some(sender),
        }
    }
}

#[async_trait]
impl Cancellable for Multiplier {
    type Result = i32;
    type Handle = NumberSender;
    type Error = anyhow::Error;

    async fn new_handle(&mut self) -> Self::Handle {
        let sender = self
            .number_sender
            .take()
            .expect("number_sender to be present");

        NumberSender { inner: sender }
    }

    async fn run(&mut self) -> Result<CancellationResult<Self::Result>, Self::Error> {
        let number = self
            .number_receiver
            .recv()
            .await
            .ok_or(anyhow::anyhow!("Channel has been closed."))?;

        Ok(CancellationResult::item(number * 2))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cancellation_token = CancellationToken::new();
    let listener = Multiplier::new();

    let input: VecDeque<_> = [42, 13, 37].into();
    let mut expected: VecDeque<_> = input.iter().map(|i| i * 2).collect();

    let mut handle = listener
        .spawn_with_callback(cancellation_token.child_token(), move |number| {
            let Some(front) = expected.pop_front() else {
                panic!("Too many inputs");
            };

            println!("Received {}.", number);
            assert_eq!(front, number);
            Ok(())
        })
        .await;

    for i in input {
        println!("Sending {}...", i);
        assert!(handle.send(i).await.is_ok());
    }

    cancellation_token.cancel();
    handle.await??;

    Ok(())
}
