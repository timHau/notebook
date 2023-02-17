use crate::core::{kernel::Kernel, notebook::Notebook};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub struct State {
    pub open_notebooks: Arc<Mutex<HashMap<String, Notebook>>>,
}

impl State {
    pub fn new() -> Self {
        Self {
            open_notebooks: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}
