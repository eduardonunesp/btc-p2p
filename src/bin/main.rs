#![allow(unused_imports, unused_variables, dead_code, unused_must_use)]

use std::{net::SocketAddr, time::SystemTime};

use anyhow::Result;
use btc_handshake::proto;
use tokio::{
    io::AsyncReadExt,
    io::AsyncWriteExt,
    net::{lookup_host, TcpStream},
    sync::mpsc::{channel, Receiver, Sender},
};
use tracing_subscriber;

const BTC_SEED: &str = "seed.bitcoin.sipa.be";
const BTC_NODE_PORT: u16 = 8333;
const BUFFER_SIZE: usize = 1;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let (tx, mut rx) = channel::<SocketAddr>(BUFFER_SIZE);

    tracing::info!("Getting seed from{}", BTC_SEED);

    let addrs = lookup_host((BTC_SEED, BTC_NODE_PORT))
        .await?
        .collect::<Vec<_>>();

    tokio::spawn(async move {
        for addr in addrs {
            if let Err(err) = tx.send(addr).await {
                tracing::error!("Failed to send address to channel: {}", err);
            }
        }
    });

    tokio::spawn(async move {
        if let Err(err) = connect(&mut rx).await {
            tracing::error!("Failed to connect: {}", err);
        }
    });

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}

async fn connect(rx: &mut Receiver<SocketAddr>) -> Result<()> {
    while let Some(addr) = rx.recv().await {
        tokio::spawn(async move {
            if let Err(err) = handshake(addr).await {
                tracing::error!("Failed to handshake: {}", err);
            }
        });
    }

    Ok(())
}

async fn handshake(socket: SocketAddr) -> Result<()> {
    let mut tcp_stream = TcpStream::connect(socket).await?;

    let mut buf = vec![0u8; 1024];
    let mut len = 0;

    tracing::info!("Connected to {}", socket);

    let version_payload = proto::VersionPayload {
        version: 70015,
        services: 0x01,
        timestamp: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_secs() as i64,
        addr_recv_serv: 0x01,
        addr_recv: match tcp_stream.local_addr()?.ip() {
            std::net::IpAddr::V4(x) => x.to_ipv6_mapped(),
            std::net::IpAddr::V6(x) => x,
        }
        .octets(),
        addr_recv_port: tcp_stream.local_addr()?.port(),
        addr_trans_serv: 0x01,
        addr_trans: match socket.ip() {
            std::net::IpAddr::V4(x) => x.to_ipv6_mapped(),
            std::net::IpAddr::V6(x) => x,
        }
        .octets(),
        addr_trans_port: socket.port(),
        nonce: 0xff,
        user_agent: "".to_string(),
        start_height: 0x00,
        relay: true,
    };

    let msg = proto::MessageHeader::new(
        proto::Command::Version,
        proto::Payload::Version(version_payload),
    );
    tcp_stream.write_all(&msg.to_bytes()?).await?;

    let n = tcp_stream.read(&mut buf[len..]).await?;
    if n == 0 {
        tracing::error!("Failed to read from socket stream");
    }

    let msg_recv = proto::MessageHeader::from_bytes(&buf[..n])?;
    tracing::info!("Received {:?}", msg_recv.payload);

    Ok(())
}
