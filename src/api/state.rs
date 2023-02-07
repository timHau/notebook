use crate::core::notebook::Notebook;
use std::{collections::HashMap, sync::Mutex};

pub struct State {
    pub open_notebooks: Mutex<HashMap<String, Notebook>>,
}

impl State {
    pub fn new() -> Self {
        Self {
            open_notebooks: Mutex::new(HashMap::new()),
        }
    }
}
