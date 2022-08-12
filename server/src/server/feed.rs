use super::client::Client;
use lib::encoding::Instruction::{self, *};

pub async fn handle_feed(client: &mut Client, feed: Vec<Instruction>) {
	for instr in feed {
		match instr {
			Instantiate(username) => {
				client.username = username;

				use tokio::time::{sleep, Duration};
				sleep(Duration::from_millis(1000)).await;
				println!("sending message!");
				client.send_local_message(format!("Hi {}", client.username));
			}
			SendMessage(content) => client.send_message(content),
			_ => {}
		}
	}
}
