use super::notebook::Scope;
use crate::core::cell::Cell;
use core::fmt;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error::Error};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Topology {
    pub cells: HashMap<String, Cell>,
}

impl Topology {
    pub fn new() -> Self {
        Self {
            cells: HashMap::new(),
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
        self.cells.insert(cell.uuid.clone(), cell.clone());
    }

    pub fn get_cell_mut(&mut self, uuid: &str) -> Option<&mut Cell> {
        self.cells.get_mut(uuid)
    }
}

impl From<Vec<&Cell>> for Topology {
    fn from(cells: Vec<&Cell>) -> Self {
        let mut topology = Topology::new();
        for cell in cells {
            topology.add_cell(cell);
        }
        topology
    }
}

#[derive(Debug, Clone)]
struct TopoNode<'a> {
    cell: &'a Cell,
    mark: TopoMark,
}

#[derive(Debug, Clone)]
enum TopoMark {
    Unmarked,
    TempMark,
    PermMark,
}

#[derive(Debug)]
struct CycleError;

impl fmt::Display for CycleError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Cycle detected")
    }
}

impl Error for CycleError {}

impl Topology {
    fn topological_sort(&self) -> Result<Vec<String>, Box<dyn Error>> {
        // let mut sorted = vec![];
        // let mut nodes = HashMap::new();

        // for (_uuid, cell) in self.cells.iter() {
        //     let node = TopoNode {
        //         cell,
        //         mark: TopoMark::Unmarked,
        //     };
        //     nodes.insert(cell.uuid.clone(), node);
        // }

        // for (cell_uuid, node) in nodes.clone().iter_mut() {
        //     Self::visit(node, &mut nodes)?;
        //     sorted.push(cell_uuid.clone());
        // }

        // Ok(sorted)
        todo!()
    }

    fn visit(
        node: &mut TopoNode,
        nodes: &mut HashMap<String, TopoNode>,
    ) -> Result<(), Box<dyn Error>> {
        // match node.mark {
        //     TopoMark::PermMark => Ok(()),
        //     TopoMark::TempMark => Err(Box::new(CycleError)),
        //     TopoMark::Unmarked => {
        //         node.mark = TopoMark::TempMark;

        //         for dep_id in node.cell.dependencies.iter() {
        //             let child_node = nodes.get_mut(dep_id).unwrap();
        //             Self::visit(child_node, nodes)?;
        //         }

        //         node.mark = TopoMark::PermMark;
        //         Ok(())
        //     }
        // }
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topo_sort() {
        // let mut scope = HashMap::new();
        // let code_cell_1 = Cell::new_reactive("a = 1", &mut scope).unwrap();
        // let code_cell_2 = Cell::new_reactive("b = a + 1", &mut scope).unwrap();

        // let mut topology = Topology::new();
        // topology.add_cell(&code_cell_1);
        // topology.add_cell(&code_cell_2);

        // println!("{:#?}", topology.cells);

        // let sorted = topology.topological_sort().unwrap();
        // let expect = vec![code_cell_1.uuid.clone(), code_cell_2.uuid.clone()];
        // assert_eq!(sorted, expect);
    }
}
