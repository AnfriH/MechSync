use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use once_cell::sync::Lazy;

use crate::config::config::{Config, ConfigError, NodeConfig};
use crate::instruments::{DrumBot, MechBass, PyNode};
use crate::midi::{Input, Output};
use crate::node::{DebugNode, DelayNode, Node};

macro_rules! types {
    ( $( $typename:ident ),* ) => {
        HashMap::from([$((stringify!($typename), $typename::factory as FactoryFunction), )*])
    }
}

// -----------------------
// Factory Implementations
// -----------------------
pub(super) static TYPES: Lazy<HashMap<&'static str, FactoryFunction>> = Lazy::new(|| types![
    Input,
    Output,
    MechBass,
    DrumBot,
    PyNode,
    DelayNode,
    DebugNode
]);

impl NodeFactory for Input {
    fn factory(_ctx: &Config, config: &NodeConfig) -> Result<Arc<dyn Node>, ConfigError> {
        let node = Input::new(config.name.as_str()).map_err(ConfigError::of)?;
        Ok(Arc::new(node))
    }
}

impl NodeFactory for Output {
    fn factory(_ctx: &Config, config: &NodeConfig) -> Result<Arc<dyn Node>, ConfigError> {
        let node = Output::new(config.name.as_str()).map_err(ConfigError::of)?;
        Ok(Arc::new(node))
    }
}

impl NodeFactory for MechBass {
    fn factory(_ctx: &Config, _config: &NodeConfig) -> Result<Arc<dyn Node>, ConfigError> {
        Ok(Arc::new(MechBass::new()))
    }
}

impl NodeFactory for DrumBot {
    fn factory(_ctx: &Config, config: &NodeConfig) -> Result<Arc<dyn Node>, ConfigError> {
        let arms = config.arms.as_ref().ok_or(ConfigError::new("Arms missing"))?;
        Ok(Arc::new(DrumBot::new(arms)))
    }
}

impl NodeFactory for DelayNode {
    fn factory(ctx: &Config, config: &NodeConfig) -> Result<Arc<dyn Node>, ConfigError> {
        let duration_raw = Duration::from_secs_f32(
            config.duration.ok_or(ConfigError::new("Duration missing"))?
        );
        let is_total = config.is_total.unwrap_or(false);

        let duration = if is_total {
            let prev_duration = *ctx.delays.get(&config.name).unwrap();
            if prev_duration > duration_raw {
                return Err(ConfigError::new(format!(
                    "Previous duration longer than total duration required ({:?} > {:?})",
                    prev_duration,
                    duration_raw
                ).as_str()));
            }

            duration_raw - prev_duration
        } else {
            duration_raw
        };
        Ok(Arc::new(DelayNode::new(duration)))
    }
}

impl NodeFactory for DebugNode {
    fn factory(_ctx: &Config, config: &NodeConfig) -> Result<Arc<dyn Node>, ConfigError> {
        Ok(Arc::new(DebugNode::new(config.name.as_str())))
    }
}

impl NodeFactory for PyNode {
    fn factory(_ctx: &Config, config: &NodeConfig) -> Result<Arc<dyn Node>, ConfigError> {
        let duration = Duration::from_secs_f32(
            config.duration.ok_or(ConfigError::new("Duration missing"))?
        );
        let source_path = config.source.as_ref().ok_or(ConfigError::new("Source missing"))?;
        let source = read_to_string(Path::new(source_path)).map_err(ConfigError::of)?;
        let pynode = PyNode::new(source.as_str(), duration).map_err(ConfigError::of)?;
        Ok(Arc::new(pynode))
    }
}

// -----------------------
type FactoryFunction = fn(&Config, &NodeConfig) -> Result<Arc<dyn Node>, ConfigError>;

pub(super) trait NodeFactory {
    fn factory(ctx: &Config, config: &NodeConfig) -> Result<Arc<dyn Node>, ConfigError>;
}