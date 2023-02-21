use crate::{core::notebook::Notebook, kernel::kernel_client::KernelClient};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub struct State {
    pub open_notebooks: Arc<Mutex<HashMap<String, Notebook>>>,
    pub kernel_client: Arc<Mutex<KernelClient>>,
}

impl State {
    pub fn new() -> Self {
        let kernel_client = KernelClient::new().expect("Could not create kernel client");

        Self {
            open_notebooks: Arc::new(Mutex::new(HashMap::new())),
            kernel_client: Arc::new(Mutex::new(kernel_client)),
        }
    }
}
