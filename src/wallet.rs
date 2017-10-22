extern crate secp256k1;
extern crate rand;

use sha2::{Sha256, Digest};

// returns the private key, public key and address
pub fn get_identity() -> (secp256k1::key::SecretKey, Vec<u8>, Vec<u8>) {
    let secp = secp256k1::Secp256k1::new();
    let mut rng = rand::thread_rng();

    // get private & public key using Rng
    let (private_key, public_key) = secp.generate_keypair(&mut rng).unwrap();
    // serialize and compress the public key
    let public_key_compressed = public_key.serialize_vec(&secp, true);

    // hash the public key to get the address
    let mut hasher = Sha256::new();
    hasher.input(&public_key_compressed);
    let hash = hasher.result();

    (
        private_key,
        public_key_compressed.as_slice().to_vec(),
        hash.as_slice().to_vec()
    )
}
