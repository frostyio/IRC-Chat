use crate::tcp_client::{server::listen_server, Event, OuterClient};
use lib::{hash, hex, hex_hash};
use rand_core::OsRng;
use rsa::{PaddingScheme, PublicKey, RsaPublicKey};
use std::{
	error::Error,
	io::{IoSlice, Read, Write},
	net::{AddrParseError, SocketAddr},
};
use tokio::net::{tcp::OwnedReadHalf, TcpStream};
use x25519_dalek::{EphemeralSecret, PublicKey as DHPublicKey};

pub fn to_socket_addr(address: String) -> Result<std::net::SocketAddr, AddrParseError> {
	address.parse::<SocketAddr>()
}

pub struct Socket {
	socket_addr: SocketAddr,
	public_key: RsaPublicKey,
	outer: Option<OuterClient>,
	read: Option<OwnedReadHalf>,
}

impl Socket {
	pub fn new(address: String, public_key: RsaPublicKey) -> Result<Self, Box<dyn Error>> {
		let socket_addr = to_socket_addr(address)?;

		Ok(Self {
			socket_addr,
			public_key,
			outer: None,
			read: None,
		})
	}

	pub async fn initalize(
		&mut self,
		outer: OuterClient,
	) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
		// tokio's TcpStream does not work here as various things do not implement Send + Sync when awaiting for certain closures I guess?
		// needed at key exchange, would error when awaiting
		let mut std_stream = std::net::TcpStream::connect(self.socket_addr)?;
		std_stream.set_nonblocking(true)?;

		println!(
			"IRC chat client listening on {}",
			std_stream.local_addr().unwrap().to_string()
		);

		let me = hex_hash(
			std_stream
				.local_addr()
				.expect("unable to get my address")
				.to_string()
				.as_bytes(),
		);

		// key exchange
		let (shared_secret, receipent) = {
			let mut rng = rand2::thread_rng();

			// create our private key & encrypt using the server's public key
			let secret = EphemeralSecret::new(OsRng);
			let public = DHPublicKey::from(&secret);
			let public_bytes = public.as_bytes();

			/*
			DH Key Exchange - current place
				1. server & client generate ephemeral key pairs
				2. server & client sends public key
				3. now both have the shared secret

			problems:
				MITM can attack this by claiming to be the server to the client and generate a fake connection to them
				and the server and forwards the data

			solution:
				1. the server will have a pre established public key already given to the client
				2. the client will connect to the server and send their DH public key encrypted via the server's public key
				3. the server will decrypt, and send back it's DH public key, thus a shared secret is acquired

				because the client is only sending encrypted data, a MITM cannot decrypt their public key (as only
				the server has the private key to decrypt it) breaking the key exchange

			theoretically the same process (but weaker) as the SSL handshake without the CA certificate verification to get the public
			key as the user gives the public keym, right?
			*/

			let padding = PaddingScheme::new_pkcs1v15_encrypt();
			let public_encrypted = self
				.public_key
				.encrypt(&mut rng, padding, &public_bytes[..])?;
			// std_stream.try_write(&public_encrypted)?;
			std_stream.write_vectored(&[IoSlice::new(&public_encrypted)])?;

			// send the client id along
			std_stream.write(me.as_bytes())?;

			// now that we have sent our public key encrypted using the dedicated server's public key
			// we wait for a response for their DHE public key
			let mut buff = [0u8; 32];
			let mut got_key_buf = false;
			println!("getting key...");
			while !got_key_buf {
				match std_stream.peek(&mut [0u8; 32]) {
					Ok(_) => {
						let _ = std_stream.read_exact(&mut buff);
						got_key_buf = true;
					}
					Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
						println!("blocking... retrying");
					}
					Err(_) => panic!("weird error?"),
				};

				std::thread::sleep(std::time::Duration::from_millis(250));
			}
			println!("got key!");
			let their_public = DHPublicKey::from(buff);

			// same as above except for their hashed id
			let mut buff = [0u8; 64];
			let mut got_id_buf = false;
			println!("getting id...");
			while !got_id_buf {
				match std_stream.peek(&mut [0u8; 64]) {
					Ok(_) => {
						let _ = std_stream.read_exact(&mut buff);
						got_id_buf = true;
					}
					Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
					Err(_) => panic!("weird error?"),
				};

				std::thread::sleep(std::time::Duration::from_millis(250));
			}
			println!("got id!");
			let id = String::from_utf8(buff.to_vec())?;

			// should zeroize our secret
			(hash(secret.diffie_hellman(&their_public).as_bytes()), id)
		};

		let stream = TcpStream::from_std(std_stream)?;
		let (read, write) = stream.into_split();
		outer.send(Event::SetWriter(write))?;

		#[cfg(debug_assertions)]
		println!(
			"the server's ({}) shared secret is: {}\nmine address is: {}",
			receipent.clone(),
			hex(&shared_secret),
			me
		);
		outer.send(Event::SetSharedKey(receipent, shared_secret))?;

		self.outer = Some(outer);
		self.read = Some(read);
		Ok(())
	}

	pub async fn listen(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
		let outer = self.outer.take().expect("Socket has not been initalized");
		let read = self.read.take().expect("Socket has not been initalized");

		println!("listening to server...");
		listen_server(read, outer).await?;
		Ok(())
	}
}
