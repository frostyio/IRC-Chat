use super::InnerClient;
use lib::encoding::Instruction::{self, *};

pub fn handle_feed(_client: &mut InnerClient, feed: Vec<Instruction>) {
	for instr in feed {
		match instr {
			RecieveMessage(username, content) => {
				println!("receiving message: {username}: {content}")
			}
			_ => {}
		}
	}
}
