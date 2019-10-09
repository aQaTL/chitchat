use actix::{prelude::*, Actor, Addr, Context, StreamHandler};
use actix_web_actors::ws;

pub struct WebsocketServer {
	pub users: Vec<Addr<WebsocketConnection>>,
}

#[derive(Message)]
struct Join(Addr<WebsocketConnection>);

impl Actor for WebsocketServer {
	type Context = Context<Self>;
}

impl Handler<Join> for WebsocketServer {
	type Result = ();

	fn handle(&mut self, join: Join, _ctx: &mut Context<Self>) -> Self::Result {
		self.users.push(join.0);
		()
	}
}

pub struct WebsocketConnection {
	pub srv_addr: Addr<WebsocketServer>,
}

impl Actor for WebsocketConnection {
	type Context = ws::WebsocketContext<Self>;

	fn started(&mut self, ctx: &mut Self::Context) {
		self.srv_addr
			.send(Join(ctx.address()))
			.into_actor(self)
			.then(|res, _act, ctx| {
				match res {
					Ok(_) => (),
					Err(e) => {
						println!("Error: {:?}", e);
						ctx.stop()
					}
				}
				fut::ok(())
				})
		.wait(ctx)
	}
}

impl StreamHandler<ws::Message, ws::ProtocolError> for WebsocketConnection {
	fn handle(&mut self, msg: ws::Message, _ctx: &mut Self::Context) {
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
