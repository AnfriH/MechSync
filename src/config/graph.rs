use std::collections::HashMap;
use std::sync::Arc;

use crate::node::Node;

pub(crate) struct Graph {
    nodes: HashMap<String, Arc<dyn Node>>
}

impl Graph {
    pub(crate) fn new() -> Self {
        Graph { nodes: HashMap::new() }
    }

    pub(super) fn bind(&self, from: &str, to: &str) -> () {
        if let (Some(from), Some(to)) = (self.nodes.get(from), self.nodes.get(to)) {
            from.bind(Arc::downgrade(to));
        }
    }

    pub(super) fn insert(&mut self, name: &str, node: Arc<dyn Node>) {
        self.nodes.insert(String::from(name), node);
    }
}