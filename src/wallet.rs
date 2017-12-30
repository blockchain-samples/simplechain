use secp256k1::Secp256k1;
use secp256k1::key::SecretKey;
use rand;
use sha2::{Sha256, Digest};

use base58::FromBase58;
use hex::FromHex;
use jfs;

use net::{NetKeyPair, NetWallet};
use errors::CoreError;

pub struct Wallet {
    pub keypair: KeyPair,
    pub address: Vec<u8>,
}

pub struct KeyPair {
    pub private_key: SecretKey,
    pub public_key: Vec<u8>,
}

// returns a new wallet
pub fn get_new_wallet() -> Result<Wallet, CoreError> {
    let secp = Secp256k1::new();
    let mut rng = rand::thread_rng();

    // get private & public key using Rng
    let (private_key, public_key) = secp.generate_keypair(&mut rng)?;
    // serialize and compress the public key
    let public_key_compressed = public_key.serialize_vec(&secp, true);

    // hash the public key to get the address
    let mut hasher = Sha256::new();
    hasher.input(&public_key_compressed);
    let hash = hasher.result();

    let keypair = KeyPair {
        private_key: private_key,
        public_key: public_key_compressed.as_slice().to_vec()
    };

    Ok(Wallet {
        keypair: keypair,
        address: hash.as_slice().to_vec()
    })
}

// return the wallet associated with the given address
// TODO handle if there is no wallet associated
pub fn get_wallet(address: &String) -> Result<Wallet, CoreError> {
    let cfg = jfs::Config {
        pretty: true,
        indent: 4,
        single: true
    };
    let storage = jfs::Store::new_with_cfg("storage/wallet", cfg).unwrap();
    let keypair: NetKeyPair = storage.get::<NetKeyPair>(&address).unwrap();

    // rebuild secp256k1 secret key
    let secp = Secp256k1::new();
    let private_key_bytes: Vec<u8> = FromHex::from_hex(&keypair.private_key)?;
    let sk = SecretKey::from_slice(&secp, &private_key_bytes).unwrap();

    // deserialize addresses and public key
    let public_key_bytes: Vec<u8> = FromHex::from_hex(&keypair.public_key)?;

    let keypair = KeyPair {
        private_key: sk,
        public_key: public_key_bytes
    };

    Ok(Wallet {
        keypair: keypair,
        address: address.from_base58()?
    })
}
