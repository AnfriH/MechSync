use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use crate::node::{DummyNode, Node};

mod node;
mod data;
mod midi;

use midi::{Input, Output};

fn main() {
    let v_in = Input::new("v_in").expect("Exploded!");
    let dummy: Arc<dyn Node> = Arc::new(DummyNode {});
    v_in.bind(Arc::downgrade(&dummy));
    sleep(Duration::from_secs(5));
}

