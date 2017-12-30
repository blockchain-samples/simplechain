use bincode::{serialize, Infinite};
use sha2::{Sha256, Digest};
use hex::{FromHex, ToHex};
use rand::{self, Rng};

use errors::CoreError;
use blockchain;
use transactions::{self, Transaction};
use net::NetBlock;
use utils;

// FIXME bad: everything is public
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Header {
    pub id: i32,
    pub timestamp: i64,
    pub previous_hash: Vec<u8>,
    pub merkle_root: Vec<u8>,
}

// TODO implement previous_hash
// TODO maybe make this private and return a "web" Block (for easier JSON) instead of this struct
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Block {
    pub header: Header,
    pub hash: Vec<u8>, // XXX place this in Header?
    pub nonce: i64, // XXX place this in Header?
    pub transactions: Vec<Transaction>,
}

impl Header {
    pub fn from(
        id: i32,
        timestamp: i64,
        previous_hash: &String,
        merkle_root: &String
    ) -> Result<Header, CoreError> {
        let previous_hash: Vec<u8> = FromHex::from_hex(previous_hash)?;
        let merkle_root: Vec<u8> = FromHex::from_hex(merkle_root)?;

        Ok(Header {
            id: id,
            timestamp: timestamp,
            previous_hash: previous_hash,
            merkle_root: merkle_root
        })
    }
}

impl Block {
    pub fn from(
        id: i32,
        timestamp: i64,
        previous_hash: String,
        merkle_root: String,
        hash: String,
        nonce: i64,
        transactions: Vec<Transaction>
    ) -> Result<Block, CoreError> {
        let previous_hash: Vec<u8> = FromHex::from_hex(previous_hash)?;
        let merkle_root: Vec<u8> = FromHex::from_hex(merkle_root)?;
        let hash: Vec<u8> = FromHex::from_hex(hash)?;

        Ok(Block {
            header: Header {
                id: id,
                timestamp: timestamp,
                previous_hash: previous_hash,
                merkle_root: merkle_root
            },
            hash: hash,
            nonce: nonce,
            transactions: transactions
        })
    }
}

// TODO add transactions dynamically to the block as they come
// (recalculate merkle_root for every new transaction and try to mine the new merkle_root)
pub fn new() -> Result<(), CoreError> {
    println!("CREATE BLOCK");

    let id: i32 = blockchain::get_previous_id()? + 1;
    let timestamp: i64 = utils::get_current_timestamp();

    // get last cached transactions from database
    let mut transactions = transactions::read_db()?;

    // create coinbase transaction for reward
    let coinbase_transaction = transactions::coinbase()?;

    // insert coinbase transaction at begining of transactions
    transactions.insert(0, coinbase_transaction);

    // delete all cached transactions
    // transactions::clean_db() // XXX don't uncomment now because we retrieve from database again below

    // create a hash list with all tx ids
    let tx_hash_list: Vec<Vec<u8>> = transactions.into_iter()
        .map(|tx| tx.id)
        .collect();

    // get merkle root of all tx using the hash list
    let merkle_root: Vec<u8> = get_merkle_root(&tx_hash_list);

    // get previous block's hash to include in header
    let previous_hash: Vec<u8> = blockchain::get_previous_hash()?;

    println!("\nBLOCK INFOS\n------");
    println!("id: {}", id);
    println!("timestamp: {}", timestamp);
    println!("merkle_root: {}", merkle_root.to_hex());
    println!("previous_hash: {}\n", previous_hash.to_hex());

    let header: Header = Header {
        id: id,
        timestamp: timestamp,
        previous_hash: previous_hash,
        merkle_root: merkle_root,
    };

    let (hash, nonce) = mine(&header)?;

    // FIXME bad! we read database two times (should use previous transactions Vec)
    let transactions = transactions::read_db()?;

    let block: Block = Block {
        header: header,
        hash: hash,
        nonce: nonce,
        transactions: transactions // FIXME use the previous transactions Vec instead!
    };

    // create network block with block
    let net_block: NetBlock = NetBlock::from_block(block);

    blockchain::add_block(net_block);
    // store_db(&block)?;

    Ok(())
}

// mine a block with the block's header
fn mine(header: &Header) -> Result<(Vec<u8>, i64), CoreError> {
    println!("MINE BLOCK");

    // serialize the block header
    let header_encoded: Vec<u8> = serialize(header, Infinite)?;

    // hash the block header
    let mut hasher = Sha256::default();
    hasher.input(&header_encoded);
    let header_hashed: Vec<u8> = hasher.result().as_slice().to_vec();

    // make a proof of work using this hash
    Ok(proof_of_work(&header_hashed)?)
}

// TODO remake this with bytes not strings
// TODO spawn new thread for concurrent mining
fn proof_of_work(hash: &Vec<u8>) -> Result<(Vec<u8>, i64), CoreError> {
    println!("PROOF OF WORK...");

    let mut rng = rand::thread_rng(); // TODO check if we can reuse this (is it secure) or should we recreate one every time
    // XXX what if `nonce: i64` isn't big enough to hold the value that will allow to find the correct hash?
    let mut nonce: i64 = 0;
    let n: usize = 4; // this is basically the difficulty (n is bigger -> less probability to find a good hash)
    let mut hash_final = hash.clone().to_hex();

    // while the leading bytes aren't some 0s
    while &hash_final[..n] != &(0..n).map(|_| "0").collect::<String>() {
        // generate a new random nonce
        nonce = rng.gen::<i64>();
        // concat the hash and the nonce
        hash_final = format!("{}{}", hash.to_hex(), nonce);

        // hash the whole thing
        let mut hasher = Sha256::default();
        hasher.input(hash_final.as_bytes());
        hash_final = hasher.result().as_slice().to_hex();
    }

    println!("FOUND {}", hash_final);
    println!("WITH NONCE {}", nonce);

    // get a Vec<u8> from the hash hex string
    let hash_final: Vec<u8> = FromHex::from_hex(hash_final)?;

    Ok((hash_final, nonce))
}

// get the root hash of every transaction's hash using a merkle tree
fn get_merkle_root(hash_list: &Vec<Vec<u8>>) -> Vec<u8> {
    let hash_list_len = hash_list.len();

    if hash_list_len > 0 {
        if hash_list_len == 1 {
            // return the first element which is the merkle root
            return hash_list[0].clone();
        } else {
            let mut hash_list_computed: Vec<Vec<u8>> = Vec::new();

            // step in the hash list 2 by 2
            // XXX use this when `step_by()` becomes stable
            // for i in (0..hash_list_len - 1).step_by(2) {
            //
            // }

            let mut i = 0;
            while i < hash_list_len - 1 {
                // hash (n, n+1) together and push the result hash in a new hash list
                let mut hasher = Sha256::default();
                hasher.input(&[&hash_list[i][..], &hash_list[i+1][..]].concat());
                hash_list_computed.push(hasher.result().as_slice().to_vec());

                i += 2;
            }

            if hash_list_len % 2 != 0 {
                // if the hash list has an odd length, duplicate the last element to make it even
                hash_list_computed.push(
                    match hash_list.last() {
                        Some(tx) => {
                            let mut hasher = Sha256::default();
                            hasher.input(&[&tx[..], &tx[..]].concat());
                            hasher.result().as_slice().to_vec()
                        },
                        None => Vec::<u8>::new()
                    }
                );
            }

            // recall the function untill the hash list has a length of 1
            get_merkle_root(&hash_list_computed)
        }
    } else {
        return Vec::<u8>::new();
    }
}

// verify a block
pub fn verify(header: &Header, mined_hash: &Vec<u8>, nonce: i64) -> Result<bool, CoreError> {
    // serialize the block header
    let header_encoded: Vec<u8> = serialize(header, Infinite)?;

    // hash the block header
    let mut hasher = Sha256::default();
    hasher.input(&header_encoded);
    let header_hashed: Vec<u8> = hasher.result().as_slice().to_vec();

    // FIXME working with strings... should concat bytes directly
    let tested_payload = format!("{}{}", header_hashed.to_hex(), nonce);

    // hash the block header
    hasher = Sha256::default();
    hasher.input(&tested_payload.as_bytes());
    let hash: Vec<u8> = hasher.result().as_slice().to_vec();

    Ok(hash == *mined_hash)
}
