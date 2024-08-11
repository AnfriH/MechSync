use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::time::Duration;
use serde::{Deserialize, Deserializer};
use serde::de::Error as SerdeError;
use crate::config::factories::TYPES;
use crate::config::graph::Graph;

#[derive(Debug)]
pub(crate) struct ConfigError {
    message: String
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ConfigError: {}", self.message)
    }
}

impl Error for ConfigError {}

impl ConfigError {
    pub(crate) fn new(message: &str) -> Self {
        ConfigError {
            message: String::from(message)
        }
    }

    pub(crate) fn of<E: Error>(err: E) -> Self {
        ConfigError {
            message: err.to_string()
        }
    }
}

pub(crate) struct Config {
    nodes: Vec<NodeConfig>,
    pub(super) delays: HashMap<String, Duration>
}

pub(crate) struct NodeConfig {
    name: String,
    type_: String,
    traits: HashMap<String, String>
}

impl Config {
    pub(crate) fn build(mut self) -> Result<Graph, ConfigError> {
        let mut graph = Graph::new();

        for node in self.nodes.iter() {
            let type_ = node.type_.as_str();
            let factory = TYPES.get(type_)
                .ok_or(ConfigError::new(&format!("Unknown type: {}", type_)))?;

            let dyn_node = factory(&self, &node.traits)?;
            if let Some(next) = node.traits.get("next") {
                self.delays.insert(next.clone(), dyn_node.delay() + *self.delays.get(&node.name).unwrap_or(&Duration::from_secs(0)));
            }
            graph.insert(
                node.name.as_str(),
                dyn_node
            )
        }

        for node in self.nodes.iter() {
            if let Some(next) = node.traits.get("next") {
                graph.bind(node.name.as_str(), next.as_str())?;
            }
        }
        Ok(graph)
    }
}

impl<'de> Deserialize<'de> for Config {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let nodes = Vec::deserialize(d)?;
        Ok(Config { nodes, delays: HashMap::new() })
    }
}

impl<'de> Deserialize<'de> for NodeConfig {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let traits: HashMap<String, String> = HashMap::deserialize(d)?;
        let name = traits.get("name").ok_or(SerdeError::missing_field("name"))?.to_string();
        let type_ = traits.get("type").ok_or(SerdeError::missing_field("type"))?.to_string();
        Ok(
            NodeConfig {
                name,
                type_,
                traits,
            }
        )
    }
}