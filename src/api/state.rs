use crate::core::{
    kernel_client::{KernelClient, KernelClientMsg},
    notebook::Notebook,
};
use std::{
    collections::HashMap,
    sync::{mpsc::Sender, Arc, Mutex},
    thread,
};

pub struct State {
    pub open_notebooks: Arc<Mutex<HashMap<String, Notebook>>>,
    pub kernel_sender: Arc<Mutex<Sender<KernelClientMsg>>>,
}

impl State {
    pub fn new() -> Self {
        let mut kernel_client = KernelClient::new().expect("Could not create kernel client");
        let sender = kernel_client.tx.clone();

        thread::spawn(move || {
            kernel_client.start();
        });

        Self {
            open_notebooks: Arc::new(Mutex::new(HashMap::new())),
            kernel_sender: Arc::new(Mutex::new(sender)),
        }
    }
}
