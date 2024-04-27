use crate::block::Block;
use crate::State;

use libp2p::{
    floodsub::{Floodsub, FloodsubEvent, Topic},
    identity,
    swarm::NetworkBehaviour,
    swarm::Swarm,
    PeerId,
};

use log::{error, info};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
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

// impl From<Init> for StateBehavior {
//     fn from(event: Init) {}
// }

impl From<ChainResponse> for EventType {
    fn from(event: ChainResponse) -> Self {
        Self::LocalChainResponse(event)
    }
}

impl From<LocalChainRequest> for EventType {
    fn from(event: LocalChainRequest) -> Self {
        Self::
    }
}

#[derive(NetworkBehaviour)]
//#[behaviour(event_process = true)]
#[behaviour(to_swarm = "EventType")]
pub struct StateBehavior {
    pub floodsub: Floodsub,
    #[behaviour(ignore)]
    pub response_sender: mpsc::UnboundedSender<ChainResponse>,
    #[behaviour(ignore)]
    pub init_sender: mpsc::UnboundedSender<bool>,
    #[behaviour(ignore)]
    pub state: State,
}

impl StateBehavior {
    pub async fn new(
        state: State,
        response_sender: mpsc::UnboundedSender<ChainResponse>,
        init_sender: mpsc::UnboundedSender<bool>,
    ) -> Self {
        let mut behavior = Self {
            state,
            floodsub: Floodsub::new(*PEER_ID),
            response_sender,
            init_sender,
        };
        behavior.floodsub.subscribe(CHAIN_TOPIC.clone());
        behavior.floodsub.subscribe(BLOCK_TOPIC.clone());
        behavior
    }
}

pub fn get_peer_list(swarm: &Swarm<StateBehavior>) -> Vec<String> {
    info!("Getting peer list...");
    let peers = swarm.behaviour().mdns.discovered_nodes();
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

pub fn handle_cmd_print_chain(swarm: &Swarm<StateBehavior>) {
    let state = &swarm.behaviour().state;
    println!("{}", state);
}

pub fn handle_cmd_create_block(swarm: &mut Swarm<StateBehavior>, cmd: &str) {
    if let Some(data) = cmd.strip_prefix("create b ") {
        let last = swarm.behaviour().state.blocks.last().expect("Expect block");
        let block = Block::new(last.id + 1, last.hash.clone(), data.to_owned());
        let json = serde_json::to_string(&block).expect("can jsonify request");
        let behavior: &mut StateBehavior = swarm.behaviour_mut();
        behavior.state.blocks.push(block);
        println!("broadcasting new block");
        behavior
            .floodsub
            .publish(BLOCK_TOPIC.clone(), json.as_bytes());
    }
}
