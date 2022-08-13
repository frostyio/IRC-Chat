use super::Sender;
use super::{
	client::{listen_client, Client},
	Event, InnerServer, Receiver,
};
use bytes::BytesMut;
use lib::{hash, hex};
use rand_core::OsRng;
use rsa::{PaddingScheme, PublicKeyParts, RsaPrivateKey};
use std::error::Error;
use tokio::{io::AsyncReadExt, net::TcpStream};
use x25519_dalek::{EphemeralSecret, PublicKey};

pub async fn new_peer(
	inner_server: &mut InnerServer,
	sender: Sender,
	stream: TcpStream,
	key: RsaPrivateKey,
	server_id: String,
) -> Result<(), Box<dyn Error>> {
	let (mut read, write) = stream.into_split();
	// key exchange

	// generate our secret/public key
	let secret = EphemeralSecret::new(OsRng);
	let public = PublicKey::from(&secret);

	// write our public key to the client
	let _ = write.try_write(public.as_bytes());

	// write our id to the client
	let _ = write.try_write(server_id.clone().as_bytes());

	// get the encryption size based of our private key & read the decrypted public key into the public buffer
	let size = key.size() + 64; // adding in their id
	println!("size: {} + 64", key.size());

	let mut buff = vec![0u8; size];
	let _ = read.readable().await;
	if size != read.read_exact(&mut buff).await? {
		panic!(
			"did not match full buffer size, got: {}, size: {}",
			buff.len(),
			size
		);
	}

	println!("size of buffer is: {}", buff.len());
	let encrypted_buf = &buff[0..key.size()];
	let id_buf = &buff[key.size()..size];
	println!("id_buf: {:?}", id_buf);
	let id = String::from_utf8(id_buf.to_vec())?;

	let padding = PaddingScheme::new_pkcs1v15_encrypt();
	let their_public_input = match key.decrypt(padding, encrypted_buf) {
		Ok(bytes) => bytes,
		Err(e) => return Err(Box::new(e)),
	};

	// change into a 32 byte array
	let mut public_buffer = [0u8; 32];
	(0usize..32usize)
		.into_iter()
		.for_each(|i| public_buffer[i] = their_public_input[i].clone());

	// get the shared secret!
	let their_public = PublicKey::from(public_buffer);
	let shared_secret = hash(secret.diffie_hellman(&their_public).as_bytes());

	#[cfg(debug_assertions)]
	println!(
		"the client's ({}) shared secret is: {}",
		id.clone(),
		hex(&shared_secret).as_str()
	);

	let client = Client::new(id.clone(), write, &shared_secret, sender.clone(), server_id);
	tokio::spawn(listen_client(id.clone(), sender, read));
	inner_server.add_client(id.clone(), client);

	Ok(())
}

pub async fn broker(mut receiver: Receiver, mut inner_server: InnerServer) {
	while let Some(event) = receiver.recv().await {
		match event {
			Event::SetServerId(id) => inner_server.set_id(id),
			Event::NewPeer(sender, stream, key) => {
				let server_id = inner_server.get_id().to_string();
				let _ = new_peer(&mut inner_server, sender, stream, key, server_id.clone()).await;
			}
			Event::RelayFeed(id, recepient_id, buf) => {
				if &recepient_id == inner_server.get_id() {
					inner_server.read_feed(&id, buf).await;
				} else {
					// future E2EE
					panic!(
						"not yet implemented, recepient id: {:?}, server id: {:?}",
						recepient_id,
						inner_server.get_id()
					);
				}
			}
			Event::SendToAll(data) => inner_server.send_instructions_to_all(data),
			Event::SendToOthers(sender_id, data) => {
				inner_server.send_instructions_to_others(&sender_id, data)
			}
		}
	}
}
