use crate::cell::Cell;
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

    pub fn save(&self, path: &str) -> Result<(), std::io::Error> {
        let json = serde_json::to_string_pretty(&self)?;
        std::fs::write(path, json)?;
        Ok(())
    }
}
