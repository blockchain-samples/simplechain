#![feature(iterator_step_by, plugin, use_extern_macros)]
#![plugin(rocket_codegen)]

#[macro_use] extern crate serde_derive;
#[macro_use] extern crate postgres_derive;
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
extern crate postgres;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate postgres_array;

mod blockchain;
mod blocks;
mod errors;
mod network;
mod transactions;
mod utils;
mod wallet;

use hex::{FromHex, ToHex};
use base58::{FromBase58, ToBase58};

fn main() {
    // let (private_key, public_key, address) = wallet::get_identity()?;
    // println!("PKEY: {}", public_key.to_hex());
    // println!("ADDR: {}", address.to_base58());

    network::server::start();
}
