use crate::tcp_client::{Event, Sender};
use eframe::egui::{self, Style, Visuals};
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

pub fn create_window(receiver: WindowReceiver) -> Application {
	let application = Application::new(receiver);

	application
}

pub fn run_window(mut window: Application, client_sender: Sender) {
	window.set_client_sender(client_sender);

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
pub struct Message(pub String, pub String);

pub struct Application {
	current_message: String,
	messages: Vec<Message>,
	client_sender: Option<Sender>,   // to send things to the tcp client
	window_receiver: WindowReceiver, // receive message updates, etc
}

impl Application {
	fn new(window_receiver: WindowReceiver) -> Self {
		Self {
			current_message: "".to_string(),
			messages: vec![],
			client_sender: None,
			window_receiver,
		}
	}

	fn set_client_sender(&mut self, client_sender: Sender) {
		self.client_sender = Some(client_sender);
	}
}

impl eframe::App for Application {
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		loop {
			match self.window_receiver.try_recv() {
				Ok(event) => match event {
					WindowEvent::DisplayMessage(auth, cont) => {
						self.messages.push(Message(auth, cont))
					}
				},
				Err(TryRecvError::Empty) => break,
				Err(TryRecvError::Disconnected) => break, // try to reconnect in the future
			}
		}

		egui::CentralPanel::default().show(ctx, |ui| {
			ui.heading(WINDOW_NAME);

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
			ui.horizontal(|ui| {
				let input = egui::TextEdit::singleline(&mut self.current_message)
					.hint_text("Message")
					.show(ui)
					.response;

				if input.lost_focus() && input.ctx.input().key_pressed(egui::Key::Enter) {
					let content = self.current_message.clone();
					// self.messages
					// 	.push(Message("frosty".to_string(), content.clone()));
					self.current_message = "".to_string();
					input.request_focus();

					if let Some(sender) = &self.client_sender {
						let _ = sender.send(Event::SendMessage(content));
					}
				}
			});
		});
	}
}
