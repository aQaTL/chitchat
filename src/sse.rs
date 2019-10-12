use std::sync::Mutex;
use std::time::{Duration, Instant};

use actix::Arbiter;
use actix_web::error::ErrorInternalServerError;
use actix_web::web::{Bytes, Data};
use serde_json::json;
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
				if user.sender.try_send(Bytes::from("data: ping\n\n")).is_ok() {
					Some(user)
				} else {
					None
				}
			})
			.collect::<Vec<User>>();
	}

	pub fn new_user(&mut self, nick: &str) -> UserDataStream {
		let (tx, rx) = mpsc::channel(100);

		tx.clone()
			.try_send(Bytes::from("data: connected\n\n"))
			.unwrap();

		self.users.push(User {
			nick: String::from(nick),
			sender: tx,
		});

		UserDataStream(rx)
	}

	pub fn send(&mut self, sender_nick: &str, msg: &str) {
		let msg = Bytes::from(
			[
				"data: ",
				json!({
					"nick": sender_nick,
					"msg": msg,
				})
				.to_string()
				.as_str(),
				"\n\n",
			]
			.concat(),
		);

		for user in &mut self.users {
			user.sender.try_send(msg.clone()).unwrap_or(());
		}
	}
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
