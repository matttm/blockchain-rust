pub mod block;
pub mod p2p;
pub mod state;

use std::time::Duration;

use crate::state::State;

use libp2p::{core::upgrade, futures::StreamExt, noise, swarm, tcp, yamux, Swarm, SwarmBuilder};
use log::{debug, error, info};
use tokio::{
    io::{stdin, AsyncBufReadExt, BufReader},
    select,
};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    info!("Initializing channels");
    // info!("Node id: {}", keys.clone());

    let mut state: State = State::new();
    state.create_genesis();

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

    loop {
        let evt = {
            select! {
                Ok(Some(line)) = stdin.next_line() => {
                    debug!("Constructing an input event");
                    Some(p2p::EventType::InputEvent(line))
                }
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
                Some(p2p::EventType::BlockAdditionEvent(block_addition)) => {
                    info!(
                        "received new block from {}",
                        block_addition.creator.to_string()
                    );
                    state.blocks.push(block_addition.block);
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
                        if !swarm.behaviour_mut().mdns.has_node(&peer_id) {
                            swarm
                                .behaviour_mut()
                                .floodsub
                                .remove_node_from_partial_view(&peer_id);
                        }
                    }
                }
                Some(p2p::EventType::IgnoreEvent) => (),
                None => (), // error!("Error occured"),
            }
        }
    }
}
