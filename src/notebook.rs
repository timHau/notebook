use nanoid::nanoid;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Notebook {
    cells: Vec<Cell>,
}

impl Notebook {
    pub fn new() -> Self {
        let first_cell = Cell::default();
        Self {
            cells: vec![first_cell],
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Cell {
    pub id: String,
    pub cellType: CellType,
    pub content: String,
    pub output: String,
    dependents: Vec<Dependent>,
}

impl Cell {
    pub fn new(cellType: CellType, content: String) -> Self {
        Self {
            id: nanoid!(30),
            cellType,
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

#[derive(Debug, Serialize)]
pub enum CellType {
    NonReactiveCode,
    ReactiveCode,
    Markdown,
}

#[derive(Debug, Serialize)]
struct Dependent {
    id: String,
}
