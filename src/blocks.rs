use bincode::{serialize, deserialize, Infinite};
use sha2::{Sha256, Digest};
use hex::{FromHex, ToHex};
use rand::{self, Rng};
use jfs::{Config, Store};

use errors::ServerError;
use transactions::{self, Transaction};
use utils;

// TODO maybe rename this to just "Header"
#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Header {
    index: u32,
    timestamp: i64,
    merkle_root: Vec<u8>,
}

// TODO implement previous_hash
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Block {
    header: Header,
    hash: Vec<u8>,
    nonce: u64,
    transactions: Vec<Transaction>
}

pub fn create() -> Result<(), ServerError> {
    println!("CREATE BLOCK");

    let index: u32 = 0;
    let timestamp: i64 = utils::get_current_timestamp();

    // get last cached transactions from database
    let transactions = transactions::read_db()?;

    // delete all cached transactions
    // transactions::clean_db() // XXX don't uncomment now because we retrieve from database again below

    // create a hash list with all tx ids
    let tx_hash_list: Vec<Vec<u8>> = transactions.into_iter()
        .map(|tx| tx.id)
        .collect();

    // get merkle root of all tx using the hash list
    let merkle_root = get_merkle_root(&tx_hash_list);

    // TODO create coinbase tx for the miner

    let header: Header = Header {
        index: index,
        timestamp: timestamp,
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

    store_db(&block);

    Ok(())
}

pub fn store_db(block: &Block) -> Result<(), ServerError> {
    println!("STORE BLOCK [DB]");

    let cfg = Config {
        pretty: false,
        indent: 2,
        single: true
    };
    let db = Store::new_with_cfg("blockchain", cfg)?;

    let id = db.save_with_id(block, &block.header.index.to_string())?;

    Ok(())

    // let ev = db.get::<Block>(&id).unwrap();
    // println!("{:?}", ev);
}

// mine a block with the block's header
fn mine(header: &Header) -> Result<(Vec<u8>, u64), ServerError> {
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

// TODO try to rewrite this with bytes, not strings
fn proof_of_work(hash: &Vec<u8>) -> Result<(Vec<u8>, u64), ServerError> {
    println!("PROOF OF WORK...");

    let mut rng = rand::thread_rng(); // TODO check if we can reuse this (is it secure) or should we recreate one every time
    let mut nonce: u64 = 0;
    let n: usize = 2;
    let mut hash_final = hash.clone().to_hex();

    // while the leading bytes aren't some 0s
    while &hash_final[..n] != &(0..n).map(|_| "0").collect::<String>() {
        // generate a new random nonce
        nonce = rng.gen::<u64>();
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
            for i in (0..hash_list_len - 1).step_by(2) {
                // hash (n, n+1) together and push the result hash in a new hash list
                let mut hasher = Sha256::default();
                hasher.input(&[&hash_list[i][..], &hash_list[i+1][..]].concat());
                hash_list_computed.push(hasher.result().as_slice().to_vec());
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
