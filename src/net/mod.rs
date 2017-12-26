pub mod server;
pub mod nodes;
mod handlers;

// TODO rename network structs to NetTransaction, NetBlock...
#[derive(Serialize, Deserialize, RustcDecodable, FromSql, ToSql, Debug, Clone)]
#[postgres(name="tx")]
pub struct Transaction {
    id: String,
    sender_addr: String,
    sender_pubkey: String,
    receiver_addr: String,
    amount: i32, // u32
    timestamp: i64,
    signature: String,
}

#[derive(Serialize, Deserialize, RustcDecodable, Debug, Clone)]
pub struct Block {
    id: i32, // u32
    timestamp: i64,
    merkle_root: String, // Vec<u8>
    hash: String, // Vec<u8>
    nonce: i64, // u64
    transactions: Vec<Transaction>, // this Transaction, not the one in transactions.rs
}
