use super::{Instruction, Opcodes, Reader};
use num_traits::FromPrimitive;

pub struct Decoder {
	reader: Reader,
	pub feed: Vec<Instruction>,
}

impl Decoder {
	pub fn from_bytes(buffer: Vec<u8>) -> Self {
		let mut s = Self {
			reader: Reader::from_bytes(buffer),
			feed: vec![],
		};
		s.parse();

		s
	}

	fn parse(&mut self) {
		while self.reader.has_next() {
			let opcode = self.reader.short();

			let instruction = match FromPrimitive::from_u16(opcode) {
				Some(Opcodes::Instantiate) => Instruction::Instantiate(self.reader.string()),
				Some(Opcodes::SendMessage) => Instruction::SendMessage(self.reader.string()),
				Some(Opcodes::ReceiveMessage) => {
					Instruction::ReceiveMessage(self.reader.string(), self.reader.string())
				}
				_ => Instruction::NOP,
			};

			self.feed.push(instruction)
		}
	}
}
