use std::thread;
use reqwest;
use rusqlite::Connection;

use super::server::Transaction;
use errors::CoreError;

#[derive(Deserialize, Debug)]
pub struct Node {
    pub address: String,
    pub port: u32,
}

// impl Node {
//     fn new(address: String, port: u32) -> Node {
//         Node {
//             address: address,
//             port: port
//         }
//     }
// }

pub fn get_nodes_from_server() -> Result<Vec<Node>, CoreError> {
    let count = 16;
    let url = format!("http://localhost:3000/nodes?count={}", count);

    let nodes: Vec<Node> = reqwest::get(&url)?.json()?;

    Ok(nodes)
}

pub fn save_nodes(nodes: &Vec<Node>) -> Result<(), CoreError> {
    let conn = Connection::open("db/storage.db")?;

    // delete previous nodes in db
    conn.execute("DELETE FROM nodes", &[])?;

    // save nodes in db
    for n in nodes {
        conn.execute(
            "INSERT INTO nodes(address, port) VALUES(?1, ?2)",
            &[&n.address, &n.port]
        )?;
    }

    Ok(())
}

fn get_nodes_from_db() -> Result<Vec<Node>, CoreError> {
    let conn = Connection::open("db/storage.db")?;

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

pub fn send_transaction(tx: Transaction) -> Result<(), CoreError> {
    let nodes = get_nodes_from_db()?;

    // spawn a thread to do not block the request
    thread::spawn(move || {
        // send transaction to known nodes
        for n in nodes {
            let url = format!("http://{}:{}/transaction", n.address, n.port);
            let client = reqwest::Client::new();

            match client.post(&url).json(&tx).send() {
                Ok(r) => {
                    println!("ok");
                },
                Err(e) => {
                    println!("{}", e);
                }
            }
        }
    });

    Ok(())
}
