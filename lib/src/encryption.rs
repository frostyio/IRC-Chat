use aes_gcm::{
	aead::{rand_core::RngCore, Aead, KeyInit, OsRng},
	Aes256Gcm, Nonce,
};

pub fn generate_nonce() -> Vec<u8> {
	let mut buff = [0u8; 12]; // 96 bit nonce / 8
	OsRng.fill_bytes(&mut buff);
	buff.to_vec()
}

pub fn encrypt(key_buff: &[u8], data: &[u8]) -> Vec<u8> {
	let mut nonce_buff = generate_nonce();
	let cipher = Aes256Gcm::new_from_slice(key_buff).expect("invalid key");
	let nonce = Nonce::from_slice(nonce_buff.as_slice());

	let mut ciphered = cipher.encrypt(nonce, data).expect("failed to encrypt");

	// append the nonce to the beginning of the ciphered buffer
	nonce_buff.append(&mut ciphered);
	nonce_buff
}

pub fn decrypt(key_buff: &[u8], buffer: Vec<u8>) -> Vec<u8> {
	let (nonce_buff, cipher_text) = buffer.split_at(12);

	let nonce = Nonce::from_slice(nonce_buff);
	let cipher = Aes256Gcm::new_from_slice(key_buff).expect("invalid key");

	match cipher.decrypt(nonce, cipher_text.as_ref()) {
		Ok(data) => data,
		Err(e) => panic!("unable to decrypt: {:#?}", e.to_string()),
	}
}

#[cfg(test)]
mod tests {
	use crate::encryption::decrypt;

	use super::encrypt;
	use aes_gcm::{aead::OsRng, Aes256Gcm, KeyInit};

	#[test]
	pub fn test_encryption() {
		let key = Aes256Gcm::generate_key(&mut OsRng);

		let slice_key = key.as_slice();
		let data = b"hello world";

		let ciphered = encrypt(slice_key, data);
		let plain = decrypt(slice_key, ciphered.clone());

		assert_eq!(
			String::from_utf8(data.to_vec()).unwrap(),
			String::from_utf8(plain).unwrap()
		);
	}
}
