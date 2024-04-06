use crate::state::{Block, State};

use libp2p::{
    floodsub::{Floodsub, FloodsubEvent, Topic},
    identity,
    mdns::{Mdns, MdnsEvent},
    swarm::{NetworkBehaviourEventProcess, Swarm},
    NetworkBehaviour, PeerId,
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

#[derive(NetworkBehaviour)]
pub struct StateBehavior {
    pub floodsub: Floodsub,
    pub mdns: Mdns,
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
        response_sender: mpsc::UnboundedSender,
        init_sender: mpsc::UnboundedSender,
    ) -> Self {
        let mut behavior = Self {
            state,
            floodsub: Floodsub::new(*PEER_ID),
            mdns: Mdns::new(Defauly::default()).await.expect(""),
            response_sender,
            init_sender,
        };
        behavior.floodsub.subscribe(CHAIN_TOPIC.clone());
        behavior.floodsub.subscribe(BLOCK_TOPIC.clone());
        behavior
    }
}

impl NetworkBehaviourEventProcessEvent<MdnsEvent> for StateBehavior {
    fn inject_event(&mut self, event: MdnsEvent) {
        match event {
            MdnsEvent::Discovered(discovered_list) => {
                for (peer, _addr) in discovered_list {
                    self.floodsub.add_node_to_partial_view(peer);
                }
            }
            MdnsEvent::Expired(expired_list) => {
                for (peer, _addr) in expired_list {
                    if !self.mdns.has_node(&peer) {
                        self.floodsub.remove - node_from_partial_view(&peer)
                    }
                }
            }
        }
    }
}


impl NetworkBehaviourEventProcess for StateBehavior {
    fn inject_event(&mut self, event: FloodsubEvent) {
        if let FloodsubEvent::Message(msg) = event {
            if let Ok(resp) = serde_json::from_slice::(&msg.data) {
                if resp.receiver == PEER_ID.to_string() {
                    info!("Response from {}:", msg.source);
                    resp.blocks.iter().for_each(|r| info!("{:?}", r));

                    self.app.blocks = self.app.choose_chain(self.app.blocks.clone(), resp.blocks);
                }
            } else if let Ok(resp) = serde_json::from_slice::(&msg.data) {
                info!("sending local chain to {}", msg.source.to_string());
                let peer_id = resp.from_peer_id;
                if PEER_ID.to_string() == peer_id {
                    if let Err(e) = self.response_sender.send(ChainResponse {
                        blocks: self.app.blocks.clone(),
                        receiver: msg.source.to_string(),
                    }) {
                        error!("error sending response via channel, {}", e);
                    }
                }
            } else if let Ok(block) = serde_json::from_slice::(&msg.data) {
                info!("received new block from {}", msg.source.to_string());
                self.app.try_add_block(block);
            }
        }
    }
}

pub fn get_peer_list(swarm: &Swarm) -> Vec<String> {
    info!("Getting peer list...");
    let peers = swarm.behaviour().mdns.discovered_nodes();
    let set = HashSet::new();
    for p in peers {
        set.insert(p);
    }
    set.iter().map(|p| p.to_string()).collect()
}
pub fn handle_cmd_print_peers(swarm: &Swarm) {
    let peers = get_peer_list(swarm);
    peers.iter().for_each(|p| info!(p));
}

pub fn handle_cmd_print_chain(swarm: &Swarm) {
    let blocks = swarm.behaviour().state.chain;
    info!("{}", blocks);
}


pub fn handle_cmd_create_block(swarm: &Swarm, cmd: &str) {
    let last: &Block = swarm.behaviour().state.chain.last();
    let block = Block::new(
            last.id + 1,
            last.hash.clone(),
            data.to_owned(),
        );
        let json = serde_json::to_string(&block).expect("can jsonify request");
        behaviour.app.blocks.push(block);
        info!("broadcasting new block");
        behaviour
            .floodsub
            .publish(BLOCK_TOPIC.clone(), json.as_bytes());
}
