use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use once_cell::sync::Lazy;

use crate::config::config::ConfigError;
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
    fn factory(kwargs: &HashMap<String, String>) -> Result<Arc<dyn Node>, ConfigError> {
        let node = Input::new(kwarg_get!(kwargs, "name")).map_err(ConfigError::of)?;
        Ok(Arc::new(node))
    }
}

impl NodeFactory for Output {
    fn factory(kwargs: &HashMap<String, String>) -> Result<Arc<dyn Node>, ConfigError> {
        let node = Output::new(kwarg_get!(kwargs, "name")).map_err(ConfigError::of)?;
        Ok(Arc::new(node))
    }
}

impl NodeFactory for MechBass {
    fn factory(kwargs: &HashMap<String, String>) -> Result<Arc<dyn Node>, ConfigError> {
        Ok(Arc::new(MechBass::new()))
    }
}

impl NodeFactory for DrumBot {
    fn factory(kwargs: &HashMap<String, String>) -> Result<Arc<dyn Node>, ConfigError> {
        Ok(Arc::new(DrumBot::new()))
    }
}

impl NodeFactory for DelayNode {
    fn factory(kwargs: &HashMap<String, String>) -> Result<Arc<dyn Node>, ConfigError> {
        let duration = kwarg_get!(kwargs, "duration").parse().map_err(ConfigError::of)?;
        Ok(Arc::new(DelayNode::new(Duration::from_secs_f32(duration))))
    }
}

impl NodeFactory for DebugNode {
    fn factory(kwargs: &HashMap<String, String>) -> Result<Arc<dyn Node>, ConfigError> {
        Ok(Arc::new(DebugNode::new(kwarg_get!(kwargs, "name"))))
    }
}

// -----------------------

type FactoryFunction = fn(&HashMap<String, String>) -> Result<Arc<dyn Node>, ConfigError>;

pub(super) trait NodeFactory {
    fn factory(kwargs: &HashMap<String, String>) -> Result<Arc<dyn Node>, ConfigError>;
}