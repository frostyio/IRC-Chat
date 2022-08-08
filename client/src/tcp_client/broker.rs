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
				println!("sending hello world!");
				inner_client.relay_data_to_all(username.as_bytes())
			}
		}
	}
}
