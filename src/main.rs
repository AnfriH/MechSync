use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use crate::midi::{Input, Output};
use crate::node::{DebugNode, Node, DelayNode};

mod node;
mod data;
mod midi;

fn main() {
    let v_in = Arc::new(Input::new("v_in").unwrap());
    let before_delay: Arc<dyn Node> = Arc::new(DebugNode::new("before_delay"));
    let delay: Arc<dyn Node> =Arc::new(DelayNode::new(Duration::from_millis(500)));
    let after_delay: Arc<dyn Node> = Arc::new(DebugNode::new("after_delay"));
    let v_out: Arc<dyn Node> = Arc::new(Output::new("v_out").unwrap());

    v_in.bind(Arc::downgrade(&before_delay));
    before_delay.bind(Arc::downgrade(&delay));
    delay.bind(Arc::downgrade(&after_delay));
    after_delay.bind(Arc::downgrade(&v_out));

    loop {
        sleep(Duration::from_secs(5));
    }
}

