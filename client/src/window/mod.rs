use crate::socket;
use crate::tcp_client::{Event, Sender};
use crate::tcp_client::{InnerClient, OuterClient};
use eframe::egui::{self, Style, Ui, Visuals};
use lib::io;
use rsa::{pkcs8::FromPublicKey, RsaPublicKey};
use tokio::sync::mpsc::{self, error::TryRecvError, UnboundedReceiver, UnboundedSender};

const WINDOW_NAME: &str = "Chat";

pub enum WindowEvent {
	DisplayMessage(String, String),
}

pub type WindowReceiver = UnboundedReceiver<WindowEvent>;
pub type WindowSender = UnboundedSender<WindowEvent>;

pub fn create_channel() -> (WindowSender, WindowReceiver) {
	mpsc::unbounded_channel::<WindowEvent>()
}

pub fn run_window(window: Application) {
	eframe::run_native(
		WINDOW_NAME,
		eframe::NativeOptions::default(),
		Box::new(|cc| {
			let style = Style {
				visuals: Visuals::dark(),
				..Style::default()
			};
			cc.egui_ctx.set_style(style);
			Box::new(window)
		}),
	);
}

pub fn create_tcp_client(username: String, server: String) -> (Sender, WindowReceiver) {
	let (window_sender, window_receiver) = create_channel();
	let public_key = {
		let file_content = io::read_public_key().expect("unable to find public key");
		RsaPublicKey::from_public_key_pem(&file_content).expect("invalid public key")
	};

	let inner = InnerClient::new(window_sender);
	let outer = OuterClient::new(inner);
	let sender = outer.sender();

	let sender_clone = sender.clone();
	tokio::spawn(async move {
		let mut socket = match socket::Socket::new(server, public_key) {
			Ok(socket) => socket,
			Err(e) => panic!("{}", e),
		};
		let _ = socket.initalize(outer).await;
		let _ = sender_clone.send(Event::Instantiate(username));

		println!("starting socket listen...");
		match socket.listen().await {
			Ok(()) => println!("successfully ran and ended client"),
			Err(e) => panic!("{}", e),
		};
	});

	(sender, window_receiver)
}

pub struct Message(pub String, pub String);

pub struct Application {
	current_username: String,
	current_ip: String,
	current_message: String,
	logged_in: bool,
	messages: Vec<Message>,
	client_sender: Option<Sender>, // to send things to the tcp client
	window_receiver: Option<WindowReceiver>, // receive message updates, etc
}

impl Application {
	pub fn new() -> Self {
		Self {
			current_username: "".to_string(),
			current_ip: "".to_string(),
			current_message: "".to_string(),
			logged_in: false,
			messages: vec![],
			client_sender: None,
			window_receiver: None,
		}
	}

	fn set_client_sender(&mut self, client_sender: Sender) {
		self.client_sender = Some(client_sender);
	}

	fn set_window_receiver(&mut self, window_receiver: WindowReceiver) {
		self.window_receiver = Some(window_receiver);
	}
}

impl Application {
	fn render_chat(&mut self, ui: &mut Ui) {
		egui::ScrollArea::vertical().show(ui, |ui| {
			if self.messages.len() == 0 {
				ui.label("There are no messages here currently :(");
			} else {
				for message in &self.messages {
					ui.horizontal(|ui| {
						ui.label(message.0.clone());
						ui.label(message.1.clone());
					});
				}
			}
		});

		ui.add_space(5.0);
		let input = egui::TextEdit::singleline(&mut self.current_message)
			.hint_text("Message")
			.show(ui)
			.response;

		if input.lost_focus() && input.ctx.input().key_pressed(egui::Key::Enter) {
			// self.messages
			// 	.push(Message("frosty".to_string(), self.current_message.clone()));

			if let Some(sender) = &self.client_sender {
				let _ = sender.send(Event::SendMessage(self.current_message.clone()));
			}

			self.current_message = "".to_string();
			input.request_focus();
		}
	}

	fn render_login(&mut self, ui: &mut Ui) {
		ui.add_space(10.0);
		ui.heading("Login");

		egui::TextEdit::singleline(&mut self.current_ip)
			.hint_text("Server IP")
			.show(ui);

		egui::TextEdit::singleline(&mut self.current_username)
			.hint_text("Username")
			.show(ui);

		if ui.button("Login").clicked() {
			let (client_sender, window_receiver) =
				create_tcp_client(self.current_username.clone(), self.current_ip.clone());
			self.set_client_sender(client_sender);
			self.set_window_receiver(window_receiver);
			self.logged_in = true;
		}
	}
}

impl eframe::App for Application {
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		if let Some(recv) = &mut self.window_receiver {
			loop {
				match recv.try_recv() {
					Ok(event) => match event {
						WindowEvent::DisplayMessage(auth, cont) => {
							self.messages.push(Message(auth, cont))
						}
					},
					Err(TryRecvError::Empty) => break,
					Err(TryRecvError::Disconnected) => break, // try to reconnect in the future
				}
			}
		}

		egui::CentralPanel::default().show(ctx, |ui| {
			ui.heading(WINDOW_NAME);

			if !self.logged_in {
				self.render_login(ui);
			} else {
				self.render_chat(ui);
			}
		});
	}
}
