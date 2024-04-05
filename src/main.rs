pub mod block;
pub mod constants;
pub mod p2p;
pub mod state;
pub mod utilities;

use crate::state::State;

use chrono::prelude::*;
use libp2p::{
    core::upgrade,
    futures::StreamExt,
    mplex,
    noise::{Keypair, NoiseConfig, X25519Spec},
    swarm::{Swarm, SwarmBuilder},
    tcp::TokioTcpConfig,
    Transport,
};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::Duration;
use tokio::{
    io::{stdin, AsyncBufReadExt, BufReader},
    select, spawn,
    sync::mpsc,
    time::sleep,
};

fn main() {
    log::info!("Initializing channels");
    log::info!("Peer id: {}" < p2p::PEER_ID.clone());
    let (response_sender, mut response_receiver) = mpsc::unbounded_channel();
    let (init_sender, mut init_receiver) = mpsc::unbounded_channel();

    let auth_keys = Keypair::new()
        .into_authentic(&p2p::KEYS)
        .expect("expect auth keys");

    // make tokio con fig

    let behavior = p2p::StateBehavior::new(State::new(), response_sender, init_sender).await();

    let transport = TokioTcpConfig::new()
        .upgrade(upgrade::Version::V1)
        .authenticate(NoiseConfig::xx(auth_keys).into_authenticated())
        .multiplex(mplex::MplexConfig::new())
        .boxed();


    // instantiate swarmbuilder
    let swarm = SwarmBuilder::new(transport, behavior, *p2p::PEER_ID)
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
        info!("sending init event");
        init_sender.send(true).expect("can send init event");
    });

    loop {
        let evt = {
            select! {
                line = stdin.next_line() => Some(p2p::EventType::Input(line)),
                _ = init_receiver.recv() => Some(),
                chain = response_receiver.recv() => {},
                event = swarm.select_next_some() => {
                    info!("Unhandled Swarm Event: {:?}", event);
                    None
                },
            }
        }
    }
}
