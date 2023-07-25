# Cancellable

A Rust library providing a generic cancellable utility.

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
