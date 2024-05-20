pub mod block;
pub mod constants;
pub mod p2p;
pub mod state;
pub mod utilities;

use crate::state::State;

use libp2p::{
    core::upgrade, futures::StreamExt, identity::Keypair, noise, swarm, tcp, yamux, Swarm,
    Transport,
};
use log::{error, info};
use std::time::Duration;
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
    info!("Peer id: {}", p2p::PEER_ID.clone());
    let (init_sender, mut init_receiver) = mpsc::unbounded_channel();

    let auth_keys = Keypair::generate_ed25519();

    let noise = noise::Config::new(&auth_keys).unwrap();

    let mut state: State = State::new();

    let behavior = p2p::StateBehavior::new().await;

    let transport = tcp::tokio::Transport::new(tcp::Config::default())
        .upgrade(upgrade::Version::V1)
        .authenticate(noise)
        //  .apply()
        .multiplex(yamux::Config::default())
        .boxed();

    // instantiate swarmbuilder
    let mut swarm = Swarm::new(
        transport,
        behavior,
        *p2p::PEER_ID,
        swarm::Config::with_executor(Box::new(|fut| {
            spawn(fut);
        })),
    );

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
                    println!("Constructing an input event");
                    Some(p2p::EventType::InputEvent(line))
                }
                Some(_str) = init_receiver.recv() => {
                    println!("Received Init event");
                    Some(p2p::EventType::InitEvent)
                },
                swarm::SwarmEvent::Behaviour(event) = swarm.select_next_some() => {
                    info!("Received {:?} from swarm", event);
                    Some(event)
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
                    if p2p::PEER_ID.to_string() == peer_id {
                        let data = p2p::ChainResponse {
                            blocks: state.blocks.clone(),
                            receiver: peer_id,
                        };
                        p2p::publish_event(&mut swarm, &p2p::CHAIN_TOPIC, data);
                    }
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
                        swarm
                            .behaviour_mut()
                            .floodsub
                            .add_node_to_partial_view(peer_id);
                    }
                }
                Some(p2p::EventType::ExpiredEvent(peers)) => {
                    for (peer_id, _addr) in peers {
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
