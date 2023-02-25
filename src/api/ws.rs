use super::routes::EvalResult;
use crate::core::{
    kernel_client::{KernelClientMsg, KernelMsg},
    notebook::Notebook,
};
use actix::{Actor, AsyncContext, Handler, StreamHandler};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use std::{
    sync::mpsc::{self, Sender},
    thread,
    time::Duration,
};
use tracing::{info, log::warn};

pub struct Ws {
    pub notebook: Notebook,
    pub tx: mpsc::Sender<KernelMsg>,
    rx: mpsc::Receiver<KernelMsg>,
}

impl Actor for Ws {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("WS session started");

        ctx.run_interval(Duration::from_secs(1), |act, ctx| {
            // for msg in act.rx.try_iter() {
            //     info!("WS msg: {:#?}", msg);
            // }
        });
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Ws {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                self.handle_text(text.to_string(), ctx);
            }
            Ok(ws::Message::Binary(bin)) => {
                println!("Binary: {:?}", bin);
            }
            Ok(ws::Message::Ping(msg)) => {
                println!("Ping: {:?}", msg);
            }
            Ok(ws::Message::Pong(msg)) => {
                println!("Pong: {:?}", msg);
            }
            Ok(ws::Message::Close(reason)) => {
                println!("Close: {:?}", reason);
            }
            Err(e) => {
                println!("Error: {:?}", e);
            }
            _ => warn!("Unhandled message {:?}", msg),
        }
    }
}

impl Handler<KernelMsg> for Ws {
    type Result = ();

    fn handle(&mut self, msg: KernelMsg, ctx: &mut Self::Context) {
        ctx.text(serde_json::to_string(&msg).unwrap());
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WsMessage {
    cmd: WsCmds,
    data: String,

    #[serde(rename = "cellUuid")]
    cell_uuid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum WsCmds {
    Run,
    Ping,
    Pong,
}

#[derive(Serialize)]
pub struct WsResponse {
    pub result: EvalResult,
    pub error: Option<String>,
}

impl Ws {
    pub fn new(notebook: &Notebook) -> Self {
        let (tx, rx) = mpsc::channel();

        Self {
            notebook: notebook.clone(),
            tx,
            rx,
        }
    }

    pub fn handle_text(&mut self, text: String, ctx: &mut ws::WebsocketContext<Self>) {
        let msg: WsMessage = match serde_json::from_str(&text) {
            Ok(msg) => msg,
            Err(e) => {
                warn!("Could not parse message: {}", e);
                return;
            }
        };

        match msg.cmd {
            WsCmds::Run => match self.notebook.eval_cell(&msg.cell_uuid, &msg.data) {
                Ok(_) => info!("Evaluated cell"),
                Err(e) => warn!("Could not evaluate cell: {}", e),
            },
            WsCmds::Ping => {
                let response = WsMessage {
                    cmd: WsCmds::Pong,
                    data: String::new(),
                    cell_uuid: String::new(),
                };

                ctx.text(serde_json::to_string(&response).unwrap());
            }
            WsCmds::Pong => {}
        }
    }
}
