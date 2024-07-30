use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

use once_cell::sync::Lazy;

use crate::midi::{Input, Output};
use crate::node::Node;

pub(super) trait NodeFactory {
    fn factory(name: &str, traits: &HashMap<String, String>) -> Result<Arc<dyn Node>, Box<dyn Error>>;
}
type FactoryFunction = fn(&str, &HashMap<String, String>) -> Result<Arc<dyn Node>, Box<dyn Error>>;

macro_rules! types {
    ( $( $typename:ident ),* ) => {
        HashMap::from([
            $(
                (stringify!($typename), $typename::factory as FactoryFunction),
            )*
        ])
    }
}

pub(super) static TYPES: Lazy<HashMap<&'static str, FactoryFunction>> = Lazy::new(|| types![
    Input,
    Output
]);
// DelayNode,
// DebugNode,
// DrumBot,
// MechBass