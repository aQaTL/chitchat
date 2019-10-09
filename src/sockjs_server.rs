use sockjs::{SockJSContext, Session, CloseReason, Message};
use actix::{Actor, Handler, Context};

pub struct Server;

impl Actor for Server {
	type Context = SockJSContext<Self>;
}

impl Default for Server {
	fn default() -> Self {
		Server
	}
}

impl Handler<Message> for Server {
	type Result = ();

	fn handle(&mut self, msg: Message, ctx: &mut Self::Context) -> Self::Result {
		println!("Message came: {}", msg.0);
		()
	}
}

impl Session for Server {
	fn opened(&mut self, ctx: &mut Context<Self>) {
		ctx.broadcast("Someone joined");
	}

	fn closed(&mut self, ctx: &mut Context<Self>, reason: CloseReason) {
		ctx.broadcast("Someone left");
	}
}

