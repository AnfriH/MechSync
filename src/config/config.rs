use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::time::Duration;
use log::trace;
use serde::{Deserialize, Deserializer};
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

#[derive(Deserialize)]
pub(crate) struct NodeConfig {
    pub(crate) name: String,
    #[serde(rename = "type")]
    pub(crate) type_: String,
    pub(crate) next: Option<String>,

    // DelayNode
    pub(crate) is_total: Option<bool>,
    pub(crate) duration: Option<f32>,

    // DrumBot
    pub(crate) arms: Option<Vec<ArmsConfig>>,

    // PyNode
    pub(crate) source: Option<String>
}

#[derive(Deserialize)]
pub(crate) struct ArmsConfig(
    #[serde(with = "tuple_vec_map")]
    pub(crate) Vec<(u8, u8)>
);

impl Config {
    pub(crate) fn build(mut self) -> Result<Graph, ConfigError> {
        let mut graph = Graph::new();

        for node in self.nodes.iter() {
            let type_ = node.type_.as_str();
            let factory = TYPES.get(type_)
                .ok_or(ConfigError::new(&format!("Unknown type: {}", type_)))?;

            let dyn_node = factory(&self, &node)?;
            trace!(target: "Config", "Loaded node {} of {}", node.name, type_);
            if let Some(next) = &node.next {
                self.delays.insert(next.clone(), dyn_node.delay() + *self.delays.get(&node.name).unwrap_or(&Duration::from_secs(0)));
                trace!(target: "Config", "Bound {} -> {}", node.name, next);
            }
            graph.insert(
                node.name.as_str(),
                dyn_node
            )
        }

        for node in self.nodes.iter() {
            if let Some(next) = &node.next {
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