use super::cell::{Cell, LocalValue};
use crate::api::ws_client::WsClient;
use actix::{Addr, Message};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error::Error, fmt, process::Command, sync::mpsc};
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
        let current_dir = std::env::current_dir()?;
        let kernel_path = current_dir.join("kernel").join("src").join("main.py");
        info!("kernel path: {:?}", kernel_path);
        let res = Command::new("python3")
            .current_dir("./kernel/src/")
            .arg("main.py")
            .spawn()
            .expect("Failed to start kernel");

        info!("res: {:?}", res);

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
        info!("num_messages: {}", num_messages);

        let msg = serde_pickle::to_vec(msg, Default::default())?;
        self.socket.send(&msg, 0)?;

        for _ in 0..num_messages {
            let mut msg = zmq::Message::new();
            self.socket.recv(&mut msg, 0)?;

            let res: MsgFromKernel = serde_pickle::from_slice(&msg, Default::default())?;
            info!("Received message from kernel: {:#?}", res);

            let ws_conn = match self.ws_mapping.get(&res.notebook_uuid) {
                Some(ws_conn) => ws_conn,
                None => {
                    warn!("Could not find ws connection");
                    continue;
                }
            };
            if let Some(err) = &res.error {
                warn!("Error from kernel: {}", err);
                ws_conn.do_send(res);
                break;
            }

            ws_conn.do_send(res);
        }

        info!("Finished sending messages to kernel {}", num_messages);
        Ok(())
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
