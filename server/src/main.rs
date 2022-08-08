use lib::io;
use rsa::{pkcs8::FromPrivateKey, PublicKeyParts, RsaPrivateKey};

pub mod server;
pub mod socket;

const ADDRESS: &str = "127.0.0.1:8080";

#[tokio::main]
async fn main() {
	let private_key = {
		let content = io::read_private_key().expect("unable to get private key");
		RsaPrivateKey::from_pkcs8_pem(&content).expect("invalid private key")
	};
	println!(
		"key size is: {} ({} bits)",
		private_key.size(),
		private_key.size() * 8
	);

	let inner = server::InnerServer::new();
	let outer = server::OuterServer::new(inner);

	let socket = match socket::Socket::new(ADDRESS.to_string(), private_key).await {
		Ok(socket) => socket,
		Err(e) => panic!("{}", e),
	};

	match socket.listen(outer).await {
		Ok(()) => println!("successfully ran and ended server"),
		Err(e) => panic!("{}", e),
	}
}
