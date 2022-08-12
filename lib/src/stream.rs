use bytes::BytesMut;
use std::io::ErrorKind;
use tokio::{io::AsyncReadExt, net::tcp::OwnedReadHalf};

pub enum StreamOperation {
	Continue,
	Break,
}

pub async fn read_stream(stream: &mut OwnedReadHalf) -> Result<(String, Vec<u8>), StreamOperation> {
	// gathering next available sizing hint
	let _ = stream.readable().await;
	let mut encrypted_size_hint = [0 as u8; 8]; // 64 bit size hint | client/src/tcp_client/mod.rs -> make_payload
	match stream.read_exact(&mut encrypted_size_hint).await {
		Ok(_) => {}
		Err(ref e)
			if e.kind() == ErrorKind::UnexpectedEof || e.kind() == ErrorKind::ConnectionReset =>
		{
			return Err(StreamOperation::Break)
		}
		Err(e) => {
			#[cfg(debug_assertions)]
			eprintln!("error while gathering size hint: {:#?}", e);

			return Err(StreamOperation::Break);
		}
	};

	// gathering the id
	let mut id = [0 as u8; 64]; // 512 bit id | client/src/tcp_client/mod.rs -> make_payload
	match stream.read_exact(&mut id).await {
		Ok(_) => {}
		Err(ref e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
			return Err(StreamOperation::Break)
		}
		Err(e) => {
			#[cfg(debug_assertions)]
			eprintln!("error while gathering id: {:#?}", e);

			return Err(StreamOperation::Continue);
		}
	};

	// gathering the encrypted buffer
	let size = u64::from_be_bytes(encrypted_size_hint);
	let mut encrypted_buf = BytesMut::with_capacity(size as usize);
	match stream.read_buf(&mut encrypted_buf).await {
		Ok(_) => {}
		Err(e) => {
			#[cfg(debug_assertions)]
			eprintln!("error while gathering encrypted buffer: {:#?}", e);

			return Err(StreamOperation::Continue);
		}
	}

	// decryption & sending to server client for feed to be read
	let the_id = match String::from_utf8(id.to_vec()) {
		Ok(id) => id,
		Err(e) => {
			#[cfg(debug_assertions)]
			eprintln!("Invalid recepient id: {:#?}", e);

			return Err(StreamOperation::Continue);
		}
	};

	Ok((the_id, encrypted_buf.to_vec()))
}
