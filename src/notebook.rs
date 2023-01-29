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
        let code_cell_1 = Cell::new(CellType::ReactiveCode, String::from("a = b"), 0);
        let code_cell_2 = Cell::new(CellType::ReactiveCode, String::from("b = 1"), 1);

        let cells = HashMap::from([
            (code_cell_1.uuid.clone(), code_cell_1.clone()),
            (code_cell_2.uuid.clone(), code_cell_2.clone()),
        ]);

        let topology = HashMap::from([
            (code_cell_1.uuid.clone(), vec![code_cell_2.uuid.clone()]),
            (code_cell_2.uuid.clone(), vec![]),
        ]);

        Self {
            uuid: nanoid!(30),
            meta_data: NotebookMetadata::default(),
            language_info: LanguageInfo::default(),
            topology,
            cells,
        }
    }

    pub fn eval(&mut self, cell_uuid: &str) {
        let cell = self.cells.get_mut(cell_uuid);
        if cell.is_none() {
            println!("Cell not found");
            return;
        }
        let cell = cell.unwrap();
        cell.eval();

        // TODO update topology if neccessary

        let dependents = self.topology.get(cell_uuid);
        if dependents.is_none() {
            return;
        }
        let dependents = dependents.unwrap();

        for dependent in dependents.iter() {
            println!("depending on {}", dependent);
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
