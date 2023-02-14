use crate::core::cell::Cell;
use core::fmt;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error::Error};
use tracing::info;

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

    pub fn build(&mut self) -> Result<(), Box<dyn Error>> {
        let topo_sorted = self.topological_sort()?;
        Ok(())
    }

    pub fn add_cell(&mut self, cell: Cell) -> Result<(), Box<dyn Error>> {
        let cells: Vec<Cell> = self.cells.values().cloned().collect();
        let mut cell = cell;
        // let deps = cell.build_dependencies(&cells)?;
        // info!("Cell {} dependencies: {:#?}", cell.uuid, deps);

        self.cells.insert(cell.uuid.clone(), cell);
        Ok(())
    }

    pub fn get_cell_mut(&mut self, uuid: &str) -> Option<&mut Cell> {
        self.cells.get_mut(uuid)
    }
}

struct TopoNode<'a> {
    cell: &'a Cell,
    mark: TopoMark,
}

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
        let mut sorted = vec![];
        let mut nodes = vec![];

        for (_uuid, cell) in self.cells.iter() {
            let node = TopoNode {
                cell,
                mark: TopoMark::Unmarked,
            };
            nodes.push(node);
        }

        for node in nodes.iter_mut() {
            Self::visit(node, &self.cells)?;
            sorted.push(node.cell.uuid.clone());
        }

        Ok(sorted)
    }

    fn visit(node: &mut TopoNode, cells: &HashMap<String, Cell>) -> Result<(), Box<dyn Error>> {
        match node.mark {
            TopoMark::PermMark => Ok(()),
            TopoMark::TempMark => Err(Box::new(CycleError)),
            TopoMark::Unmarked => {
                node.mark = TopoMark::TempMark;
                for dep_id in node.cell.dependencies.iter() {
                    let child = match cells.get(dep_id) {
                        Some(child) => child,
                        None => return Err(Box::new(CycleError)),
                    };

                    let mut child_node = TopoNode {
                        cell: child,
                        mark: TopoMark::Unmarked,
                    };
                    match Self::visit(&mut child_node, cells) {
                        Ok(_) => (),
                        Err(e) => return Err(e),
                    }
                }
                node.mark = TopoMark::PermMark;
                Ok(())
            }
        }
    }
}
