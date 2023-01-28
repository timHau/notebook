use nanoid::nanoid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Cell {
    metadata: CellMetadata,
    pub uuid: String,
    pub cell_type: CellType,
    pub content: String,
    pub output: String,
    dependents: Vec<Dependent>,
}

impl Cell {
    pub fn new(cell_type: CellType, content: String) -> Self {
        Self {
            metadata: CellMetadata::default(),
            uuid: nanoid!(30),
            cell_type,
            content,
            output: String::new(),
            dependents: Vec::new(),
        }
    }

    pub fn update_content(&mut self, content: &str) {
        self.content = content.to_string();
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CellMetadata {
    pub collapsed: bool,
}

impl Default for CellMetadata {
    fn default() -> Self {
        Self { collapsed: false }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CellType {
    NonReactiveCode,
    ReactiveCode,
    Markdown,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Dependent {
    id: String,
}
