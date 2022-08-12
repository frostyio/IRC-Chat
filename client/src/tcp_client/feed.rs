use super::InnerClient;
use lib::encoding::Instruction::{self, *};

pub fn handle_feed(client: &mut InnerClient, feed: Vec<Instruction>) {
	for instr in feed {
		match instr {
			_ => {}
		}
	}
}
