use super::{kernel::Kernel, notebook::Scope};
use crate::core::{cell::Cell, errors::TopologyErrors};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, VecDeque},
    error::Error,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Topology {
    pub cells: HashMap<String, Cell>,
    pub display_order: Vec<String>,
}

impl Topology {
    pub fn new() -> Self {
        Self {
            cells: HashMap::new(),
            display_order: Vec::new(),
        }
    }

    fn build_dependencies(&mut self, scope: &mut Scope) -> Result<(), Box<dyn Error>> {
        for cell in self.cells.values_mut() {
            cell.build_dependencies(scope)?;
        }
        Ok(())
    }

    fn build_dependents(&mut self) {
        let mut cells = self.cells.clone();
        for cell in cells.clone().values().clone() {
            cell.build_dependents(&mut cells);
        }
        self.cells = cells;
    }

    pub fn build(&mut self, scope: &mut Scope) -> Result<(), Box<dyn Error>> {
        self.build_dependencies(scope)?;
        self.build_dependents();
        Ok(())
    }

    pub fn add_cell(&mut self, cell: &Cell) {
        self.display_order.push(cell.uuid.clone()); // TODO should be possible to add cell anywhere
        self.cells.insert(cell.uuid.clone(), cell.clone());
    }

    pub fn get_cell_mut(&mut self, uuid: &str) -> Option<&mut Cell> {
        self.cells.get_mut(uuid)
    }

    pub fn eval_cell(&mut self, kernel: &mut Kernel, uuid: &str) -> Result<(), Box<dyn Error>> {
        let cells = self.cells.clone();

        let cell = match self.cells.get_mut(uuid) {
            Some(cell) => cell,
            None => return Err(Box::new(TopologyErrors::CellNotFound)),
        };

        let mut dependencies = Vec::with_capacity(cell.dependencies.len());
        for dep_uuid in cell.dependencies.clone().iter() {
            let dep_cell = match cells.get(dep_uuid) {
                Some(dep_cell) => dep_cell,
                None => return Err(Box::new(TopologyErrors::CellNotFound)),
            };

            dependencies.push(dep_cell);
        }

        kernel.eval(cell, &dependencies);

        Ok(())
    }
}

impl From<Vec<&Cell>> for Topology {
    fn from(cells: Vec<&Cell>) -> Self {
        let mut topology = Topology::new();
        for cell in cells {
            topology.add_cell(cell);
            topology.display_order.push(cell.uuid.clone());
        }
        topology
    }
}

impl Topology {
    fn topological_sort(&self) -> Result<Vec<String>, Box<dyn Error>> {
        let mut sorted = vec![];

        let cells = self.cells.clone();
        let nodes = cells.values().collect::<Vec<_>>();

        let mut in_degree = HashMap::new();
        for cell_uuid in self.display_order.iter() {
            let cell = match cells.get(cell_uuid) {
                Some(cell) => cell,
                None => return Err(Box::new(TopologyErrors::CellNotFound)),
            };
            in_degree.insert(cell_uuid.clone(), cell.dependencies.len());
        }

        let mut queue = VecDeque::new();
        for (cell_uuid, degree) in in_degree.iter() {
            if *degree == 0 {
                let cell = match cells.get(cell_uuid) {
                    Some(cell) => cell,
                    None => return Err(Box::new(TopologyErrors::CellNotFound)),
                };
                queue.push_back(cell);
            }
        }

        let mut count = 0;
        while !queue.is_empty() {
            let cell = queue.pop_front().unwrap();
            sorted.push(cell.uuid.clone());
            eprintln!("sorted: {:#?}", cell);

            for dep_uuid in cell.dependents.clone().iter() {
                let degree = in_degree.get_mut(dep_uuid).unwrap();
                *degree -= 1;

                if *degree == 0 {
                    let dep_cell = match cells.get(dep_uuid) {
                        Some(dep_cell) => dep_cell,
                        None => return Err(Box::new(TopologyErrors::CellNotFound)),
                    };
                    let dep_node = dep_cell;
                    queue.push_back(dep_node);
                }
            }

            count += 1;
        }

        if count != nodes.len() {
            return Err(Box::new(TopologyErrors::CycleDetected));
        }

        Ok(sorted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topo_sort() {
        let mut scope = HashMap::new();
        let code_cell_1 = Cell::new_reactive("a = 1", &mut scope).unwrap();
        let code_cell_2 = Cell::new_reactive("b = a + 1", &mut scope).unwrap();

        let mut topology = Topology::from(vec![&code_cell_1, &code_cell_2]);
        topology.build(&mut scope).unwrap();

        let sorted = topology.topological_sort().unwrap();
        let expect = vec![code_cell_1.uuid.clone(), code_cell_2.uuid.clone()];
        assert_eq!(sorted, expect);
    }

    #[test]
    fn test_topo_sort_2() {
        let mut scope = HashMap::new();
        let code_cell_1 = Cell::new_reactive("a = 1", &mut scope).unwrap();
        let code_cell_2 = Cell::new_reactive("b = a + c", &mut scope).unwrap();
        let code_cell_3 = Cell::new_reactive("c = 4", &mut scope).unwrap();

        let mut topology = Topology::from(vec![&code_cell_1, &code_cell_2, &code_cell_3]);
        topology.build(&mut scope).unwrap();

        let sorted = topology.topological_sort().unwrap();
        assert_eq!(sorted.last(), Some(&code_cell_2.uuid));
    }

    #[test]
    fn test_topo_sort_3() {
        let mut scope = HashMap::new();
        let code_cell_1 = Cell::new_reactive("a = 1", &mut scope).unwrap();
        let code_cell_2 = Cell::new_reactive("b = a + c", &mut scope).unwrap();
        let code_cell_3 = Cell::new_reactive("c = d", &mut scope).unwrap();
        let code_cell_4 = Cell::new_reactive("d = 4", &mut scope).unwrap();

        let mut topology =
            Topology::from(vec![&code_cell_1, &code_cell_2, &code_cell_3, &code_cell_4]);
        topology.build(&mut scope).unwrap();

        let sorted = topology.topological_sort().unwrap();
        assert_eq!(sorted.last(), Some(&code_cell_2.uuid));
        assert_eq!(sorted.get(2).unwrap(), &code_cell_3.uuid.clone());
    }

    #[test]
    fn test_topo_sort_cycle_detected() {
        let mut scope = HashMap::new();
        let code_cell_1 = Cell::new_reactive("a = 1", &mut scope).unwrap();
        let code_cell_2 = Cell::new_reactive("b = a + c", &mut scope).unwrap();
        let code_cell_3 = Cell::new_reactive("c = d", &mut scope).unwrap();
        let code_cell_4 = Cell::new_reactive("d = b", &mut scope).unwrap();

        let mut topology =
            Topology::from(vec![&code_cell_1, &code_cell_2, &code_cell_3, &code_cell_4]);
        topology.build(&mut scope).unwrap();

        let sorted = topology.topological_sort();
        assert!(sorted.is_err());
        assert!(sorted.err().unwrap().is::<TopologyErrors>());
    }
}
