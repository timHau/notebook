use crate::cell::Cell;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Topology {
    pub trees: HashMap<String, Vec<String>>,
    pub cells: HashMap<String, Cell>,
}

impl Topology {
    pub fn eval(&mut self, cell_uuid: &str) {
        let cell = self.cells.get_mut(cell_uuid);
        if cell.is_none() {
            println!("Cell not found");
            return;
        }
        let cell = cell.unwrap();
        cell.eval();

        // TODO update topology if neccessary

        let dependents = self.trees.get(cell_uuid);
        if dependents.is_none() {
            return;
        }
        let dependents = dependents.unwrap();

        for dependent in dependents.iter() {
            println!("depending on {}", dependent);
        }
    }
}
