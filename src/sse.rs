use std::sync::Mutex;
use std::time::{Duration, Instant};

use actix::Arbiter;
use actix_web::error::ErrorInternalServerError;
use actix_web::web::{Bytes, Data};
use serde::Serialize;
use tokio::prelude::*;
use tokio::sync::mpsc;
use tokio::timer::Interval;

pub struct Broadcaster {
	pub users: Vec<User>,
}

impl Default for Broadcaster {
	fn default() -> Self {
		Broadcaster { users: Vec::new() }
	}
}

impl Broadcaster {
	pub fn new() -> Data<Mutex<Self>> {
		let broadcaster = Data::new(Mutex::new(Broadcaster::default()));
		Broadcaster::start_heartbeat(broadcaster.clone());
		broadcaster
	}

	fn start_heartbeat(broadcaster: Data<Mutex<Broadcaster>>) {
		let task = Interval::new(Instant::now(), Duration::from_secs(10))
			.for_each(move |_instant| {
				broadcaster.lock().unwrap().remove_dead_users();
				Ok(())
			})
			.map_err(|e| println!("Heartbeat error: {}", e));

		Arbiter::spawn(task)
	}

	fn remove_dead_users(&mut self) {
		self.users = self
			.users
			.iter()
			.filter_map(|user| {
				let mut user = user.clone();
				if user
					.sender
					.try_send(event_data(Msg::new(MsgType::Ping)))
					.is_ok()
				{
					Some(user)
				} else {
					None
				}
			})
			.collect::<Vec<User>>();
	}

	pub fn new_user(&mut self, nick: &str) -> UserDataStream {
		let (mut tx, rx) = mpsc::channel(100);

		tx.try_send(event_data(Msg::new(MsgType::Connected)))
			.unwrap();

		self.users.push(User {
			nick: String::from(nick),
			sender: tx,
		});

		UserDataStream(rx)
	}

	pub fn send(&mut self, nick: &str, msg: &str) {
		let msg = event_data(Msg::user_msg(nick, msg));

		for user in &mut self.users {
			user.sender.try_send(msg.clone()).unwrap_or(());
		}
	}
}

#[derive(Serialize)]
pub struct Msg<T> {
	pub r#type: MsgType,
	pub data: Option<T>,
}

impl<'a> Msg<UserMsg<'a>> {
	pub fn user_msg(nick: &'a str, msg: &'a str) -> Self {
		let user_msg = UserMsg { nick, msg };
		Msg {
			r#type: MsgType::Message,
			data: Some(user_msg),
		}
	}
}

impl Msg<()> {
	pub fn new(r#type: MsgType) -> Self {
		Msg { r#type, data: None }
	}
}

#[derive(Serialize)]
pub enum MsgType {
	Connected,
	Ping,
	Message,
	YourNickIsTaken,
}

#[derive(Serialize)]
pub struct UserMsg<'a> {
	pub nick: &'a str,
	pub msg: &'a str,
}

#[derive(Clone)]
pub struct User {
	pub nick: String,
	sender: mpsc::Sender<Bytes>,
}

pub struct UserDataStream(mpsc::Receiver<Bytes>);

impl Stream for UserDataStream {
	type Item = Bytes;
	type Error = actix_web::Error;

	fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
		self.0.poll().map_err(ErrorInternalServerError)
	}
}

pub fn event_data(msg: Msg<impl Serialize>) -> Bytes {
	Bytes::from(
		[
			"data: ",
			serde_json::to_string(&msg).unwrap().as_str(),
			"\n\n",
		]
		.concat(),
	)
}
