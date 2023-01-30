use std::collections::HashMap;

use crate::{
    cell::{Cell, CellType},
    topology::Topology,
};
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
            name: String::from(""),
            version: String::from(""),
            file_extension: String::from(""),
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
    topology: Topology,
}

impl Notebook {
    pub fn new() -> Self {
        let code_cell_1 = Cell::new(CellType::ReactiveCode, String::from("a = b"), 0);
        let code_cell_2 = Cell::new(CellType::ReactiveCode, String::from("b = 1"), 1);

        let cells = HashMap::from([
            (code_cell_1.uuid.clone(), code_cell_1.clone()),
            (code_cell_2.uuid.clone(), code_cell_2.clone()),
        ]);

        let trees = HashMap::from([
            (code_cell_1.uuid.clone(), vec![code_cell_2.uuid.clone()]),
            (code_cell_2.uuid.clone(), vec![]),
        ]);

        let topology = Topology { trees, cells };

        Self {
            uuid: nanoid!(30),
            meta_data: NotebookMetadata::default(),
            language_info: LanguageInfo::default(),
            topology,
        }
    }

    pub fn eval(&mut self, cell_uuid: &str) {
        self.topology.eval(cell_uuid)
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
