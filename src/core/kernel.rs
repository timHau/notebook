use crate::core::statement_pos::ExecutionType;

use super::cell::Cell;
use itertools::Itertools;
use pyo3::{prelude::*, types::PyDict};
use std::error::Error;
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

    pub fn eval(&self, cell: &Cell, dependencies: &[&Cell]) -> Result<(), Box<dyn Error>> {
        Python::with_gil(|py| {
            let locals = cell.locals.clone().unwrap();
            let locals = locals.as_ref(py);

            for dependency in dependencies.iter() {
                let dep_locals = dependency.locals.clone().unwrap();
                locals.update(dep_locals.as_ref(py).as_mapping()).unwrap();
            }

            let sorted_statements = cell
                .statements
                .iter()
                .sorted_by(|pos_1, pos_2| {
                    let row_1 = pos_1.row;
                    let row_2 = pos_2.row;
                    row_1.cmp(&row_2)
                })
                .collect::<Vec<_>>();

            info!("Sorted statements: {:#?}", sorted_statements);
            for statement in sorted_statements {
                let code = statement.extract_code(&cell.content);

                info!("Code: {}", code);
                match statement.execution_type {
                    ExecutionType::Eval => {
                        match py.eval(&code, Some(self.globals.as_ref(py)), Some(locals)) {
                            Ok(code) => {
                                info!("Eval Code: {:#?}, locals: {:#?}", code, locals);
                            }
                            Err(e) => warn!("Error: {}", e),
                        };
                    }
                    ExecutionType::Exec => {
                        match py.run(&code, Some(self.globals.as_ref(py)), Some(locals)) {
                            Ok(code) => {
                                info!("Exec Code: {:#?}, locals: {:#?}", code, locals);
                            }
                            Err(e) => warn!("Error: {}", e),
                        };
                    }
                }
            }
        });

        Ok(())
    }
}

impl Default for Kernel {
    fn default() -> Self {
        Self::new()
    }
}
