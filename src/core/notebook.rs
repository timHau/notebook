use super::{
    cell::{CellType, LocalValue},
    kernel_client::{KernelClient, KernelClientMsg},
};
use crate::core::{cell::Cell, kernel_client::MsgToKernel, topology::Topology};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error::Error, sync::mpsc::Sender};
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

    #[serde(skip)]
    pub kernel_sender: Option<Sender<KernelClientMsg>>,
}

impl Notebook {
    pub fn new(kernel_sender: Sender<KernelClientMsg>) -> Self {
        let mut scope = Scope::default();
        let mut topology = Topology::from_vec(
            vec![
                Cell::new_reactive("def add(a, b):\n  return a + b", &mut scope).unwrap(),
                Cell::new_reactive("a = 1 + 2\nb = 5\nc = 12", &mut scope).unwrap(),
                Cell::new_reactive("add(5, 2)", &mut scope).unwrap(),
                Cell::new_reactive("sum = 0\nfor i in range(10):\n  sum += 1", &mut scope).unwrap(),
                Cell::new_reactive("print(123)", &mut scope).unwrap(),
                Cell::new_reactive(
                    "import asyncio\n\nasync def main():\n  print('hello')\n\nasyncio.run(main())",
                    &mut scope,
                )
                .unwrap(),
            ],
            &mut scope,
        )
        .unwrap();
        topology.build(&mut scope).unwrap();

        let uuid = nanoid!(30);
        Self {
            uuid,
            meta_data: NotebookMetadata::default(),
            scope,
            language_info: LanguageInfo {
                name: String::from("python"),
                // version,
                file_extension: String::from(".py"),
            },
            topology,
            title: String::from("Untitled Notebook"),
            kernel_sender: Some(kernel_sender),
        }
    }

    pub fn eval_cell(&mut self, cell_uuid: &str, next_content: &str) -> Result<(), Box<dyn Error>> {
        // update cell content if it has changed
        self.topology
            .update_cell(cell_uuid, next_content, &mut self.scope)?;

        // get an topological order of the cell uuids and execute them in order
        let execution_seq = self.topology.execution_seq(cell_uuid)?;

        let execution_cells = execution_seq
            .iter()
            .map(|uuid| self.topology.cells.get(uuid).unwrap().clone())
            .collect::<Vec<_>>();

        let locals_of_deps = execution_cells
            .iter()
            .map(|cell| {
                let dependencies = self.topology.get_dependencies(&cell.uuid);
                Self::locals_from_dependencies(&cell, &dependencies)
            })
            .collect::<Vec<_>>();

        let msg = KernelClientMsg::MsgToKernel(MsgToKernel {
            notebook_uuid: self.uuid.clone(),
            cell_uuid: cell_uuid.to_string(),
            locals_of_deps,
            execution_cells,
        });
        let kernel_sender = self.kernel_sender.as_ref().unwrap();
        kernel_sender.send(msg)?;

        Ok(())
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
