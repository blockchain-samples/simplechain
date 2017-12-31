use bincode::{serialize, deserialize, Infinite};
use sha2::{Sha256, Digest};
use rusqlite::Connection;
use base58::{FromBase58, ToBase58};
use hex::{FromHex, ToHex};
use secp256k1;
use secp256k1::key::{SecretKey, PublicKey};
use jfs;

use net::{NetTransaction, NetKeyPair};
use errors::CoreError;
use utils;

// FIXME too many public fields

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct TransactionContent {
    pub sender_addr: Vec<u8>,
    pub sender_pubkey: Vec<u8>,
    pub receiver_addr: Vec<u8>,
    pub amount: i32,
    pub timestamp: i64
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct TransactionSigned {
    pub content: TransactionContent,
    pub signature: Vec<u8>
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Transaction {
    pub id: Vec<u8>,
    pub transaction: TransactionSigned // bad field name...
}

impl TransactionContent {
    // sign a transaction using schnorr signature
    pub fn get_signature(
        &self,
        private_key: SecretKey
    ) -> Result<Vec<u8>, CoreError> {
        println!("SIGN TRANSACTION");

        let secp = secp256k1::Secp256k1::new();
        // serialize the tx content
        let tx_content_encoded: Vec<u8> = serialize(&self, Infinite)?;

        // hash the tx content
        let mut hasher = Sha256::new();
        hasher.input(&tx_content_encoded);
        let tx_content_hashed = hasher.result();

        // create the input message with the hashed tx content
        let input = secp256k1::Message::from_slice(tx_content_hashed.as_slice())?;

        // return the signature created with the input message and private key
        Ok(secp.sign_schnorr(&input, &private_key)?.serialize())
    }
}

impl TransactionSigned {
    // hash a transaction to create its id
    pub fn get_id(&self) -> Result<Vec<u8>, CoreError> {
        // serialize the signed tx
        let tx_signed_encoded: Vec<u8> = serialize(&self, Infinite)?;

        // hash everything to return the id
        let mut hasher = Sha256::new();
        hasher.input(&tx_signed_encoded);
        Ok(hasher.result().as_slice().to_vec())
    }
}

impl Transaction {
    // create a transaction from raw bytes
    pub fn from_bytes(data: &Vec<u8>) -> Result<Transaction, CoreError> {
        // read data and deserialize into a Transaction struct
        let tx: Transaction = deserialize(&data[..])?;
        Ok(tx)
    }

    // verify a transaction using the signature and the public key
    pub fn is_valid(&self) -> Result<bool, CoreError> {
        println!("VERIFY TRANSACTION");

        let secp = secp256k1::Secp256k1::new();
        // serialize the tx content
        let tx_encoded: Vec<u8> = serialize(&self.transaction.content, Infinite)?;

        // hash the tx content
        let mut hasher = Sha256::new();
        hasher.input(&tx_encoded);
        let tx_hashed = hasher.result();

        // create the input message using the hashed tx content
        let input = secp256k1::Message::from_slice(tx_hashed.as_slice())?;

        // retrieve sig and pbkey from the tx
        let signature = secp256k1::schnorr::Signature::deserialize(&self.transaction.signature);
        let public_key = PublicKey::from_slice(
            &secp, &self.transaction.content.sender_pubkey
        )?;

        // verify the input message using the signature and pbkey
        Ok(
            match secp.verify_schnorr(&input, &signature, &public_key) {
                Ok(()) => true,
                _ => false
            }
        )
    }

    // store a transaction on database (cache) for further block creation
    // TODO rewrite this with redis
    pub fn store_db(&self) -> Result<(), CoreError> {
        println!("STORE TRANSACTION [DB]");
        // TODO rewrite this with connection pools
        // TODO get the db address string from config.json
        let conn = Connection::open("db/storage.db")?;

        let id = &self.id.to_hex();
        let sender_addr = &self.transaction.content.sender_addr.to_base58();
        let sender_pubkey = &self.transaction.content.sender_pubkey.to_hex();
        let receiver_addr = &self.transaction.content.receiver_addr.to_base58();
        let amount = &self.transaction.content.amount;
        let timestamp = &self.transaction.content.timestamp;
        let signature = &self.transaction.signature.to_hex();

        conn.execute("INSERT INTO transactions(
            id, sender_addr, sender_pubkey, receiver_addr, amount, timestamp, signature
        ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            &[&*id, &*sender_addr, &*sender_pubkey, &*receiver_addr, &*amount, &*timestamp, &*signature])?;

        Ok(())
    }
}

// create a transaction, sign it, hash it and return a network version of it
pub fn new(
    sender_privkey: SecretKey,
    sender_pubkey: Vec<u8>,
    sender_addr: Vec<u8>,
    receiver_addr: Vec<u8>,
    amount: i32
) -> Result<NetTransaction, CoreError> {
    println!("CREATE TRANSACTION");

    let timestamp: i64 = utils::get_current_timestamp();

    let tx_content = TransactionContent {
        sender_addr: sender_addr,
        sender_pubkey: sender_pubkey,
        receiver_addr: receiver_addr,
        amount: amount,
        timestamp: timestamp
    };

    // sign the current tx content
    let signature: Vec<u8> = tx_content.get_signature(sender_privkey)?;

    // create a signed tx with the signature
    let tx_signed = TransactionSigned {
        content: tx_content,
        signature: signature
    };

    // get the tx id (hash) using the signed tx content
    let id: Vec<u8> = tx_signed.get_id()?;

    // TEST
    println!("-- TRANSACTION --");
    println!("id: {}", id.to_hex());
    println!("sender_addr: {}", tx_signed.content.sender_addr.to_base58());
    println!("sender_pubkey: {}", tx_signed.content.sender_pubkey.to_hex());
    println!("receiver_addr: {}", tx_signed.content.receiver_addr.to_base58());
    println!("amount: {}", tx_signed.content.amount);
    println!("timestamp: {}", tx_signed.content.timestamp);
    println!("signature: {}", tx_signed.signature.to_hex());

    // return the final network transaction
    let id = id.to_hex();
    let sender_addr = tx_signed.content.sender_addr.to_base58();
    let sender_pubkey = tx_signed.content.sender_pubkey.to_hex();
    let receiver_addr = tx_signed.content.receiver_addr.to_base58();
    let amount = tx_signed.content.amount;
    let timestamp = tx_signed.content.timestamp;
    let signature = tx_signed.signature.to_hex();

    Ok(NetTransaction {
        id: id,
        sender_addr: sender_addr,
        sender_pubkey: sender_pubkey,
        receiver_addr: receiver_addr,
        amount: amount,
        timestamp: timestamp,
        signature: signature
    })
}

// return a Transaction struct filled with given field values
pub fn from(
    id: &String,
    sender_addr: &String,
    sender_pubkey: &String,
    receiver_addr: &String,
    amount: i32,
    timestamp: i64,
    signature: &String,
) -> Result<Transaction, CoreError> {
    let id: Vec<u8> = FromHex::from_hex(id)?;
    let sender_addr: Vec<u8> = sender_addr.from_base58()?;
    let sender_pubkey: Vec<u8> = FromHex::from_hex(sender_pubkey)?;
    let receiver_addr: Vec<u8> = receiver_addr.from_base58()?;
    let signature: Vec<u8> = FromHex::from_hex(signature)?;

    Ok(Transaction {
        id: id,
        transaction: TransactionSigned {
            content: TransactionContent {
                sender_addr: sender_addr,
                sender_pubkey: sender_pubkey,
                receiver_addr: receiver_addr,
                amount: amount,
                timestamp: timestamp
            },
            signature: signature,
        },
    })
}

pub fn coinbase() -> Result<Transaction, CoreError> {
    println!("CREATE COINBASE TRANSACTION");
    // retrieve wallet entry from storage
    // XXX this is ugly, we read for the address and then read again for private_key and public_key
    // we should do everything at once without reading twice
    let cfg = jfs::Config {
        pretty: true,
        indent: 4,
        single: true
    };
    let storage = jfs::Store::new_with_cfg("storage/wallet", cfg).unwrap();
    let wallets = storage.all::<NetKeyPair>().unwrap();

    let wallet = wallets.iter().nth(0);

    let address = match wallet {
        Some(w) => w.0.from_base58()?,
        None => return Err(CoreError::WalletError)
    };

    // create coinbase value
    let coinbase = String::from("coinbase").into_bytes();

    let timestamp: i64 = utils::get_current_timestamp();

    let tx_content = TransactionContent {
        sender_addr: coinbase.clone(),
        sender_pubkey: coinbase.clone(),
        receiver_addr: address,
        amount: 50, // XXX FIXME amount is hardcoded for now
        timestamp: timestamp
    };

    let tx_signed = TransactionSigned {
        content: tx_content,
        signature: coinbase
    };

    // get the tx id (hash) using the signed tx content
    let id: Vec<u8> = tx_signed.get_id()?;

    // return the final tx
    Ok(Transaction {
        id: id,
        transaction: tx_signed
    })
}

// TODO rewrite this with redis
// XXX maybe return a NetTransaction directly?
// read all cached database transactions
pub fn read_db() -> Result<Vec<Transaction>, CoreError> {
    println!("READ TRANSACTIONS [DB]");
    let conn = Connection::open("db/storage.db")?;

    let mut transactions: Vec<Transaction> = Vec::new();

    let mut stmt = conn.prepare(
        "SELECT id, sender_addr, sender_pubkey, receiver_addr, amount, timestamp, signature
        FROM transactions"
    )?;

    let net_txs = stmt.query_map(&[], |row| {
        let id: String = row.get(0);
        let sender_addr: String = row.get(1);
        let sender_pubkey: String = row.get(2);
        let receiver_addr: String = row.get(3);
        let amount: i32 = row.get(4);
        let timestamp: i64 = row.get(5);
        let signature: String = row.get(6);

        NetTransaction {
            id: id,
            sender_addr: sender_addr,
            sender_pubkey: sender_pubkey,
            receiver_addr: receiver_addr,
            amount: amount,
            timestamp: timestamp,
            signature: signature
        }
    })?;

    for net_tx in net_txs {
        let net_tx = net_tx?;

        let tx = Transaction {
            id: FromHex::from_hex(net_tx.id)?,
            transaction: TransactionSigned {
                content: TransactionContent {
                    sender_addr: net_tx.sender_addr.from_base58()?,
                    sender_pubkey: FromHex::from_hex(net_tx.sender_pubkey)?,
                    receiver_addr: net_tx.receiver_addr.from_base58()?,
                    amount: net_tx.amount,
                    timestamp: net_tx.timestamp
                },
                signature: FromHex::from_hex(net_tx.signature)?
            }
        };

        transactions.push(tx);
    }

    Ok(transactions)
}

// delete all cached transactions from database
pub fn clean_db() -> Result<(), CoreError> {
    println!("CLEAN TRANSACTIONS [DB]");
    // TODO rewrite this with connection pools
    // TODO get the db address string from config.json
    let conn = Connection::open("db/storage.db")?;

    conn.execute("DELETE FROM transactions", &[])?;
    Ok(())
}
