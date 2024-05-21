use chrono::Utc;
use std::fmt;

use crate::block::Block;

pub struct State {
    pub blocks: Vec<Block>,
}

impl State {
    pub fn new() -> Self {
        Self { blocks: vec![] }
    }
    pub fn create_genesis(&mut self) {
        println!("Creating genesis block");
        let genesis_block = Block {
            id: 0,
            timestamp: Utc::now().timestamp(),
            previous_hash: String::from("genesis"),
            data: String::from("genesis!"),
            nonce: 2836,
            hash: "0000f816a87f806bb0073dcf026a64fb40c946b5abee2573702828694d5b4c43".to_string(),
        };
        self.blocks.push(genesis_block)
    }
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), std::fmt::Error> {
        let mut result = String::new();
        result.push_str("Start of chain\n");
        for _i in 0..self.blocks.len() {
            let block = self.blocks.get(_i).unwrap();
            result = format!(
                "{pre}Block {id} -- hash {hash}\n",
                pre = result,
                id = block.id,
                hash = block.hash
            );
        }
        result.push_str("End of chain ");
        write!(f, "{}", result)
    }
}
