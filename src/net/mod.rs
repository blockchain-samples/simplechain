pub mod server;
pub mod nodes;
mod handlers;

#[derive(Serialize, Deserialize, RustcDecodable, FromSql, ToSql, Debug, Clone)]
#[postgres(name="tx")]
pub struct NetTransaction {
    pub id: String,
    pub sender_addr: String,
    pub sender_pubkey: String,
    pub receiver_addr: String,
    pub amount: i32, // u32
    pub timestamp: i64,
    pub signature: String,
}

#[derive(Serialize, Deserialize, RustcDecodable, Debug, Clone)]
pub struct NetBlock {
    id: i32, // u32
    timestamp: i64,
    merkle_root: String, // Vec<u8>
    hash: String, // Vec<u8>
    nonce: i64, // u64
    transactions: Vec<NetTransaction>,
}
