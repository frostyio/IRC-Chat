use crate::window::WindowEvent;

use super::InnerClient;
use lib::encoding::Instruction::{self, *};

pub fn handle_feed(client: &mut InnerClient, feed: Vec<Instruction>) {
	for instr in feed {
		match instr {
			ReceiveMessage(username, content) => {
				println!("receiving message: {username}: {content}");
				let _ = client
					.window_sender
					.send(WindowEvent::DisplayMessage(username, content));
			}
			_ => {}
		}
	}
}
