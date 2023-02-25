use crate::core::{kernel_client::KernelMsg, notebook::Notebook};
use actix::{Actor, Handler, StreamHandler};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use tracing::{info, log::warn};

pub struct WsClient {
    pub notebook: Notebook,
}

impl Actor for WsClient {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("WS session started");

        // ctx.run_interval(Duration::from_secs(1), |act, ctx| {
        //     act.tx.send(KernelMsg::Ping).unwrap();
        //     ctx.text(serde_json::to_string(&KernelMsg::Ping).unwrap());
        // });
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsClient {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                self.handle_text(text.to_string(), ctx);
            }

            Err(e) => {
                println!("Error: {:?}", e);
            }

            _ => warn!("Unhandled message {:?}", msg),
        }
    }
}

impl Handler<KernelMsg> for WsClient {
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
    cell_uuid: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum WsCmds {
    Run,
    Ping,
    Pong,
}

impl WsClient {
    pub fn new(notebook: &Notebook) -> Self {
        Self {
            notebook: notebook.clone(),
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
            WsCmds::Run => match self.notebook.eval_cell(&msg.cell_uuid.unwrap(), &msg.data) {
                Ok(_) => info!("Evaluated cell"),
                Err(e) => warn!("Could not evaluate cell: {}", e),
            },
            WsCmds::Ping => {
                let response = WsMessage {
                    cmd: WsCmds::Pong,
                    data: String::new(),
                    cell_uuid: Some(String::new()),
                };

                ctx.text(serde_json::to_string(&response).unwrap());
            }
            WsCmds::Pong => {}
        }
    }
}
