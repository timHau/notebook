use crate::core::statement_pos::ExecutionType;

use super::cell::Cell;
use itertools::Itertools;
use pyo3::{prelude::*, types::PyDict};
use std::{collections::HashMap, error::Error};
use tracing::{info, warn};

#[derive(Debug, Clone)]
pub struct Kernel {
    pub version: String,
    globals: Py<PyDict>,
}

impl Kernel {
    pub fn new() -> Self {
        let version_info = Python::with_gil(move |py| {
            let sys = py.import("sys").unwrap();
            let version = sys.getattr("version").unwrap();
            version.to_string()
        });
        let version = version_info.split(' ').collect::<Vec<&str>>()[0];
        Self {
            version: version.to_string(),
            globals: Python::with_gil(|py| PyDict::new(py).into()),
        }
    }

    pub fn eval(
        &self,
        cell: &Cell,
        dependencies: &[&Cell],
    ) -> Result<HashMap<String, String>, Box<dyn Error>> {
        let res = Python::with_gil(|py| -> PyResult<HashMap<String, String>> {
            let locals = cell.locals.clone().unwrap();
            let locals = locals.as_ref(py);

            // merge dependencies locals
            for dependency in dependencies.iter() {
                let dep_locals = dependency.locals.clone().unwrap();
                locals.update(dep_locals.as_ref(py).as_mapping()).unwrap();
            }

            // sort statements by row
            let sorted_statements = cell
                .statements
                .iter()
                .sorted_by(|pos_1, pos_2| {
                    let row_1 = pos_1.row;
                    let row_2 = pos_2.row;
                    row_1.cmp(&row_2)
                })
                .collect::<Vec<_>>();

            // exec/eval each statement after the other
            let mut res = HashMap::new();
            for statement in sorted_statements {
                let code = statement.extract_code(&cell.content);

                match statement.execution_type {
                    ExecutionType::Eval => {
                        match py.eval(&code, Some(self.globals.as_ref(py)), Some(locals)) {
                            Ok(code) => res.insert("RETURN".to_string(), code.to_string()),
                            Err(err) => return Err(err),
                        };
                    }
                    ExecutionType::Exec => {
                        match py.run(&code, Some(self.globals.as_ref(py)), Some(locals)) {
                            Ok(_) => {
                                for binding in cell.bindings.iter() {
                                    if let Some(value) = locals.get_item(binding) {
                                        res.insert(binding.to_string(), value.to_string());
                                    }
                                }
                            }
                            Err(err) => return Err(err),
                        };
                    }
                }
            }

            info!("Code: {}, Result: {:?}", cell.content, res);

            Ok(res)
        })?;

        Ok(res)
    }
}

impl Default for Kernel {
    fn default() -> Self {
        Self::new()
    }
}
