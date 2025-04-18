use super::*;

/// Handles encoding/decoding and transmission/reception of TCP messages
pub async fn session<I, O, R, T>(
    stream: TcpStream,
    handle_incoming: R,
    await_outgoing: T,
) -> Result<(), ()>
where
    I: for<'a> Deserialize<'a>,
    O: Serialize,
    R: HandleIncoming<I>,
    T: GetOutgoing<O>,
{
    let tx_tcp = stream.clone();
    let rx_tcp = stream;

    let tx_task = transmit(tx_tcp, await_outgoing);
    let rx_task = receive(rx_tcp, handle_incoming);

    or(tx_task, rx_task).await
}

async fn receive<I, R>(
    mut rx_tcp: TcpStream,
    mut handle_incoming: R,
) -> Result<(), ()>
where
    I: for<'a> Deserialize<'a>,
    R: HandleIncoming<I>,
{
    loop {
        let mut len = [0u8; 8];
        rx_tcp.read_exact(&mut len).await.map_err(drop)?;
        let len = u64::from_be_bytes(len) as usize;

        let mut buffer = vec![0u8; len];
        rx_tcp.read_exact(&mut buffer).await.map_err(drop)?;
        let incoming = serde_json::from_slice(&buffer).map_err(drop)?;

        handle_incoming.handle_incoming(incoming).await;
    }
}

async fn transmit<O, T>(
    mut tx_tcp: TcpStream,
    mut await_outgoing: T,
) -> Result<(), ()>
where
    O: Serialize,
    T: GetOutgoing<O>,
{
    while let Some(incoming) = await_outgoing.get_outgoing().await {
        let bytes = serde_json::to_vec(&incoming).map_err(drop)?;
        let len = (bytes.len() as u64).to_be_bytes();

        tx_tcp.write_all(&len).await.map_err(drop)?;
        tx_tcp.write_all(&bytes).await.map_err(drop)?;
    }

    Ok(())
}
