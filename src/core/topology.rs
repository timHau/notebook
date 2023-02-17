use super::notebook::Scope;
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

    /// initialize a topology from a vector of cells
    /// we assume that every cell has all its local variables writen to the scope
    /// but it might happen that not all dependencies are found
    /// therefore we need to build the dependencies and dependents
    /// example where this is needed:
    ///
    /// cell 1: a = 1
    /// cell 2: b = a + c  (a is found, c is not found)
    /// cell 3: c = 2
    pub fn from_vec(cells: Vec<&Cell>, scope: &mut Scope) -> Result<Self, Box<dyn Error>> {
        let mut topology = Topology::new();
        for cell in cells {
            topology.display_order.push(cell.uuid.clone());
            topology.cells.insert(cell.uuid.clone(), cell.clone());
        }
        topology.build(scope)?;

        let _ = topology.topological_sort()?; // check for cycles

        Ok(topology)
    }

    /// add a cell to the topology without building the topology again
    fn add_cell(&mut self, cell: &Cell) {
        self.cells.insert(cell.uuid.clone(), cell.clone());
        self.display_order.push(cell.uuid.clone());
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

    pub fn get_cell_mut(&mut self, uuid: &str) -> Option<&mut Cell> {
        self.cells.get_mut(uuid)
    }

    pub fn get_cell(&self, uuid: &str) -> Option<&Cell> {
        self.cells.get(uuid)
    }

    pub fn get_dependencies(&self, uuid: &str) -> Vec<&Cell> {
        let cell = match self.cells.get(uuid) {
            Some(cell) => cell,
            None => {
                return Vec::new();
            }
        };
        cell.dependencies
            .iter()
            .map(|uuid| &self.cells[uuid])
            .collect::<Vec<_>>()
    }

    pub fn execution_seq(&self, cell_uuid: &str) -> Result<Vec<String>, Box<dyn Error>> {
        let cell = match self.cells.get(cell_uuid) {
            Some(cell) => cell,
            None => return Err(Box::new(TopologyErrors::CellNotFound)),
        };

        let mut nodes = Vec::with_capacity(cell.dependents.len() + cell.dependencies.len() + 1);

        for dependent in cell.dependents.clone() {
            let cell = match self.cells.get(&dependent) {
                Some(cell) => cell,
                None => return Err(Box::new(TopologyErrors::CellNotFound)),
            };
            nodes.push(cell);
        }

        for dependency in cell.dependencies.clone() {
            let cell = match self.cells.get(&dependency) {
                Some(cell) => cell,
                None => return Err(Box::new(TopologyErrors::CellNotFound)),
            };
            nodes.push(cell);
        }

        nodes.push(cell);

        let mut update_topology = Self::new();
        for node in nodes {
            update_topology.add_cell(&node.clone());
        }

        let sorted = update_topology.topological_sort()?;
        Ok(sorted)
    }

    fn topological_sort(&self) -> Result<Vec<String>, Box<dyn Error>> {
        let mut sorted = vec![];

        let cells = self.cells.clone();
        let nodes = cells.values().collect::<Vec<_>>();

        let mut degree = HashMap::new();
        for cell_uuid in self.display_order.iter() {
            let cell = match cells.get(cell_uuid) {
                Some(cell) => cell,
                None => return Err(Box::new(TopologyErrors::CellNotFound)),
            };
            let deg = cell
                .dependencies
                .iter()
                .filter(|&uuid| cells.contains_key(uuid))
                .count();
            degree.insert(cell_uuid.clone(), deg);
        }

        let mut queue = VecDeque::new();
        for (cell_uuid, degree) in degree.iter() {
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

            for dependent_uuid in cell.dependents.clone().iter() {
                let degree = match degree.get_mut(dependent_uuid) {
                    Some(degree) => degree,
                    None => continue,
                };
                *degree -= 1;

                if *degree == 0 {
                    let dependent = match cells.get(dependent_uuid) {
                        Some(dependent) => dependent,
                        None => return Err(Box::new(TopologyErrors::CellNotFound)),
                    };
                    queue.push_back(dependent);
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

        let topology = Topology::from_vec(vec![&code_cell_1, &code_cell_2], &mut scope).unwrap();

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

        let topology =
            Topology::from_vec(vec![&code_cell_1, &code_cell_2, &code_cell_3], &mut scope).unwrap();

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

        let topology = Topology::from_vec(
            vec![&code_cell_1, &code_cell_2, &code_cell_3, &code_cell_4],
            &mut scope,
        )
        .unwrap();

        let sorted = topology.topological_sort().unwrap();
        assert_eq!(sorted.last(), Some(&code_cell_2.uuid));
        assert_eq!(sorted.get(2).unwrap(), &code_cell_3.uuid.clone());
    }

    #[test]
    fn test_topo_sort_4_add_cell() {
        let mut scope = HashMap::new();
        let code_cell_1 = Cell::new_reactive("a = 1", &mut scope).unwrap();
        let code_cell_2 = Cell::new_reactive("b = a + c", &mut scope).unwrap();
        let code_cell_3 = Cell::new_reactive("c = d", &mut scope).unwrap();

        let topology =
            Topology::from_vec(vec![&code_cell_1, &code_cell_2, &code_cell_3], &mut scope).unwrap();

        let sorted = topology.topological_sort().unwrap();
        assert_eq!(sorted.last(), Some(&code_cell_2.uuid));
    }

    #[test]
    fn test_topo_sort_cycle_detected() {
        let mut scope = HashMap::new();
        let code_cell_1 = Cell::new_reactive("a = 1", &mut scope).unwrap();
        let code_cell_2 = Cell::new_reactive("b = a + c", &mut scope).unwrap();
        let code_cell_3 = Cell::new_reactive("c = d", &mut scope).unwrap();
        let code_cell_4 = Cell::new_reactive("d = b", &mut scope).unwrap();

        let topology = Topology::from_vec(
            vec![&code_cell_1, &code_cell_2, &code_cell_3, &code_cell_4],
            &mut scope,
        );
        assert!(topology.is_err());
        assert!(topology.err().unwrap().is::<TopologyErrors>());
    }

    #[test]
    fn test_cycle_build_should_fail() {
        let mut scope = HashMap::new();
        let code_cell_1 = Cell::new_reactive("a = 1", &mut scope).unwrap();
        let code_cell_2 = Cell::new_reactive("b = a + c", &mut scope).unwrap();
        let code_cell_3 = Cell::new_reactive("c = d", &mut scope).unwrap();
        let code_cell_4 = Cell::new_reactive("d = b", &mut scope).unwrap();

        let topology = Topology::from_vec(
            vec![&code_cell_1, &code_cell_2, &code_cell_3, &code_cell_4],
            &mut scope,
        );
        assert!(topology.is_err());
        assert!(topology.err().unwrap().is::<TopologyErrors>());
    }

    #[test]
    fn test_execution_seq() {
        let mut scope = HashMap::new();
        let code_cell_1 = Cell::new_reactive("a = 1", &mut scope).unwrap();
        let code_cell_2 = Cell::new_reactive("b = a + c", &mut scope).unwrap();
        let code_cell_3 = Cell::new_reactive("c = 3", &mut scope).unwrap();
        let code_cell_4 = Cell::new_reactive("d = 4", &mut scope).unwrap();

        let topology = Topology::from_vec(
            vec![&code_cell_1, &code_cell_2, &code_cell_3, &code_cell_4],
            &mut scope,
        )
        .unwrap();

        let execution_seq = topology.execution_seq(&code_cell_3.uuid).unwrap();

        assert_eq!(
            execution_seq,
            vec![code_cell_3.uuid.clone(), code_cell_2.uuid.clone()]
        );
    }

    #[test]
    fn test_execution_seq_2() {
        let mut scope = HashMap::new();
        let code_cell_1 = Cell::new_reactive("a = b + 1", &mut scope).unwrap();
        let code_cell_2 = Cell::new_reactive("b = 2", &mut scope).unwrap();
        let code_cell_3 = Cell::new_reactive("c = 1", &mut scope).unwrap();

        let topology =
            Topology::from_vec(vec![&code_cell_1, &code_cell_2, &code_cell_3], &mut scope).unwrap();

        let execution_seq = topology.execution_seq(&code_cell_2.uuid).unwrap();

        assert_eq!(
            execution_seq,
            vec![code_cell_2.uuid.clone(), code_cell_1.uuid.clone()]
        );
    }
}
