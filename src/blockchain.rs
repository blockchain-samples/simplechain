use r2d2::{Config, Pool};
use r2d2_postgres::{TlsMode, PostgresConnectionManager};
use postgres_array::Array;
use hex::FromHex;

use net::{NetBlock, NetTransaction};
use errors::CoreError;

fn get_db_pool() -> Result<Pool<PostgresConnectionManager>, CoreError> {
    let config = Config::default();
    let manager = PostgresConnectionManager::new(
        "postgres://mgul@localhost/blockchain",
        TlsMode::None
    ).unwrap();

    match Pool::new(config, manager) {
        Ok(pool) => Ok(pool),
        Err(e) => Err(CoreError::DatabaseError) // maybe just panic! as we can't establish a connection to database
    }
}

pub fn add_block(block: NetBlock) -> Result<(), CoreError> {
    let pool = get_db_pool()?;
    let conn = pool.get().unwrap();

    let query = "INSERT INTO blocks(id, timestamp, previous_hash, merkle_root, hash, nonce, transactions)
        VALUES($1, $2, $3, $4, $5, $6, $7)";

    conn.execute(query, &[
        &block.id,
        &block.timestamp,
        &block.previous_hash,
        &block.merkle_root,
        &block.hash,
        &block.nonce,
        &Array::from_vec(block.transactions, 0)
    ])?;

    Ok(())
}

pub fn get_previous_id() -> Result<i32, CoreError> {
    let pool = get_db_pool()?;
    let conn = pool.get().unwrap();

    let query = "SELECT id FROM blocks ORDER BY id DESC";
    let rows = conn.query(query, &[])?;

    if !rows.is_empty() {
        let row = rows.get(0);
        let id: i32 = row.get("id"); // TODO access by index (more efficient)

        Ok(id)
    } else {
        Ok(0)
    }
}

pub fn get_previous_hash() -> Result<Vec<u8>, CoreError> {
    let pool = get_db_pool()?;
    let conn = pool.get().unwrap();

    let query = "SELECT hash FROM blocks ORDER BY id DESC";
    let rows = conn.query(query, &[])?;

    if !rows.is_empty() {
        let row = rows.get(0);
        let hash: String = row.get("hash"); // TODO access by index (more efficient)
        let hash_bytes: Vec<u8> = FromHex::from_hex(hash)?;

        Ok(hash_bytes)
    } else {
        // genesis
        let zero_hash: Vec<u8> = vec![0];
        Ok(zero_hash)
    }
}

#[derive(Debug)]
struct LedgerEntry {
    sender_addr: String,
    receiver_addr: String,
    amount: i32
}

pub fn scan() -> Result<(), CoreError> {
    let pool = get_db_pool()?;
    let conn = pool.get().unwrap();

    let query = "SELECT transactions FROM blocks";
    let rows = conn.query(query, &[])?;

    let mut ledger: Vec<Vec<(String, String, i32)>> = Vec::new();

    if !rows.is_empty() {
        for (i, _) in rows.iter().enumerate() {
            let transactions: Vec<NetTransaction> = rows.get(i).get(0);

            let block_ledger: Vec<(String, String, i32)> = transactions.into_iter().map(|tx| {
                (tx.sender_addr, tx.receiver_addr, tx.amount)
            }).collect();

            ledger.push(block_ledger);
        }
    } else {
        return Err(CoreError::IoError); // XXX proper handle
    }

    let mut ins: Vec<(String, i32)> = Vec::new();
    let mut outs: Vec<(String, i32)> = Vec::new();

    println!("{:?}", ledger);
    println!("------\n");

    for block_ledger in ledger {
        for ledger_entry in block_ledger {
            ins.push((ledger_entry.1, ledger_entry.2));
            outs.push((ledger_entry.0, ledger_entry.2));
        }
    }

    let mut balances: Vec<(String, i32)> = Vec::new();
    let mut in_balances: Vec<(String, i32)> = Vec::new();
    let mut addresses: Vec<String> = Vec::new();

    println!("INS: {:?}\n", ins);
    println!("OUTS: {:?}\n", outs);

    for (i, e) in ins.into_iter().enumerate() {
        println!("[{}] | {:?}", i, e);
        if !addresses.contains(&e.0) {
            addresses.push(e.0);
        }
    }

    println!("\n");

    for (i, e) in outs.into_iter().enumerate() {
        println!("[{}] | {:?}", i, e);
        if !addresses.contains(&e.0) && e.0 != "0" {
            addresses.push(e.0);
        }
    }

    println!("\n");

    println!("{:?}", addresses);

    println!("\n");

    println!("{:?}\n", balances);

    Ok(())
}
