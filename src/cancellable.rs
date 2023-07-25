use async_trait::async_trait;
use tokio_util::sync::CancellationToken;

use crate::{cancellation_result::CancellationResult, CancellableHandle};

/// Defines an interface for a cancellable service with an optional callback.
#[async_trait]
pub trait Cancellable {
    /// Type of values that _can_ be yielded by the service.
    type Result;

    /// Type of a handle for communicating with the service.
    type Handle: std::fmt::Debug;

    /// Error returned by [`Self::run`] method.
    type Error: std::fmt::Debug + std::fmt::Display + Send;

    /// Performs a single unit of work.
    ///
    /// The return value of this method controls whether the service will
    /// continue to loop. If the returned value is either
    /// `Ok(CancellableResult::Break)` or `Err(Self::Error)`, then the service
    /// will complete.
    ///
    /// See [CancellationResult::Break].
    ///
    /// # Returns
    ///
    /// Returned value controls whether the work loop will continue. If the
    /// returned value is `Err(Self::Error)` or `Ok(CancellationResult::Break)`,
    /// then the work loop finishes. The method can optionally yield a value by
    /// returning [`CancellationResult::Item`].
    ///
    /// [CancellableResult::Break]: crate::CancellationResult#variant.Break
    /// [`CancellationResult::Item`]: crate::CancellationResult#variant.Item
    async fn run(&mut self) -> Result<CancellationResult<Self::Result>, Self::Error>;

    /// Constructs a new handle for communicating with the service.
    ///
    /// This method is intended to be called only once. If it's called more than
    /// once, then the behavior is undefined.
    async fn new_handle(&mut self) -> Self::Handle;

    /// Consumes the service and spawns its work loop.
    ///
    /// It's equivalent to [`Self::spawn_with_callback`] in every way, besides
    /// the callback.
    async fn spawn(mut self, cancellation_token: CancellationToken) -> CancellableHandle<Self>
    where
        Self: Sized + Send + 'static,
    {
        self.spawn_with_callback(cancellation_token, |_| Ok(()))
            .await
    }

    /// Consumes the service and spawns its work loop.
    ///
    /// Schedules a new background task, that repetitively calls [`Self::run`]
    /// and performs its work. If `cancellation_token` is cancelled, then the
    /// service completes instantly.
    ///
    /// # Arguments
    ///
    /// * `cancellation_token` - provides a way of cancelling the service mid
    /// work.
    /// * `callback` - if the service yields a new value, then it's passed to
    /// the callback. If the callback returns `Err`, then service completes with
    /// the same error.
    ///
    /// # Returns
    ///
    /// Handle that can be used to await for the service to complete.
    async fn spawn_with_callback<F>(
        mut self,
        cancellation_token: CancellationToken,
        mut callback: F,
    ) -> CancellableHandle<Self>
    where
        Self: Sized + Send + 'static,
        F: FnMut(Self::Result) -> Result<(), Self::Result> + Send + 'static,
    {
        let inner = self.new_handle().await;

        let join_handle = tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = cancellation_token.cancelled() => {
                        break
                    }
                    result = self.run() => {
                        match result {
                            Ok(CancellationResult::Item(result)) => {
                                if let Err(_result) = callback(result) {
                                    break
                                }
                            },
                            Ok(CancellationResult::Continue) => {}
                            Ok(CancellationResult::Break) => break,
                            Err(e) => return Err(e)
                        }
                    }
                }
            }

            Ok(())
        });

        CancellableHandle::new(join_handle, inner)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{
            atomic::{AtomicBool, Ordering},
            Arc,
        },
        time::Duration,
    };

    use tokio::time::timeout;
    use tokio_util::sync::CancellationToken;

    use crate::{Cancellable, CancellationResult};

    struct MockCancellable {
        flag: Arc<AtomicBool>,
    }

    impl MockCancellable {
        fn new(flag: bool) -> Self {
            Self {
                flag: Arc::new(AtomicBool::new(flag)),
            }
        }
    }

    #[async_trait::async_trait]
    impl Cancellable for MockCancellable {
        type Result = Arc<AtomicBool>;
        type Handle = ();
        type Error = std::io::Error;

        async fn run(&mut self) -> Result<CancellationResult<Self::Result>, Self::Error> {
            let () = std::future::pending().await;
            self.flag.store(true, Ordering::Relaxed);
            Ok(CancellationResult::Break)
        }

        async fn new_handle(&mut self) -> Self::Handle {}
    }

    #[tokio::test]
    async fn should_finish_when_cancelled() {
        // Arrange
        let flag = Arc::new(AtomicBool::new(false));
        let flag_clone = Arc::clone(&flag);

        let cancellable = MockCancellable::new(false);
        let cancellation_token = CancellationToken::new();

        // Act
        let handle = cancellable
            .spawn_with_callback(cancellation_token.clone(), move |_| {
                flag_clone.store(true, Ordering::SeqCst);
                Ok(())
            })
            .await;

        cancellation_token.cancel();

        // Assert
        assert!(!flag.load(Ordering::SeqCst));
        handle.await.unwrap().unwrap();
    }

    struct ErrorCancellable {}

    #[async_trait::async_trait]
    impl Cancellable for ErrorCancellable {
        type Result = ();
        type Handle = ();
        type Error = anyhow::Error;

        async fn run(&mut self) -> Result<CancellationResult<()>, Self::Error> {
            Err(anyhow::anyhow!("ErrorCancellable error"))
        }

        async fn new_handle(&mut self) -> Self::Handle {}
    }

    #[tokio::test]
    async fn should_propagate_error_from_task() {
        // Arrange
        let cancellable = ErrorCancellable {};
        let cancellation_token = CancellationToken::new();

        // Act
        let handle = cancellable.spawn(cancellation_token).await;

        // Assert
        let result = handle.await.unwrap();
        assert!(result.is_err());
    }

    struct BreakCancellable {}

    #[async_trait::async_trait]
    impl Cancellable for BreakCancellable {
        type Result = ();
        type Handle = ();
        type Error = anyhow::Error;

        async fn run(&mut self) -> Result<CancellationResult<()>, Self::Error> {
            Ok(CancellationResult::Break)
        }

        async fn new_handle(&mut self) -> Self::Handle {}
    }

    #[tokio::test]
    async fn should_finish_future_when_breaks() {
        // Arrange
        let cancellable = BreakCancellable {};
        let cancellation_token = CancellationToken::new();

        // Act
        let handle = cancellable.spawn(cancellation_token).await;

        // Assert
        let result = handle.await.unwrap();
        assert!(result.is_ok());
    }

    struct ContinueCancellable {}

    #[async_trait::async_trait]
    impl Cancellable for ContinueCancellable {
        type Result = ();
        type Handle = ();
        type Error = anyhow::Error;

        async fn run(&mut self) -> Result<CancellationResult<()>, Self::Error> {
            tokio::task::yield_now().await;
            Ok(CancellationResult::Continue)
        }

        async fn new_handle(&mut self) -> Self::Handle {}
    }

    #[tokio::test]
    async fn should_pending_when_continues() {
        // Arrange
        let cancellable = ContinueCancellable {};
        let cancellation_token = CancellationToken::new();

        // Act
        let handle = cancellable.spawn(cancellation_token.clone()).await;

        // Assert
        let t = timeout(Duration::from_millis(150), handle).await;
        assert!(t.is_err());
    }
}
