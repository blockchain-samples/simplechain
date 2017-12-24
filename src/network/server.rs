use std::io::Read;
use std::thread;
use rocket;
use rocket::config::{Config, Environment};
use rocket_contrib::Json;
use hex::{FromHex, ToHex};
use base58::{FromBase58, ToBase58};
use reqwest;
use serde_json;
use rusqlite::Connection;
use postgres_array::Array;
use postgres_derive;

use super::nodes;
use blockchain;
use blocks;
use errors::ServerError;
use transactions;
use wallet;

#[derive(Serialize, Deserialize, FromSql, ToSql, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
pub struct Block {
    id: i32, // u32
    timestamp: i64,
    merkle_root: String, // Vec<u8>
    hash: String, // Vec<u8>
    nonce: i64, // u64
    transactions: Vec<Transaction>, // this Transaction, not the one in transactions.rs
}

// Get general infos about node
#[get("/")]
fn get_index() -> String {
    String::from("boo")
}

// Send a transaction to node
#[post("/transaction", data="<transaction>")]
fn post_transaction(transaction: Json<Transaction>) -> Result<(), ServerError> {
    let tx_json = transaction.into_inner();

    let tx = transactions::Transaction::from(
        &tx_json.id,
        &tx_json.sender_addr,
        &tx_json.sender_pubkey,
        &tx_json.receiver_addr,
        tx_json.amount,
        tx_json.timestamp,
        &tx_json.signature
    )?;
    println!("{:?}", tx.verify());

    // if(tx.verify()) {
    //     put code here after
    // }

    // XXX
    let nodes = nodes::get_nodes_from_server()?;
    nodes::save_nodes(&nodes)?;

    // send transaction to known nodes
    nodes::send_transaction(tx_json)?;

    // save transaction in db
    tx.store_db()?;

    Ok(())
}

#[post("/block", data="<block>")]
fn post_block(block: Json<Block>) -> Result<(), ServerError> {
    let block_json = block.into_inner();
    let pool = blockchain::get_db_pool()?;

    let block_header = blocks::Header::from(
        block_json.id, block_json.timestamp, &block_json.merkle_root
    )?;

    let mined_hash: Vec<u8> = FromHex::from_hex(&block_json.hash)?;

    let verified = blocks::verify(&block_header, &mined_hash, block_json.nonce)?;

    if verified {
        let query = "INSERT INTO blocks(id, timestamp, merkle_root, hash, nonce, transactions)
            VALUES($1, $2, $3, $4, $5, $6)";   
    
        thread::spawn(move || {
        let conn = pool.get().unwrap();
        match conn.execute(query, &[
            &block_json.id,
            &block_json.timestamp,
            &block_json.merkle_root,
            &block_json.hash,
            &block_json.nonce,
            &Array::from_vec(block_json.transactions, 0)
        ]) {
                Ok(_) => {},
                Err(e) => println!("{:?}", e) // FIXME do proper error handling
            }
        });
    } else {
        println!("Invalid block");
    }

    // thread::spawn(move || {
    //     let conn = pool.get().unwrap();
    //     match conn.execute(query, &[
    //         &block_json.id,
    //         &block_json.timestamp,
    //         &block_json.merkle_root,
    //         &block_json.hash,
    //         &block_json.nonce,
    //         &Array::from_vec(block_json.transactions, 0)
    //     ]) {
    //         Ok(_) => {},
    //         Err(e) => println!("{:?}", e) // FIXME do proper error handling
    //     }
    // });

    Ok(())
}

pub fn start() -> Result<(), ServerError> {
    println!("STARTING NODE...");

    // TODO read address and port in config.json or other file
    let config = Config::build(Environment::Staging)
        .address("10.0.0.1")
        .port(8000)
        .finalize()?;

    let server = rocket::custom(config, true);
    server.mount("/", routes![get_index, post_transaction, post_block]).launch();

    Ok(())
}
