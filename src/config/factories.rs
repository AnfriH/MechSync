use std::collections::HashMap;
use std::sync::Arc;

use once_cell::sync::Lazy;

use crate::config::config::ConfigError;
use crate::midi::{Input, Output};
use crate::node::Node;

macro_rules! types {
    ( $( $typename:ident ),* ) => {
        HashMap::from([$((stringify!($typename), $typename::factory as FactoryFunction), )*])
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
    Output
]);

impl NodeFactory for Input {
    fn factory(kwargs: &HashMap<String, String>) -> Result<Arc<dyn Node>, ConfigError> {
        let node = Input::new(kwargs.get("name").unwrap()).map_err(ConfigError::of)?;
        Ok(Arc::new(node))
    }
}

impl NodeFactory for Output {
    fn factory(kwargs: &HashMap<String, String>) -> Result<Arc<dyn Node>, ConfigError> {
        let node = Output::new(kwargs.get("name").unwrap()).map_err(ConfigError::of)?;
        Ok(Arc::new(node))
    }
}

// -----------------------

type FactoryFunction = fn(&HashMap<String, String>) -> Result<Arc<dyn Node>, ConfigError>;

pub(super) trait NodeFactory {
    fn factory(kwargs: &HashMap<String, String>) -> Result<Arc<dyn Node>, ConfigError>;
}