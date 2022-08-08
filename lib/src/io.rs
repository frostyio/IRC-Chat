use std::{fs, path::Path};

const PUBLIC_KEY: &str = "key.pub";
const PRIVATE_KEY: &str = "key";

pub fn read_file(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
	Ok(fs::read_to_string(path)?)
}

pub fn read_public_key() -> Result<String, Box<dyn std::error::Error>> {
	read_file(Path::new(PUBLIC_KEY))
}

pub fn read_private_key() -> Result<String, Box<dyn std::error::Error>> {
	read_file(Path::new(PRIVATE_KEY))
}
