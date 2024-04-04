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
    // TODO: read from stdin
    let mut stdin = BufReader::new(stdin()).lines();
}
