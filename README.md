# BTC P2P

## Description
This is a simple lib for the Bitcoin network. It is used to establish a connection between two nodes and to exchange information about the current state of the node.

## Run
To run the handshake example, run the following command:

```bash
cargo run --example handshake
```

## Docker version
It is possible to run the handshake example in a docker container. To do so, run the following commands:

```bash
docker build -t btc_handshake:latest .
docker run --rm btc_handshake:latest
```

## Protocol
Implementation follows Bitcoin p2p networking specification at https://developer.bitcoin.org/reference/p2p_networking.html

### Messages
- Version message: Is sent by the initiator of the connection. It contains information about the node and its current state.
- Verack message: Is sent by the responder of the connection. It is a simple acknowledgement of the version message.

## Simple handshake

0. Lookup at DNS seeds for a list of nodes.
1. The initiator sends a version message to the nodes in the list.
2. The nodes respond with a valid version message.
3. The initiator sends a verack message to the nodes that responded to a valid version message.
4. After this point other messages can be exchange between the nodes.