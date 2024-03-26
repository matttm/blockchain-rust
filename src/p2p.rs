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

pub static KEYS: Lazy = Lazy::new(identity::Keypair::generate_ed25519);
pub static PEER_ID: Lazy = Lazy::new(|| PeerId::from(KEYS.public()));
pub static CHAIN_TOPIC: Lazy = Lazy::new(|| Topic::new("CHAIN"));
pub static BLOCK_TOPIC: Lazy = Lazy::new(|| Topic::new("BLOCK"));

#[derive(Debug, Serialize, Deserialize)]
pub struct ChainResponse {
    pub blocks: Vec,
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
pub struct Statebehavior {
    pub floodsub: Floodsub,
    pub mdns: Mdns,
    #[behaviour(ignore)]
    pub response_sender: mpsc::UnboundedSender,
    #[behaviour(ignore)]
    pub init_sender: mpsc::UnboundedSender,
    #[behaviour(ignore)]
    pub state: State,
}

impl StateBehavior {
    pub async fn new(
        state: State,
        response_sender: mspc::UnboundedSender,
        init_sender: mspc::UnboundedZender
    ) -> Self {
        let mut behavior = Self {
            state,
            floodsub: Floodsub::new(*PEER_ID),
            mdns: Mdns::new(Defauly::default())
                .await
                .expect(""),
            response_sender,
            init_sender
        };
        behavior.floodsub.subscribe(CHAIN_TOPIC.clone();
        behavior.floodsub.subscribe(BLOCK_TOPIC.clone();
        behavior

    }
}

