use std::thread;
use base58::{FromBase58, ToBase58};
use hex::{FromHex, ToHex};
use rouille::{input, Request, Response};
use postgres_array::Array;

use super::{nodes, NetTransaction, NetBlock};
use errors::ServerError;
use transactions;
use blockchain;
use blocks;
use wallet;

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
    let block: NetBlock = input::json_input(req)?;
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

// local handlers (only accessible locally)
pub mod local {
    use std::ops::Index;
    use base58::{FromBase58, ToBase58};
    use hex::{FromHex, ToHex};
    use rouille::{input, Request, Response};
    use jfs;
    use secp256k1;

    use super::nodes;
    use errors::ServerError;
    use transactions;
    use wallet;

    #[derive(RustcEncodable)]
    struct Wallet {
        private_key: String,
        public_key: String,
        address: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct KeyPair {
        private_key: String,
        public_key: String,
    }

    pub fn get_wallet(req: &Request) -> Result<Response, ServerError> {
        // TODO handle the fact that the user calls this by mistake (his previous wallet will be lost)
        let (private_key, public_key, address) = wallet::get_wallet()?;

        let private_key: String = private_key.index(..).to_hex();
        let public_key: String = public_key.to_hex();
        let address: String = address.to_base58();

        let wallet = Wallet {
            private_key: private_key.clone(),
            public_key: public_key.clone(),
            address: address
        };

        let keypair = KeyPair {
            private_key: private_key,
            public_key: public_key
        };

        let cfg = jfs::Config {
            pretty: true,
            indent: 4,
            single: true
        };
        // TODO handle user moving the file
        // TODO check if folder `storage` exists or create it before
        let storage = jfs::Store::new_with_cfg("storage/wallet", cfg).unwrap();
        storage.save_with_id(&keypair, &wallet.address).unwrap();

        Ok(Response::json(&wallet))
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
        // TODO handle if there is no wallet associated
        let cfg = jfs::Config {
            pretty: true,
            indent: 4,
            single: true
        };
        let storage = jfs::Store::new_with_cfg("storage/wallet", cfg).unwrap();
        let keypair: KeyPair = storage.get::<KeyPair>(&tx_body.sender_addr).unwrap();

        // rebuild secp256k1 secret key
        let secp = secp256k1::Secp256k1::new();
        let private_key_bytes: Vec<u8> = FromHex::from_hex(&keypair.private_key)?;
        let sk = secp256k1::key::SecretKey::from_slice(&secp, &private_key_bytes).unwrap();

        // deserialize addresses and public key
        let public_key_bytes: Vec<u8> = FromHex::from_hex(&keypair.public_key)?;
        let sender_addr_bytes: Vec<u8> = tx_body.sender_addr.from_base58()?;
        let receiver_addr_bytes: Vec<u8> = tx_body.receiver_addr.from_base58()?;

        // create transaction for signature
        let net_tx = transactions::new(sk, public_key_bytes, sender_addr_bytes, receiver_addr_bytes, tx_body.amount)?;

        // broadcast transaction to network
        nodes::send_transaction(net_tx)?;

        Ok(Response::text(""))
    }
}
