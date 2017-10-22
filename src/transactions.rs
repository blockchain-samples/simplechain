use std::fs::{self, File};
use std::io::prelude::*;
use std::io::ErrorKind;
use std::path::Path;
use bincode::{serialize, deserialize, Infinite};
use sha2::{Sha256, Digest};
use rusqlite::Connection;
use base58::ToBase58;
use hex::ToHex;
use secp256k1;

use utils;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct TransactionContent {
    sender_addr: Vec<u8>,
    sender_pubkey: Vec<u8>,
    receiver_addr: Vec<u8>,
    amount: u32,
    timestamp: i64
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct TransactionSigned {
    content: TransactionContent,
    signature: Vec<u8>
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Transaction {
    pub id: Vec<u8>,
    transaction: TransactionSigned
}

// struct TransactionContentReadable<'a>(&'a TransactionContent);
//
// impl<'a> Serialize for TransactionContentReadable<'a> {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//         where S: Serializer
//     {
//         let mut state = serializer.serialize_struct("TransactionContent", 5)?;
//         state.serialize_field("sender_addr", &self.0.sender_addr.to_base58())?;
//         state.serialize_field("sender_pubkey", &self.0.sender_pubkey.to_base58())?;
//         state.serialize_field("receiver_addr", &self.0.receiver_addr.to_base58())?;
//         state.serialize_field("amount", &self.0.amount)?;
//         state.serialize_field("timestamp", &self.0.timestamp)?;
//         state.end()
//     }
// }
//
// struct TransactionSignedReadable<'a>(&'a TransactionSigned);
//
// impl<'a> Serialize for TransactionSignedReadable<'a> {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//         where S: Serializer
//     {
//         let mut state = serializer.serialize_struct("TransactionSigned", 2)?;
//         state.serialize_field("content", &self.0.content)?;
//         state.serialize_field("signature", &self.0.signature.to_hex())?;
//         state.end()
//     }
// }
//
// pub struct TransactionReadable<'a>(&'a Transaction);
//
// impl<'a> Serialize for TransactionReadable<'a> {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//         where S: Serializer
//     {
//         let mut state = serializer.serialize_struct("Transaction", 2)?;
//         state.serialize_field("id", &self.0.id.to_hex())?;
//         state.serialize_field("transaction", &self.0.transaction)?;
//         state.end()
//     }
// }

// TODO use impl syntax instead of transactions::fn() syntax

// create a transaction, sign it, hash it and return it
pub fn create(
    sender_privkey: secp256k1::key::SecretKey,
    sender_pubkey: Vec<u8>,
    sender_addr: Vec<u8>,
    receiver_addr: Vec<u8>,
    amount: u32
) -> Transaction {
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
    let signature: Vec<u8> = get_signature(&tx_content, sender_privkey);
    // create a signed tx with the sig
    let tx_signed = TransactionSigned {
        content: tx_content,
        signature: signature
    };

    // get the tx id (hash) using the signed tx content
    let id: Vec<u8> = get_id(&tx_signed);

    //  TODO maybe rewrite this by removing the struct nesting
    // this will be easier for cross-language
    // Transaction {
    //     id: ...
    //     sender_addr: ...
    //     ...
    //     signature: ...
    // }
    // but be careful maybe recreating Transaction with TransactionContent's
    // and TransactionSigned's fields will make the signature obsolete

    // create the final tx
    Transaction {
        id: id,
        transaction: tx_signed
    }
}

// create a transaction from raw bytes
pub fn create_from_bytes(data: &Vec<u8>) -> Transaction {
    // read data and deserialize into a Transaction struct
    let tx: Transaction = deserialize(&data[..]).unwrap();
    tx
}

// verify a transaction using the signature and the public key
pub fn verify(tx: &Transaction) -> bool {
    println!("VERIFY TRANSACTION");

    let secp = secp256k1::Secp256k1::new();
    // serialize the tx content
    let tx_encoded: Vec<u8> = serialize(&tx.transaction.content, Infinite).unwrap();

    // hash the tx content
    let mut hasher = Sha256::new();
    hasher.input(&tx_encoded);
    let tx_hashed = hasher.result();

    // create the input message using the hashed tx content
    let input = secp256k1::Message::from_slice(tx_hashed.as_slice()).unwrap();
    // retrieve sig and pbkey from the tx
    let signature = secp256k1::schnorr::Signature::deserialize(&tx.transaction.signature);
    let public_key = secp256k1::key::PublicKey::from_slice(
        &secp, &tx.transaction.content.sender_pubkey
    ).unwrap();

    // verify the input message using the sig and pbkey
    match secp.verify_schnorr(&input, &signature, &public_key) {
        Ok(()) => true,
        _ => false
    }
}

// store a transaction on database (cache) for further block creation
pub fn store_db(tx: &Transaction) {
    println!("STORE TRANSACTION [DB]");
    let conn = Connection::open("coin.db").unwrap();

    let id = &tx.id.to_hex();
    let sender_addr = &tx.transaction.content.sender_addr.to_base58();
    let sender_pubkey = &tx.transaction.content.sender_pubkey.to_hex();
    let receiver_addr = &tx.transaction.content.receiver_addr.to_base58();
    let amount = &tx.transaction.content.amount;
    let timestamp = &tx.transaction.content.timestamp;
    let signature = &tx.transaction.signature.to_hex();

    conn.execute("INSERT INTO transactions(
        id, sender_addr, sender_pubkey, receiver_addr, amount, timestamp, signature
    ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        &[&*id, &*sender_addr, &*sender_pubkey, &*receiver_addr, &*amount, &*timestamp, &*signature])
        .unwrap();
}

// read all cached (on database) transactions
pub fn read_db() -> Vec<Transaction> {
    println!("READ TRANSACTIONS [DB]");
    let conn = Connection::open("coin.db").unwrap();

    let mut transactions: Vec<Transaction> = Vec::new();

    let mut stmt = conn.prepare(
        "SELECT id, sender_addr, sender_pubkey, receiver_addr, amount, timestamp, signature
        FROM transactions"
    ).unwrap();

    let rows = stmt.query_map(&[], |row| {
        let id: String = row.get(0);
        let sender_addr: String = row.get(1);
        let sender_pubkey: String = row.get(2);
        let receiver_addr: String = row.get(3);
        let amount: u32 = row.get(4);
        let timestamp: i64 = row.get(5);
        let signature: String = row.get(6);

        Transaction {
            id: id.into_bytes(),
            transaction: TransactionSigned {
                content: TransactionContent {
                    sender_addr: sender_addr.into_bytes(),
                    sender_pubkey: sender_pubkey.into_bytes(),
                    receiver_addr: receiver_addr.into_bytes(),
                    amount: amount,
                    timestamp: timestamp
                },
                signature: signature.into_bytes()
            }
        }
    }).unwrap();

    for tx in rows {
        transactions.push(tx.unwrap());
    }

    transactions
}

// delete all cached transactions from database
pub fn clean_db() {
    println!("CLEAN TRANSACTIONS [DB]");
    let conn = Connection::open("coin.db").unwrap();

    conn.execute("DELETE FROM transactions", &[]).unwrap();
}

// store a transaction on disk (cache) for further block creation
pub fn store_disk(tx: &Transaction) {
    println!("STORE TRANSACTION [DISK]");
    let tx_encoded: Vec<u8> = serialize(&tx, Infinite).unwrap();
    let tx_dir_path = Path::new("./transactions");

    let ready: bool = match fs::create_dir(tx_dir_path) {
        Ok(_) => true,
        Err(e) => match e.kind() {
            ErrorKind::AlreadyExists => true,
            _ => false,
        },
    };

    if ready {
        let tx_dir = fs::read_dir(tx_dir_path).unwrap();
        let tx_file_path = tx_dir_path.join(format!("tx{}.bin", tx_dir.count() + 1));
        let mut tx_file = File::create(tx_file_path).unwrap();

        tx_file.write_all(&tx_encoded).unwrap();
    }
}

// read all cached (on disk) transactions
pub fn read_disk() -> Result<Vec<Transaction>, String> {
    println!("READ TRANSACTIONS [DISK]");
    let tx_dir_path = Path::new("./transactions");

    let ready: bool = match fs::read_dir(tx_dir_path) {
        Ok(_) => true,
        Err(e) => false,
    };

    if ready {
        let tx_dir = fs::read_dir(tx_dir_path).unwrap();
        let mut transactions: Vec<Transaction> = Vec::new();

        for tx_file in tx_dir {
            let mut tx_file = File::open(tx_file.unwrap().path()).unwrap();
            let mut buffer = vec![0; 1024];

            tx_file.read(&mut buffer);

            let tx: Transaction = deserialize(&buffer[..]).unwrap();
            transactions.push(tx);
        }

        Ok(transactions)
    } else {
        Err(String::from("Error"))
    }
}

// delete all cached transactions from disk
pub fn clean_disk() {
    println!("CLEAN TRANSACTIONS [DISK]");
}

// sign a transaction using schnorr signature
fn get_signature(
    tx_content: &TransactionContent,
    private_key: secp256k1::key::SecretKey
) -> Vec<u8> {
    println!("SIGN TRANSACTION");

    let secp = secp256k1::Secp256k1::new();
    // serialize the tx content
    let tx_content_encoded: Vec<u8> = serialize(tx_content, Infinite).unwrap();

    // hash the tx content
    let mut hasher = Sha256::new();
    hasher.input(&tx_content_encoded);
    let tx_content_hashed = hasher.result();

    // create the input message with the hashed tx content
    let input = secp256k1::Message::from_slice(tx_content_hashed.as_slice()).unwrap();

    // return the signature created with the input message and private key
    secp.sign_schnorr(&input, &private_key).unwrap().serialize()
}

// hash a transaction to create its id
fn get_id(tx_signed: &TransactionSigned) -> Vec<u8> {
    // serialize the signed tx
    let tx_signed_encoded: Vec<u8> = serialize(tx_signed, Infinite).unwrap();

    // hash everything to return the id
    let mut hasher = Sha256::new();
    hasher.input(&tx_signed_encoded);
    hasher.result().as_slice().to_vec()
}
