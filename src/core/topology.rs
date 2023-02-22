use super::notebook::Scope;
use crate::core::{cell::Cell, errors::TopologyErrors};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    error::Error,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Topology {
    pub cells: HashMap<String, Cell>,
    pub display_order: Vec<String>,
    pub dependencies: HashMap<String, HashSet<String>>,
    pub dependents: HashMap<String, HashSet<String>>,
}

impl Topology {
    pub fn new() -> Self {
        Self {
            cells: HashMap::new(),
            display_order: Vec::new(),
            dependencies: HashMap::new(),
            dependents: HashMap::new(),
        }
    }

    pub fn from_vec(cells: Vec<Cell>, scope: &mut Scope) -> Result<Self, Box<dyn Error>> {
        let mut topology = Topology::new();
        for cell in cells {
            topology.display_order.push(cell.uuid.clone());
            topology.cells.insert(cell.uuid.clone(), cell);
        }
        topology.build(scope)?;

        let _ = topology.topological_sort()?; // check for cycles

        Ok(topology)
    }

    pub fn get_cell_mut(&mut self, uuid: &str) -> Option<&mut Cell> {
        self.cells.get_mut(uuid)
    }

    pub fn update_cell(
        &mut self,
        cell_uuid: &str,
        next_content: &str,
        scope: &mut Scope,
    ) -> Result<(), Box<dyn Error>> {
        if let Some(cell) = self.get_cell_mut(cell_uuid) {
            if next_content == cell.content {
                return Ok(());
            }

            cell.update_content(next_content, scope)?;
            self.build(scope)?;
        }

        Ok(())
    }

    pub fn get_dependencies(&self, uuid: &str) -> Vec<&Cell> {
        self.dependencies
            .get(uuid)
            .unwrap_or(&HashSet::new())
            .iter()
            .filter_map(|uuid| self.cells.get(uuid))
            .collect()
    }

    pub fn get_dependents(&self, uuid: &str) -> Vec<&Cell> {
        self.dependents
            .get(uuid)
            .unwrap_or(&HashSet::new())
            .iter()
            .filter_map(|uuid| self.cells.get(uuid))
            .collect()
    }

    pub fn build(&mut self, scope: &mut Scope) -> Result<(), Box<dyn Error>> {
        self.dependencies.clear();
        self.dependents.clear();

        for cell in self.cells.values() {
            for required_var in cell.required.iter() {
                if let Some(other_uuid) = scope.get(required_var) {
                    if other_uuid == &cell.uuid {
                        continue;
                    }

                    self.dependents
                        .entry(other_uuid.clone())
                        .or_insert_with(HashSet::new)
                        .insert(cell.uuid.clone());

                    self.dependencies
                        .entry(cell.uuid.clone())
                        .or_insert_with(HashSet::new)
                        .insert(other_uuid.clone());
                }
            }
        }

        Ok(())
    }

    pub fn execution_seq(&self, cell_uuid: &str) -> Result<Vec<String>, Box<dyn Error>> {
        let cell = match self.cells.get(cell_uuid) {
            Some(cell) => cell,
            None => return Err(Box::new(TopologyErrors::CellNotFound)),
        };

        let mut dependencies = self.get_dependencies(cell_uuid);
        let mut dependents = self.get_dependents(cell_uuid);

        let mut nodes = Vec::with_capacity(dependencies.len() + dependents.len() + 1);
        nodes.push(cell);
        nodes.append(&mut dependencies);
        nodes.append(&mut dependents);

        let mut update_topology = Self::new();
        update_topology.dependencies = self.dependencies.clone();
        update_topology.dependents = self.dependents.clone();
        for node in nodes {
            update_topology
                .cells
                .insert(node.uuid.clone(), node.clone());
            update_topology.display_order.push(node.uuid.clone());
        }

        let sorted = update_topology.topological_sort()?;
        Ok(sorted)
    }

    fn topological_sort(&self) -> Result<Vec<String>, Box<dyn Error>> {
        let mut sorted = vec![];

        let cells = self.cells.clone();
        let nodes = cells.values().collect::<Vec<_>>();

        // setup hashmap uuid -> (number of dependencies, cell)
        let mut degree = HashMap::new();
        for cell_uuid in self.display_order.iter() {
            let cell = match cells.get(cell_uuid) {
                Some(cell) => cell,
                None => return Err(Box::new(TopologyErrors::CellNotFound)),
            };

            let dependencies = self.get_dependencies(&cell_uuid);
            degree.insert(cell_uuid.clone(), (dependencies.len(), cell));
        }

        // queue of cells with no dependencies
        let mut queue = VecDeque::new();
        for (_cell_uuid, (degree, cell)) in degree.iter() {
            if *degree == 0 {
                queue.push_back(*cell);
            }
        }

        let mut count = 0;
        while !queue.is_empty() {
            let cell = queue.pop_front().unwrap();
            sorted.push(cell.uuid.clone());

            for dependent in self.get_dependents(&cell.uuid) {
                let (degree, dependent) = match degree.get_mut(&dependent.uuid) {
                    Some(degree) => degree,
                    None => continue,
                };
                *degree -= 1;

                if *degree == 0 {
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

    pub fn reorder_cells(&mut self, cell_uuids: &[String]) {
        self.display_order = cell_uuids.to_vec();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trivial_deps() {
        let mut scope = HashMap::new();
        let code_cell_1 = Cell::new_reactive("a = 1", &mut scope).unwrap();
        let code_cell_2 = Cell::new_reactive("b = 2", &mut scope).unwrap();

        let expected_dependencies: HashMap<String, HashSet<String>> = HashMap::new();
        let expected_dependents: HashMap<String, HashSet<String>> = HashMap::new();

        let topology = Topology::from_vec(vec![code_cell_1, code_cell_2], &mut scope).unwrap();

        assert_eq!(topology.dependencies, expected_dependencies);
        assert_eq!(topology.dependents, expected_dependents);
    }

    #[test]
    fn test_build_deps_simple() {
        let mut scope = HashMap::new();
        let code_cell_1 = Cell::new_reactive("a = 1", &mut scope).unwrap();
        let code_cell_2 = Cell::new_reactive("b = a + 1", &mut scope).unwrap();

        let mut expected_dependencies: HashMap<String, HashSet<String>> = HashMap::new();
        expected_dependencies.insert(
            code_cell_2.uuid.clone(),
            vec![code_cell_1.uuid.clone()].into_iter().collect(),
        );

        let mut expected_dependents: HashMap<String, HashSet<String>> = HashMap::new();
        expected_dependents.insert(
            code_cell_1.uuid.clone(),
            vec![code_cell_2.uuid.clone()].into_iter().collect(),
        );

        let topology = Topology::from_vec(vec![code_cell_1, code_cell_2], &mut scope).unwrap();

        assert_eq!(topology.dependencies, expected_dependencies);
        assert_eq!(topology.dependents, expected_dependents);
    }

    #[test]
    fn test_build_deps_while() {
        let mut scope = HashMap::new();
        let code_cell_1 = Cell::new_reactive("a = 1", &mut scope).unwrap();
        let code_cell_2 = Cell::new_reactive("while True:\n a += 1", &mut scope).unwrap();

        let mut expected_dependencies: HashMap<String, HashSet<String>> = HashMap::new();
        expected_dependencies.insert(
            code_cell_2.uuid.clone(),
            vec![code_cell_1.uuid.clone()].into_iter().collect(),
        );

        let mut expected_dependents: HashMap<String, HashSet<String>> = HashMap::new();
        expected_dependents.insert(
            code_cell_1.uuid.clone(),
            vec![code_cell_2.uuid.clone()].into_iter().collect(),
        );

        let topology = Topology::from_vec(vec![code_cell_1, code_cell_2], &mut scope).unwrap();

        assert_eq!(topology.dependencies, expected_dependencies);
        assert_eq!(topology.dependents, expected_dependents);
    }

    #[test]
    fn test_get_dependencies() {
        let mut scope = HashMap::new();
        let code_cell_1 = Cell::new_reactive("a = 1", &mut scope).unwrap();
        let code_cell_2 = Cell::new_reactive("b = a + 1", &mut scope).unwrap();

        let cell_uuid_2 = code_cell_2.uuid.clone();
        let code_cell_1_clone = code_cell_1.clone();
        let expected_deps = vec![&code_cell_1_clone];

        let topology = Topology::from_vec(vec![code_cell_1, code_cell_2], &mut scope).unwrap();

        let dependencies = topology.get_dependencies(&cell_uuid_2);
        assert_eq!(dependencies[0].uuid, expected_deps[0].uuid);
    }

    #[test]
    fn test_get_dependents() {
        let mut scope = HashMap::new();
        let code_cell_1 = Cell::new_reactive("a = 1", &mut scope).unwrap();
        let code_cell_2 = Cell::new_reactive("b = a + 1", &mut scope).unwrap();

        let cell_uuid_1 = code_cell_1.uuid.clone();
        let code_cell_2_clone = code_cell_2.clone();
        let expected_deps = vec![&code_cell_2_clone];

        let topology = Topology::from_vec(vec![code_cell_1, code_cell_2], &mut scope).unwrap();

        let dependents = topology.get_dependents(&cell_uuid_1);
        assert_eq!(dependents[0].uuid, expected_deps[0].uuid);
    }

    #[test]
    fn test_topo_sort() {
        let mut scope = HashMap::new();
        let code_cell_1 = Cell::new_reactive("a = 1", &mut scope).unwrap();
        let code_cell_2 = Cell::new_reactive("b = a + 1", &mut scope).unwrap();

        let expect_order = vec![code_cell_1.uuid.clone(), code_cell_2.uuid.clone()];
        let topology = Topology::from_vec(vec![code_cell_1, code_cell_2], &mut scope).unwrap();

        let sorted = topology.topological_sort().unwrap();
        assert_eq!(sorted, expect_order);
    }

    #[test]
    fn test_topo_sort_2() {
        let mut scope = HashMap::new();
        let code_cell_1 = Cell::new_reactive("a = 1", &mut scope).unwrap();
        let code_cell_2 = Cell::new_reactive("b = a + c", &mut scope).unwrap();
        let code_cell_3 = Cell::new_reactive("c = 4", &mut scope).unwrap();

        let expected_last = code_cell_2.uuid.clone();

        let topology =
            Topology::from_vec(vec![code_cell_1, code_cell_2, code_cell_3], &mut scope).unwrap();

        let sorted = topology.topological_sort().unwrap();
        assert_eq!(sorted.last().unwrap(), &expected_last);
    }

    #[test]
    fn test_topo_sort_3() {
        let mut scope = HashMap::new();
        let code_cell_1 = Cell::new_reactive("a = 1", &mut scope).unwrap();
        let code_cell_2 = Cell::new_reactive("b = a + c", &mut scope).unwrap();
        let code_cell_3 = Cell::new_reactive("c = d", &mut scope).unwrap();
        let code_cell_4 = Cell::new_reactive("d = 4", &mut scope).unwrap();

        let expected_last = code_cell_2.uuid.clone();
        let expected_3rd = code_cell_3.uuid.clone();

        let topology = Topology::from_vec(
            vec![code_cell_1, code_cell_2, code_cell_3, code_cell_4],
            &mut scope,
        )
        .unwrap();

        let sorted = topology.topological_sort().unwrap();
        assert_eq!(sorted.last().unwrap(), &expected_last);
        assert_eq!(sorted.get(2).unwrap(), &expected_3rd);
    }

    #[test]
    fn test_topo_sort_4_add_cell() {
        let mut scope = HashMap::new();
        let code_cell_1 = Cell::new_reactive("a = 1", &mut scope).unwrap();
        let code_cell_2 = Cell::new_reactive("b = a + c", &mut scope).unwrap();
        let code_cell_3 = Cell::new_reactive("c = d", &mut scope).unwrap();

        let expected_last = code_cell_2.uuid.clone();

        let topology =
            Topology::from_vec(vec![code_cell_1, code_cell_2, code_cell_3], &mut scope).unwrap();

        let sorted = topology.topological_sort().unwrap();
        assert_eq!(sorted.last().unwrap(), &expected_last);
    }

    #[test]
    fn test_topo_sort_cycle_detected() {
        let mut scope = HashMap::new();
        let code_cell_1 = Cell::new_reactive("a = 1", &mut scope).unwrap();
        let code_cell_2 = Cell::new_reactive("b = a + c", &mut scope).unwrap();
        let code_cell_3 = Cell::new_reactive("c = d", &mut scope).unwrap();
        let code_cell_4 = Cell::new_reactive("d = b", &mut scope).unwrap();

        let topology = Topology::from_vec(
            vec![code_cell_1, code_cell_2, code_cell_3, code_cell_4],
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
            vec![code_cell_1, code_cell_2, code_cell_3, code_cell_4],
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

        let target_uuid = code_cell_3.uuid.clone();
        let expected_seq = vec![code_cell_3.uuid.clone(), code_cell_2.uuid.clone()];

        let topology = Topology::from_vec(
            vec![code_cell_1, code_cell_2, code_cell_3, code_cell_4],
            &mut scope,
        )
        .unwrap();

        let execution_seq = topology.execution_seq(&target_uuid).unwrap();

        assert_eq!(execution_seq, expected_seq);
    }

    #[test]
    fn test_execution_seq_2() {
        let mut scope = HashMap::new();
        let code_cell_1 = Cell::new_reactive("a = b + 1", &mut scope).unwrap();
        let code_cell_2 = Cell::new_reactive("b = 2", &mut scope).unwrap();
        let code_cell_3 = Cell::new_reactive("c = 1", &mut scope).unwrap();

        let target_uuid = code_cell_2.uuid.clone();
        let expected_seq = vec![target_uuid.clone(), code_cell_1.uuid.clone()];

        let topology =
            Topology::from_vec(vec![code_cell_1, code_cell_2, code_cell_3], &mut scope).unwrap();

        let execution_seq = topology.execution_seq(&target_uuid).unwrap();

        assert_eq!(execution_seq, expected_seq,);
    }

    #[test]
    fn test_assign_code_dependencies() -> Result<(), Box<dyn Error>> {
        let mut scope = Scope::new();

        let cell_1 = Cell::new_reactive("a = 1", &mut scope)?;
        let cell_2 = Cell::new_reactive("b = a", &mut scope)?;
        let cell_1_uuid = cell_1.uuid.clone();
        let cell_2_uuid = cell_2.uuid.clone();

        let topology = Topology::from_vec(vec![cell_1, cell_2], &mut scope)?;

        let expected_dependencies = vec![cell_1_uuid.clone()];
        let dependencies = topology.get_dependencies(&cell_2_uuid);
        Ok(assert_eq!(dependencies[0].uuid, expected_dependencies[0]))
    }

    #[test]
    fn test_assign_add_code_dependencies() -> Result<(), Box<dyn Error>> {
        let mut scope = Scope::new();

        let cell_1 = Cell::new_reactive("a = 1", &mut scope)?;
        let cell_2 = Cell::new_reactive("b = a + 1", &mut scope)?;
        let cell_1_uuid = cell_1.uuid.clone();
        let cell_2_uuid = cell_2.uuid.clone();

        let topology = Topology::from_vec(vec![cell_1, cell_2], &mut scope)?;

        assert_eq!(scope.get("a").unwrap(), &cell_1_uuid);
        assert_eq!(scope.get("b").unwrap(), &cell_2_uuid);

        let expected_dependencies = vec![cell_1_uuid.clone()];
        let dependencies = topology.get_dependencies(&cell_2_uuid);
        Ok(assert_eq!(dependencies[0].uuid, expected_dependencies[0]))
    }

    // #[test]
    // fn test_assign_add_two_code_dependencies() -> Result<(), Box<dyn Error>> {
    //     let mut scope = Scope::new();

    //     let cell_1 = Cell::new_reactive("a = 1");
    //     let mut cell_2 = Cell::new_reactive("b = a + c");
    //     let cell_3 = Cell::new_reactive("c = 1");

    //     cell_2.build_dependencies(&mut scope)?;

    //     let expect = HashSet::from([cell_1.uuid.to_string(), cell_3.uuid.to_string()]);

    //     assert_eq!(scope.get("a"), Some(&cell_1.uuid));
    //     assert_eq!(scope.get("b"), Some(&cell_2.uuid));
    //     assert_eq!(scope.get("c"), Some(&cell_3.uuid));
    //     Ok(assert_eq!(cell_2.dependencies, expect))
    // }

    // #[test]
    // fn test_import_dependencies() -> Result<(), Box<dyn Error>> {
    //     let mut scope = Scope::new();

    //     let cell_1 = Cell::new_reactive("import numpy as np");
    //     let mut cell_2 = Cell::new_reactive("p = np.pi");

    //     cell_2.build_dependencies(&mut scope)?;

    //     let expect = HashSet::from([cell_1.uuid]);

    //     Ok(assert_eq!(cell_2.dependencies, expect))
    // }

    // #[test]
    // fn test_attr_dependencies() -> Result<(), Box<dyn Error>> {
    //     let mut scope = Scope::new();

    //     let cell_1 = Cell::new_reactive("import numpy as np");
    //     let mut cell_2 = Cell::new_reactive("np.pi");

    //     cell_2.build_dependencies(&mut scope)?;

    //     let expect = HashSet::from([cell_1.uuid.to_string()]);

    //     Ok(assert_eq!(cell_2.dependencies, expect))
    // }

    // #[test]
    // fn test_list_dependencies() -> Result<(), Box<dyn Error>> {
    //     let mut scope = Scope::new();

    //     let cell_1 = Cell::new_reactive("a = 1");
    //     let mut cell_2 = Cell::new_reactive("b = [a, c]");
    //     let cell_3 = Cell::new_reactive("c = 2");

    //     cell_2.build_dependencies(&mut scope)?;

    //     let expect = HashSet::from([cell_1.uuid.to_string(), cell_3.uuid.to_string()]);

    //     Ok(assert_eq!(cell_2.dependencies, expect))
    // }

    // #[test]
    // fn test_tuple_dependencies() -> Result<(), Box<dyn Error>> {
    //     let mut scope = Scope::new();

    //     let cell_1 = Cell::new_reactive("a = 1");
    //     let mut cell_2 = Cell::new_reactive("b = (a, c)");
    //     let cell_3 = Cell::new_reactive("c = 2");

    //     cell_2.build_dependencies(&mut scope)?;

    //     let expect = HashSet::from([cell_1.uuid.to_string(), cell_3.uuid.to_string()]);

    //     Ok(assert_eq!(cell_2.dependencies, expect))
    // }

    // #[test]
    // fn test_set_dependencies() -> Result<(), Box<dyn Error>> {
    //     let mut scope = Scope::new();

    //     let cell_1 = Cell::new_reactive("a = 1");
    //     let mut cell_2 = Cell::new_reactive("b = {a, c}");
    //     let cell_3 = Cell::new_reactive("c = 2");

    //     cell_2.build_dependencies(&mut scope)?;

    //     let expect = HashSet::from([cell_1.uuid.to_string(), cell_3.uuid.to_string()]);

    //     Ok(assert_eq!(cell_2.dependencies, expect))
    // }

    // #[test]
    // fn test_unary_dependencies() -> Result<(), Box<dyn Error>> {
    //     let mut scope = Scope::new();

    //     let cell_1 = Cell::new_reactive("a = 1");
    //     let mut cell_2 = Cell::new_reactive("b = -a");

    //     cell_2.build_dependencies(&mut scope)?;

    //     let expect = HashSet::from([cell_1.uuid.to_string()]);

    //     Ok(assert_eq!(cell_2.dependencies, expect))
    // }

    // #[test]
    // fn test_boolop_dependencies() -> Result<(), Box<dyn Error>> {
    //     let mut scope = Scope::new();

    //     let cell_1 = Cell::new_reactive("a = 1");
    //     let cell_2 = Cell::new_reactive("b = 2");
    //     let mut cell_3 = Cell::new_reactive("c = a and b");

    //     cell_3.build_dependencies(&mut scope)?;

    //     let expect = HashSet::from([cell_1.uuid.to_string(), cell_2.uuid.to_string()]);

    //     Ok(assert_eq!(cell_3.dependencies, expect))
    // }

    // #[test]
    // fn test_namedexpr_dependencies() -> Result<(), Box<dyn Error>> {
    //     let mut scope = Scope::new();

    //     let cell_1 = Cell::new_reactive("a = 1");
    //     let mut cell_2 = Cell::new_reactive("(b := a)");

    //     cell_2.build_dependencies(&mut scope)?;

    //     let expect = HashSet::from([cell_1.uuid.to_string()]);

    //     Ok(assert_eq!(cell_2.dependencies, expect))
    // }

    // #[test]
    // fn test_ifexpr_dependencies_1() -> Result<(), Box<dyn Error>> {
    //     let mut scope = Scope::new();

    //     let cell_1 = Cell::new_reactive("a = 1");
    //     let cell_2 = Cell::new_reactive("b = 2");
    //     let mut cell_3 = Cell::new_reactive("c = a if b else 3");

    //     cell_3.build_dependencies(&mut scope)?;

    //     let expect = HashSet::from([cell_1.uuid.to_string(), cell_2.uuid.to_string()]);

    //     Ok(assert_eq!(cell_3.dependencies, expect))
    // }

    // #[test]
    // fn test_ifexpr_dependencies_2() -> Result<(), Box<dyn Error>> {
    //     let mut scope = Scope::new();

    //     let cell_1 = Cell::new_reactive("a = 1");
    //     let cell_2 = Cell::new_reactive("b = 2");
    //     let mut cell_3 = Cell::new_reactive("c = a if 3 else b");

    //     cell_3.build_dependencies(&mut scope)?;

    //     let expect = HashSet::from([cell_1.uuid.to_string(), cell_2.uuid.to_string()]);

    //     Ok(assert_eq!(cell_3.dependencies, expect))
    // }

    // #[test]
    // fn test_compare_dependencies_1() -> Result<(), Box<dyn Error>> {
    //     let mut scope = Scope::new();

    //     let cell_1 = Cell::new_reactive("a = 1");
    //     let cell_2 = Cell::new_reactive("b = 2");
    //     let mut cell_3 = Cell::new_reactive("c = a < b");

    //     cell_3.build_dependencies(&mut scope)?;

    //     let expect = HashSet::from([cell_1.uuid.to_string(), cell_2.uuid.to_string()]);

    //     Ok(assert_eq!(cell_3.dependencies, expect))
    // }

    // #[test]
    // fn test_compare_dependencies_2() -> Result<(), Box<dyn Error>> {
    //     let mut scope = Scope::new();

    //     let cell_1 = Cell::new_reactive("a = 1");
    //     let cell_2 = Cell::new_reactive("b = 2");
    //     let mut cell_3 = Cell::new_reactive("c = a >= b");

    //     cell_3.build_dependencies(&mut scope)?;

    //     let expect = HashSet::from([cell_1.uuid.to_string(), cell_2.uuid.to_string()]);

    //     Ok(assert_eq!(cell_3.dependencies, expect))
    // }

    // #[test]
    // fn test_slice_lower_dependencies() -> Result<(), Box<dyn Error>> {
    //     let mut scope = Scope::new();

    //     let cell_1 = Cell::new_reactive("c = 1");
    //     let mut cell_2 = Cell::new_reactive("a = [1, 2, 3]\nb = a[c:]");

    //     cell_2.build_dependencies(&mut scope)?;

    //     let expect = HashSet::from([cell_1.uuid.to_string()]);

    //     Ok(assert_eq!(cell_2.dependencies, expect))
    // }

    // #[test]
    // fn test_slice_upper_dependencies() -> Result<(), Box<dyn Error>> {
    //     let mut scope = Scope::new();

    //     let cell_1 = Cell::new_reactive("c = 1");
    //     let mut cell_2 = Cell::new_reactive("a = [1, 2, 3]\nb = a[:c]");

    //     cell_2.build_dependencies(&mut scope)?;

    //     let expect = HashSet::from([cell_1.uuid.to_string()]);

    //     Ok(assert_eq!(cell_2.dependencies, expect))
    // }

    // #[test]
    // fn test_slice_step_dependencies() -> Result<(), Box<dyn Error>> {
    //     let mut scope = Scope::new();

    //     let cell_1 = Cell::new_reactive("c = 1");
    //     let mut cell_2 = Cell::new_reactive("a = [1, 2, 3]\nb = a[0:c:2]");

    //     cell_2.build_dependencies(&mut scope)?;

    //     let expect = HashSet::from([cell_1.uuid.to_string()]);

    //     Ok(assert_eq!(cell_2.dependencies, expect))
    // }

    // #[test]
    // fn test_formattedvalue_dependencies() -> Result<(), Box<dyn Error>> {
    //     let mut scope = Scope::new();

    //     let cell_1 = Cell::new_reactive("a = 1");
    //     let mut cell_2 = Cell::new_reactive("b = f'{a}'");

    //     cell_2.build_dependencies(&mut scope)?;

    //     let expect = HashSet::from([cell_1.uuid.to_string()]);

    //     Ok(assert_eq!(cell_2.dependencies, expect))
    // }

    // #[test]
    // fn test_joinedstr_dependencies() -> Result<(), Box<dyn Error>> {
    //     let mut scope = Scope::new();

    //     let cell_1 = Cell::new_reactive("a = 1");
    //     let mut cell_2 = Cell::new_reactive("b = f'{a}' + 'a'");

    //     cell_2.build_dependencies(&mut scope)?;

    //     let expect = HashSet::from([cell_1.uuid.to_string()]);

    //     Ok(assert_eq!(cell_2.dependencies, expect))
    // }

    // #[test]
    // fn test_dict_dependencies_1() -> Result<(), Box<dyn Error>> {
    //     let mut scope = Scope::new();

    //     let cell_1 = Cell::new_reactive("a = 1");
    //     let mut cell_2 = Cell::new_reactive("b = {a: 1}");

    //     cell_2.build_dependencies(&mut scope)?;

    //     let expect = HashSet::from([cell_1.uuid.to_string()]);

    //     Ok(assert_eq!(cell_2.dependencies, expect))
    // }

    // #[test]
    // fn test_dict_dependencies_2() -> Result<(), Box<dyn Error>> {
    //     let mut scope = Scope::new();

    //     let cell_1 = Cell::new_reactive("a = 1");
    //     let mut cell_2 = Cell::new_reactive("b = {1: a}");

    //     cell_2.build_dependencies(&mut scope)?;

    //     let expect = HashSet::from([cell_1.uuid.to_string()]);

    //     Ok(assert_eq!(cell_2.dependencies, expect))
    // }

    // #[test]
    // fn test_listcomp_dependencies() -> Result<(), Box<dyn Error>> {
    //     let mut scope = Scope::new();

    //     let cell_1 = Cell::new_reactive("a = 1");
    //     let mut cell_2 = Cell::new_reactive("b = [a for _ in [1, 2, 3]]");

    //     cell_2.build_dependencies(&mut scope)?;

    //     let expect = HashSet::from([cell_1.uuid.to_string()]);

    //     Ok(assert_eq!(cell_2.dependencies, expect))
    // }

    // #[test]
    // fn test_setcomp_dependencies() -> Result<(), Box<dyn Error>> {
    //     let mut scope = Scope::new();

    //     let cell_1 = Cell::new_reactive("a = 1");
    //     let mut cell_2 = Cell::new_reactive("b = {a for _ in [1, 2, 3]}");

    //     cell_2.build_dependencies(&mut scope)?;

    //     let expect = HashSet::from([cell_1.uuid.to_string()]);

    //     Ok(assert_eq!(cell_2.dependencies, expect))
    // }

    // #[test]
    // fn test_listcomp_if_dependencies() -> Result<(), Box<dyn Error>> {
    //     let mut scope = Scope::new();

    //     let cell_1 = Cell::new_reactive("a = 1");
    //     let mut cell_2 = Cell::new_reactive("b = [a for _ in [1, 2, 3] if c > 1]");
    //     let cell_3 = Cell::new_reactive("c = 2");

    //     cell_2.build_dependencies(&mut scope)?;

    //     let expect = HashSet::from([cell_1.uuid.to_string(), cell_3.uuid.to_string()]);

    //     Ok(assert_eq!(cell_2.dependencies, expect))
    // }

    // #[test]
    // fn test_listcomp_mult_if_dependencies() -> Result<(), Box<dyn Error>> {
    //     let mut scope = Scope::new();

    //     let cell_1 = Cell::new_reactive("a = 1");
    //     let mut cell_2 = Cell::new_reactive("b = [a for _ in [1, 2, 3] if c > 1 if d > 2]");
    //     let cell_3 = Cell::new_reactive("c = 2");
    //     let cell_4 = Cell::new_reactive("d = 3");

    //     cell_2.build_dependencies(&mut scope)?;

    //     let expect = HashSet::from([
    //         cell_1.uuid.to_string(),
    //         cell_3.uuid.to_string(),
    //         cell_4.uuid.to_string(),
    //     ]);

    //     Ok(assert_eq!(cell_2.dependencies, expect))
    // }

    // #[test]
    // fn test_dictcomp_dependencies() -> Result<(), Box<dyn Error>> {
    //     let mut scope = Scope::new();

    //     let cell_1 = Cell::new_reactive("a = 1");
    //     let mut cell_2 = Cell::new_reactive("b = {i: a for i in [1, 2, 3]}");

    //     cell_2.build_dependencies(&mut scope)?;

    //     let expect = HashSet::from([cell_1.uuid.to_string()]);

    //     Ok(assert_eq!(cell_2.dependencies, expect))
    // }

    // #[test]
    // fn test_dictcomp_dependencies_2() -> Result<(), Box<dyn Error>> {
    //     let mut scope = Scope::new();

    //     let cell_1 = Cell::new_reactive("a = 1");
    //     let mut cell_2 = Cell::new_reactive("b = {a: 1 for i in [1, 2, 3]}");

    //     cell_2.build_dependencies(&mut scope)?;

    //     let expect = HashSet::from([cell_1.uuid.to_string()]);

    //     Ok(assert_eq!(cell_2.dependencies, expect))
    // }

    // #[test]
    // fn test_dictcomp_if_dependencies() -> Result<(), Box<dyn Error>> {
    //     let mut scope = Scope::new();

    //     let cell_1 = Cell::new_reactive("a = 1");
    //     let mut cell_2 = Cell::new_reactive("b = {2: 1 for i in [1, 2, 3] if a > 0}");

    //     cell_2.build_dependencies(&mut scope)?;

    //     let expect = HashSet::from([cell_1.uuid.to_string()]);

    //     Ok(assert_eq!(cell_2.dependencies, expect))
    // }

    // #[test]
    // fn test_lambda_dependencies() -> Result<(), Box<dyn Error>> {
    //     let mut scope = Scope::new();

    //     let cell_1 = Cell::new_reactive("a = 1");
    //     let mut cell_2 = Cell::new_reactive("b = lambda x: a");

    //     cell_2.build_dependencies(&mut scope)?;

    //     let expect = HashSet::from([cell_1.uuid.to_string()]);

    //     Ok(assert_eq!(cell_2.dependencies, expect))
    // }

    // #[test]
    // fn test_call_dependencies() -> Result<(), Box<dyn Error>> {
    //     let mut scope = Scope::new();

    //     let cell_1 = Cell::new_reactive("a = lambda x: 1");
    //     let mut cell_2 = Cell::new_reactive("b = a(1)");

    //     cell_2.build_dependencies(&mut scope)?;

    //     let expect = HashSet::from([cell_1.uuid.to_string()]);

    //     Ok(assert_eq!(cell_2.dependencies, expect))
    // }

    // #[test]
    // fn test_call_dependencies_2() -> Result<(), Box<dyn Error>> {
    //     let mut scope = Scope::new();

    //     let cell_1 = Cell::new_reactive("a = lambda x: 1");
    //     let mut cell_2 = Cell::new_reactive("b = a(c)");
    //     let cell_3 = Cell::new_reactive("c = 1");

    //     cell_2.build_dependencies(&mut scope)?;

    //     let expect = HashSet::from([cell_1.uuid.to_string(), cell_3.uuid.to_string()]);

    //     Ok(assert_eq!(cell_2.dependencies, expect))
    // }

    // #[test]
    // fn test_generatorexp_dependencies() -> Result<(), Box<dyn Error>> {
    //     let mut scope = Scope::new();

    //     let cell_1 = Cell::new_reactive("a = 1");
    //     let mut cell_2 = Cell::new_reactive("b = (a for _ in [1, 2, 3])");

    //     cell_2.build_dependencies(&mut scope)?;

    //     let expect = HashSet::from([cell_1.uuid.to_string()]);

    //     Ok(assert_eq!(cell_2.dependencies, expect))
    // }

    // #[test]
    // fn test_await_dependencies() -> Result<(), Box<dyn Error>> {
    //     let mut scope = Scope::new();

    //     let cell_1 = Cell::new_reactive("a = 1");
    //     let mut cell_2 = Cell::new_reactive("b = await a");

    //     cell_2.build_dependencies(&mut scope)?;

    //     let expect = HashSet::from([cell_1.uuid.to_string()]);

    //     Ok(assert_eq!(cell_2.dependencies, expect))
    // }

    // #[test]
    // fn test_yield_dependencies() -> Result<(), Box<dyn Error>> {
    //     let mut scope = Scope::new();

    //     let cell_1 = Cell::new_reactive("a = 1");
    //     let mut cell_2 = Cell::new_reactive("b = yield a");

    //     cell_2.build_dependencies(&mut scope)?;

    //     let expect = HashSet::from([cell_1.uuid.to_string()]);

    //     Ok(assert_eq!(cell_2.dependencies, expect))
    // }

    // #[test]
    // fn test_yieldfrom_dependencies() -> Result<(), Box<dyn Error>> {
    //     let mut scope = Scope::new();

    //     let cell_1 = Cell::new_reactive("a = 1");
    //     let mut cell_2 = Cell::new_reactive("b = yield from a");

    //     cell_2.build_dependencies(&mut scope)?;

    //     let expect = HashSet::from([cell_1.uuid.to_string()]);

    //     Ok(assert_eq!(cell_2.dependencies, expect))
    // }

    // #[test]
    // fn test_starred_dependencies() -> Result<(), Box<dyn Error>> {
    //     let mut scope = Scope::new();

    //     let cell_1 = Cell::new_reactive("a = 1");
    //     let mut cell_2 = Cell::new_reactive("b = [*a]");

    //     cell_2.build_dependencies(&mut scope)?;

    //     let expect = HashSet::from([cell_1.uuid.to_string()]);

    //     Ok(assert_eq!(cell_2.dependencies, expect))
    // }

    // #[test]
    // fn test_build_dependents() -> Result<(), Box<dyn Error>> {
    //     let mut scope = Scope::new();

    //     let cell_1 = Cell::new_reactive("a = b + 1");
    //     let cell_2 = Cell::new_reactive("b = 1");

    //     let mut topology = Topology::from_vec(vec![&cell_1, &cell_2]);
    //     topology.build(&mut scope)?;

    //     let expect = HashSet::from([cell_1.uuid.to_string()]);
    //     let c_2 = topology.cells.get(&cell_2.uuid).unwrap();
    //     Ok(assert_eq!(c_2.dependents, expect))
    // }

    // #[test]
    // fn test_build_dependents_2() -> Result<(), Box<dyn Error>> {
    //     let mut scope = Scope::new();

    //     let cell_1 = Cell::new_reactive("a = b + c");
    //     let cell_2 = Cell::new_reactive("b = 1");
    //     let cell_3 = Cell::new_reactive("c = 2");

    //     let mut topology = Topology::from_vec(vec![&cell_1, &cell_2, &cell_3]);
    //     topology.build(&mut scope)?;

    //     let expect = HashSet::from([cell_1.uuid.to_string()]);
    //     let c_2 = topology.cells.get(&cell_2.uuid).unwrap();
    //     let c_3 = topology.cells.get(&cell_3.uuid).unwrap();
    //     assert_eq!(c_2.dependents, expect);
    //     Ok(assert_eq!(c_3.dependents, expect))
    // }

    // #[test]
    // fn test_augassign_dependencies() -> Result<(), Box<dyn Error>> {
    //     let mut scope = Scope::new();

    //     let cell_1 = Cell::new_reactive("a = 1");
    //     let mut cell_2 = Cell::new_reactive("b += a");

    //     cell_2.build_dependencies(&mut scope)?;

    //     let expect = HashSet::from([cell_1.uuid.to_string()]);

    //     Ok(assert_eq!(cell_2.dependencies, expect))
    // }

    #[test]
    fn test_funndef_dependencies() {
        let mut scope = Scope::new();

        let cell_1 = Cell::new_reactive("a = 1", &mut scope).unwrap();
        let cell_2 = Cell::new_reactive("def b(c, d): return a", &mut scope).unwrap();
        let cell_1_uuid = cell_1.uuid.to_string();
        let cell_2_uuid = cell_2.uuid.to_string();

        let topology = Topology::from_vec(vec![cell_1, cell_2], &mut scope).unwrap();

        let expected_dependencies = vec![cell_1_uuid.clone()];
        let dependencies = topology.get_dependencies(&cell_2_uuid);
        assert_eq!(dependencies[0].uuid, expected_dependencies[0]);
    }

    #[test]
    fn test_funndef_2_dependencies() {
        let mut scope = Scope::new();

        let cell_1 = Cell::new_reactive("add(1,2)", &mut scope).unwrap();
        let cell_2 = Cell::new_reactive("def add(a, b): return a + b", &mut scope).unwrap();
        let cell_1_uuid = cell_1.uuid.to_string();
        let cell_2_uuid = cell_2.uuid.to_string();

        let topology = Topology::from_vec(vec![cell_1, cell_2], &mut scope).unwrap();

        let expected_dependents = vec![cell_1_uuid.clone()];
        let dependencies = topology.get_dependents(&cell_2_uuid);
        assert_eq!(dependencies[0].uuid, expected_dependents[0]);
    }

    // #[test]
    // fn test_asyncfndef_dependencies() -> Result<(), Box<dyn Error>> {
    //     let mut scope = Scope::new();

    //     let cell_1 = Cell::new_reactive("a = 1");
    //     let mut cell_2 = Cell::new_reactive("async def b(c, d): return a");

    //     cell_2.build_dependencies(&mut scope)?;

    //     let expect = HashSet::from([cell_1.uuid.to_string()]);

    //     Ok(assert_eq!(cell_2.dependencies, expect))
    // }

    // #[test]
    // fn test_annassign_dependencies() -> Result<(), Box<dyn Error>> {
    //     let mut scope = Scope::new();

    //     let cell_1 = Cell::new_reactive("a = 1");
    //     let mut cell_2 = Cell::new_reactive("b: int = a");

    //     cell_2.build_dependencies(&mut scope)?;

    //     let expect = HashSet::from([cell_1.uuid.to_string()]);

    //     Ok(assert_eq!(cell_2.dependencies, expect))
    // }

    // #[test]
    // fn test_while_dependencies() -> Result<(), Box<dyn Error>> {
    //     let mut scope = Scope::new();

    //     let cell_1 = Cell::new_reactive("a = 1");
    //     let mut cell_2 = Cell::new_reactive("while a: pass");

    //     cell_2.build_dependencies(&mut scope)?;

    //     let expect = HashSet::from([cell_1.uuid.to_string()]);

    //     Ok(assert_eq!(cell_2.dependencies, expect))
    // }

    // #[test]
    // fn test_while_dependencies_2() -> Result<(), Box<dyn Error>> {
    //     let mut scope = Scope::new();

    //     let cell_1 = Cell::new_reactive("a = 1");
    //     let mut cell_2 = Cell::new_reactive("while True:\n  a += 1");

    //     cell_2.build_dependencies(&mut scope)?;

    //     let expect = HashSet::from([cell_1.uuid.to_string()]);

    //     Ok(assert_eq!(cell_2.dependencies, expect))
    // }

    // #[test]
    // fn test_forloop_dependencies() -> Result<(), Box<dyn Error>> {
    //     let mut scope = Scope::new();

    //     let cell_1 = Cell::new_reactive("a = [1, 2, 3]");
    //     let mut cell_2 = Cell::new_reactive("for i in a: pass");

    //     cell_2.build_dependencies(&mut scope)?;

    //     let expect = HashSet::from([cell_1.uuid.to_string()]);

    //     Ok(assert_eq!(cell_2.dependencies, expect))
    // }

    // #[test]
    // fn test_forloop_dependencies_2() -> Result<(), Box<dyn Error>> {
    //     let mut scope = Scope::new();

    //     let cell_1 = Cell::new_reactive("a = [1, 2, 3]");
    //     let mut cell_2 = Cell::new_reactive("for i in [4,5,6]: a");

    //     cell_2.build_dependencies(&mut scope)?;

    //     let expect = HashSet::from([cell_1.uuid.to_string()]);

    //     Ok(assert_eq!(cell_2.dependencies, expect))
    // }

    // #[test]
    // fn test_forloop_dependencies_3() -> Result<(), Box<dyn Error>> {
    //     let mut scope = Scope::new();

    //     let cell_1 = Cell::new_reactive("a = [1, 2, 3]");
    //     let mut cell_2 = Cell::new_reactive("for i in [4,5,6]:\n  for j in a: j");

    //     cell_2.build_dependencies(&mut scope)?;

    //     let expect = HashSet::from([cell_1.uuid.to_string()]);

    //     Ok(assert_eq!(cell_2.dependencies, expect))
    // }

    // #[test]
    // fn test_class_dependencies() -> Result<(), Box<dyn Error>> {
    //     let mut scope = Scope::new();

    //     let cell_1 = Cell::new_reactive("a = 1");
    //     let mut cell_2 = Cell::new_reactive(
    //         "class b:\n  def __init__(self):\n    self.a = a",
    //         &mut scope,
    //     )?;

    //     cell_2.build_dependencies(&mut scope)?;

    //     let expect = HashSet::from([cell_1.uuid.to_string()]);

    //     Ok(assert_eq!(cell_2.dependencies, expect))
    // }

    // #[test]
    // fn test_class_dependencies_2() -> Result<(), Box<dyn Error>> {
    //     let mut scope = Scope::new();

    //     let cell_1 = Cell::new_reactive("class a:\n  def __init__(self): pass");
    //     let mut cell_2 = Cell::new_reactive(
    //         "class b:\n  def __init__(self):\n    self.a = a",
    //         &mut scope,
    //     )?;

    //     cell_2.build_dependencies(&mut scope)?;

    //     let expect = HashSet::from([cell_1.uuid.to_string()]);

    //     Ok(assert_eq!(cell_2.dependencies, expect))
    // }

    // #[test]
    // fn test_class_dependencies_inheritance() -> Result<(), Box<dyn Error>> {
    //     let mut scope = Scope::new();

    //     let cell_1 = Cell::new_reactive("class a:\n  def __init__(self): pass");
    //     let mut cell_2 = Cell::new_reactive("class b(a):\n  def __init__(self):\n    pass");

    //     cell_2.build_dependencies(&mut scope)?;

    //     let expect = HashSet::from([cell_1.uuid.to_string()]);

    //     Ok(assert_eq!(cell_2.dependencies, expect))
    // }
}
