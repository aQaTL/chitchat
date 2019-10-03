use actix::{Actor, Addr, System};
use actix_web::{web, App, HttpRequest, HttpServer, Responder};
use actix_web_actors::ws;
use std::io;

mod websocket_server;

use websocket_server::*;

fn main() -> io::Result<()> {
	let sys = System::new(env!("CARGO_PKG_NAME"));

	let ws_server_actor = WebsocketServer {}.start();

	HttpServer::new(move || {
		App::new()
			.data(ws_server_actor.clone())
			.route("/ws", web::get().to(websocket_connect))
			.service(actix_files::Files::new("/", "/frontend"))
	})
	.bind(":80")?
	.start();

	sys.run()
}

fn websocket_connect(
	req: HttpRequest,
	stream: web::Payload,
	srv: web::Data<Addr<WebsocketServer>>,
) -> impl Responder {
	ws::start(WebsocketConnection {}, &req, stream)
}
