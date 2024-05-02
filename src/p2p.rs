use crate::block::Block;
use crate::State;

use libp2p::{
    core::{Endpoint, Multiaddr},
    floodsub::{
        protocol::{FloodsubProtocol, FloodsubRpc},
        Floodsub, FloodsubEvent, FloodsubMessage, Topic,
    },
    identity,
    swarm::{
        behaviour::FromSwarm, ConnectionDenied, ConnectionId, NetworkBehaviour, OneShotHandler,
        Swarm,
    },
    PeerId,
};

use log::{error, info};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::convert::From;
use std::task::{Context, Poll};
use tokio::sync::mpsc;

pub static KEYS: Lazy<identity::Keypair> = Lazy::new(identity::Keypair::generate_ed25519);
pub static PEER_ID: Lazy<PeerId> = Lazy::new(|| PeerId::from(KEYS.public()));
pub static CHAIN_TOPIC: Lazy<Topic> = Lazy::new(|| Topic::new("CHAIN"));
pub static BLOCK_TOPIC: Lazy<Topic> = Lazy::new(|| Topic::new("BLOCK"));

#[derive(Debug, Serialize, Deserialize)]
pub struct ChainResponse {
    pub blocks: Vec<Block>,
    pub receiver: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LocalChainRequest {
    pub from_peer_id: String,
}

pub enum EventType {
    LocalChainResponse(ChainResponse),
    Input(String),
    Init,
}

impl From<FloodsubEvent> for EventType {
    fn from(event: FloodsubEvent) -> Self {
        match event {
            FloodsubEvent::Message(msg) => {
                if let Ok(resp) = serde_json::from_slice::<ChainResponse>(&msg.data) {
                    if resp.receiver == PEER_ID.to_string() {
                        info!("Response from {}:", msg.source);
                        resp.blocks.iter().for_each(|r| info!("{:?}", r));

                        self.state.blocks =
                            State::choose_chain(self.state.blocks.clone(), resp.blocks);
                    }
                } else if let Ok(resp) = serde_json::from_slice::<LocalChainRequest>(&msg.data) {
                    info!("sending local chain to {}", msg.source.to_string());
                    let peer_id = resp.from_peer_id;
                    if PEER_ID.to_string() == peer_id {
                        if let Err(e) = self.response_sender.send(ChainResponse {
                            blocks: self.state.blocks.clone(),
                            receiver: msg.source.to_string(),
                        }) {
                            error!("error sending response via channel, {}", e);
                        }
                    }
                } else if let Ok(block) = serde_json::from_slice(&msg.data) {
                    info!("received new block from {}", msg.source.to_string());
                    self.state.add_block(block);
                }
            }
            _ => println!("Someone subscribed or unsubscribed"),
        }
    }
}

#[derive(NetworkBehaviour)]
//#[behaviour(event_process = true)]
#[behaviour(to_swarm = "EventType")]
pub struct StateBehavior {
    pub floodsub: Floodsub,
}

impl StateBehavior {
    pub async fn new() -> Self {
        let mut behavior = Self {
            floodsub: Floodsub::new(*PEER_ID),
        };
        behavior.floodsub.subscribe(CHAIN_TOPIC.clone());
        behavior.floodsub.subscribe(BLOCK_TOPIC.clone());
        behavior
    }
}
//

pub fn get_peer_list(swarm: &Swarm<StateBehavior>) -> Vec<String> {
    info!("Getting peer list...");
    let peers = swarm.connected_peers();
    let mut set = HashSet::new();
    for p in peers {
        set.insert(p);
    }
    set.iter().map(|p| p.to_string()).collect()
}
pub fn handle_cmd_print_peers(swarm: &Swarm<StateBehavior>) {
    let peers = get_peer_list(swarm);
    println!("Peer count: {}", peers.len());
    peers.iter().for_each(|p| println!("{}", p));
}

pub fn handle_cmd_print_chain(state: &State, swarm: &Swarm<StateBehavior>) {
    println!("{}", state);
}

pub fn handle_cmd_create_block(state: &mut State, swarm: &mut Swarm<StateBehavior>, cmd: &str) {
    if let Some(data) = cmd.strip_prefix("create b ") {
        let last = state.blocks.last().expect("Expect block");
        let block = Block::new(last.id + 1, last.hash.clone(), data.to_owned());
        let json = serde_json::to_string(&block).expect("can jsonify request");
        let behavior: &mut StateBehavior = swarm.behaviour_mut();
        state.blocks.push(block);
        println!("broadcasting new block");
        behavior
            .floodsub
            .publish(BLOCK_TOPIC.clone(), json.as_bytes());
    }
}
