use lib::{
	encoding::{Decoder, Instruction},
	encryption::decrypt,
};

use crate::tcp_client::feed::handle_feed;

use super::{Event, InnerClient, Receiver};

pub async fn broker(mut receiver: Receiver, mut inner_client: InnerClient) {
	while let Some(event) = receiver.recv().await {
		match event {
			Event::SetWriter(writer) => inner_client.set_writer(writer),
			Event::SetSharedKey(recepient, key) => {
				// currently the only recepient is the server
				if key.len() != 32 {
					println!("invalid key given: {:?} size: {:?}", key, key.len());
					return;
				}
				inner_client.set_key(recepient, key);
			}
			Event::Instantiate(username) => {
				// sort of a hello world
				inner_client.send_instructions_to_all(vec![Instruction::Instantiate(username)]);
			}
			Event::ReadFeed(sender_id, buf) => {
				if let Some(secret) = inner_client.get_key(&sender_id) {
					let data = decrypt(secret, buf);
					let feed = Decoder::from_bytes(data);
					handle_feed(&mut inner_client, feed.feed);
				}
			}
		}
	}
}
