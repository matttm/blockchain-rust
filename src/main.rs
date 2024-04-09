pub mod block;
pub mod constants;
pub mod p2p;
pub mod state;
pub mod utilities;

use crate::p2p::EventType;
use crate::state::State;

use libp2p::{
    core::upgrade,
    futures::StreamExt,
    mplex,
    noise::{Keypair, NoiseConfig, X25519Spec},
    swarm::{Swarm, SwarmBuilder},
    tcp::TokioTcpConfig,
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
    println!("Initializing channels");
    println!("Peer id: {}", p2p::PEER_ID.clone());
    let (response_sender, mut response_receiver) = mpsc::unbounded_channel();
    let (init_sender, mut init_receiver) = mpsc::unbounded_channel();

    let auth_keys = Keypair::<X25519Spec>::new()
        .into_authentic(&p2p::KEYS)
        .expect("expect auth keys");

    // make tokio con fig

    let behavior =
        p2p::StateBehavior::new(State::new(), response_sender, init_sender.clone()).await;

    let transport = TokioTcpConfig::new()
        .upgrade(upgrade::Version::V1)
        .authenticate(NoiseConfig::xx(auth_keys).into_authenticated())
        .multiplex(mplex::MplexConfig::new())
        .boxed();

    // instantiate swarmbuilder
    let mut swarm = SwarmBuilder::new(transport, behavior, *p2p::PEER_ID)
        .executor(Box::new(|fut| {
            spawn(fut);
        }))
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
                // chain = response_receiver.recv() => {
                //         Some(EventType::LocalChainResponse(chain.expect("Chain exists")))
                // },
                // event = swarm.select_next_some() => {
                //     info!("Unhandled Swarm Event: {:?}", event);
                //     None
                // },
            }
        };
        // now do something with the evt
        if let Some(ref _trigger) = evt {
            match evt {
                Some(p2p::EventType::Init) => {
                    println!("Received init event");
                    let peers = p2p::get_peer_list(&swarm);
                    swarm.behaviour_mut().state.create_genesis();

                    println!("connected nodes: {}", peers.len());
                    if !peers.is_empty() {
                        let req = p2p::LocalChainRequest {
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
                //     let json = serde_json::to_string(&res).expect("can stringify response");
                //     swarm
                //         .behaviour_mut()
                //         .floodsub
                //         .publish(p2p::CHAIN_TOPIC.clone(), json.as_bytes());
                }
                Some(p2p::EventType::Input(line)) => {
                    println!("Received user input");
                    match line.as_str() {
                        "ls p" => p2p::handle_cmd_print_peers(&swarm),
                        cmd if cmd.starts_with("ls c") => p2p::handle_cmd_print_chain(&swarm),
                        cmd if cmd.starts_with("create b") => {
                            p2p::handle_cmd_create_block(&mut swarm, cmd)
                        }
                        _ => error!("unknown command"),
                    }
                },
                None => error!("Error occured"),
            }
        }
    }
}
