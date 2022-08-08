## IRC Chat
### Security is of least concern (although I still try I have no experience). Do not use for production.

Needs an RSA key-pair in root directory named key & key.pub
The client will only use the key.pub while the server will only use the private key

cargo run --bin server
cargo run --bin client