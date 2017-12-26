use std::io::Read;
use std::thread;
use hex::{FromHex, ToHex};
use base58::{FromBase58, ToBase58};
use reqwest;
use serde_json;
use rusqlite::Connection;
use postgres_array::Array;
use postgres_derive;
use rouille::{input, Server, Request, Response};

use super::nodes;
use blockchain;
use blocks;
use errors::ServerError;
use transactions;
use wallet;

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

fn get_index(req: &Request) -> Result<Response, ServerError> {
    Ok(Response::text("Get /"))
}

fn post_transaction(req: &Request) -> Result<Response, ServerError> {
    let tx_body: Transaction = input::json_input(req)?;

    let tx = transactions::from(
        &tx_body.id,
        &tx_body.sender_addr,
        &tx_body.sender_pubkey,
        &tx_body.receiver_addr,
        tx_body.amount,
        tx_body.timestamp,
        &tx_body.signature
    )?;

    if(tx.verify()?) {
        // XXX
        let nodes = nodes::get_nodes_from_server()?;
        nodes::save_nodes(&nodes)?;

        let (private_key, public_key, address) = wallet::get_identity()?;
        println!("PKEY: {}", public_key.to_hex());
        println!("ADDR: {}", address.to_base58());

        let receiver_addr: Vec<u8> = "6X8BeC3UjgZR3XyB6vhGrA1JJbzmeroVtM6uvJdcJtDe".from_base58()?;

        let tx_tmp = transactions::new(
            private_key, public_key, address, receiver_addr, 100
        )?;

        // send transaction to known nodes
        nodes::send_transaction(tx_body)?;

        // save transaction in db
        tx.store_db()?;

        Ok(Response::text(""))
    } else {
        Err(ServerError::InvalidTransaction)
    }
}

fn post_block(req: &Request) -> Result<Response, ServerError> {
    let block: Block = input::json_input(req)?;
    let pool = blockchain::get_db_pool()?;

    let block_header = blocks::Header::from(
        block.id, block.timestamp, &block.merkle_root
    )?;

    let mined_hash: Vec<u8> = FromHex::from_hex(&block.hash)?;

    let verified = blocks::verify(&block_header, &mined_hash, block.nonce)?;

    if verified {
        let query = "INSERT INTO blocks(id, timestamp, merkle_root, hash, nonce, transactions)
            VALUES($1, $2, $3, $4, $5, $6)";

        thread::spawn(move || {
            let conn = pool.get().unwrap();
            match conn.execute(query, &[
                &block.id,
                &block.timestamp,
                &block.merkle_root,
                &block.hash,
                &block.nonce,
                &Array::from_vec(block.transactions, 0)
            ]) {
                Ok(_) => {},
                Err(e) => println!("{:?}", e) // FIXME do proper error handling
            }
        });

        Ok(Response::text(""))
    } else {
        Err(ServerError::InvalidBlock)
    }
}

// route incoming request to matching handler
fn route(req: &Request) -> Result<Response, ServerError> {
    router!(req,
        (GET) (/) => { get_index(req) },
        (POST) (/transaction) => { post_transaction(req) },
        (POST) (/block) => { post_block(req) },
        _ => Err(ServerError::NotFound) // Err(NotFound)
    )
}

// handle incoming request
fn handle(req: &Request) -> Response {
    println!("[+] {} {}", req.method(), req.raw_url());

    match route(req) {
        Ok(res) => res,
        Err(e) => {
            match e {
                ServerError::NotFound => {
                    Response::empty_404()
                },
                ServerError::InvalidTransaction => {
                    Response::empty_400()
                },
                ServerError::InvalidBlock => {
                    Response::empty_400()
                }
                _ => {
                    println!("error: {:?}", e);
                    Response::text("error")
                }
            }
        }
    }
}

// start the http server
pub fn start() {
    println!("STARTING NODE...");

    let server = Server::new("10.0.0.1:8000", |req| {
        handle(&req)
    }).unwrap();

    server.run();
}
