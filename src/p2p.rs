use crate::block::Block;
use crate::State;

use libp2p::{
    core::{Endpoint, Multiaddr},
    floodsub::{
        protocol::{FloodsubProtocol, FloodsubRpc},
        Floodsub, FloodsubEvent, Topic,
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
}

// impl From<Init> for StateBehavior {
//     fn from(event: Init) {}
// }

impl From<ChainResponse> for EventType {
    fn from(event: ChainResponse) -> Self {
        Self::LocalChainResponse(event)
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

impl NetworkBehaviour for StateBehavior {
    type ConnectionHandler = OneShotHandler<FloodsubProtocol, FloodsubRpc, FloodsubEvent>;
    type ToSwarm = EventType;
    // Required methods
    fn handle_established_inbound_connection(
        &mut self,
        _connection_id: ConnectionId,
        peer: PeerId,
        local_addr: &Multiaddr,
        remote_addr: &Multiaddr,
    ) -> Result<Self::ConnectionHandler, ConnectionDenied> {
        self.floodsub.handle_established_inbound_connection(
            _connection_id,
            peer,
            local_addr,
            remote_addr,
        )
    }
    fn handle_established_outbound_connection(
        &mut self,
        _connection_id: ConnectionId,
        peer: PeerId,
        addr: &Multiaddr,
        role_override: Endpoint,
    ) -> Result<Self::ConnectionHandler, ConnectionDenied> {
        self.floodsub.handle_established_outbound_connection(
            _connection_id,
            peer,
            addr,
            role_override,
        )
    }
    fn on_swarm_event(&mut self, event: FromSwarm<'_>) {
        self.floodsub.on_swarm_event(event)
    }
    fn on_connection_handler_event(
        &mut self,
        _peer_id: PeerId,
        _connection_id: ConnectionId,
        _event: <Self::ConnectionHandler as ConnectionHandler>::ToBehaviour,
    ) {
        self.floodsub
            .on_connection_handler_event(_peer_id, _connection_id, _event)
    }
    fn poll(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<ToSwarm<Self::ToSwarm, <Self::ConnectionHandler as ConnectionHandler>::FromBehaviour>>
    {
        self.floodsub.poll(cx)
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
