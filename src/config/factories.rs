use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use once_cell::sync::Lazy;

use crate::config::config::{Config, ConfigError};
use crate::instruments::{DrumBot, MechBass};
use crate::midi::{Input, Output};
use crate::node::{DebugNode, DelayNode, Node};

macro_rules! types {
    ( $( $typename:ident ),* ) => {
        HashMap::from([$((stringify!($typename), $typename::factory as FactoryFunction), )*])
    }
}

macro_rules! kwarg_get {
    ( $kwargs:expr, $arg:literal ) => {
        $kwargs.get($arg).ok_or(ConfigError::new(format!("Unable to find required kwarg: {}", $arg).as_str()))?.as_str()
    }
}

// -----------------------
// Factory Implementations
// -----------------------
//
// TODO: Consider creating a #[derive!()] trait to allow easy dynamic constructors
//  (will have to fight the syn library to make it work, might be out of scope)
pub(super) static TYPES: Lazy<HashMap<&'static str, FactoryFunction>> = Lazy::new(|| types![
    Input,
    Output,
    MechBass,
    DrumBot,
    DelayNode,
    DebugNode
]);

impl NodeFactory for Input {
    fn factory(ctx: &Config, kwargs: &HashMap<String, String>) -> Result<Arc<dyn Node>, ConfigError> {
        let node = Input::new(kwarg_get!(kwargs, "name")).map_err(ConfigError::of)?;
        Ok(Arc::new(node))
    }
}

impl NodeFactory for Output {
    fn factory(ctx: &Config, kwargs: &HashMap<String, String>) -> Result<Arc<dyn Node>, ConfigError> {
        let node = Output::new(kwarg_get!(kwargs, "name")).map_err(ConfigError::of)?;
        Ok(Arc::new(node))
    }
}

impl NodeFactory for MechBass {
    fn factory(ctx: &Config, kwargs: &HashMap<String, String>) -> Result<Arc<dyn Node>, ConfigError> {
        Ok(Arc::new(MechBass::new()))
    }
}

impl NodeFactory for DrumBot {
    fn factory(ctx: &Config, kwargs: &HashMap<String, String>) -> Result<Arc<dyn Node>, ConfigError> {
        Ok(Arc::new(DrumBot::new()))
    }
}

impl NodeFactory for DelayNode {
    fn factory(ctx: &Config, kwargs: &HashMap<String, String>) -> Result<Arc<dyn Node>, ConfigError> {
        let duration_raw = Duration::from_secs_f32(
            kwarg_get!(kwargs, "duration").parse().map_err(ConfigError::of)?
        );
        let is_total = kwargs.get("is_total")
            .map(|e| e.parse::<bool>().unwrap_or(false))
            .unwrap_or(false);

        let duration = if is_total {
            let prev_duration = *ctx.delays.get(kwargs.get("name").unwrap()).unwrap();
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
    fn factory(ctx: &Config, kwargs: &HashMap<String, String>) -> Result<Arc<dyn Node>, ConfigError> {
        Ok(Arc::new(DebugNode::new(kwarg_get!(kwargs, "name"))))
    }
}

// -----------------------

type FactoryFunction = fn(&Config, &HashMap<String, String>) -> Result<Arc<dyn Node>, ConfigError>;

pub(super) trait NodeFactory {
    fn factory(ctx: &Config, kwargs: &HashMap<String, String>) -> Result<Arc<dyn Node>, ConfigError>;
}