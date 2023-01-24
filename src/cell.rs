use nanoid::nanoid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Cell {
    metadata: CellMetadata,
    pub id: String,
    pub cell_type: CellType,
    pub content: String,
    pub output: String,
    dependents: Vec<Dependent>,
}

impl Cell {
    pub fn new(cell_type: CellType, content: String) -> Self {
        Self {
            metadata: CellMetadata::default(),
            id: nanoid!(30),
            cell_type,
            content,
            output: String::new(),
            dependents: Vec::new(),
        }
    }
}

impl Default for Cell {
    fn default() -> Self {
        let content = String::from(
            "# Welcome to the notebook! \n This is a markdown cell. You can write __markdown__ here and it will be rendered as **HTML**.",
        );
        Self::new(CellType::Markdown, content)
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
