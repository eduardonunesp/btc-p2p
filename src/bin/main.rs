use std::{net::SocketAddr, time::SystemTime};

use anyhow::{bail, Result};
use btc_proto_handshake::proto;
use tokio::{
    io::AsyncReadExt,
    io::AsyncWriteExt,
    net::{lookup_host, TcpStream},
    sync::mpsc::{channel, Receiver},
};
use tracing_subscriber;

const BTC_SEED: &str = "seed.bitcoin.sipa.be";
const BTC_NODE_PORT: u16 = 8333;
const BUFFER_SIZE: usize = 1;

const PROTOCOL_VERSION: i32 = 70015;
const NODE_SERVICE: u64 = 0x01;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let (socket_chan_tx, mut socket_chan_rx) = channel::<SocketAddr>(BUFFER_SIZE);

    tracing::info!("Getting seed from{}", BTC_SEED);

    let addrs = lookup_host((BTC_SEED, BTC_NODE_PORT))
        .await?
        .collect::<Vec<_>>();

    tokio::spawn(async move {
        for addr in addrs {
            if let Err(err) = socket_chan_tx.send(addr).await {
                tracing::error!("Failed to send address to channel: {}", err);
            }
        }
    });

    tokio::spawn(async move {
        if let Err(err) = connect(&mut socket_chan_rx).await {
            tracing::error!("Failed to connect: {}", err);
        }
    });

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}

async fn connect(rx: &mut Receiver<SocketAddr>) -> Result<()> {
    while let Some(socket) = rx.recv().await {
        tokio::spawn(async move {
            if let Err(err) = handshake(socket).await {
                tracing::error!("Failed to handshake: {} with {}", err, socket);
            }
        });
    }

    Ok(())
}

async fn handshake(socket: SocketAddr) -> Result<()> {
    let mut tcp_stream = TcpStream::connect(socket).await?;

    tracing::info!("Sending version to {}", socket);

    let version_payload = proto::VersionPayload {
        version: PROTOCOL_VERSION,
        services: NODE_SERVICE,
        timestamp: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_secs() as i64,
        addr_recv_serv: NODE_SERVICE,
        addr_recv: match tcp_stream.local_addr()?.ip() {
            std::net::IpAddr::V4(x) => x.to_ipv6_mapped(),
            std::net::IpAddr::V6(x) => x,
        }
        .octets(),
        addr_recv_port: tcp_stream.local_addr()?.port(),
        addr_trans_serv: NODE_SERVICE,
        addr_trans: match socket.ip() {
            std::net::IpAddr::V4(x) => x.to_ipv6_mapped(),
            std::net::IpAddr::V6(x) => x,
        }
        .octets(),
        addr_trans_port: socket.port(),
        nonce: rand::random(),
        user_agent: "".to_string(),
        start_height: 0x00,
        relay: false,
    };

    let msg = proto::Message::new(proto::Payload::Version(version_payload));
    let msg_recv = send_and_receive(&mut tcp_stream, msg).await?;
    tracing::info!("Received version {:?} from {}", msg_recv.payload, socket);

    tracing::info!("Sending verack to {}", socket);
    let msg = proto::Message::new(proto::Payload::VerAck);
    let msg_recv = send_and_receive(&mut tcp_stream, msg).await?;
    tracing::info!("Received version {:?} from {}", msg_recv.payload, socket);

    Ok(())
}

async fn send_and_receive(
    tcp_stream: &mut TcpStream,
    msg_send: proto::Message,
) -> Result<proto::Message> {
    let mut buffer = vec![0u8; 1024];

    tcp_stream.write_all(&msg_send.to_bytes()?).await?;

    let n = tcp_stream.read(&mut buffer[..]).await?;
    if n == 0 {
        tracing::error!("Failed to read from socket stream");
        bail!("Failed to read from socket stream");
    }

    let msg_recv = proto::Message::from_bytes(&buffer[..n])?;

    Ok(msg_recv)
}
