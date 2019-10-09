use actix::{Actor, Addr, System};
use actix_web::{web, App, HttpRequest, HttpServer, Responder};
use actix_web_actors::ws;
use std::io;
use serde::Deserialize;

mod websocket_server;
mod sockjs_server;

use websocket_server::*;
use sockjs::SockJSManager;

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

	let ws_server_actor = WebsocketServer { users: Vec::with_capacity(10) }.start();

	let sockjs_session_manager_addr = SockJSManager::<sockjs_server::Server>::start_default();

	let bind_addr = format!("{}:{}", config.ip, config.port);

	HttpServer::new(move || {
		let manager = sockjs_session_manager_addr.clone();
		App::new()
			.data(ws_server_actor.clone())
			//.route("/ws", web::get().to(websocket_connect))
			.route("/sockjs", sockjs::SockJS::new(manager.clone()))
			.service(actix_files::Files::new("/", "frontend").index_file("index.html"))
	})
	.bind(&bind_addr)?
	.start();

	println!("Running on: {}", bind_addr);

	sys.run()
}

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
		stream)
}
