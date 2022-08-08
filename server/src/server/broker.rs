use super::{
	client::{listen_client, Client},
	Event, InnerServer, Receiver,
};
use bytes::BytesMut;
use lib::hex_hash;
use rand_core::OsRng;
use rsa::{PaddingScheme, PublicKeyParts};
use tokio::io::AsyncReadExt;
use x25519_dalek::{EphemeralSecret, PublicKey};

pub async fn broker(mut receiver: Receiver, mut inner_server: InnerServer) {
	while let Some(event) = receiver.recv().await {
		match event {
			Event::NewPeer(sender, stream, key) => match stream.peer_addr() {
				Ok(addr) => {
					let id = hex_hash(addr.to_string().as_bytes());

					let (mut read, write) = stream.into_split();

					// key exchange

					// generate our secret/public key
					let secret = EphemeralSecret::new(OsRng);
					let public = PublicKey::from(&secret);

					// write our public key to the client
					let _ = write.try_write(public.as_bytes());

					// get the encryption size based of our private key & read the decrypted public key into the public buffer
					let size = key.size();
					let mut buff = BytesMut::with_capacity(size);
					let _ = read.readable().await;
					let _ = read.read_buf(&mut buff).await;

					let padding = PaddingScheme::new_pkcs1v15_encrypt();
					let their_public_input = match key.decrypt(padding, &buff) {
						Ok(bytes) => bytes,
						Err(e) => return eprintln!("error: {:?}", e.to_string()),
					};
					let mut public_buffer = [0u8; 32];
					(0usize..32usize)
						.into_iter()
						.for_each(|i| public_buffer[i] = their_public_input[i].clone());

					// get the shared secret!
					let their_public = PublicKey::from(public_buffer);
					let shared_secret = secret.diffie_hellman(&their_public);

					let client = Client::new(id.clone(), write, shared_secret.as_bytes());
					tokio::spawn(listen_client(id.clone(), sender, read));
					inner_server.add_client(id.clone(), client);

					// temporary
					inner_server.relay_data_to_recepient(&id, b"hello world");
				}
				Err(_) => {}
			},
		}
	}
}
