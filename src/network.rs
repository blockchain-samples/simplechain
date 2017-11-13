use std::io::Read;
use rocket;
use rocket_contrib::Json;
use hex::{FromHex, ToHex};
use base58::{FromBase58, ToBase58};
use reqwest;
use serde_json;
use rusqlite::Connection;

use transactions;
use wallet;

#[derive(Deserialize, Debug)]
struct Transaction {
    id: String,
    sender_addr: String,
    sender_pubkey: String,
    receiver_addr: String,
    amount: u32,
    timestamp: i64,
    signature: String,
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
fn post_index(transaction: Json<Transaction>) {
    let tx = transaction.into_inner();

    let tx = transactions::new(
        tx.id, tx.sender_addr, tx.sender_pubkey, tx.receiver_addr, tx.amount, tx.timestamp, tx.signature
    );

    println!("{}", transactions::verify(&tx));

    // if(transactions::verify(&tx)) {
    //     put code here after
    // }

    let nodes = get_nodes_from_server();
    save_nodes(nodes);

    // send transaction to known nodes
    // for n in nodes {
    //     let url = format!("http://{}:{}/transaction", n.address, n.port);
    //     let mut res = String::new();
    //
    //     reqwest::get(&url).unwrap()
    //         .read_to_string(&mut res)
    //         .unwrap();
    // }
}

fn get_nodes_from_server() -> Vec<Node> {
    let count = 16;
    let url = format!("http://localhost:3000/nodes?count={}", count);
    let mut res = String::new();

    let nodes: Vec<Node> = reqwest::get(&url).unwrap().json().unwrap();

    nodes
}

fn save_nodes(nodes: Vec<Node>) {
    let conn = Connection::open("storage.db").unwrap();

    // delete previous nodes in db
    conn.execute("DELETE FROM nodes", &[]).unwrap();

    // save nodes in db
    for n in &nodes {
        conn.execute(
            "INSERT INTO nodes(address, port) VALUES(?1, ?2)",
            &[&n.address, &n.port]
        ).unwrap();
    }
}

fn get_nodes_from_db() -> Vec<Node> {
    let conn = Connection::open("storage.db").unwrap();

    let mut stmt = conn.prepare("SELECT address, port FROM nodes").unwrap();
    let mut nodes: Vec<Node> = Vec::new();

    let rows = stmt.query_map(&[], |row| {
        let address: String = row.get(0);
        let port: u32 = row.get(1);

        Node {
            address: address,
            port: port
        }
    }).unwrap();

    for n in rows {
        nodes.push(n.unwrap());
    }

    nodes
}

pub fn start_node() {
    println!("STARTING NODE...");
    rocket::ignite().mount("/", routes![get_index, post_index]).launch();
}
