use crate::tcp_client::{InnerClient, OuterClient};
use lib::io;
use rsa::{pkcs8::FromPublicKey, RsaPublicKey};

mod socket;
mod tcp_client;
mod window;

const ADDRESS: &str = "127.0.0.1:8080";

#[tokio::main]
async fn main() {
	let public_key = {
		let file_content = io::read_public_key().expect("unable to find public key");
		RsaPublicKey::from_public_key_pem(&file_content).expect("invalid public key")
	};

	let (window_sender, window_receiver) = window::create_channel();

	let inner = InnerClient::new(window_sender);
	let outer = OuterClient::new(inner);
	let client_sender = outer.sender();

	tokio::spawn(async move {
		let mut socket = match socket::Socket::new(ADDRESS.to_string(), public_key) {
			Ok(socket) => socket,
			Err(e) => panic!("{}", e),
		};

		match socket.listen(outer).await {
			Ok(()) => println!("successfully ran and ended client"),
			Err(e) => panic!("{}", e),
		};
	});

	let app = window::create_window(window_receiver);
	window::run_window(app, client_sender)

	// loop {}
}
