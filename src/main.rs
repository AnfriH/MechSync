use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use crate::midi::{Input, Output};
use crate::node::{DebugNode, Node};

mod node;
mod data;
mod midi;

fn main() {
    may::config().set_workers(4);

    let v_in = Arc::new(Input::new("v_in").unwrap());
    let debug: Arc<dyn Node> = Arc::new(DebugNode::new("debugger"));
    let v_out: Arc<dyn Node> = Arc::new(Output::new("v_out").unwrap());

    v_in.bind(Arc::downgrade(&debug));
    debug.bind(Arc::downgrade(&v_out));

    loop {
        sleep(Duration::from_secs(5));
    }
}

