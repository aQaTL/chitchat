use actix::{Actor, Addr, System};
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_actors::ws;
use serde::Deserialize;
use std::io;
use actix_web::web::{Data, Path};
use std::sync::Mutex;

mod websocket_server;
//mod sockjs_server;
mod sse;

use crate::sse::Broadcaster;
use websocket_server::*;

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

fn new_client(broadcaster: Data<Mutex<Broadcaster>>) -> impl Responder {
	let rx = broadcaster.lock().unwrap().new_user();

	HttpResponse::Ok()
		.header("content-type", "text/event-stream")
		.no_chunking()
		.streaming(rx)
}

fn broadcast_msg(msg: Path<String>, broadcaster: Data<Mutex<Broadcaster>>) -> impl Responder {
	broadcaster.lock().unwrap().send(&msg.into_inner());
	HttpResponse::Ok().body("msg sent")
}
