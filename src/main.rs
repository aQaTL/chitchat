use std::io;
use std::sync::Mutex;

use actix::{Actor, Addr, System};
use actix_session::{CookieSession, Session};
use actix_web::web::{Data, Path};
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_actors::ws;
use serde::Deserialize;

use websocket_server::*;

use crate::sse::{Broadcaster, Msg, MsgType};

mod websocket_server;
//mod sockjs_server;
mod sse;

#[derive(Deserialize)]
struct Config {
	ip: String,
	port: u16,
}

fn main() -> io::Result<()> {
	let config = {
		let data = std::fs::read("config.toml")?;
		toml::from_slice::<Config>(data.as_slice())?
	};

	let sys = System::new(env!("CARGO_PKG_NAME"));

	let ws_server_actor = WebsocketServer {
		users: Vec::with_capacity(10),
	}
	.start();

	//	let sockjs_session_manager_addr = SockJSManager::<sockjs_server::Server>::start_default();

	let broadcaster = sse::Broadcaster::new();

	let bind_addr = format!("{}:{}", config.ip, config.port);

	HttpServer::new(move || {
		//		let manager = sockjs_session_manager_addr.clone();
		App::new()
			.wrap(CookieSession::signed(&[0; 32]).secure(false))
			.data(ws_server_actor.clone())
			.register_data(broadcaster.clone())
			//			.route("/ws", web::get().to(websocket_connect))
			//			.route("/sockjs", sockjs::SockJS::new(manager.clone()))
			.route("/events", web::get().to(new_client))
			.route("/broadcast/{msg}", web::get().to(broadcast_msg))
			.service(actix_files::Files::new("/", "frontend").index_file("index.html"))
	})
	.bind(&bind_addr)?
	.start();

	println!("Running on: {}", bind_addr);

	sys.run()
}

#[allow(dead_code)]
fn websocket_connect(
	req: HttpRequest,
	stream: web::Payload,
	srv: web::Data<Addr<WebsocketServer>>,
) -> impl Responder {
	ws::start(
		WebsocketConnection {
			srv_addr: srv.get_ref().clone(),
		},
		&req,
		stream,
	)
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
	if let Some(nick) = session.get::<String>("nick")? {
		if broadcaster.users.iter().any(|user| user.nick == nick) {
			return Ok(HttpResponse::Ok()
				.content_type("application/json")
				.body(serde_json::to_string(&Msg::new(MsgType::YourNickIsTaken))?));
		}
	}

	session.set("nick", params.nick.clone())?;

	let rx = broadcaster.new_user(&params.nick);

	Ok(HttpResponse::Ok()
		.header("content-type", "text/event-stream")
		.no_chunking()
		.streaming(rx))
}

fn broadcast_msg(
	msg: Path<String>,
	broadcaster: Data<Mutex<Broadcaster>>,
	session: Session,
) -> Result<impl Responder, actix_web::Error> {
	let nick = session.get::<String>("nick")?.unwrap_or_default();
	broadcaster
		.lock()
		.unwrap()
		.send(nick.as_str(), &msg.into_inner());

	Ok(HttpResponse::Ok().body(format!("msg sent from {}", nick,)))
}
