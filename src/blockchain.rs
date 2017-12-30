use r2d2::{Config, Pool};
use r2d2_postgres::{TlsMode, PostgresConnectionManager};
use postgres_array::Array;
use hex::FromHex;

use net::NetBlock;
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
        let id: i32 = row.get("id");

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
        let hash: String = row.get("hash");
        let hash_bytes: Vec<u8> = FromHex::from_hex(hash)?;

        Ok(hash_bytes)
    } else {
        // genesis
        let zero_hash: Vec<u8> = vec![0];
        Ok(zero_hash)
    }
}
