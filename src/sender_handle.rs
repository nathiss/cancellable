use async_trait::async_trait;

#[async_trait]
pub trait SenderHandle {
    type Item;

    async fn send(&mut self, item: Self::Item) -> Result<(), Self::Item>;
}
