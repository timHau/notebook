use super::{cell::CellType, kernel::Kernel, topology};
use crate::core::{cell::Cell, topology::Topology};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error::Error};
use tracing::info;

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
    title: String,

    #[serde(skip)]
    kernel: Kernel,
    #[serde(skip)]
    scope: Scope,
}

impl Notebook {
    pub fn new(kernel: Kernel) -> Self {
        let mut scope = Scope::default();
        // let code_cell_1 = Cell::new(CellType::ReactiveCode, String::from("import matplotlib.pyplot as plt\nimport numpy as np\nx = np.arange(0,4*np.pi,0.1)\ny = np.sin(x)\nplt.plot(x,y)\nplt.show()"), 0);
        // let code_cell_1 = Cell::new_reactive("a = b + 1", &mut scope, 0).unwrap();
        let code_cell_1 = Cell::new_reactive("a = b + 1", &mut scope).unwrap();
        let code_cell_2 = Cell::new_reactive("b = 2", &mut scope).unwrap();
        let code_cell_3 = Cell::new_reactive("c = 1", &mut scope).unwrap();

        let mut topology =
            Topology::from_vec(vec![&code_cell_1, &code_cell_2, &code_cell_3], &mut scope).unwrap();
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
            title: String::from("Untitled Notebook"),
        }
    }

    pub fn get_cell_mut(&mut self, cell_uuid: &str) -> Option<&mut Cell> {
        self.topology.get_cell_mut(cell_uuid)
    }

    pub fn eval_cell(&self, cell: &mut Cell) -> Result<(), Box<dyn Error>> {
        match cell.cell_type {
            CellType::NonReactiveCode => todo!(),
            CellType::ReactiveCode => self.eval_reactive_cell(cell),
            CellType::Markdown => todo!(),
        }
    }

    fn eval_reactive_cell(&self, cell: &mut Cell) -> Result<(), Box<dyn Error>> {
        let execution_seq = self.topology.execution_seq(&cell.uuid)?;

        let execution_seq = execution_seq
            .iter()
            .map(|uuid| self.topology.get_cell(uuid).unwrap())
            .collect::<Vec<_>>();

        info!(
            "Execution sequence: {:?}",
            execution_seq.iter().map(|c| &c.content).collect::<Vec<_>>()
        );

        for cell in execution_seq {
            let dependencies = self.topology.get_dependencies(&cell.uuid);
            self.kernel.eval(cell, &dependencies);
        }

        Ok(())
    }
}

impl From<&str> for Notebook {
    fn from(path: &str) -> Self {
        let json = std::fs::read_to_string(path).unwrap();
        serde_json::from_str(&json).unwrap()
    }
}
