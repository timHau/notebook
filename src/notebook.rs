use std::collections::HashMap;

use crate::cell::{Cell, CellType};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct LanguageInfo {
    name: String,
    version: String,
    file_extension: String,
}

impl Default for LanguageInfo {
    fn default() -> Self {
        Self {
            name: String::from("python"),
            version: String::from("3.10.6"),
            file_extension: String::from(".py"),
        }
    }
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

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Notebook {
    pub uuid: String,
    language_info: LanguageInfo,
    meta_data: NotebookMetadata,

    topology: HashMap<String, Vec<String>>,
    cells: HashMap<String, Cell>,
}

impl Notebook {
    pub fn new() -> Self {
        let code_cell = Cell::new(CellType::ReactiveCode, String::from("a = 1"), 0);
        let cells = HashMap::from([(code_cell.uuid.clone(), code_cell.clone())]);
        Self {
            uuid: nanoid!(30),
            meta_data: NotebookMetadata::default(),
            language_info: LanguageInfo::default(),
            topology: HashMap::new(),
            cells,
        }
    }

    pub fn eval(&mut self, cell_uuid: &str) {
        let cell = self.cells.get_mut(cell_uuid);
        if let Some(cell) = cell {
            cell.eval();
        } else {
            println!("Cell not found");
            println!("cell_mapping: {:#?}", self.cells);
        }
    }

    pub fn save(&self, path: &str) -> Result<(), std::io::Error> {
        // let json = serde_json::to_string_pretty(&self)?;
        // std::fs::write(path, json)?;
        Ok(())
    }
}

impl From<&str> for Notebook {
    fn from(path: &str) -> Self {
        let json = std::fs::read_to_string(path).unwrap();
        serde_json::from_str(&json).unwrap()
    }
}
