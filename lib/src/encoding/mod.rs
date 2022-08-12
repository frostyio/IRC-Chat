use num_derive::FromPrimitive;

mod reader;
mod writer;
pub use reader::Reader;
pub use writer::Writer;

mod decode;
mod encoder;
pub use decode::Decoder;
pub use encoder::Encoder;

#[derive(Debug)]
pub enum Instruction {
	NOP,
	Instantiate(String),            // username
	SendMessage(String),            // content
	ReceiveMessage(String, String), // author, content
}

#[derive(FromPrimitive)]
pub enum Opcodes {
	NOP = 0,
	Instantiate = 1,
	SendMessage = 2,
	ReceiveMessage = 3,
}

mod test {
	#[cfg(test)]
	use super::{Decoder, Encoder, Instruction, Reader, Writer};

	#[test]
	fn test_encoder_and_decoder() {
		let feed = vec![Instruction::ReceiveMessage(
			"frosty".to_string(),
			"stinky spike".to_string(),
		)];

		let encoded = Encoder::from_feed(feed).writer.dump();
		let decoded = Decoder::from_bytes(encoded).feed;
		println!("{:#?}", decoded);
	}

	#[test]
	fn test_writer_and_reader() {
		let mut writer = Writer::new();
		writer.u64(5000000);
		writer.byte(200);
		writer.short(500);
		writer.f32(100.52);
		writer.i32(500);
		writer.string("spike stinks");

		let mut reader = Reader::from_bytes(writer.dump());

		print!("{} ", reader.u64());
		print!("{} ", reader.byte());
		print!("{} ", reader.short());
		print!("{} ", reader.f32());
		print!("{} ", reader.i32());
		print!("{} ", reader.string());
		println!();
	}
}
