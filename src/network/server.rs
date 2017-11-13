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

use super::nodes;
use errors::ServerError;
use transactions;
use wallet;

#[derive(Serialize, Deserialize, Debug)]
pub struct Transaction {
    id: String,
    sender_addr: String,
    sender_pubkey: String,
    receiver_addr: String,
    amount: u32,
    timestamp: i64,
    signature: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Block {

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
    nodes::save_nodes(&nodes);

    // send transaction to known nodes
    nodes::send_transaction(tx_json);

    // save transaction in db
    tx.store_db();

    Ok(())
}

#[post("/block", data="<block>")]
fn post_block(block: Json<Block>) {

}

pub fn start() -> Result<(), ServerError> {
    println!("STARTING NODE...");

    // TODO read address and port in config.json or other file
    let config = Config::build(Environment::Staging)
        .address("10.0.0.1")
        .port(8000)
        .finalize()?;

    let server = rocket::custom(config, true);
    server.mount("/", routes![get_index, post_transaction]).launch();

    Ok(())
}
