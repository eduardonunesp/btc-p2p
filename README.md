# BTC PROTO HANDSHAKE

## Description
This is a simple handshake protocol for the Bitcoin network. It is used to establish a connection between two nodes and to exchange information about the current state of the node.

## Run

```bash
cargo run
```

## Protocol
Implemented in the `protocol` module. From specification at https://developer.bitcoin.org/reference/p2p_networking.html

- Version message: Is sent by the initiator of the connection. It contains information about the node and its current state.
- Verack message: Is sent by the responder of the connection. It is a simple acknowledgement of the version message.

## Steps

0. Lookup at DNS seeds for a list of nodes.
1. The initiator sends a version message to the nodes in the list.
2. The nodes respond with a valid version message.
3. The initiator sends a verack message to the nodes that responded to a valid version message.
4. After this point other messages can be exchange between the nodes.


