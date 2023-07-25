use std::{
    ops::{Deref, DerefMut},
    task::Poll,
};

use pin_project::pin_project;
use tokio::task::{JoinError, JoinHandle};

use crate::Cancellable;

/// Service handle that allows to await for the service to join after it has
/// been cancelled.
///
/// Awaiting this future does not guarantee that the service will ever join. It
/// the callers responsibility to ensure that the service either has been
/// cancelled, or it will join on its own.
#[pin_project]
#[derive(Debug)]
pub struct CancellableHandle<T>
where
    T: Cancellable,
{
    #[pin]
    join_handle: JoinHandle<Result<(), <T as Cancellable>::Error>>,
    inner: <T as Cancellable>::Handle,
}

impl<T> CancellableHandle<T>
where
    T: Cancellable,
{
    pub(crate) fn new(
        join_handle: JoinHandle<Result<(), <T as Cancellable>::Error>>,
        inner: <T as Cancellable>::Handle,
    ) -> Self {
        Self { join_handle, inner }
    }
}

impl<T> std::future::Future for CancellableHandle<T>
where
    T: Cancellable,
{
    type Output = Result<Result<(), <T as Cancellable>::Error>, JoinError>;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        this.join_handle.poll(cx)
    }
}

impl<T> Deref for CancellableHandle<T>
where
    T: Cancellable,
{
    type Target = <T as Cancellable>::Handle;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for CancellableHandle<T>
where
    T: Cancellable,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio_util::sync::CancellationToken;

    use crate::{Cancellable, CancellableHandle, CancellationResult};

    struct MockCancellable {}

    #[async_trait::async_trait]
    impl Cancellable for MockCancellable {
        type Result = ();
        type Handle = ();
        type Error = anyhow::Error;

        async fn run(&mut self) -> Result<CancellationResult<Self::Result>, Self::Error> {
            Ok(CancellationResult::Continue)
        }

        async fn new_handle(&mut self) -> Self::Handle {}
    }

    #[tokio::test]
    async fn aborts_on_drop() {
        // Arrange
        let cancellation_token = CancellationToken::new();
        let cancellation_token_clone = cancellation_token.clone();

        let task = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(200)).await;
            cancellation_token_clone.cancel();
            Err(anyhow::anyhow!(""))
        });
        let handle = CancellableHandle::<MockCancellable>::new(task, ());

        // Act
        drop(handle);
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Assert
        assert!(!cancellation_token.is_cancelled());
    }
}
