use std::collections::HashMap;

use rsa::RsaPrivateKey;
use tokio::{net::TcpStream, sync::mpsc};

use self::client::Client;
mod broker;
mod client;

#[derive(Debug)]
pub enum Event {
	NewPeer(Sender, TcpStream, RsaPrivateKey),
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
}

impl InnerServer {
	pub fn new() -> Self {
		Self {
			clients: HashMap::new(),
		}
	}

	pub fn add_client(&mut self, id: String, client: Client) {
		println!("adding new client with id: {id}");
		self.clients.insert(id.clone(), client);
	}

	pub fn relay_data_to_recepient(&mut self, id: &String, buff: &[u8]) {
		if let Some(client) = self.clients.get_mut(id) {
			client.make_and_send(buff);
		}
	}

	pub fn relay_data_to_all(&mut self, buff: &[u8]) {
		for (_, client) in self.clients.iter_mut() {
			client.make_and_send(buff);
		}
	}
}
