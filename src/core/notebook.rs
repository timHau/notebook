use super::{cell::CellType, errors::NotebookErrors};
use crate::{
    api::routes::EvalResult,
    core::{cell::Cell, statement_pos::ExecutionType, topology::Topology},
    kernel::kernel_client::{KernelClient, KernelMessage},
};
use nanoid::nanoid;
use pyo3::{types::PyDict, PyResult, Python};
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
            if let Some(next_cell) = self.topology.get_cell_mut(&uuid) {
                match next_cell.cell_type {
                    CellType::ReactiveCode => {
                        let dependencies = topology.get_dependencies(&next_cell.uuid);
                        let locals = Self::locals_from_dependencies(&dependencies)?;
                        let msg = KernelMessage {
                            content: next_cell.content.clone(),
                            locals: locals.clone(),
                            execution_type: ExecutionType::Exec,
                        };
                        let res = kernel_client.send_to_kernel(&msg)?;
                        info!("res: {:#?}", res);
                        // let locals = res.locals;
                        // next_cell.locals = Some(locals);

                        // let sorted_statements = next_cell.sorted_statements();

                        // for statement in sorted_statements {
                        //     let code = statement.extract_code(&next_cell.content);
                        //     let msg = KernelMessage {
                        //         content: code,
                        //         locals: locals.clone(),
                        //         execution_type: statement.execution_type,
                        //     };
                        //     let res = kernel_client.send_to_kernel(&msg)?;
                        //     info!("res: {:#?}", res);
                        // let locals = res.locals;
                        // next_cell.locals = Some(locals);
                        // result.insert(next_cell.uuid.clone(), cell_res);
                        // }
                    }
                    _ => return Err(Box::new(NotebookErrors::NotYetImplemented)),
                }
            }
        }

        Ok(result)
    }

    fn locals_from_dependencies(
        dependencies: &[&Cell],
    ) -> Result<HashMap<String, Value>, Box<dyn Error>> {
        let res = Python::with_gil(|py| -> PyResult<HashMap<String, String>> {
            let locals = PyDict::new(py);

            // merge dependencies locals
            for dependency in dependencies.iter() {
                let dep_locals = dependency.locals.clone().unwrap();
                locals.update(dep_locals.as_ref(py).as_mapping()).unwrap();
            }

            Ok(locals.extract().unwrap())
        })?;

        Ok(res
            .into_iter()
            .map(|(k, v)| (k, Value::String(v)))
            .collect())
    }

    pub fn eval(
        &self,
        cell: &Cell,
        dependencies: &[&Cell],
    ) -> Result<HashMap<String, String>, Box<dyn Error>> {
        // let res = Python::with_gil(|py| -> PyResult<HashMap<String, String>> {
        //     let locals = cell.locals.clone().unwrap();
        //     let locals = locals.as_ref(py);

        //     // merge dependencies locals
        //     for dependency in dependencies.iter() {
        //         let dep_locals = dependency.locals.clone().unwrap();
        //         locals.update(dep_locals.as_ref(py).as_mapping()).unwrap();
        //     }

        //     // sort statements by row
        //     let sorted_statements = cell
        //         .statements
        //         .iter()
        //         .sorted_by(|pos_1, pos_2| {
        //             let row_1 = [pos_1.row_start, pos_1.row_end];
        //             let row_2 = [pos_2.row_start, pos_2.row_end];
        //             row_1.cmp(&row_2)
        //         })
        //         .collect::<Vec<_>>();

        //     // exec/eval each statement after the other
        //     let mut res = HashMap::new();
        //     for statement in sorted_statements {
        //         let code = statement.extract_code(&cell.content);

        //         println!("code: {}, statement: {:?}", code, statement);
        //         match statement.execution_type {
        //             ExecutionType::Eval => {
        //                 match py.eval(&code, Some(self.globals.as_ref(py)), Some(locals)) {
        //                     Ok(code) => res.insert("RETURN".to_string(), code.to_string()),
        //                     Err(err) => {
        //                         warn!("Error: {:#?}", err);
        //                         return Err(err);
        //                     }
        //                 };
        //             }
        //             ExecutionType::Exec => {
        //                 match py.run(&code, Some(self.globals.as_ref(py)), Some(locals)) {
        //                     Ok(_) => {
        //                         for binding in cell.bindings.iter() {
        //                             if let Some(value) = locals.get_item(binding) {
        //                                 res.insert(binding.to_string(), value.to_string());
        //                             }
        //                         }
        //                     }
        //                     Err(err) => {
        //                         warn!("Error: {:#?}", err);
        //                         return Err(err);
        //                     }
        //                 };
        //             }
        //             ExecutionType::Import => todo!(),
        //         }
        //     }

        //     Ok(res)
        // });

        // match res {
        //     Ok(res) => Ok(res),
        //     Err(err) => Err(Box::new(err)),
        // }
        todo!()
    }
}

impl From<&str> for Notebook {
    fn from(path: &str) -> Self {
        let json = std::fs::read_to_string(path).unwrap();
        serde_json::from_str(&json).unwrap()
    }
}
