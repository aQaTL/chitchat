use std::sync::Mutex;
use std::time::{Duration, Instant};

use actix::Arbiter;
use actix_web::error::ErrorInternalServerError;
use actix_web::web::{Bytes, Data};
use tokio::prelude::*;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::timer::Interval;

pub struct Broadcaster {
	users: Vec<Sender<Bytes>>,
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
				if user.try_send(Bytes::from("data: ping\n\n")).is_ok() {
					Some(user)
				} else {
					None
				}
			})
			.collect::<Vec<Sender<Bytes>>>();
	}

	pub fn new_user(&mut self) -> User {
		let (tx, rx) = mpsc::channel(100);

		tx.clone()
			.try_send(Bytes::from("data: connected\n\n"))
			.unwrap();

		self.users.push(tx);
		User(rx)
	}

	pub fn send(&mut self, msg: &str) {
		let msg = Bytes::from(["data: ", msg, "\n\n"].concat());

		for user in &mut self.users {
			user.try_send(msg.clone()).unwrap_or(());
		}
	}
}

pub struct User(Receiver<Bytes>);

impl Stream for User {
	type Item = Bytes;
	type Error = actix_web::Error;

	fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
		self.0.poll().map_err(ErrorInternalServerError)
	}
}
