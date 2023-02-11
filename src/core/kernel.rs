use pyo3::prelude::*;

#[derive(Debug)]
pub struct Kernel {
    pub version: String,
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
        }
    }
}
