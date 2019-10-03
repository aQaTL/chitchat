use actix::{Actor, Context, StreamHandler};
use actix_web_actors::ws;

pub struct WebsocketServer {}

impl Actor for WebsocketServer {
	type Context = Context<Self>;
}

pub struct WebsocketConnection {}

impl Actor for WebsocketConnection {
	type Context = ws::WebsocketContext<Self>;

	fn started(&mut self, ctx: &mut Self::Context) {}
}

impl StreamHandler<ws::Message, ws::ProtocolError> for WebsocketConnection {
	fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
		use ws::Message::*;
		match msg {
			Ping(msg) => println!("ping: {}", msg),
			Pong(msg) => println!("png: {}", msg),
			Text(msg) => println!("text: {}", msg),
			Binary(_bytes) => {}
			Close(reason) => println!("close: {:?}", reason),
			Nop => (),
		}
	}
}
