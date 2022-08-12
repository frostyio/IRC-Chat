use sha3::{Digest, Sha3_256};
pub mod encoding;
pub mod encryption;
pub mod io;
pub mod stream;

pub fn hash(data: &[u8]) -> Vec<u8> {
	Sha3_256::new_with_prefix(data).finalize().to_vec()
}

pub fn hex(data: &[u8]) -> String {
	format!("{:x?}", data)
}

// output size: 256 * 2 = 512 bits || 64 bytes, accounting for hex which outputs 2 bytes
pub fn hex_hash(data: &[u8]) -> String {
	let hash = &Sha3_256::new_with_prefix(data).finalize();
	format!("{:x}", hash)
}
