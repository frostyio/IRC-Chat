use self::broker::broker;
use lib::encryption;
use std::collections::HashMap;
use tokio::{net::tcp::OwnedWriteHalf, sync::mpsc};
mod broker;

#[derive(Debug)]
pub enum Event {
	SetWriter(OwnedWriteHalf),
	SetSharedKey(String, Vec<u8>), // recepient, key
	Instantiate(String),           // username
}
pub type Sender = mpsc::UnboundedSender<Event>;
pub type Receiver = mpsc::UnboundedReceiver<Event>;

pub struct OuterClient(Sender);

impl OuterClient {
	pub fn new(inner_client: InnerClient) -> Self {
		let (sender, receiver) = mpsc::unbounded_channel::<Event>();
		tokio::spawn(broker(receiver, inner_client));

		Self(sender)
	}

	pub fn send(&self, event: Event) -> Result<(), tokio::sync::mpsc::error::SendError<Event>> {
		self.0.send(event)
	}

	// pub fn sender(&self) -> Sender {
	// 	self.0.clone()
	// }
}

fn make_payload(recepient: &String, key: &[u8], buff: &[u8]) -> Vec<u8> {
	let mut payload = recepient.as_bytes().to_vec();
	payload.append(&mut encryption::encrypt(key, buff));
	payload
}

pub struct InnerClient {
	keys: HashMap<String, Vec<u8>>,
	writer: Option<OwnedWriteHalf>,
}
impl InnerClient {
	pub fn new() -> Self {
		Self {
			keys: HashMap::new(),
			writer: None,
		}
	}

	pub fn set_writer(&mut self, writer: OwnedWriteHalf) {
		self.writer = Some(writer);
	}

	pub fn set_key(&mut self, recepient: String, key: Vec<u8>) {
		self.keys.insert(recepient, key);
	}

	pub fn relay_data_to_all(&mut self, buff: &[u8]) {
		if let Some(write) = &self.writer {
			for (recepient, key) in self.keys.iter() {
				let payload = make_payload(recepient, key, buff);

				match write.try_write(&payload) {
					Ok(_) => {}
					Err(e) => eprintln!("{}", e.to_string()),
				};
			}
		}
	}
}
