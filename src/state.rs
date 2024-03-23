use chrono::Utc;
use log::{error, warn};
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::constants::DIFFICULTY_PREFIX;
use crate::utilities::{calculate_hash, hash_to_binary};

pub struct State {
    pub blocks: Vec<Block>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub id: u64,
    pub hash: String,
    pub previous_hash: String,
    pub timestamp: i64,
    pub data: String,
    pub nonce: u64,
}

impl State {
    pub fn new() -> Self {
        Self { blocks: vec![] }
    }
    fn create_genesis(&mut self) {
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
    fn add_block(&mut self, block: Block) {
        let latest_block = self.blocks.last().expect("There is atleast one block");
        if State::is_block_valid(&block, &latest_block) {
            self.blocks.push(block)
        } else {
            panic!("Error: tried adding invalid block");
        }
    }
    fn is_block_valid(block: &Block, previous_block: &Block) -> bool {
        if block.previous_hash != previous_block.hash {
            warn!("Block with id {} is invalid due to prooperty previous_hash not matching previous block's hash", block.id);
            return false;
        }
        if !hash_to_binary(&hex::decode(block.hash).expect("")).starts_with(DIFFICULTY_PREFIX) {
            warn!("Error: Block has the wrong didficulty prefix");
            return false;
        }
        if block.id != previous_block.id + 1 {
            warn!("Error: new block is not previous block id plus one");
            return false;
        }
        if block.id
            != hex::encode(calculate_hash(
                block.id,
                block.timestamp,
                &block.previous_hash,
                &block.data,
                block.nonce,
            ))
        {
            warn!("Error");
            return false;
        }
        true
    }
    pub fn is_chain_valid(chain: &[Block]) -> bool {
        println!("Processing chain of length {}", chain.len());
        for i in 0..chain.len() {
            if i == 0 {
                continue;
            }
            let prv = chain.get(i - 1).expect("");
            let cur = chain.get(i).expect("");
            if State::is_block_valid(&cur, &prv) {
                warn!(
                    "Error: validation error occured on block with id {}",
                    cur.id
                );
                return false;
            }
        }
        true
    }
    pub fn choose_chain(local: Vec<Block>, remote: Vec<Block>) -> Vec<Block> {
        let is_local_valid = State::is_chain_valid(&local);
        let is_remote_valid = State::is_chain_valid(&remote);
        if is_remote_valid && is_local_valid {
            if local.len() > remote.len() {
                return local;
            } else {
                return remote;
            }
        }
        if is_remote_valid {
            return remote;
        }
        if is_local_valid {
            return local;
        }
        panic!("Error: no valid blockchsin to use");
    }
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "Start of chain <id>");
        for i in 0..self.blocks.len() {
            write!(
                f,
                "Block {} -- hash {}",
                i,
                self.blocks.get(i).unwrap().hash
            );
        }
        write!(f, "End of chain <id>")
    }
}
