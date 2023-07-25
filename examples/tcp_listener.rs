use std::{error::Error, net::SocketAddr, time::Duration};

use cancellable::{async_trait, Cancellable, CancellationResult, CancellationToken};
use tokio::{
    net::{TcpListener, TcpStream},
    time::sleep,
};

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

fn handle_connection(_stream: TcpStream, addr: SocketAddr) {
    print!("New connection from {}.", addr);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cancellation_token = CancellationToken::new();

    let listener = Listener::new().await?;

    let handle = listener
        .spawn_with_callback(cancellation_token.child_token(), |(stream, addr)| {
            handle_connection(stream, addr);
            Ok(())
        })
        .await;

    sleep(Duration::from_secs(10)).await;

    cancellation_token.cancel();
    handle.await??;

    Ok(())
}
