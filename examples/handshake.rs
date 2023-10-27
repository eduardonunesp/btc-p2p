use btc_p2p::{Command, Message, Network, Payload, ServiceFlags, VersionPayload};
use crossbeam_utils::sync::WaitGroup;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::{
    io::AsyncReadExt,
    io::AsyncWriteExt,
    net::{lookup_host, TcpStream},
    sync::mpsc::channel,
    time::timeout,
};

/// This example connects to a Bitcoin node and performs a handshake.
const BTC_SEED: &str = "seed.bitcoin.sipa.be";

/// The port of the Bitcoin node.
const BTC_NODE_PORT: u16 = 8333;

/// The size of the channels used to communicate between the threads.
const CHANNELS_BUFFER_SIZE: usize = 1;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let (socket_chan_tx, mut socket_chan_rx) = channel::<SocketAddr>(CHANNELS_BUFFER_SIZE);

    tracing::info!("Getting seed from{}", BTC_SEED);

    // Get the addresses of the Bitcoin nodes.
    let addrs = lookup_host((BTC_SEED, BTC_NODE_PORT))
        .await?
        .collect::<Vec<_>>();

    // Spawn a thread that sends the addresses to the channel.
    tokio::spawn(async move {
        for addr in addrs {
            if let Err(err) = socket_chan_tx.send(addr).await {
                tracing::error!("Failed to send address to channel: {}", err);
            }
        }
    });

    let wg = WaitGroup::new();

    // Spawn a thread for each address received from the channel.
    while let Some(socket) = socket_chan_rx.recv().await {
        let wg = wg.clone();

        tokio::spawn(async move {
            match timeout(Duration::from_secs(3), handshake(socket)).await {
                Ok(Ok(_)) => {
                    tracing::info!("Handshake successful with for {}", socket);
                }
                Ok(Err(err)) => {
                    tracing::error!("Handshake failed with {} for {}", socket, err);
                }
                Err(_) => {
                    tracing::error!("Handshake timed out for {}", socket);
                }
            }

            drop(wg);
        });
    }

    // Wait for all the threads to finish.
    wg.wait();

    Ok(())
}

// Performs a handshake with the Bitcoin node.
async fn handshake(socket: SocketAddr) -> anyhow::Result<()> {
    tracing::info!("Connecting to {}", socket);

    // Connect to the Bitcoin node.
    let mut tcp_stream = TcpStream::connect(socket).await?;

    // Build the version message, which is the first message sent to the Bitcoin node.
    let version_msg = Message::new(
        Network::MainNet,
        Command::Version,
        VersionPayload::build(
            ServiceFlags::NODE_NETWORK,
            ServiceFlags::NODE_NETWORK,
            tcp_stream.local_addr()?,
            ServiceFlags::NODE_NETWORK,
            socket,
            rand::random(),
            0x0,
            true,
        ),
    );

    // Send the version message and wait for the response.
    tracing::info!("Sending version to {}", socket);
    let msg_recv = send_and_receive(&mut tcp_stream, version_msg).await?;
    tracing::info!("Received version {:?} from {}", msg_recv.payload, socket);

    // Send the verack message, to confirm the version message.
    let verack_msg = Message::new(Network::MainNet, Command::VerAck, Payload::VerAck);
    tracing::info!("Sending verack to {}", socket);
    let msg_recv = send_and_receive(&mut tcp_stream, verack_msg).await?;
    tracing::info!("Received verack {:?} from {}", msg_recv.payload, socket);

    Ok(())
}

async fn send_and_receive(
    tcp_stream: &mut TcpStream,
    msg_send: Message,
) -> anyhow::Result<Message> {
    let mut buffer = vec![0u8; 1024];

    tcp_stream.write_all(&msg_send.to_bytes()?).await?;

    let n = tcp_stream.read(&mut buffer[..]).await?;
    if n == 0 {
        tracing::error!("Failed to read from socket stream");
        anyhow::bail!("Failed to read from socket stream");
    }

    let msg_recv = Message::from_bytes(&buffer[..n])?;

    Ok(msg_recv)
}
