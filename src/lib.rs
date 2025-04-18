//! Send and receive serde-compatible objects over TCP (async)
//!
//! # Example
//!
//! ```rust
//! let addr = ("127.0.0.1", 9000);
//! let stream = std::net::TcpStream::connect(addr).unwrap();
//! let stream = async_net::TcpStream::try_from(stream).unwrap();
//!
//! type Incoming = String;
//! type Outgoing = u32;
//!
//! let (task, tx, mut rx) = tropocol::async_fifo::session::<Outgoing, Incoming>(stream);
//! // spawn this task in an asynchronous executor
//!
//! tx.send(0u32 as Outgoing);
//!
//! async {
//!     let s: Incoming = rx.recv().await.unwrap();
//! };
//!
//! ```
//!

use futures_lite::{AsyncWriteExt, AsyncReadExt};
use futures_lite::future::or;
use std::future::Future;
use async_net::TcpStream;

use serde::{Serialize, Deserialize};

pub use async_net;

/// For custom implementations
pub mod raw;

#[cfg(feature = "async-fifo")]
pub mod async_fifo;

#[cfg(feature = "async-channel")]
pub mod async_channel;

/// Alias trait for `Send` futures
pub trait SendFut<O>: Future<Output = O> + Send {}
impl<O, Y: Future<Output = O> + Send> SendFut<O> for Y {}

/// Object which produces outgoing messages
pub trait GetOutgoing<O> {
    fn get_outgoing(&mut self) -> impl SendFut<Option<O>>;
}

/// Object which handles incoming messages
pub trait HandleIncoming<I> {
    fn handle_incoming(&mut self, incoming: I) -> impl SendFut<()>;
}
