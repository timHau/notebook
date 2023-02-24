use super::cell::LocalValue;
use crate::core::errors::NotebookErrors;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error::Error, fmt};
use tracing::info;
use zmq::Socket;

pub struct KernelClient {
    socket: Socket,
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

        Ok(Self { socket })
    }

    pub fn send_to_kernel(&self, msg: &KernelMessage) -> Result<KernelResponse, Box<dyn Error>> {
        info!("sending: {:#?}", msg);
        let msg = serde_json::to_string(msg)?;
        self.socket.send(&msg, 0)?;
        let msg = self.socket.recv_string(0)?;
        match msg {
            Ok(msg) => {
                let res: KernelResponse = serde_json::from_str(&msg)?;
                if let Some(error) = res.error {
                    return Err(Box::new(NotebookErrors::KernelError(error)));
                }

                info!("received: {:#?}", res);
                Ok(res)
            }
            Err(_e) => Err(Box::new(KernelClientErrors::CouldNotParse)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelResponse {
    pub locals: HashMap<String, LocalValue>,
    error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionType {
    Exec,
    Eval,
    Definition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelMessage {
    pub content: String,
    pub locals: HashMap<String, LocalValue>,
    pub execution_type: ExecutionType,
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
