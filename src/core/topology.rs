use crate::core::{cell::Cell, graph::Graph};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Topology {
    pub cells: HashMap<String, Cell>,
    pub graph: Graph,
}

impl Topology {
    pub fn new() -> Self {
        Self {
            cells: HashMap::new(),
            graph: Graph::new(),
        }
    }

    pub fn eval(&mut self, cell: &Cell) -> Result<(), Box<dyn error::Error>> {
        cell.parse()?;

        // let dependents = match self.adj_list.get(cell.uuid) {
        //     Some(dependents) => dependents,
        //     None => return,
        // };

        // for dependent in dependents.iter() {
        //     println!("depending on {}", dependent);
        // }
        Ok(())
    }

    pub fn add_cell(
        &mut self,
        cell: Cell,
        child_uuids: Option<&Vec<String>>,
    ) -> Result<(), Box<dyn error::Error>> {
        self.graph.add_node(&cell.uuid, child_uuids)?;
        self.cells.insert(cell.uuid.clone(), cell);
        Ok(())
    }

    pub fn get_cell(&self, uuid: &str) -> Option<&Cell> {
        self.cells.get(uuid)
    }
}
