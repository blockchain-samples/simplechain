pub mod server;
pub mod nodes;
mod handlers;

use hex::ToHex;
use base58::ToBase58;
use blocks::Block;

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
    pub id: i32, // u32
    pub timestamp: i64,
    pub previous_hash: String,
    pub merkle_root: String, // Vec<u8>
    pub hash: String, // Vec<u8>
    pub nonce: i64, // u64
    pub transactions: Vec<NetTransaction>,
}

#[derive(Debug, Serialize, Deserialize, RustcEncodable, Clone)]
pub struct NetKeyPair {
    pub private_key: String,
    pub public_key: String,
}

#[derive(RustcEncodable)]
pub struct NetWallet {
    pub keypair: NetKeyPair,
    pub address: String,
}

impl NetBlock {
    pub fn from_block(block: Block) -> NetBlock {
        let previous_hash = block.header.previous_hash.to_hex();
        let merkle_root = block.header.merkle_root.to_hex();
        let hash = block.hash.to_hex();

        // XXX converting every Transaction to NetTransaction seem to be overkill
        // maybe find a better solution that requires less iterations?
        let net_txs: Vec<NetTransaction> = block.transactions.into_iter().map(|tx| {
            let id = tx.id.to_hex();
            let sender_addr = tx.transaction.content.sender_addr.to_base58();
            let sender_pubkey = tx.transaction.content.sender_pubkey.to_hex();
            let receiver_addr = tx.transaction.content.receiver_addr.to_base58();
            let signature = tx.transaction.signature.to_hex();

            NetTransaction {
                id: id,
                sender_addr: sender_addr,
                sender_pubkey: sender_pubkey,
                receiver_addr: receiver_addr,
                amount: tx.transaction.content.amount,
                timestamp: tx.transaction.content.timestamp,
                signature: signature
            }
        }).collect();

        NetBlock {
            id: block.header.id,
            timestamp: block.header.timestamp,
            previous_hash: previous_hash,
            merkle_root: merkle_root,
            hash: hash,
            nonce: block.nonce,
            transactions: net_txs
        }
    }
}
