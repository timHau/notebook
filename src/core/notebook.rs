use std::collections::HashMap;

use super::kernel::Kernel;
use crate::core::{cell::Cell, topology::Topology};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};

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

pub type Scope = HashMap<String, String>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Notebook {
    pub uuid: String,
    language_info: LanguageInfo,
    meta_data: NotebookMetadata,
    topology: Topology,

    #[serde(skip)]
    kernel: Kernel,
    #[serde(skip)]
    scope: Scope,
}

impl Notebook {
    pub fn new(kernel: Kernel) -> Self {
        let mut scope = Scope::default();
        // let code_cell_1 = Cell::new(CellType::ReactiveCode, String::from("import matplotlib.pyplot as plt\nimport numpy as np\nx = np.arange(0,4*np.pi,0.1)\ny = np.sin(x)\nplt.plot(x,y)\nplt.show()"), 0);
        let code_cell_1 = Cell::new_reactive("a = b + 1", &mut scope).unwrap();
        let code_cell_2 = Cell::new_reactive("b = 2", &mut scope).unwrap();
        let code_cell_3 = Cell::new_reactive("c = 1", &mut scope).unwrap();

        let mut topology = Topology::from(vec![&code_cell_1, &code_cell_2, &code_cell_3]);
        topology.build(&mut scope).unwrap();

        let version = kernel.version.clone();
        Self {
            uuid: nanoid!(30),
            meta_data: NotebookMetadata::default(),
            kernel,
            scope,
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
}

impl From<&str> for Notebook {
    fn from(path: &str) -> Self {
        let json = std::fs::read_to_string(path).unwrap();
        serde_json::from_str(&json).unwrap()
    }
}
