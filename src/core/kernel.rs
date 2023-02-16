use super::cell::{Cell, Dependencies};
use pyo3::{prelude::*, types::PyDict};

#[derive(Debug, Clone)]
pub struct Kernel {
    pub version: String,
    globals: Py<PyDict>,
}

impl Kernel {
    pub fn start() -> Self {
        let version_info = Python::with_gil(move |py| {
            let sys = py.import("sys").unwrap();
            let version = sys.getattr("version").unwrap();
            version.to_string()
        });
        let version = version_info.split(" ").collect::<Vec<&str>>()[0];
        Self {
            version: version.to_string(),
            globals: Python::with_gil(|py| PyDict::new(py).into()),
        }
    }

    pub fn eval(&mut self, cell: &mut Cell, dependencies: &[Py<PyDict>]) {
        Python::with_gil(|py| {
            let locals = cell.locals.clone().unwrap();
            let locals = locals.as_ref(py);

            for dep in dependencies.iter() {
                let dep = dep.as_ref(py).as_mapping();
                locals.update(dep).unwrap();
            }

            match py.run(&cell.content, Some(self.globals.as_ref(py)), Some(locals)) {
                Ok(res) => println!("Success with result: {:?}", res),
                Err(e) => println!("Error: {}", e),
            };
            // println!("Locals: {:#?}", locals.as_ref(py));
        });
    }
}

impl Default for Kernel {
    fn default() -> Self {
        Self::start()
    }
}
