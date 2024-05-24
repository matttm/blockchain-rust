use crate::block::Block;
use crate::State;

use libp2p::{
    core::Multiaddr,
    floodsub::{Floodsub, FloodsubEvent, Topic},
    identity::{self, Keypair},
    mdns,
    swarm::{NetworkBehaviour, Swarm},
    PeerId,
};

use log::{debug, error, info};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::convert::From;

pub static CHAIN_TOPIC: Lazy<Topic> = Lazy::new(|| Topic::new("CHAIN"));
pub static BLOCK_TOPIC: Lazy<Topic> = Lazy::new(|| Topic::new("BLOCK"));

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockAddition {
    pub creator: String,
    pub block: Block,
}

#[derive(Debug)]
pub enum EventType {
    BlockAdditionEvent(BlockAddition),
    // the following is never sent over the swarm (only used locally)
    InputEvent(String),
    // the following is never sent over the swarm (only used locally)
    DiscoveredEvent(Vec<(PeerId, Multiaddr)>),
    ExpiredEvent(Vec<(PeerId, Multiaddr)>),
    IgnoreEvent,
}

impl From<mdns::Event> for EventType {
    fn from(event: mdns::Event) -> Self {
        match event {
            mdns::Event::Discovered(peers) => {
                debug!("Received Discovered MDNS event");
                return EventType::DiscoveredEvent(peers);
            }
            mdns::Event::Expired(peers) => {
                debug!("Received Expired mDNS event");
                return EventType::ExpiredEvent(peers);
            }
        }
    }
}
impl From<FloodsubEvent> for EventType {
    fn from(event: FloodsubEvent) -> Self {
        debug!("Received FloodsubEvent");
        if let FloodsubEvent::Message(msg) = event {
            // let topic = String::from(msg.topics[0].id());
            let topic = msg.topics[0].id();
            let data = &msg.data;
            // TODO: REMOVE HARD CODE CASES
            info!("Event with topic {topic} received");
            match topic {
                "BLOCK" => {
                    if let Ok(block_addition) = serde_json::from_slice::<BlockAddition>(&data) {
                        return EventType::BlockAdditionEvent(block_addition);
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
    pub fn new(keys: &Keypair) -> Self {
        let mut behavior = Self {
            floodsub: Floodsub::new(keys.public().to_peer_id().clone()),
            mdns: mdns::tokio::Behaviour::new(
                mdns::Config::default(),
                keys.public().to_peer_id().clone(),
            )
            .expect("should create mdns"),
        };
        debug!("Subscribing to block topic");
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
        let json = serde_json::to_string(&block).expect("can jsonify request");
        let behavior: &mut StateBehavior = swarm.behaviour_mut();
        debug!("Sending payload: {}", &json);

        behavior
            .floodsub
            .publish(BLOCK_TOPIC.clone(), json.clone().into_bytes());
    }
}

pub fn publish_event(swarm: &mut Swarm<StateBehavior>, topic: &Topic, data: impl Serialize) {
    let json = serde_json::to_string(&data).expect("can jsonify request");
    let behavior: &mut StateBehavior = swarm.behaviour_mut();
    debug!("Sending payload: {}", &json);

    behavior
        .floodsub
        .publish(topic.clone(), json.clone().into_bytes());
}
