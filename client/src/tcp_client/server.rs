use super::{Event, OuterClient};
use lib::stream::{self, StreamOperation};
use tokio::net::tcp::OwnedReadHalf;

pub async fn listen_server(
	mut read: OwnedReadHalf,
	outer: OuterClient,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
	loop {
		let (sender_id, encrypted_buf) = match stream::read_stream(&mut read).await {
			Ok((r, e)) => (r, e),
			Err(StreamOperation::Continue) => continue,
			Err(StreamOperation::Break) => break,
		};

		outer.send(Event::ReadFeed(sender_id, encrypted_buf))?;
	}

	Ok(())
}
