use super::{
    cell::{CellType, LocalValue},
    errors::NotebookErrors,
};
use crate::{
    api::routes::EvalResult,
    core::{
        cell::Cell,
        kernel_client::{ExecutionType, KernelClient, KernelMessage},
        topology::Topology,
    },
};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, error::Error};
use tracing::info;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
struct LanguageInfo {
    name: String,
    // version: String,
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
                Cell::new_reactive("sum = 0\nfor i in range(10):\n  sum += 1", &mut scope).unwrap(),
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

        Self {
            uuid: nanoid!(30),
            meta_data: NotebookMetadata::default(),
            scope,
            language_info: LanguageInfo {
                name: String::from("python"),
                // version,
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
        kernel_client: &KernelClient,
    ) -> Result<EvalResult, Box<dyn Error>> {
        // update cell content if it has changed
        self.topology
            .update_cell(cell_uuid, next_content, &mut self.scope)?;

        let mut result = HashMap::new();

        // get an topological order of the cell uuids and execute them in order
        let execution_seq = self.topology.execution_seq(cell_uuid)?;
        for uuid in execution_seq {
            let topology = self.topology.clone();
            if let Some(cell) = self.topology.get_cell_mut(&uuid) {
                info!("cell: {:#?}", cell);
                match cell.cell_type {
                    CellType::ReactiveCode => {
                        let dependencies = topology.get_dependencies(&cell.uuid);
                        info!("dependencies: {:#?}", dependencies);
                        for statement in cell.statements.iter() {
                            let locals = Self::locals_from_dependencies(&cell, &dependencies);
                            let msg = KernelMessage {
                                content: statement.content.clone(),
                                locals: locals.clone(),
                                execution_type: statement.execution_type.clone(),
                            };
                            let res = kernel_client.send_to_kernel(&msg)?;
                            info!("res: {:#?}", res);
                            cell.locals.extend(res.locals.clone());
                            info!("cell.locals: {:#?}", cell.locals);
                            result.insert(cell.uuid.clone(), cell.locals.clone());
                        }
                    }
                    _ => return Err(Box::new(NotebookErrors::NotYetImplemented)),
                }
            }
        }

        Ok(result)
    }

    fn locals_from_dependencies(
        cell: &Cell,
        dependencies: &[&Cell],
    ) -> HashMap<String, LocalValue> {
        let mut locals = HashMap::new();
        locals.extend(cell.locals.clone());

        for dependency in dependencies.iter() {
            for (key, value) in dependency.locals.clone().iter() {
                if cell.required.contains(key) {
                    locals.insert(key.clone(), value.clone());
                }
            }
        }
        locals
    }

    pub fn reorder_cells(&mut self, cell_uuids: &[String]) {
        self.topology.reorder_cells(cell_uuids);
    }
}

impl From<&str> for Notebook {
    fn from(path: &str) -> Self {
        let json = std::fs::read_to_string(path).unwrap();
        serde_json::from_str(&json).unwrap()
    }
}
