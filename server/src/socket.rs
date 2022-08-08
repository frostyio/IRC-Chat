use crate::server::{Event, OuterServer};
use rsa::RsaPrivateKey;
use std::{
	error::Error,
	net::{AddrParseError, SocketAddr},
};
use tokio::net::TcpListener;

pub fn to_socket_addr(address: String) -> Result<std::net::SocketAddr, AddrParseError> {
	address.parse::<SocketAddr>()
}

pub struct Socket {
	listener: TcpListener,
	private_key: RsaPrivateKey,
}

impl Socket {
	pub async fn new(address: String, private_key: RsaPrivateKey) -> Result<Self, Box<dyn Error>> {
		let socket_addr = to_socket_addr(address)?;
		let listener = TcpListener::bind(socket_addr).await?;

		Ok(Self {
			listener,
			private_key,
		})
	}

	pub async fn listen(&self, outer: OuterServer) -> Result<(), Box<dyn std::error::Error>> {
		println!(
			"IRC chat server listening on {}",
			self.listener.local_addr().unwrap().to_string()
		);

		loop {
			match self.listener.accept().await {
				Ok((stream, _)) => outer.send(Event::NewPeer(
					outer.sender(),
					stream,
					self.private_key.to_owned(),
				))?,
				Err(_) => {}
			};
		}
	}
}
