use super::cell::{Cell, LocalValue};
use crate::api::ws_client::WsClient;
use actix::{Addr, Message};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error::Error, fmt, process::Command, sync::mpsc, thread};
use tracing::{info, log::warn};
use zmq::Socket;

pub struct KernelClient {
    sub_socket: Socket,
    req_socket: Socket,
    rx: mpsc::Receiver<KernelClientMsg>,
    pub tx: mpsc::Sender<KernelClientMsg>,
    ws_mapping: HashMap<String, Addr<WsClient>>, // notebook_uuid, ws sender
}

impl KernelClient {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let current_dir = std::env::current_dir()?;
        let kernel_path = current_dir.join("kernel").join("src").join("main.py");
        info!("kernel path: {:?}", kernel_path);

        let zmq_port_sub = std::env::var("ZMQ_PORT_SUB")
            .unwrap_or_else(|_| "8081".to_string())
            .parse::<u16>()?;
        let zmq_port_req = std::env::var("ZMQ_PORT_REQ")
            .unwrap_or_else(|_| "8081".to_string())
            .parse::<u16>()?;

        let ctx = zmq::Context::new();

        let sub_socket = ctx.socket(zmq::SUB)?;
        sub_socket.connect(&format!("tcp://localhost:{:?}", zmq_port_sub))?;
        sub_socket.set_subscribe(b"")?;

        let req_socket = ctx.socket(zmq::REQ)?;
        req_socket.connect(&format!("tcp://localhost:{:?}", zmq_port_req))?;

        thread::spawn(|| {
            Command::new("python3")
                .current_dir("./kernel/src/")
                .arg("main.py")
                .spawn()
                .expect("Failed to start kernel");
        });

        let (tx, rx) = mpsc::channel();

        Ok(Self {
            sub_socket,
            req_socket,
            rx,
            tx,
            ws_mapping: HashMap::new(),
        })
    }

    pub fn start(&mut self) {
        loop {
            info!("waiting in start for message");
            match self.rx.recv() {
                Ok(msg) => {
                    info!("Received message: {:#?}", msg);
                    match msg {
                        KernelClientMsg::InitWs(uuid, sender) => {
                            self.ws_mapping.insert(uuid, sender);
                        }
                        KernelClientMsg::MsgToKernel(msg) => {
                            let _res = self.send_to_kernel(&msg);
                            let res = self.receive_from_kernel();
                            info!("res: {:?}", res);
                        }
                    }
                }
                Err(_e) => {
                    info!("Could not receive message");
                }
            }
        }
    }

    pub fn receive_from_kernel(&self) -> Result<(), Box<dyn Error>> {
        loop {
            info!("Waiting for response from kernel");

            let mut msg = zmq::Message::new();
            self.sub_socket.recv(&mut msg, 0)?;

            info!("msg: {:?}", msg);
            let res: MsgFromKernel = serde_pickle::from_slice(&msg, Default::default())?;
            info!("Received message from kernel: {:#?}", res);
            if res.ended {
                info!("Kernel ended");
                break;
            }

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

        Ok(())
    }

    pub fn send_to_kernel(&self, msg: &MsgToKernel) -> Result<(), Box<dyn Error>> {
        info!("sending message to kernel: {:#?}", msg);

        let msg = serde_pickle::to_vec(msg, Default::default())?;
        self.req_socket.send(&msg, 0)?;
        let res = self.req_socket.recv_bytes(0)?;
        info!("Received response from kernel: {:?}", res);

        Ok(())
    }
}

impl fmt::Debug for KernelClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KernelClient")
            // .field("socket", &self.socket)
            .field("ws_mapping", &self.ws_mapping)
            .finish()
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MsgFromKernel {
    pub notebook_uuid: String,
    pub cell_uuid: String,
    pub locals: HashMap<String, LocalValue>,
    pub error: Option<String>,
    pub ended: bool,
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
