use sha3::{Digest, Sha3_256};
pub mod encryption;
pub mod io;

pub fn hash(data: &[u8]) -> Vec<u8> {
	Sha3_256::new_with_prefix(data).finalize().to_vec()
}

pub fn hex_hash(data: &[u8]) -> String {
	let hash = &Sha3_256::new_with_prefix(data).finalize();
	format!("{:x}", hash)
}
