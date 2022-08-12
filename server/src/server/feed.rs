use super::client::Client;
use lib::encoding::Instruction::{self, *};

pub fn handle_feed(client: &mut Client, feed: Vec<Instruction>) {
	for instr in feed {
		match instr {
			Instantiate(username) => client.username = username,
			SendMessage(content) => client.send_message(content),
			_ => {}
		}
	}
}
