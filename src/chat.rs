use std::sync::Mutex;
use std::time::{Duration, Instant};

use actix::Arbiter;
use actix_web::error::ErrorInternalServerError;
use actix_web::web::{Bytes, Data};
use chrono::{prelude::*, Datelike, TimeZone};
use serde::Serialize;
use tokio::prelude::*;
use tokio::sync::mpsc;
use tokio::timer::Interval;
use crate::models;

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
		let midnight = chrono::Local
			.ymd(now.year(), now.month(), now.day())
			.and_hms(23, 59, 59);
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

	pub fn new_user<'a>(&'a mut self, nick: &str) -> (UserDataStream, &'a User) {
		let (mut tx, rx) = mpsc::channel(100);

		tx.try_send(event_data(Msg::new(MsgType::Ping)))
			.unwrap();

		self.users.push(User {
			nick: String::from(nick),
			sender: tx.clone(),
		});

		(UserDataStream(rx), &self.users.last().unwrap())
	}

	pub fn send(&mut self, nick: String, msg: String) {
		let user_msg = UserMsg {
			nick,
			msg,
			time: Utc::now(),
		};

		let msg = event_data(Msg::user_msg(&user_msg));

		self.history.push(user_msg);

		for user in &mut self.users {
			user.sender.try_send(msg.clone()).unwrap_or(());
		}
	}

	pub fn send_paste(&mut self, paste: models::Paste) {
		let msg = event_data(Msg::paste_msg(&paste));

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

impl<'a> Msg<&'a UserMsg> {
	pub fn user_msg(msg: &'a UserMsg) -> Self {
		Msg {
			r#type: MsgType::Message,
			data: Some(msg),
		}
	}
}

impl Msg<()> {
	fn new(r#type: MsgType) -> Self {
		Msg { r#type, data: None }
	}
}

impl<'a> Msg<(&'a Vec<UserMsg>, &'a Vec<models::Paste>)> {
	pub fn connected(history: &'a Vec<UserMsg>, pastes: &'a Vec<models::Paste>) -> Self {
		Msg {
			r#type: MsgType::Connected,
			data: Some((history, pastes)),
		}
	}
}

impl<'a> Msg<&'a models::Paste> {
	pub fn paste_msg(paste: &'a models::Paste) -> Self {
		Msg {
			r#type: MsgType::Paste,
			data: Some(paste),
		}
	}
}

#[derive(Serialize)]
enum MsgType {
	Connected,
	Ping,
	Message,
	Paste,
}

#[derive(Serialize)]
pub struct UserMsg {
	pub nick: String,
	pub msg: String,
	pub time: DateTime<Utc>,
}

#[derive(Clone)]
pub struct User {
	pub nick: String,
	pub sender: mpsc::Sender<Bytes>,
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
