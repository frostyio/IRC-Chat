use super::{Instruction, Opcodes, Writer};

pub struct Encoder {
	pub writer: Writer,
}

impl Encoder {
	pub fn from_feed(feed: Vec<Instruction>) -> Self {
		let mut s = Self {
			writer: Writer::new(),
		};
		s.parse(feed);

		s
	}

	fn parse(&mut self, feed: Vec<Instruction>) {
		for instruction in feed {
			match instruction {
				Instruction::Instantiate(username) => {
					self.writer.short(Opcodes::Instantiate as u16);
					self.writer.string(&username);
				}
				Instruction::SendMessage(message) => {
					self.writer.short(Opcodes::SendMessage as u16);
					self.writer.string(&message);
				}
				Instruction::ReceiveMessage(sender, message) => {
					self.writer.short(Opcodes::ReceiveMessage as u16);
					self.writer.string(&sender);
					self.writer.string(&message);
				}
				_ => {}
			}
		}
	}
}
