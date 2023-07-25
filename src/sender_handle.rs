use async_trait::async_trait;

/// Helper trait that can be used to define a common interface for handles to
/// send values to their services.
#[async_trait]
pub trait SenderHandle {
    /// Type of values send to the service.
    type Item;

    /// Sends `item` to the service.
    ///
    /// # Arguments
    ///
    /// * `item` - value to be send to the service.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the operation succeeded. If it failed to send the
    /// value it returns it back, wrapped as `Err(Self::Item)`.
    async fn send(&mut self, item: Self::Item) -> Result<(), Self::Item>;
}
