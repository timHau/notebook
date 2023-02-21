use pyo3::{types::PyDict, Py, PyResult};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error::Error, fmt, thread};
use tracing::info;
use zmq::Socket;

pub struct KernelClient {
    client: Socket,
    // server: Socket,
}

impl KernelClient {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let _ = Self::launch_python_pub()?;

        let zmq_port = std::env::var("ZMQ_PORT")
            .unwrap_or_else(|_| "80801".to_string())
            .parse::<u16>()?;
        let ctx = zmq::Context::new();
        // let server = ctx.socket(zmq::REP).expect("Could not create socket");
        // server
        //     .bind("tcp://localhost:8082")
        //     .expect("Could not connect to socket");
        // info!("[ZMQ] Server Bound to port 8082");

        let client = ctx.socket(zmq::REQ).expect("Could not create socket");
        client
            .connect("tcp://localhost:8081")
            .expect("Could not connect to socket");
        info!("[ZMQ] Client Connected to port 8081");

        Ok(Self { client })
    }

    fn launch(&self) -> Result<(), Box<dyn Error>> {
        // thread::spawn(move || loop {
        //     let msg = server
        //         .recv_string(0)
        //         .expect("Could not receive message")
        //         .expect("Could not parse message");
        //     info!("Received message: {:?}", msg);
        //     let res_msg = format!("Status Success: {}", msg);
        //     server.send(&res_msg, 0).expect("Could not send message");
        // });

        Ok(())
    }

    pub fn send_to_kernel(&self, msg: &KernelMessage) -> Result<String, Box<dyn Error>> {
        info!("Sending message: {:?}", msg.content);
        let msg = serde_json::to_string(msg)?;
        self.client.send(&msg, 0)?;

        let res = self.client.recv_string(0)?;
        match res {
            Ok(msg) => {
                info!("Received message: {:?}", msg);
                Ok(msg)
            }
            Err(_e) => Err(Box::new(KernelClientErrors::CouldNotParse)),
        }
    }

    fn launch_python_pub() -> PyResult<()> {
        // TODO should be able to use this to launch the python kernel

        // let file_path = Path::new("./src/kernel/src/main.py");
        // let file_content = fs::read_to_string(file_path)?;

        // Python::with_gil(|py| -> PyResult<Py<PyAny>> {
        //     let main: Py<PyAny> = PyModule::from_code(py, &file_content, "main.py", "main")?
        //         .getattr("main")?
        //         .into();
        //     main.call0(py)
        // })
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelMessage {
    pub content: String,
    pub locals: HashMap<String, String>,
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
