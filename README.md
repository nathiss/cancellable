# Cancellable

[![ci-master](https://github.com/nathiss/cancellable/actions/workflows/ci-master.yaml/badge.svg)](https://github.com/nathiss/cancellable/actions/workflows/ci-master.yaml)
[![Crates.io](https://img.shields.io/crates/v/cancellable)](https://crates.io/crates/cancellable)
[![docs.rs](https://docs.rs/cancellable/badge.svg)](https://docs.rs/cancellable/)
![Crates.io](https://img.shields.io/crates/l/cancellable)

A Rust library providing a generic cancellable utility.

The goal of this library is to provide a unified way of defining background
services that are managed by the [`tokio`](https://tokio.rs/) runtime.

## Examples

```rust
use std::{error::Error, net::SocketAddr};

use cancellable::{async_trait, Cancellable, CancellationResult};
use tokio::net::{TcpListener, TcpStream};

struct Listener {
    tcp_listener: TcpListener,
}

impl Listener {
    async fn new() -> Result<Self, Box<dyn Error>> {
        let tcp_listener = TcpListener::bind("127.0.0.1:5000").await?;

        Ok(Self { tcp_listener })
    }
}

#[async_trait]
impl Cancellable for Listener {
    type Result = (TcpStream, SocketAddr);
    type Handle = ();
    type Error = std::io::Error;

    async fn new_handle(&mut self) -> Self::Handle {}

    async fn run(&mut self) -> Result<CancellationResult<Self::Result>, Self::Error> {
        let (addr, stream) = self.tcp_listener.accept().await?;

        Ok(CancellationResult::item((addr, stream)))
    }
}
```

## License

See [LICENSE.txt](./LICENSE.txt) file.
