use std::io;
use std::sync::Mutex;

use actix::System;
use actix_session::{CookieSession, Session};
use actix_web::web::Data;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::Deserialize;

use crate::sse::Broadcaster;

mod sse;

#[derive(Deserialize)]
struct Config {
	ip: String,
	port: u16,
}

fn main() -> io::Result<()> {
	let config = {
		let data = match std::fs::read("config.toml") {
			Ok(data) => data,
			Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
				println!("config.toml not found, using config_template.toml");
				std::fs::read("config_template.toml")?
			}
			Err(e) => return Err(e),
		};
		toml::from_slice::<Config>(data.as_slice())?
	};

	let sys = System::new(env!("CARGO_PKG_NAME"));

	let broadcaster = sse::Broadcaster::new();

	let bind_addr = format!("{}:{}", config.ip, config.port);

	HttpServer::new(move || {
		App::new()
			.wrap(CookieSession::signed(&[0; 32]).secure(false))
			.register_data(broadcaster.clone())
			.route("/events", web::get().to(new_client))
			.route("/send_msg", web::post().to(send_msg))
			.service(actix_files::Files::new("/", "frontend").index_file("index.html"))
	})
	.bind(&bind_addr)?
	.start();

	println!("Running on: {}", bind_addr);

	sys.run()
}

#[derive(Deserialize)]
struct NewClientQueryParams {
	nick: String,
}

fn new_client(
	params: web::Query<NewClientQueryParams>,
	broadcaster: Data<Mutex<Broadcaster>>,
	session: Session,
) -> Result<impl Responder, actix_web::Error> {
	let mut broadcaster = broadcaster.lock().unwrap();
	//	if let Some(nick) = session.get::<String>("nick")? {
	//		if broadcaster.users.iter().any(|user| user.nick == nick) {
	//			return Ok(HttpResponse::Ok()
	//				.content_type("application/json")
	//				.body(serde_json::to_string(&Msg::new(MsgType::YourNickIsTaken))?));
	//		}
	//	}

	session.set("nick", params.nick.clone())?;

	let rx = broadcaster.new_user(&params.nick);

	Ok(HttpResponse::Ok()
		.header("content-type", "text/event-stream")
		.no_chunking()
		.streaming(rx))
}

fn send_msg(
	msg: web::Json<String>,
	broadcaster: Data<Mutex<Broadcaster>>,
	session: Session,
) -> Result<impl Responder, actix_web::Error> {
	let nick = session.get::<String>("nick")?.unwrap_or_default();
	broadcaster.lock().unwrap().send(nick, msg.0.clone());

	Ok(HttpResponse::Ok())
}
