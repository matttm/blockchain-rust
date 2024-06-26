use chrono::Utc;
use log::{error, warn};
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::block::Block;
use crate::constants::DIFFICULTY_PREFIX;
use crate::utilities::{calculate_hash, hash_to_binary};

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
    pub fn add_block(&mut self, block: Block) {
        let latest_block = self.blocks.last().expect("Should have a block");
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
        if !hash_to_binary(&hex::decode(&block.hash).expect("")).starts_with(DIFFICULTY_PREFIX) {
            warn!("Error: Block has the wrong didficulty prefix");
            return false;
        }
        if block.id != previous_block.id + 1 {
            warn!("Error: new block is not previous block id plus one");
            return false;
        }
        if block.hash
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
        for i in 1..chain.len() {
            let prv = chain.get(i - 1).expect("");
            let cur = chain.get(i).expect("");
            if !State::is_block_valid(&cur, &prv) {
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
