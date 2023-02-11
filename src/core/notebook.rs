use super::kernel::Kernel;
use crate::core::{
    cell::{Cell, CellType},
    topology::Topology,
};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use std::error;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
struct LanguageInfo {
    name: String,
    version: String,
    file_extension: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct NotebookMetadata {
    format_version: String,
}

impl Default for NotebookMetadata {
    fn default() -> Self {
        Self {
            format_version: String::from("0.0.1"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Notebook {
    pub uuid: String,
    language_info: LanguageInfo,
    meta_data: NotebookMetadata,
    topology: Topology,

    #[serde(skip)]
    kernel: Kernel,
}

impl Notebook {
    pub fn new(kernel: Kernel) -> Self {
        let code_cell_1 = Cell::new(CellType::ReactiveCode, String::from("a = b\nc = 1"), 0);
        let code_cell_2 = Cell::new(CellType::ReactiveCode, String::from("b = 1"), 1);

        let mut topology = Topology::new();
        topology
            .add_cell(code_cell_1, Some(&vec![code_cell_2.uuid.clone()]))
            .unwrap();
        topology.add_cell(code_cell_2, None).unwrap();

        let version = kernel.version.clone();
        Self {
            uuid: nanoid!(30),
            meta_data: NotebookMetadata::default(),
            kernel,
            language_info: LanguageInfo {
                name: String::from("python"),
                version,
                file_extension: String::from(".py"),
            },
            topology,
        }
    }

    pub fn get_cell_mut(&mut self, cell_uuid: &str) -> Option<&mut Cell> {
        self.topology.get_cell_mut(cell_uuid)
    }

    pub fn eval(&mut self, cell: &mut Cell) -> Result<(), Box<dyn error::Error>> {
        self.topology.eval(cell, &mut self.kernel)
    }
}

impl From<&str> for Notebook {
    fn from(path: &str) -> Self {
        let json = std::fs::read_to_string(path).unwrap();
        serde_json::from_str(&json).unwrap()
    }
}
