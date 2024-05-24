pub mod block;
pub mod constants;
pub mod p2p;
pub mod state;
pub mod utilities;

use std::time::Duration;

use crate::state::State;

use libp2p::{core::upgrade, futures::StreamExt, noise, swarm, tcp, yamux, Swarm, SwarmBuilder};
use log::{debug, error, info};
use tokio::{
    io::{stdin, AsyncBufReadExt, BufReader},
    select, spawn,
    sync::mpsc,
    time::sleep,
};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    info!("Initializing channels");
    let (init_sender, mut init_receiver) = mpsc::unbounded_channel();

    let mut state: State = State::new();

    let mut swarm = SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )
        .expect("can change tcp")
        .with_behaviour(|keys| Ok(p2p::StateBehavior::new(keys)))
        .expect("added behavior")
        .with_swarm_config(|config| config.with_idle_connection_timeout(Duration::from_secs(60)))
        .build();
    //  read from stdin
    let mut stdin = BufReader::new(stdin()).lines();

    Swarm::listen_on(
        &mut swarm,
        "/ip4/0.0.0.0/tcp/0"
            .parse()
            .expect("can get a local socket"),
    )
    .expect("swarm can be started");

    spawn(async move {
        sleep(Duration::from_secs(1)).await;
        info!("sending init event");
        init_sender.send(true).expect("can send init event");
    });

    loop {
        let evt = {
            select! {
                Ok(Some(line)) = stdin.next_line() => {
                    debug!("Constructing an input event");
                    Some(p2p::EventType::InputEvent(line))
                }
                Some(_str) = init_receiver.recv() => {
                    debug!("Received Init event");
                    Some(p2p::EventType::InitEvent)
                },
                event_sw = swarm.select_next_some() => {
                    debug!("Received {:?} from swarm", event_sw);
                    if let swarm::SwarmEvent::Behaviour(event) = event_sw {
                        Some(event)
                    } else {
                        Some(p2p::EventType::IgnoreEvent)
                    }
                },
            }
        };
        // now do something with the evt
        if let Some(ref _trigger) = evt {
            match evt {
                Some(p2p::EventType::InitEvent) => {
                    info!("Received init event");
                    let peers = p2p::get_peer_list(&swarm);
                    state.create_genesis();

                    info!("connected nodes: {}", peers.len());
                    if !peers.is_empty() {
                        let req = p2p::ChainRequest {
                            from_peer_id: peers
                                .iter()
                                .last()
                                .expect("at least one peer")
                                .to_string(),
                        };
                        p2p::publish_event(&mut swarm, &p2p::CHAIN_TOPIC, req);
                    }
                }
                Some(p2p::EventType::ChainResponseEvent(res)) => {
                    state.blocks = State::choose_chain(state.blocks, res.blocks);
                }
                Some(p2p::EventType::InputEvent(line)) => {
                    info!("Received user input");
                    match line.as_str() {
                        "ls p" => p2p::handle_cmd_print_peers(&swarm),
                        "ls c" => p2p::handle_cmd_print_chain(&mut state, &swarm),
                        cmd if cmd.starts_with("create b") => {
                            p2p::handle_cmd_create_block(&mut state, &mut swarm, cmd)
                        }
                        _ => error!("unknown command"),
                    }
                }
                Some(p2p::EventType::ChainRequestEvent(req)) => {
                    let peer_id = req.from_peer_id;
                    info!("sending local chain to {}", peer_id);
                    let data = p2p::ChainResponse {
                        blocks: state.blocks.clone(),
                        receiver: peer_id,
                    };
                    p2p::publish_event(&mut swarm, &p2p::CHAIN_TOPIC, data);
                }
                Some(p2p::EventType::BlockAdditionEvent(block_addition)) => {
                    info!(
                        "received new block from {}",
                        block_addition.creator.to_string()
                    );
                    state.add_block(block_addition.block);
                }
                Some(p2p::EventType::DiscoveredEvent(peers)) => {
                    for (peer_id, _addr) in peers {
                        debug!("adding {peer_id} to floodsub");
                        swarm
                            .behaviour_mut()
                            .floodsub
                            .add_node_to_partial_view(peer_id);
                    }
                }
                Some(p2p::EventType::ExpiredEvent(peers)) => {
                    for (peer_id, _addr) in peers {
                        debug!("Rmoving {peer_id} from floodsub");
                        swarm
                            .behaviour_mut()
                            .floodsub
                            .remove_node_from_partial_view(&peer_id);
                    }
                }
                Some(p2p::EventType::IgnoreEvent) => (),
                None => error!("Error occured"),
            }
        }
    }
}
