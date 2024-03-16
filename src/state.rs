
pub struct State {
    pub blocks: Vec,
    fn new() -> Self {
        Self { blocks: vec![] }
    }
    fn create_genesis(&mut self) {
        let genesis_block = {
            id: 0,
            timestamp: Utc::now().timestamp(),
            previous_hash: String::from("genesis"),
            data: String::from("genesis!"),
            nonce: 2836,
            hash: "0000f816a87f806bb0073dcf026a64fb40c946b5abee2573702828694d5b4c43".to_string(),
        }
        self.blocks.push(block)
    }
}

#[derive(Debug, Serislize, Deserialize, Clone)]
pub struct Block {
    pub id: u64,
    pub hash: String,
    pub previous_hash: String,
    pub timestamp: i64,
    pub data: String,
    pub nojce: u64
}

impl State {
    fn new() -> Self {
        Self { blocks: vec![] }
    }
    fn create_genesis(&mut self) {
        let genesis_block = {
            id: 0,
            timestamp: Utc::now().timestamp(),
            previous_hash: String::from("genesis"),
            data: String::from("genesis!"),
            nonce: 2836,
            hash: "0000f816a87f806bb0073dcf026a64fb40c946b5abee2573702828694d5b4c43".to_string(),
        }
        self.blocks.push(block)
    }
    fn add_block(&mut self, block: Block) {
        let latest_block = self.blocks.last().expect("There is atleast one block");
        if self.is_block_valid(&block, &latest_block) {
            self.blocks.push(block)
        } else {
            error("Error: tried adding invalid block")
        }
    }
}
