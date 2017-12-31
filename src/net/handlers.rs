use std::thread;
use hex::FromHex;
use rouille::{input, Request, Response};

use super::{nodes, NetTransaction, NetBlock};
use errors::ServerError;
use transactions;
use blockchain;
use blocks;

pub fn get_index(req: &Request) -> Result<Response, ServerError> {
    Ok(Response::text("Get /"))
}

pub fn post_transaction(req: &Request) -> Result<Response, ServerError> {
    let tx_body: NetTransaction = input::json_input(req)?;

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
    // TODO check if sender is allowed to send that amount
    // TODO check if transaction is not already on blockchain
    if tx.is_valid()? {
        // let nodes = nodes::get_nodes_from_server()?;
        // nodes::save_nodes(&nodes)?;

        blockchain::scan()?;

        // send transaction to known nodes
        // nodes::send_transaction(tx_body)?;

        // save transaction in db
        // tx.store_db()?;

        // create a new block with the new transaction
        // TODO use threads (safely)
        // kill previous thread if we respawn one to recreate a block, otherwise the user will keep mining old block
        blocks::new()?;

        Ok(Response::text(""))
    } else {
        Err(ServerError::InvalidTransaction)
    }
}

pub fn post_block(req: &Request) -> Result<Response, ServerError> {
    let block: NetBlock = input::json_input(req)?;

    let block_header = blocks::Header::from(
        block.id, block.timestamp, &block.previous_hash, &block.merkle_root
    )?;

    let mined_hash: Vec<u8> = FromHex::from_hex(&block.hash)?;

    let verified = blocks::verify(&block_header, &mined_hash, block.nonce)?;

    if verified {
        // XXX is this safe?
        thread::spawn(move || {
            blockchain::add_block(block); // can't use ? here
        });

        Ok(Response::text(""))
    } else {
        Err(ServerError::InvalidBlock)
    }
}

// local handlers (only accessible locally)
// provides an interface for the user to easily create new transactions, new wallets, etc.
pub mod local {
    use std::ops::Index;
    use base58::{FromBase58, ToBase58};
    use hex::ToHex;
    use rouille::{input, Request, Response};
    use jfs;

    use super::nodes;
    use net::{NetKeyPair, NetWallet};
    use errors::ServerError;
    use transactions;
    use wallet;

    pub fn get_new_wallet(req: &Request) -> Result<Response, ServerError> {
        // TODO handle the fact that the user calls this by mistake (his previous wallet will be lost)
        let wallet = wallet::get_new_wallet()?;

        let private_key: String = wallet.keypair.private_key.index(..).to_hex();
        let public_key: String = wallet.keypair.public_key.to_hex();
        let address: String = wallet.address.to_base58();

        let net_keypair = NetKeyPair {
            private_key: private_key,
            public_key: public_key
        };

        let net_wallet = NetWallet {
            keypair: net_keypair.clone(),
            address: address
        };

        // TODO FIXME do storage in `wallet.rs` at wallet creation, not here
        let cfg = jfs::Config {
            pretty: true,
            indent: 4,
            single: true
        };
        // TODO handle user moving the file
        // TODO check if folder `storage` exists or create it before
        let storage = jfs::Store::new_with_cfg("storage/wallet", cfg).unwrap();
        storage.save_with_id(&net_keypair, &net_wallet.address).unwrap();

        Ok(Response::json(&net_wallet))
    }

    pub fn get_wallet(req: &Request, address: String) -> Result<Response, ServerError> {
        let wallet = wallet::get_wallet(&address)?;

        let private_key: String = wallet.keypair.private_key.index(..).to_hex();
        let public_key: String = wallet.keypair.public_key.to_hex();

        let net_keypair = NetKeyPair {
            private_key: private_key,
            public_key: public_key
        };

        let net_wallet = NetWallet {
            keypair: net_keypair,
            address: address
        };

        Ok(Response::json(&net_wallet))
    }

    #[derive(Debug, RustcDecodable)]
    struct Transaction {
        sender_addr: String,
        receiver_addr: String,
        amount: i32,
    }

    pub fn post_transaction(req: &Request) -> Result<Response, ServerError> {
        let tx_body: Transaction = input::json_input(req)?;

        // get wallet associated with given address from storage
        let wallet = wallet::get_wallet(&tx_body.sender_addr)?;

        // deserialize addresses
        let sender_addr_bytes: Vec<u8> = tx_body.sender_addr.from_base58()?;
        let receiver_addr_bytes: Vec<u8> = tx_body.receiver_addr.from_base58()?;

        // create transaction for signature
        let net_tx = transactions::new(wallet.keypair.private_key, wallet.keypair.public_key, sender_addr_bytes, receiver_addr_bytes, tx_body.amount)?;

        // broadcast transaction to network
        nodes::send_transaction(net_tx)?;

        Ok(Response::text(""))
    }
}
