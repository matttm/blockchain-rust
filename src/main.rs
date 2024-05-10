pub mod block;
pub mod constants;
pub mod p2p;
pub mod state;
pub mod utilities;

use crate::p2p::EventType;
use crate::state::State;

use libp2p::{
    core::{transport, upgrade},
    futures::StreamExt,
    identity::{ed25519, Keypair},
    noise, swarm, tcp, yamux, Swarm, SwarmBuilder, Transport,
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
    let (response_sender, mut response_receiver) = mpsc::unbounded_channel();
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
        println!("sending init event");
        init_sender.send(true).expect("can send init event");
    });

    loop {
        let evt = {
            select! {
                line = stdin.next_line() => {
                    println!("Constructing an input event");
                    Some(p2p::EventType::Input(line.expect("Input exists").unwrap()))
                }
                _ = init_receiver.recv() => {
                    println!("Received Init event");
                    Some(p2p::EventType::Init)
                },
                event = swarm.select_next_some() => {
                    info!("Received event from zwarm");
                    None
                },
            }
        };
        // now do something with the evt
        if let Some(ref _trigger) = evt {
            match evt {
                Some(p2p::EventType::Init) => {
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

                        let json = serde_json::to_string(&req).expect("can jsonify request");
                        swarm
                            .behaviour_mut()
                            .floodsub
                            .publish(p2p::CHAIN_TOPIC.clone(), json.as_bytes());
                    }
                }
                Some(p2p::EventType::LocalChainResponse(res)) => {
                    let json = serde_json::to_string(&res).expect("can stringify response");
                    swarm
                        .behaviour_mut()
                        .floodsub
                        .publish(p2p::CHAIN_TOPIC.clone(), json.as_bytes());
                }
                Some(p2p::EventType::Input(line)) => {
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
                Some(p2p::EventType::LocalChainRequest(req)) => {
                    let peer_id = req.from_peer_id;
                    info!("sending local chain to {}", peer_id);
                    if p2p::PEER_ID.to_string() == peer_id {
                        if let Err(e) = behavior.floodsub.publish(ChainResponse {
                            blocks: state.blocks.clone(),
                            receiver: peer_id,
                        }) {
                            error!("error sending response via channel, {}", e);
                        }
                    }
                },
                Some(p2p::EventType::LocalBlockAddition(blockAddition)) => {
                    info!("received new block from {}", blockAddition.creator.to_string());
                    state.add_block(blockAddition.block);
                },
                Some(p2p::EventType::Ignore) => (),
                None => error!("Error occured"),
            }
        }
    }
}
