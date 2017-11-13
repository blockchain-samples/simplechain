#![feature(iterator_step_by, plugin)]
#![plugin(rocket_codegen)]

#[macro_use] extern crate serde_derive;
#[macro_use] extern crate rocket_contrib;
extern crate rocket;
extern crate serde_bytes;
extern crate serde_json;
extern crate serde;
extern crate bincode;
extern crate time;
extern crate secp256k1;
extern crate base58;
extern crate sha2;
extern crate rusqlite;
extern crate hex;
extern crate rand;
extern crate jfs;
extern crate reqwest;

mod blocks;
mod network;
mod transactions;
mod utils;
mod wallet;

use std::net::{TcpListener, TcpStream};

use std::io::prelude::*;
use bincode::{serialize, deserialize, Infinite};

use hex::{FromHex, ToHex};
use base58::{FromBase58, ToBase58};

// #[derive(Serialize, Deserialize, PartialEq, Debug)]
// struct Header {
//     length: usize,
//     timestamp: i64,
//     command: Vec<u8>,
// }
//
// #[derive(Serialize, Deserialize, PartialEq, Debug)]
// struct Packet {
//     header: Header,
//     data: Vec<u8>,
// }
//
// fn handle_peer(mut stream: TcpStream) {
//     println!("[+] Node connected");
//
//     let mut buffer = vec![0; 512];
//     let _ = stream.read(&mut buffer);
//
//     println!("{:?}", buffer);
//
//     let packet: Packet = deserialize(&buffer[..]).unwrap();
//
//     println!("PACKET SIZE: {}", packet.header.length);
//
//     match String::from_utf8(packet.header.command).unwrap().as_ref() {
//         "send(transaction)" => {
//             println!("RECEIVED TRANSACTION");
//             let tx = transactions::create_from_bytes(&packet.data);
//
//             if transactions::verify(&tx) {
//                 println!("[TRANSACTION VALID]");
//                 // transactions::store_db(&tx);
//                 // transactions::clean_db();
//                 let block = blocks::create();
//
//                 // TODO send "ok" back to sender
//                 let _ = stream.write(String::from("OK").as_bytes());
//             }
//         },
//         "send(block)" => {
//             println!("RECEIVED BLOCK");
//         },
//         _ => println!("UNKNOWN COMMAND"),
//     };
// }

fn main() {
    let (private_key, public_key, address) = wallet::get_identity();
    println!("PKEY: {}", public_key.to_hex());
    println!("ADDR: {}", address.to_base58());

    network::start_node();
}
