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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Notebook {
    pub uuid: String,
    language_info: LanguageInfo,
    meta_data: NotebookMetadata,
    cells: Vec<Cell>,
}

impl From<&str> for Notebook {
    fn from(path: &str) -> Self {
        let json = std::fs::read_to_string(path).unwrap();
        serde_json::from_str(&json).unwrap()
    }
}

impl Notebook {
    pub fn new() -> Self {
        let first_cell = Cell::default();
        Self {
            uuid: nanoid!(30),
            meta_data: NotebookMetadata::default(),
            language_info: LanguageInfo::default(),
            cells: vec![first_cell],
        }
    }

    pub fn update_cell(&mut self, cell_uuid: &str, content: &str) -> Result<(), String> {
        let cell = self.cells.iter_mut().find(|c| c.uuid == cell_uuid);
        if cell.is_none() {
            return Err(String::from("Cell not found"));
        }
        cell.unwrap().update_content(content);
        Ok(())
    }

    pub fn add_cell(&mut self, cell_type: CellType) {
        let cell = Cell::new(cell_type, String::new());
        self.cells.push(cell);
    }

    pub fn save(&self, path: &str) -> Result<(), std::io::Error> {
        let json = serde_json::to_string_pretty(&self)?;
        std::fs::write(path, json)?;
        Ok(())
    }
}
