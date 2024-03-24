mod state;

pub mod block;
pub mod constants;
pub mod utilities;

use crate::state::State;

fn main() {
    let mut state = State::new();
    print!("state: {}", state);
}
