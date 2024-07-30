use std::collections::HashMap;

use serde::{Deserialize, Deserializer};
use serde::de::Error;

use crate::config::graph::Graph;
use crate::config::types::TYPES;

pub(crate) struct Config {
    nodes: Vec<NodeConfig>
}

pub(crate) struct NodeConfig {
    name: String,
    type_: String,
    traits: HashMap<String, String>
}

impl Config {
    pub(crate) fn build(&self) -> Result<Graph, ()> {
        let mut graph = Graph::new();

        for node in self.nodes.iter() {
            println!("{} - {}", node.name, node.type_);
            graph.insert(
                node.name.as_str(),
                TYPES.get(node.type_.as_str()).expect("TODO: ERROR HANDLING")(
                    &node.traits
                ).expect("TODO: WE CAN'T ASSUME THE CONSTRUCTOR DOESN'T BLOW UP")
            )
        }
        for node in self.nodes.iter() {
            if let Some(next) = node.traits.get("next") {
                graph.bind(node.name.as_str(), next.as_str());
            }
        }
        Ok(graph)
    }
}

impl<'de> Deserialize<'de> for Config {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let nodes = Vec::deserialize(d)?;
        Ok(Config { nodes })
    }
}

impl<'de> Deserialize<'de> for NodeConfig {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let traits: HashMap<String, String> = HashMap::deserialize(d)?;
        let name = traits.get("name").ok_or(Error::missing_field("name"))?.to_string();
        let type_ = traits.get("type").ok_or(Error::missing_field("type"))?.to_string();
        Ok(
            NodeConfig {
                name,
                type_,
                traits,
            }
        )
    }
}