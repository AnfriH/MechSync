use std::sync::Arc;
use crate::node::Node;

mod node;
mod data;
mod midi;

fn main() {
    midi::virtual_input("DemoInput").expect("Fail");

    let demo_node: Arc<dyn Node> = Arc::new(node::DummyNode {});
    midi::bind_input("DemoInput", Arc::downgrade(&demo_node)).expect("Input should exist");

    loop {}
}

