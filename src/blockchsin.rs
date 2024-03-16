
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

pub struct Block {
    pub id: u64,
    pub hash: String,
    pub previous_hash: String,
    pub timestamp: i64,
    pub data: String,
    pub nojce: u64
}

