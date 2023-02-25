use super::{
    cell::{Cell, LocalValue},
    statement::Statement,
};
use crate::{api::ws_client::WsClient, core::errors::NotebookErrors};
use actix::{Addr, Message};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error::Error, fmt, sync::mpsc};
use tracing::{info, log::warn};
use zmq::Socket;

pub struct KernelClient {
    socket: Socket,
    rx: mpsc::Receiver<KernelClientMsg>,
    pub tx: mpsc::Sender<KernelClientMsg>,
    ws_mapping: HashMap<String, Addr<WsClient>>, // notebook_uuid, ws sender
}

impl KernelClient {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let zmq_port = std::env::var("ZMQ_PORT")
            .unwrap_or_else(|_| "8081".to_string())
            .parse::<u16>()?;
        let ctx = zmq::Context::new();
        let socket = ctx.socket(zmq::PAIR)?;
        // socket.bind(&format!("tcp://*:{:?}", zmq_port))?;
        socket.connect(&format!("tcp://localhost:{:?}", zmq_port))?;

        let (tx, rx) = mpsc::channel();

        Ok(Self {
            socket,
            rx,
            tx,
            ws_mapping: HashMap::new(),
        })
    }

    pub fn start(&mut self) {
        loop {
            match self.rx.recv() {
                Ok(msg) => match msg {
                    KernelClientMsg::InitWs(uuid, sender) => {
                        self.ws_mapping.insert(uuid, sender);
                    }
                    KernelClientMsg::MsgToKernel(msg) => {
                        let _res = self.send_to_kernel(&msg);
                    }
                    _ => warn!("Unhandled message {:?}", msg),
                },
                Err(_e) => {
                    info!("Could not receive message");
                }
            }
        }
    }

    pub fn send_to_kernel(&self, msg: &MsgToKernel) -> Result<(), Box<dyn Error>> {
        info!("sending message to kernel: {:#?}", msg);
        let num_messages = msg
            .execution_cells
            .iter()
            .fold(0, |acc, cell| acc + cell.statements.len());

        let msg = serde_json::to_string(msg)?;
        self.socket.send(&msg, 0)?;

        for _ in 0..num_messages {
            let msg = self.socket.recv_string(0)?;
            self.handle_msg(msg)?;
        }

        Ok(())
    }

    fn handle_msg(&self, msg: Result<String, Vec<u8>>) -> Result<(), Box<dyn Error>> {
        match msg {
            Ok(msg) => {
                let res: MsgFromKernel = serde_json::from_str(&msg)?;
                let ws_conn = match self.ws_mapping.get(&res.notebook_uuid) {
                    Some(ws_conn) => ws_conn,
                    None => {
                        return Err(Box::new(NotebookErrors::KernelError(
                            "No ws connection".to_string(),
                        )))
                    }
                };
                ws_conn.do_send(res);

                Ok(())
            }
            Err(_e) => Err(Box::new(KernelClientErrors::CouldNotParse)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionType {
    Exec,
    Eval,
    Definition,
}

#[derive(Debug, Clone)]
pub enum KernelClientMsg {
    InitWs(String, Addr<WsClient>),
    MsgToKernel(MsgToKernel),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MsgToKernel {
    pub notebook_uuid: String,
    pub cell_uuid: String,
    pub execution_cells: Vec<Cell>,
    pub locals_of_deps: Vec<HashMap<String, LocalValue>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MsgFromKernel {
    pub notebook_uuid: String,
    pub cell_uuid: String,
    pub locals: HashMap<String, LocalValue>,
    pub error: Option<String>,
}

impl Message for MsgFromKernel {
    type Result = ();
}

#[derive(Debug)]
pub enum KernelClientErrors {
    CouldNotParse,
}

impl fmt::Display for KernelClientErrors {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            KernelClientErrors::CouldNotParse => write!(f, "Could not parse message"),
        }
    }
}

impl Error for KernelClientErrors {}
