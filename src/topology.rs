use crate::cell::Cell;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Topology {
    pub adj_list: HashMap<String, Vec<String>>,
    pub cells: HashMap<String, Cell>,
}

impl Topology {
    pub fn eval(&mut self, cell_uuid: &str) {
        let cell = match self.cells.get_mut(cell_uuid) {
            Some(cell) => cell,
            None => return,
        };
        cell.eval();

        // TODO update topology if neccessary

        let dependents = match self.adj_list.get(cell_uuid) {
            Some(dependents) => dependents,
            None => return,
        };

        for dependent in dependents.iter() {
            println!("depending on {}", dependent);
        }
    }

    pub fn add_to_tree(&mut self, parent_uuid: &str, child_uuid: &str) {
        let parent_deps = match self.adj_list.get(parent_uuid) {
            Some(parent_deps) => parent_deps,
            None => return,
        };

        // add child id to parent's dependencies
        let mut next_adj_list = self.adj_list.clone();
        let mut next_parent_deps = parent_deps.clone();
        next_parent_deps.push(child_uuid.to_string());
        next_adj_list.insert(parent_uuid.to_string(), next_parent_deps);
        if Self::has_cycle(&next_adj_list) {
            return println!("Cycle detected");
        }

        self.adj_list = next_adj_list;
    }

    pub fn has_cycle(adj_list: &HashMap<String, Vec<String>>) -> bool {
        let mut visited = HashMap::new();
        let mut rec_stack = HashMap::new();

        for (node, _) in adj_list.iter() {
            if Self::has_cycle_util(node, adj_list, &mut visited, &mut rec_stack) {
                return true;
            }
        }

        false
    }

    pub fn has_cycle_util(
        node: &str,
        adj_list: &HashMap<String, Vec<String>>,
        visited: &mut HashMap<String, bool>,
        rec_stack: &mut HashMap<String, bool>,
    ) -> bool {
        match visited.get(node) {
            Some(_) => return false,
            None => {
                visited.insert(node.to_string(), true);
                rec_stack.insert(node.to_string(), true);
            }
        }

        let children = match adj_list.get(node) {
            Some(children) => children,
            None => return false,
        };

        for child in children.iter() {
            if rec_stack.get(child).is_some() {
                return true;
            }

            match rec_stack.get(child) {
                Some(_) => return true,
                None => {
                    if Self::has_cycle_util(child, adj_list, visited, rec_stack) {
                        return true;
                    }
                }
            }
        }

        rec_stack.remove(node);

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_cycle() {
        let mut adj_list = HashMap::new();
        adj_list.insert("a".to_string(), vec!["b".to_string()]);
        adj_list.insert("b".to_string(), vec!["c".to_string()]);
        adj_list.insert("c".to_string(), vec!["a".to_string()]);
        assert_eq!(Topology::has_cycle(&adj_list), true);

        let mut adj_list = HashMap::new();
        adj_list.insert("a".to_string(), vec!["b".to_string()]);
        adj_list.insert("b".to_string(), vec!["c".to_string()]);
        adj_list.insert("c".to_string(), vec![]);
        assert_eq!(Topology::has_cycle(&adj_list), false);
    }
}
