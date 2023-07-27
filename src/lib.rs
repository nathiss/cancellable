//! This crate provides a way of defining an interface fo//!^r a background service.
//! //!
//! The main entrypoint of this create is [`cancellable::Cancellable`] trait. It
//! is an [`async-trait`](https://docs.rs/async-trait/latest/async_trait/) that
//! depends on [tokio](https://tokio.rs).
//!
//! # Examples
//!
//! ```
//! use std::{error::Error, net::SocketAddr};
//!
//! use cancellable::{async_trait, Cancellable, CancellationResult};
//! use tokio::net::{TcpListener, TcpStream};
//!
//! struct Listener {
//!     tcp_listener: TcpListener,
//! }
//!
//! impl Listener {
//!     async fn new() -> Result<Self, Box<dyn Error>> {
//!         let tcp_listener = TcpListener::bind("127.0.0.1:5000").await?;
//!
//!         Ok(Self { tcp_listener })
//!     }
//! }
//!
//! #[async_trait]
//! impl Cancellable for Listener {
//!     type Result = (TcpStream, SocketAddr);
//!     type Handle = ();
//!     type Error = std::io::Error;
//!
//!     async fn new_handle(&mut self) -> Self::Handle {}
//!
//!     async fn run(&mut self) -> Result<CancellationResult<Self::Result>, Self::Error> {
//!         let (addr, stream) = self.tcp_listener.accept().await?;
//!
//!         Ok(CancellationResult::item((addr, stream)))
//!     }
//! }
//! ```

#![warn(missing_docs)]

mod cancellable;
mod cancellable_handle;
mod cancellation_result;

pub use crate::cancellable::Cancellable;
pub use crate::cancellable_handle::CancellableHandle;
pub use crate::cancellation_result::CancellationResult;
pub use async_trait::async_trait;
pub use tokio_util::sync::CancellationToken;
