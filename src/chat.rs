use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::models;
use actix::Arbiter;
use actix_web::web::{Bytes, Data};
use chrono::{prelude::*, Datelike, TimeZone};
use serde::Serialize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::task::Context;
use tokio::macros::support::{Pin, Poll};
use tokio::stream::Stream;
use tokio::sync::mpsc;
use tokio::time;

static ID_COUNTER: AtomicU64 = AtomicU64::new(0);

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
		Arbiter::spawn(Self::heartbeat(broadcaster));
	}

	async fn heartbeat(broadcaster: Data<Mutex<Broadcaster>>) {
		let mut timer = time::interval(Duration::from_secs(10));
		loop {
			let _ = timer.tick().await;
			broadcaster.lock().unwrap().remove_dead_users();
		}
	}

	fn start_history_cleaner(broadcaster: Data<Mutex<Broadcaster>>) {
		Arbiter::spawn(Self::history_cleaner(broadcaster));
	}

	async fn history_cleaner(broadcaster: Data<Mutex<Broadcaster>>) {
		let now = chrono::Local::now();
		let midnight = chrono::Local
			.ymd(now.year(), now.month(), now.day())
			.and_hms(23, 59, 59);
		let now_std = Instant::now();
		let i = now_std + ((midnight - now).to_std().expect("Failed to calc date"));

		let mut timer = time::interval_at(
			time::Instant::from_std(i),
			Duration::from_secs(60 * 60 * 24),
		);
		loop {
			let _ = timer.tick().await;
			broadcaster.lock().unwrap().history.clear();
		}
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

	pub fn new_user(&mut self, nick: &str) -> (UserDataStream, &mut User) {
		let (mut tx, rx) = mpsc::channel(100);

		tx.try_send(event_data(Msg::new(MsgType::Ping))).unwrap();

		self.users.push(User {
			id: ID_COUNTER.fetch_add(1, Ordering::SeqCst),
			nick: String::from(nick),
			color: None,
			sender: tx.clone(),
		});

		(UserDataStream(rx), self.users.last_mut().unwrap())
	}

	pub fn send(&mut self, id: u64, msg: String) {
		let user = self.users.iter_mut().find(|u| u.id == id).unwrap();

		let user_msg = UserMsg {
			nick: user.nick.clone(),
			custom_nick_color: user.color.clone(),
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

impl<'a> Msg<&'a Vec<UserMsg>> {
	pub fn connected(history: &'a Vec<UserMsg>) -> Self {
		Msg {
			r#type: MsgType::Connected,
			data: Some(history),
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

impl<'a> Msg<&'a str> {
	pub fn color_change_msg(color: &'a str) -> Self {
		Msg {
			r#type: MsgType::ColorChange,
			data: Some(color),
		}
	}

	pub fn nick_change_msg(nick: &'a str) -> Self {
		Msg {
			r#type: MsgType::NickChange,
			data: Some(nick),
		}
	}
}

#[derive(Serialize)]
enum MsgType {
	Connected,
	Ping,
	Message,
	Paste,
	NickChange,
	ColorChange,
}

#[derive(Serialize)]
pub struct UserMsg {
	pub nick: String,
	pub custom_nick_color: Option<String>,
	pub msg: String,
	pub time: DateTime<Utc>,
}

#[derive(Clone)]
pub struct User {
	pub id: u64,
	pub nick: String,
	pub color: Option<String>,
	pub sender: mpsc::Sender<Bytes>,
}

pub struct UserDataStream(mpsc::Receiver<Bytes>);

impl Stream for UserDataStream {
	type Item = Result<Bytes, actix_web::Error>;

	fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
		self.0.poll_recv(cx).map(|opt| opt.map(|b| Ok(b)))
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
