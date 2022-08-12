use std::collections::HashMap;

use lib::encoding::{Encoder, Instruction};
use rsa::RsaPrivateKey;
use tokio::{net::TcpStream, sync::mpsc};

use self::client::Client;
mod broker;
mod client;
mod feed;

#[derive(Debug)]
pub enum Event {
	SetServerId(String),
	NewPeer(Sender, TcpStream, RsaPrivateKey),
	RelayFeed(String, String, Vec<u8>), // ClientId, RecepientId, Encrypted Data
	SendToAll(Vec<Instruction>),
	SendToOthers(String, Vec<Instruction>),
}
pub type Sender = mpsc::UnboundedSender<Event>;
pub type Receiver = mpsc::UnboundedReceiver<Event>;

/*
Handles all the exterior functionality & event controlling,
used by a Socket
*/
pub struct OuterServer(Sender);

impl OuterServer {
	pub fn new(inner_server: InnerServer) -> Self {
		let (sender, receiver) = mpsc::unbounded_channel::<Event>();
		let _handle = tokio::spawn(broker::broker(receiver, inner_server));

		Self(sender)
	}

	pub fn send(&self, event: Event) -> Result<(), tokio::sync::mpsc::error::SendError<Event>> {
		self.0.send(event)
	}

	pub fn sender(&self) -> Sender {
		self.0.clone()
	}
}

/*
Handles all streams & client related data,
used by the broker
*/
pub struct InnerServer {
	clients: HashMap<String, client::Client>,
	id: String,
}

impl InnerServer {
	pub fn new() -> Self {
		Self {
			clients: HashMap::new(),
			id: String::from(""),
		}
	}

	pub fn add_client(&mut self, id: String, client: Client) {
		#[cfg(debug_assertions)]
		println!("adding new client with id: {id}");

		self.clients.insert(id.clone(), client);
	}

	pub async fn read_feed(&mut self, id: &str, buff: Vec<u8>) {
		if let Some(client) = self.clients.get_mut(id) {
			client.read_feed(buff).await;
		} else {
			#[cfg(debug_assertions)]
			eprintln!("attempting to read feed to invalid client with id: {id}");
		}
	}

	pub fn relay_data_to_recepient(&mut self, id: &String, buff: &[u8]) {
		if let Some(client) = self.clients.get_mut(id) {
			client.make_and_send(&self.id, buff);
		} else {
			#[cfg(debug_assertions)]
			eprintln!("attempting to relay to invalid client with id: {id}");
		}
	}

	pub fn relay_data_to_all(&mut self, buff: &[u8]) {
		for (_, client) in self.clients.iter_mut() {
			client.make_and_send(&self.id, buff);
		}
	}

	pub fn relay_data_to_others(&mut self, id: &str, buff: &[u8]) {
		for (_, client) in self.clients.iter_mut() {
			if client.get_id() != id {
				client.make_and_send(&self.id, buff);
			}
		}
	}

	pub fn set_id(&mut self, id: String) {
		self.id = id
	}

	pub fn get_id(&self) -> &str {
		&self.id
	}

	pub fn send_instructions_to_all(&mut self, feed: Vec<Instruction>) {
		let data = Encoder::from_feed(feed).writer.dump();
		self.relay_data_to_all(&data)
	}

	pub fn send_instructions_to_others(&mut self, sender_id: &str, feed: Vec<Instruction>) {
		let data = Encoder::from_feed(feed).writer.dump();
		self.relay_data_to_others(sender_id, &data)
	}
}
