use super::{TcpStream, SendFut, Serialize, Deserialize};
use ::async_fifo::{Sender, Receiver};

/// Object which produces outgoing messages
impl<O: Send + Unpin> super::GetOutgoing<O> for Receiver<O> {
    async fn get_outgoing(&mut self) -> Option<O> {
        self.recv().await.ok()
    }
}

/// Object which handles incoming messages
impl<I: Send> super::HandleIncoming<I> for Sender<I> {
    async fn handle_incoming(&mut self, incoming: I) {
        self.send(incoming);
    }
}

pub fn session<O, I>(tcp_stream: TcpStream) -> (impl SendFut<Result<(), ()>>, Sender<O>, Receiver<I>)
where
    O: Send + Unpin + Serialize + 'static,
    I: Send + for<'a> Deserialize<'a> + 'static,
{
    let (tx_outgoing, rx_outgoing) = ::async_fifo::new();
    let (tx_incoming, rx_incoming) = ::async_fifo::new();

    let task = super::raw::session(tcp_stream, tx_incoming, rx_outgoing);

    (task, tx_outgoing, rx_incoming)
}
