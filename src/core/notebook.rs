use super::{cell::CellType, kernel::Kernel};
use crate::{
    api::routes::EvalResult,
    core::{cell::Cell, topology::Topology},
};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error::Error};

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
    pub scope: Scope,
}

impl Notebook {
    pub fn new() -> Self {
        let mut scope = Scope::default();
        // let code_cell_1 = Cell::new(
        //     CellType::ReactiveCode,
        //     String::from("def add(a, b): return a + b"),
        //     &mut scope,
        // )
        // .unwrap();
        let code_cell_1 = Cell::new_reactive("def add(a, b):\n  return a + b", &mut scope).unwrap();
        let code_cell_2 = Cell::new_reactive("a = 1 + 2\nb = 5\nc = 12", &mut scope).unwrap();
        let code_cell_3 = Cell::new_reactive("add(5, 2)", &mut scope).unwrap();
        let code_cell_4 =
            Cell::new_reactive("sum = 0\nfor i in range(10):\n  sum += a", &mut scope).unwrap();

        let mut topology = Topology::from_vec(
            vec![code_cell_1, code_cell_2, code_cell_3, code_cell_4],
            &mut scope,
        )
        .unwrap();
        topology.build(&mut scope).unwrap();

        let kernel = Kernel::new();
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

    pub fn eval_cell(&self, cell: &mut Cell) -> Result<EvalResult, Box<dyn Error>> {
        match cell.cell_type {
            CellType::NonReactiveCode => todo!(),
            CellType::ReactiveCode => self.eval_reactive_cell(cell),
            CellType::Markdown => todo!(),
        }
    }

    fn eval_reactive_cell(&self, cell: &mut Cell) -> Result<EvalResult, Box<dyn Error>> {
        let execution_seq = self.topology.execution_seq(&cell.uuid)?;

        let execution_seq = execution_seq
            .iter()
            .map(|uuid| self.topology.get_cell(uuid).unwrap())
            .collect::<Vec<_>>();

        let mut result = HashMap::new();
        for cell in execution_seq {
            let dependencies = self.topology.get_dependencies(&cell.uuid);
            let cell_res = self.kernel.eval(cell, &dependencies)?;
            result.insert(cell.uuid.clone(), cell_res);
        }

        Ok(result)
    }
}

impl From<&str> for Notebook {
    fn from(path: &str) -> Self {
        let json = std::fs::read_to_string(path).unwrap();
        serde_json::from_str(&json).unwrap()
    }
}
