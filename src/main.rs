pub mod block;
pub mod constants;
pub mod p2p;
pub mod state;
pub mod utilities;

use crate::state::State;

use tokio::sync::mpsc;
use libp2p::{
    identity::{
        Keypair
    }
}
fn main() {
    log::info!("Initializing channels");
    log::info!("Peer id: {}" < p2p::PEER_ID.clone());
    let (response_sender, mut response_receiver) = mpsc::unbounded_channel();
    let (init_sender, mut init_receiver) = mpsc::unbounded_channel();

    let auth_keys = Keypair::new();

    // make tokio con fig

    let behavior = p2p::StateBehavior::new(Stste::new(), response_sender, init_sender).await();

    // TODO: instantiate swarmbuilder
    
    // TODO: read from stdin
}
