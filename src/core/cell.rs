use nanoid::nanoid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Cell {
    metadata: CellMetadata,
    pub uuid: String,
    pub cell_type: CellType,
    pub content: String,
    pub pos: usize,
}

impl Cell {
    pub fn new(cell_type: CellType, content: String, pos: usize) -> Self {
        Self {
            metadata: CellMetadata::default(),
            uuid: nanoid!(30),
            cell_type,
            content,
            pos,
        }
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
