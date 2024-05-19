use crate::block::Block;
use crate::State;

use libp2p::{
    core::{Endpoint, Multiaddr},
    floodsub::{
        protocol::{FloodsubProtocol, FloodsubRpc},
        Floodsub, FloodsubEvent, FloodsubMessage, Topic,
    },
    identity, mdns,
    swarm::{
        behaviour::FromSwarm, ConnectionDenied, ConnectionId, NetworkBehaviour, OneShotHandler,
        Swarm,
    },
    PeerId,
};

use log::{error, info};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::convert::From;
use std::task::{Context, Poll};
use std::{clone, collections::HashSet, fmt};
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
    pub creator: String,
    pub block: Block,
}

#[derive(Debug)]
pub enum EventType {
    BlockAdditionEvent(BlockAddition),
    ChainRequestEvent(ChainRequest),
    // the following can be used to send a response
    ChainResponseEvent(ChainResponse),
    // the following is never sent over the swarm (only used locally)
    InputEvent(String),
    // the following is never sent over the swarm (only used locally)
    InitEvent,
    DiscoveredEvent(Vec<(PeerId, Multiaddr)>),
    ExpiredEveent(Vec<(PeerId, Multiaddr)>),
    IgnoreEvent,
}

impl From<mdns::Event> for EventType {
    fn from(event: mdns::Event) -> Self {
        match event {
            mdns::Event::Discovered(peers) => {
                info!("Connecting to peer");
                return EventType::DiscoveredEvent(peers);
            }
            mdns::Event::Expired(peers) => {
                info!("A peer expired");
                return EventType::ExpiredEveent(peers);
            }
        }
    }
}
impl From<FloodsubEvent> for EventType {
    fn from(event: FloodsubEvent) -> Self {
        if let FloodsubEvent::Message(msg) = event {
            // let topic = String::from(msg.topics[0].id());
            let topic = msg.topics[0].id();
            let data = &msg.data;
            // TODO: REMOVE HARD CODE CASES
            info!("Event with topic {topic} received");
            match topic {
                "CHAIN" => {
                    if let Ok(chainMessage) = serde_json::from_slice::<ChainResponse>(&data) {
                        return EventType::ChainResponseEvent(chainMessage);
                    } else if let Ok(chainRequest) = serde_json::from_slice::<ChainRequest>(&data) {
                        return EventType::ChainRequestEvent(chainRequest);
                    }
                }
                "BLOCK" => {
                    if let Ok(blockAddition) = serde_json::from_slice::<BlockAddition>(&data) {
                        return EventType::BlockAdditionEvent(blockAddition);
                    }
                }
                _ => {
                    return EventType::IgnoreEvent;
                }
            }
        }
        EventType::IgnoreEvent
    }
}

#[derive(NetworkBehaviour)]
//#[behaviour(event_process = true)]
#[behaviour(to_swarm = "EventType")]
pub struct StateBehavior {
    pub floodsub: Floodsub,
    pub mdns: mdns::tokio::Behaviour,
}

impl StateBehavior {
    pub async fn new() -> Self {
        let mut behavior = Self {
            floodsub: Floodsub::new(*PEER_ID),
            mdns: mdns::tokio::Behaviour::new(mdns::Config::default(), PEER_ID.clone())
                .expect("should create mdns"),
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
        state.blocks.push(block.clone());
        info!("broadcasting new block");
        let event = BlockAddition {
            creator: PEER_ID.to_string(),
            block: block,
        };
        __publish_event(swarm, &BLOCK_TOPIC, event)
    }
}

pub fn __publish_event(swarm: &mut Swarm<StateBehavior>, topic: &Topic, data: impl Serialize) {
    let json = serde_json::to_string(&data).expect("can jsonify request");
    let behavior: &mut StateBehavior = swarm.behaviour_mut();

    behavior
        .floodsub
        .publish(topic.clone(), json.clone().into_bytes());
}
