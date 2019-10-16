use std::sync::Mutex;
use std::time::{Duration, Instant};

use actix::Arbiter;
use actix_web::error::ErrorInternalServerError;
use actix_web::web::{Bytes, Data};
use serde::Serialize;
use tokio::prelude::*;
use tokio::sync::mpsc;
use tokio::timer::Interval;
use chrono::{Datelike, TimeZone};

pub struct Broadcaster {
	pub users: Vec<User>,
	pub history: Vec<UserMsg>,
}

impl Default for Broadcaster {
	fn default() -> Self {
		Broadcaster {
			users: Vec::new(),
			history: Vec::new(),
		}
	}
}

impl Broadcaster {
	pub fn new() -> Data<Mutex<Self>> {
		let broadcaster = Data::new(Mutex::new(Broadcaster::default()));
		Broadcaster::start_heartbeat(broadcaster.clone());
		Broadcaster::start_history_cleaner(broadcaster.clone());
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

	fn start_history_cleaner(broadcaster: Data<Mutex<Broadcaster>>) {
		let now = chrono::Local::now();
		let midnight = chrono::Local.ymd(now.year(), now.month(), now.day()).and_hms(23, 59, 59);
		let now_std = Instant::now();
		let i = now_std + ((midnight - now).to_std().expect("Failed to calc date"));

		let task = Interval::new(i, Duration::from_secs(60 * 60 * 24))
			.for_each(move |_instant| {
				broadcaster.lock().unwrap().history.clear();
				Ok(())
			})
			.map_err(|_| ());

		Arbiter::spawn(task);
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

		tx.try_send(event_data(Msg::connected(&self.history)))
			.unwrap();

		self.users.push(User {
			nick: String::from(nick),
			sender: tx,
		});

		UserDataStream(rx)
	}

	pub fn send(&mut self, nick: &str, msg: &str) {
		self.history.push(UserMsg {
			nick: String::from(nick),
			msg: String::from(msg),
		});

		let msg = event_data(Msg::user_msg(nick, msg));

		for user in &mut self.users {
			user.sender.try_send(msg.clone()).unwrap_or(());
		}
	}
}

#[derive(Serialize)]
pub struct Msg<T> {
	r#type: MsgType,
	data: Option<T>,
}

impl<'a> Msg<UserMsgRef<'a>> {
	pub fn user_msg(nick: &'a str, msg: &'a str) -> Self {
		let user_msg = UserMsgRef { nick, msg };
		Msg {
			r#type: MsgType::Message,
			data: Some(user_msg),
		}
	}
}

impl Msg<()> {
	fn new(r#type: MsgType) -> Self {
		Msg { r#type, data: None }
	}
}

impl<'a> Msg<&'a Vec<UserMsg>> {
	pub fn connected(history: &'a Vec<UserMsg>) -> Self {
		Msg {
			r#type: MsgType::Connected,
			data: Some(history),
		}
	}
}

#[derive(Serialize)]
enum MsgType {
	Connected,
	Ping,
	Message,
	YourNickIsTaken,
}

#[derive(Serialize)]
pub struct UserMsgRef<'a> {
	pub nick: &'a str,
	pub msg: &'a str,
}

#[derive(Serialize)]
pub struct UserMsg {
	pub nick: String,
	pub msg: String,
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
