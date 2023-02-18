use actix::{Actor, StreamHandler};
use actix_web_actors::ws;

pub struct Ws {}

impl Actor for Ws {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Ws {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, _ctx: &mut Self::Context) {
        println!("WEBSOCKET MESSAGE: {:?}", msg);
    }
}
