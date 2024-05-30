# blockchain-rust

Development Status: Inactive

The mvp was achieved, but a later addition is intended to integrate this with matttm/vote-chain

## Description

This is a small program written in Rust using libp2p to create a decentralized blockchain. Each node will use the multicast DNS (mDNS) protocol to automatically discover other nodes on the local network and the noise protocol to provide encryption. Floodsub is the main protocol being used for communication of the blockchain data itself.

## Getting Started

To best run this project. clone the code and open two terminals ;n the root of the project. The binary needs to be built and ran but with the environment variable set, as to get practical log information.
```
cargo build

❯ RUST_LOG=info ./target/debug/blockchain-rust                                                                                                                                
```
Each terminal this is ran in acts as a unique node and through mDNS, they will connect to eachother if on the same network.

### Commands

When running. of course you want to issue commands, so here is a list of commands.
```
ls c    -- print current chain

ls p   -- printr list of current connected peers

create b <data   --- send a block with payload 'data' to peers and add it to the local chain
```
## Authors

-   Matt Maloney : matttm

## Contribute

If you want to contribute, just send me a message.
