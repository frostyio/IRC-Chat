use super::Sender;
use crate::server::{feed::handle_feed, Event};
use lib::{
	encoding::{Decoder, Encoder, Instruction},
	encryption::{self, decrypt},
	stream::{self, StreamOperation},
};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};

pub async fn listen_client(id: String, sender: Sender, mut stream: OwnedReadHalf) {
	#[cfg(debug_assertions)]
	println!("listening to client {id}");

	loop {
		let (recepient_id, encrypted_buf) = match stream::read_stream(&mut stream).await {
			Ok((r, e)) => (r, e),
			Err(StreamOperation::Continue) => continue,
			Err(StreamOperation::Break) => break,
		};

		let _ = sender.send(Event::RelayFeed(
			id.clone(),
			recepient_id,
			encrypted_buf.to_vec(),
		));
	}

	// todo: remove client from server's client tables to cleanup
}

pub struct Client {
	#[allow(dead_code)]
	id: String,
	server_id: String,
	write: OwnedWriteHalf,
	shared_secret: Vec<u8>,
	#[allow(dead_code)]
	sender: Sender,
	pub username: String,
}

impl Client {
	pub fn new(
		id: String,
		write: OwnedWriteHalf,
		shared_secret: &[u8],
		sender: Sender,
		server_id: String,
	) -> Self {
		Self {
			id,
			server_id,
			write,
			shared_secret: shared_secret.to_vec(),
			sender,
			username: "Unknown".to_string(),
		}
	}

	// size hint (64 bits) | sender (512 bits) | encrypted buffer
	pub fn make_payload(&self, sender: &str, buff: &[u8]) -> Vec<u8> {
		let sender = sender.as_bytes().to_vec();
		let encrypted_buf = encryption::encrypt(&self.shared_secret, buff);
		let len = encrypted_buf.len() as u64;
		[len.to_be_bytes().to_vec(), sender, encrypted_buf].concat()
	}

	pub fn make_and_send(&mut self, sender: &str, buff: &[u8]) {
		let payload = self.make_payload(sender, buff);
		match self.write.try_write(&payload) {
			Ok(_) => {}
			Err(e) => eprintln!("{:?}", e.to_string()),
		}
	}

	pub async fn read_feed(&mut self, buff: Vec<u8>) {
		let decrypted_buff = decrypt(&self.shared_secret, buff);
		let decoder = Decoder::from_bytes(decrypted_buff);

		handle_feed(self, decoder.feed).await;
	}

	pub fn send_to_all(&mut self, feed: Vec<Instruction>) {
		let _ = self.sender.send(Event::SendToAll(feed));
	}

	pub fn send_to_others(&mut self, feed: Vec<Instruction>) {
		let _ = self.sender.send(Event::SendToOthers(self.id.clone(), feed));
	}

	pub fn send_message(&mut self, content: String) {
		self.send_to_all(vec![Instruction::ReceiveMessage(
			self.username.clone(),
			content,
		)])
	}

	pub fn send_local_instructions(&mut self, feed: Vec<Instruction>) {
		let data = Encoder::from_feed(feed).writer.dump();
		let id = self.server_id.clone();
		self.make_and_send(&id, &data);
	}

	pub fn send_local_message(&mut self, content: String) {
		self.send_local_instructions(vec![Instruction::ReceiveMessage(
			"Server".to_string(),
			content,
		)])
	}

	pub fn get_id(&self) -> &str {
		&self.id
	}
}
