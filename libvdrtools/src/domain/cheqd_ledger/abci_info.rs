#[derive(Serialize, Deserialize, Debug)]
pub struct ABCIInfo {
    pub response: Response,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    pub last_block_height: String,
}