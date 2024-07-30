use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use crate::config::types::NodeFactory;
use crate::midi::{Input, Output};
use crate::node::Node;

/* TODO: Consider creating a #[derive!()] trait to allow easy dynamic constructors
 *  (will have to fight the syn library to make it work, might be out of scope)
 */

impl NodeFactory for Input {
    fn factory(kwargs: &HashMap<String, String>) -> Result<Arc<dyn Node>, Box<dyn Error>> {
        Ok(Arc::new(Input::new(kwargs.get("name").unwrap())?))
    }
}

impl NodeFactory for Output {
    fn factory(kwargs: &HashMap<String, String>) -> Result<Arc<dyn Node>, Box<dyn Error>> {
        Ok(Arc::new(Output::new(kwargs.get("name").unwrap())?))
    }
}