mod socket;
mod tcp_client;
mod window;

#[tokio::main]
async fn main() {
	window::run_window(window::Application::new());
}
