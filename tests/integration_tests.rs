use std::time::Duration;

use cancellable::{Cancellable, CancellationToken};
use tokio::{
    sync::mpsc::{error::SendError, unbounded_channel},
    time::timeout,
};

use crate::common::MockCancellable;

mod common;

#[tokio::test]
async fn should_receive_sent_items() -> Result<(), anyhow::Error> {
    // Arrange
    let (sender, mut receiver) = unbounded_channel();

    let cancellable = MockCancellable::new();
    let mut handle = cancellable
        .spawn_with_callback(CancellationToken::new(), move |item| {
            match sender.send(item) {
                Ok(()) => Ok(()),
                Err(SendError(item)) => Err(item),
            }
        })
        .await;

    // Act
    handle.send(42).await.unwrap();

    // Assert
    assert_eq!(42 * 2, receiver.recv().await.unwrap());

    Ok(())
}

#[tokio::test]
async fn should_complete_when_produced_error() -> Result<(), anyhow::Error> {
    // Arrange
    let (sender, _) = unbounded_channel();

    let cancellable = MockCancellable::new();
    let mut handle = cancellable
        .spawn_with_callback(CancellationToken::new(), move |item| {
            match sender.send(item) {
                Ok(()) => Ok(()),
                Err(SendError(item)) => Err(item),
            }
        })
        .await;

    // Act
    handle.send(0).await.unwrap();

    // Assert
    assert!(handle.await?.is_err());

    Ok(())
}

#[tokio::test]
async fn should_await_when_produces_continue() -> Result<(), anyhow::Error> {
    // Arrange
    let (sender, mut receiver) = unbounded_channel();

    let cancellable = MockCancellable::new();
    let mut handle = cancellable
        .spawn_with_callback(CancellationToken::new(), move |item| {
            match sender.send(item) {
                Ok(()) => Ok(()),
                Err(SendError(item)) => Err(item),
            }
        })
        .await;

    // Act
    handle.send(-1).await.unwrap();

    // Assert
    let t = timeout(Duration::from_millis(100), receiver.recv()).await;
    assert!(t.is_err());

    Ok(())
}

#[tokio::test]
async fn should_pass_result_after_continue() -> Result<(), anyhow::Error> {
    // Arrange
    let (sender, mut receiver) = unbounded_channel();

    let cancellable = MockCancellable::new();
    let mut handle = cancellable
        .spawn_with_callback(CancellationToken::new(), move |item| {
            match sender.send(item) {
                Ok(()) => Ok(()),
                Err(SendError(item)) => Err(item),
            }
        })
        .await;

    handle.send(-1).await.unwrap();

    // Act
    handle.send(42).await.unwrap();

    // Assert
    assert_eq!(42 * 2, receiver.recv().await.unwrap());

    Ok(())
}

#[tokio::test]
async fn should_complete_when_produces_break() -> Result<(), anyhow::Error> {
    // Arrange
    let (sender, mut receiver) = unbounded_channel();

    let cancellable = MockCancellable::new();
    let handle = cancellable
        .spawn_with_callback(CancellationToken::new(), move |item| {
            match sender.send(item) {
                Ok(()) => Ok(()),
                Err(SendError(item)) => Err(item),
            }
        })
        .await;

    // Act
    drop(handle);

    // Assert
    assert!(receiver.recv().await.is_none());

    Ok(())
}

#[tokio::test]
async fn should_await_when_not_cancelled() -> Result<(), anyhow::Error> {
    // Arrange
    let cancellable = MockCancellable::new();

    // Act
    let handle = cancellable.spawn(CancellationToken::new()).await;

    // Assert
    let t = timeout(Duration::from_millis(100), handle).await;
    assert!(t.is_err());

    Ok(())
}

#[tokio::test]
async fn should_complete_task_when_cancelled() -> Result<(), anyhow::Error> {
    // Arrange
    let cancellation_token = CancellationToken::new();

    let cancellable = MockCancellable::new();
    let handle = cancellable.spawn(cancellation_token.clone()).await;

    // Act
    cancellation_token.cancel();

    // Assert
    handle.await??;

    Ok(())
}

#[tokio::test]
async fn should_complete_when_cancelled_via_handle() -> Result<(), anyhow::Error> {
    // Arrange
    let cancellable = MockCancellable::new();
    let handle = cancellable.spawn(CancellationToken::new()).await;

    // Act
    handle.cancel();

    // Assert
    handle.await??;

    Ok(())
}
