mod state;

use crate::state::State;

fn main() {
    let mut state = State::new();
    print!("state: {}", state);
}
