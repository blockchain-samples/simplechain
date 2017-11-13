use std::io::Read;
use std::thread;
use rocket;
use rocket_contrib::Json;
use hex::{FromHex, ToHex};
use base58::{FromBase58, ToBase58};
use reqwest;
use serde_json;
use rusqlite::Connection;

use errors::ServerError;
use transactions;
use wallet;

#[derive(Serialize, Deserialize, Debug)]
struct Transaction {
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

#[derive(Deserialize, Debug)]
struct Node {
    address: String,
    port: u32,
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

    let tx = transactions::new(
        &tx_json.id,
        &tx_json.sender_addr,
        &tx_json.sender_pubkey,
        &tx_json.receiver_addr,
        tx_json.amount,
        tx_json.timestamp,
        &tx_json.signature
    )?;
    println!("{:?}", transactions::verify(&tx));

    // if(transactions::verify(&tx)) {
    //     put code here after
    // }

    let nodes = get_nodes_from_server()?;
    save_nodes(&nodes);

    // spawn a thread to do not block the request
    thread::spawn(move || {
        // send transaction to known nodes
        for n in nodes {
            let url = format!("http://{}:{}/transaction", n.address, n.port);

            let client = reqwest::Client::new();

            match client.post(&url).json(&tx_json).send() {
                Ok(r) => {
                    println!("ok");
                },
                Err(e) => {
                    println!("{}", e);
                }
            }
        }
    });

    // save transaction in db
    transactions::store_db(&tx);

    Ok(())
}

#[post("/block", data="<block>")]
fn post_block(block: Json<Block>) {

}

fn get_nodes_from_server() -> Result<Vec<Node>, ServerError> {
    let count = 16;
    let url = format!("http://localhost:3000/nodes?count={}", count);
    let mut res = String::new();

    let nodes: Vec<Node> = reqwest::get(&url)?.json()?;

    Ok(nodes)
}

fn save_nodes(nodes: &Vec<Node>) -> Result<(), ServerError> {
    let conn = Connection::open("storage.db")?;

    // delete previous nodes in db
    conn.execute("DELETE FROM nodes", &[])?;

    // save nodes in db
    for n in nodes {
        conn.execute(
            "INSERT INTO nodes(address, port) VALUES(?1)",
            &[&n.address, &n.port]
        )?;
    }

    Ok(())
}

fn get_nodes_from_db() -> Result<Vec<Node>, ServerError> {
    let conn = Connection::open("storage.db")?;

    let mut stmt = conn.prepare("SELECT address, port FROM nodes")?;
    let mut nodes: Vec<Node> = Vec::new();

    let rows = stmt.query_map(&[], |row| {
        let address: String = row.get(0);
        let port: u32 = row.get(1);

        Node {
            address: address,
            port: port
        }
    })?;

    for n in rows {
        nodes.push(n?);
    }

    Ok(nodes)
}

pub fn start_node() {
    println!("STARTING NODE...");
    rocket::ignite().mount("/", routes![get_index, post_transaction]).launch();
}
