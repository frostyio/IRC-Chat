use lib::encryption;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};

use super::Sender;

pub async fn listen_client(_id: String, _sender: Sender, stream: OwnedReadHalf) {
	loop {
		let _ = stream.readable().await;
	}
}

pub struct Client {
	id: String,
	write: OwnedWriteHalf,
	secret: Vec<u8>,
}

impl Client {
	pub fn new(id: String, write: OwnedWriteHalf, secret: &[u8]) -> Self {
		Self {
			id,
			write,
			secret: secret.to_vec(),
		}
	}

	pub fn make_payload(&self, buff: &[u8]) -> Vec<u8> {
		let mut payload = self.id.as_bytes().to_vec();
		payload.append(&mut encryption::encrypt(&self.secret, buff));
		payload
	}

	pub fn make_and_send(&mut self, buff: &[u8]) {
		let payload = self.make_payload(buff);
		match self.write.try_write(&payload) {
			Ok(_) => {}
			Err(e) => eprintln!("{:?}", e.to_string()),
		}
	}
}
