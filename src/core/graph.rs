use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Graph {
    pub adj_list: HashMap<String, Vec<String>>,
}

impl Graph {
    pub fn new() -> Self {
        Self {
            adj_list: HashMap::new(),
        }
    }

    pub fn get(&self, uuid: &str) -> Option<&Vec<String>> {
        self.adj_list.get(uuid)
    }

    pub fn add_node(
        &mut self,
        parent_uuid: &str,
        child_uuids: Option<&Vec<String>>,
    ) -> Result<(), Box<dyn error::Error>> {
        let mut next_adj_list = self.adj_list.clone();

        let deps = self
            .adj_list
            .entry(parent_uuid.to_string())
            .or_insert(vec![]);

        let mut next_deps = deps.clone();
        if child_uuids.is_some() {
            next_deps.extend(child_uuids.unwrap().clone());
        }
        next_adj_list.insert(parent_uuid.to_string(), next_deps);
        if Self::has_cycle(&next_adj_list) {
            return Err("Cycle detected".into());
        }

        self.adj_list = next_adj_list;

        Ok(())
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
        assert_eq!(Graph::has_cycle(&adj_list), true);

        let mut adj_list = HashMap::new();
        adj_list.insert("a".to_string(), vec!["b".to_string()]);
        adj_list.insert("b".to_string(), vec!["c".to_string()]);
        adj_list.insert("c".to_string(), vec![]);
        assert_eq!(Graph::has_cycle(&adj_list), false);
    }
}
