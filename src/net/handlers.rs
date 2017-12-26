use std::thread;
use hex::FromHex;
use rouille::{input, Request, Response};
use postgres_array::Array;

use super::nodes;
use errors::ServerError;
use transactions;
use blockchain;
use blocks;

pub fn get_index(req: &Request) -> Result<Response, ServerError> {
    Ok(Response::text("Get /"))
}

pub fn post_transaction(req: &Request) -> Result<Response, ServerError> {
    let tx_body: super::Transaction = input::json_input(req)?;

    let tx = transactions::from(
        &tx_body.id,
        &tx_body.sender_addr,
        &tx_body.sender_pubkey,
        &tx_body.receiver_addr,
        tx_body.amount,
        tx_body.timestamp,
        &tx_body.signature
    )?;

    // check if transaction is valid
    if tx.verify()? {
        // XXX
        let nodes = nodes::get_nodes_from_server()?;
        nodes::save_nodes(&nodes)?;

        // let (private_key, public_key, address) = wallet::get_identity()?;
        // println!("PKEY: {}", public_key.to_hex());
        // println!("ADDR: {}", address.to_base58());
        //
        // let receiver_addr: Vec<u8> = "6X8BeC3UjgZR3XyB6vhGrA1JJbzmeroVtM6uvJdcJtDe".from_base58()?;
        //
        // let tx_tmp = transactions::new(
        //     private_key, public_key, address, receiver_addr, 100
        // )?;

        // send transaction to known nodes
        nodes::send_transaction(tx_body)?;

        // save transaction in db
        tx.store_db()?;

        Ok(Response::text(""))
    } else {
        Err(ServerError::InvalidTransaction)
    }
}

pub fn post_block(req: &Request) -> Result<Response, ServerError> {
    let block: super::Block = input::json_input(req)?;
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
