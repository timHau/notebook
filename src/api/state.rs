use crate::core::{kernel::Kernel, notebook::Notebook};
use std::{collections::HashMap, sync::Mutex};

pub struct State {
    pub open_notebooks: Mutex<HashMap<String, Notebook>>,
    pub kernel: Mutex<Kernel>,
}

impl State {
    pub fn new() -> Self {
        Self {
            open_notebooks: Mutex::new(HashMap::new()),
            kernel: Mutex::new(Kernel::start()),
        }
    }
}
