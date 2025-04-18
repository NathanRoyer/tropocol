use futures_lite::{AsyncWriteExt, AsyncReadExt};
use futures_lite::future::or;
use std::future::Future;
use async_net::TcpStream;

use serde::{Serialize, Deserialize};

pub use async_net;

/// Alias trait for `Send` futures
pub trait SendFut<O>: Future<Output = O> + Send {}
impl<O, Y: Future<Output = O> + Send> SendFut<O> for Y {}

/// Closure which produces outgoing messages
pub trait GetOutgoing<O, Y: SendFut<O>>: FnMut() -> Y {}
impl<O, Y: SendFut<O>, F: FnMut() -> Y> GetOutgoing<O, Y> for F {}

/// Closure which handles incoming messages
pub trait HandleIncoming<I, Y: SendFut<()>>: FnMut(I) -> Y {}
impl<I, Y: SendFut<()>, F: FnMut(I) -> Y> HandleIncoming<I, Y> for F {}

/// Handles encoding/decoding and transmission/reception of TCP messages
///
/// # Example
///
/// ```rust
/// let addr = ("127.0.0.1", 9000);
/// let stream = std::net::TcpStream::connect(addr).unwrap();
/// let stream = async_net::TcpStream::try_from(stream).unwrap();
///
/// type Incoming = String;
/// type Outgoing = u32;
///
/// let handle_incoming = |incoming: Incoming| async move {
///     println!("Incoming: {}", incoming);
/// };
///
/// let await_outgoing = || async {
///     Some(0 as Outgoing)
/// };
///
/// let session_task = tropocol::session(stream, handle_incoming, await_outgoing);
/// // spawn this task in an asynchronous executor
/// ```
///
pub async fn session<I, O, R, T, Y1, Y2>(
    stream: TcpStream,
    handle_incoming: R,
    await_outgoing: T,
) -> Result<(), ()>
where
    I: for<'a> Deserialize<'a>,
    O: Serialize,
    R: HandleIncoming<I, Y1>,
    T: GetOutgoing<Option<O>, Y2>,
    Y1: SendFut<()>,
    Y2: SendFut<Option<O>>,
{
    let tx_tcp = stream.clone();
    let rx_tcp = stream;

    let tx_task = transmit(tx_tcp, await_outgoing);
    let rx_task = receive(rx_tcp, handle_incoming);

    or(tx_task, rx_task).await
}

async fn receive<I, R, Y>(
    mut rx_tcp: TcpStream,
    mut handle_incoming: R,
) -> Result<(), ()>
where
    I: for<'a> Deserialize<'a>,
    R: HandleIncoming<I, Y>,
    Y: SendFut<()>,
{
    loop {
        let mut len = [0u8; 8];
        rx_tcp.read_exact(&mut len).await.map_err(drop)?;
        let len = u64::from_be_bytes(len) as usize;

        let mut buffer = vec![0u8; len];
        rx_tcp.read_exact(&mut buffer).await.map_err(drop)?;
        let incoming = serde_json::from_slice(&buffer).map_err(drop)?;

        handle_incoming(incoming).await;
    }
}

async fn transmit<O, T, Y>(
    mut tx_tcp: TcpStream,
    mut await_outgoing: T,
) -> Result<(), ()>
where
    O: Serialize,
    T: GetOutgoing<Option<O>, Y>,
    Y: SendFut<Option<O>>,
{
    while let Some(incoming) = await_outgoing().await {
        let bytes = serde_json::to_vec(&incoming).map_err(drop)?;
        let len = (bytes.len() as u64).to_be_bytes();

        tx_tcp.write_all(&len).await.map_err(drop)?;
        tx_tcp.write_all(&bytes).await.map_err(drop)?;
    }

    Ok(())
}
