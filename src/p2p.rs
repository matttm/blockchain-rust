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
use std::{clone, collections::HashSet};
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
pub struct ChainRequest {
    pub from_peer_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockAddition {
    pub block: Block,
}

pub enum EventType {
    LocalBlockAddition(BlockAddition),
    LocalChainRequest(ChainRequest),
    // the following can be used to send a response
    LocalChainResponse(ChainResponse),
    // the following is never sent over the swarm (only used locally)
    Input(String),
    // the following is never sent over the swarm (only used locally)
    Init,
    Ignore
}

impl From<FloodsubEvent> for EventType {
    fn from(event: FloodsubEvent) -> Self {
        if let FloodsubEvent::Message(msg) = event {
            // let topic = String::from(msg.topics[0].id());
            let topic = msg.topics[0].id();
            let data = &msg.data;
            // TODO: REMOVE HARD CODE
            info!("Event with topic {topic} received");
            match topic {
                "CHAIN" => {
                    if let Ok(chainMessage) = serde_json::from_slice::<ChainResponse>(&data) {
                        return EventType::LocalChainResponse(chainMessage);
                    } else if let Ok(chainRequest) = serde_json::from_slice::<ChainRequest>(&data) {
                        return EventType::LocalChainRequest(chainRequest);
                    }
                },
                "BLOCK" => {
                    return EventType::Ignore;
                }
            }
        }
        EventType::Ignore
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
    info!("Peer count: {}", peers.len());
    peers.iter().for_each(|p| println!("{}", p));
}

pub fn handle_cmd_print_chain(state: &State, swarm: &Swarm<StateBehavior>) {
    info!("{}", state);
}

pub fn handle_cmd_create_block(state: &mut State, swarm: &mut Swarm<StateBehavior>, cmd: &str) {
    if let Some(data) = cmd.strip_prefix("create b ") {
        let last = state.blocks.last().expect("Expect block");
        let block = Block::new(last.id + 1, last.hash.clone(), data.to_owned());
        let json = serde_json::to_string(&block).expect("can jsonify request");
        let behavior: &mut StateBehavior = swarm.behaviour_mut();
        state.blocks.push(block);
        info!("broadcasting new block");
        behavior
            .floodsub
            .publish(BLOCK_TOPIC.clone(), json.as_bytes());
    }
}
