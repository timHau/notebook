use crate::core::{cell::LocalValue, kernel_client::KernelClient, notebook::Notebook};
use actix::{Actor, StreamHandler};
use actix_web_actors::ws::{self, WebsocketContext};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::log::warn;

pub struct Ws {}

impl Actor for Ws {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Ws {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                // self.handle_text(text.to_string(), ctx);
                println!("Text: {:?}", text)
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

impl Ws {}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WsMessage {
    cmd: WsCmds,
    data: String,

    #[serde(rename = "notebookUuid")]
    notebook_uuid: String,

    #[serde(rename = "cellUuid")]
    cell_uuid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum WsCmds {
    Run,
}

#[derive(Serialize)]
pub struct WsResponse {
    pub result: EvalResult,
    pub error: Option<String>,
}

pub type EvalResult = HashMap<String, HashMap<String, LocalValue>>; // cell_uuid -> (var_name -> var_value)
