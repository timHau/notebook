use super::{cell::CellType, errors::NotebookErrors, kernel::Kernel};
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
        let mut topology = Topology::from_vec(
            vec![
                Cell::new_reactive("def add(a, b):\n  return a + b", &mut scope).unwrap(),
                Cell::new_reactive("a = 1 + 2\nb = 5\nc = 12", &mut scope).unwrap(),
                Cell::new_reactive("add(5, 2)", &mut scope).unwrap(),
                Cell::new_reactive("sum = 0\nfor i in range(10):\n  sum += a", &mut scope).unwrap(),
                Cell::new_reactive("print(123)", &mut scope).unwrap(),
                Cell::new_reactive(
                    "from torch import nn\nfrom torch.utils.data import DataLoader\nfrom torchvision import datasets\nfrom torchvision.transforms import ToTensor\n\ntraining_data = datasets.FashionMNIST(\n  root='data',\n  train=True,\n  download=True,\n  transform=ToTensor\n)",
                    &mut scope,
                )
                .unwrap(),
                Cell::new_reactive(
                    "import asyncio\n\nasync def main():\n  print('hello')\n\nasyncio.run(main())",
                    &mut scope,
                ).unwrap(),
            ],
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

    pub fn eval_cell(
        &mut self,
        cell_uuid: &str,
        next_content: &str,
    ) -> Result<EvalResult, Box<dyn Error>> {
        // update cell content if it has changed
        self.topology
            .update_cell(cell_uuid, next_content, &mut self.scope)?;

        let mut result = HashMap::new();

        // get an topological order of the cell uuids and execute them in order
        let execution_seq = self.topology.execution_seq(cell_uuid)?;
        for uuid in execution_seq {
            let topology = self.topology.clone();
            if let Some(next_cell) = self.topology.get_cell_mut(&uuid) {
                match next_cell.cell_type {
                    CellType::ReactiveCode => {
                        let dependencies = topology.get_dependencies(&next_cell.uuid);
                        let cell_res = self.kernel.eval(next_cell, &dependencies)?;
                        result.insert(next_cell.uuid.clone(), cell_res);
                    }
                    _ => return Err(Box::new(NotebookErrors::NotYetImplemented)),
                }
            }
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
