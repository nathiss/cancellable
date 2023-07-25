use async_trait::async_trait;

/// Helper trait that can be used to define a common interface for handles to
/// send values to their services.
///
/// # Examples
///
/// ```
/// use cancellable::{async_trait, Cancellable, CancellationResult, SenderHandle};
/// use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
///
/// #[derive(Debug)]
/// struct NumberSender {
///     inner: UnboundedSender<i32>,
/// }
///
/// #[async_trait]
/// impl SenderHandle for NumberSender {
///     type Item = i32;
///
///     async fn send(&mut self, item: Self::Item) -> Result<(), Self::Item> {
///         self.inner.send(item).map_err(|_| item)?;
///         Ok(())
///     }
/// }
///
/// struct Multiplier {
///     number_receiver: UnboundedReceiver<i32>,
///     number_sender: Option<UnboundedSender<i32>>,
/// }
///
/// impl Multiplier {
///     fn new() -> Self {
///         let (sender, receiver) = unbounded_channel();
///
///         Self {
///             number_receiver: receiver,
///             number_sender: Some(sender),
///         }
///     }
/// }
///
/// #[async_trait]
/// impl Cancellable for Multiplier {
///     type Result = i32;
///     type Handle = NumberSender;
///     type Error = anyhow::Error;
///
///     async fn new_handle(&mut self) -> Self::Handle {
///         let sender = self
///             .number_sender
///             .take()
///             .expect("number_sender to be present");
///
///         NumberSender { inner: sender }
///     }
///
///     async fn run(&mut self) -> Result<CancellationResult<Self::Result>, Self::Error> {
///         let number = self
///             .number_receiver
///             .recv()
///             .await
///             .ok_or(anyhow::anyhow!("Channel has been closed."))?;
///
///         Ok(CancellationResult::item(number * 2))
///     }
/// }
/// ```
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
    /// Returns `Ok(())` if the operation succeeded. If it's failed to send the
    /// value, it returns it back wrapped as `Err(Self::Item)`.
    async fn send(&mut self, item: Self::Item) -> Result<(), Self::Item>;
}
