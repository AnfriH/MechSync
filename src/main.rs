use std::io::Read;
use std::thread;
use crate::config::graph::Graph;
use crate::node::Node;

mod node;
mod data;
mod midi;
mod instruments;
mod config;

fn main() {
    let _graph = Graph::from_yaml("
        - name: Dummy Input
          type: Input
          next: Dummy Output

        - name: MechBass Node
          type: MechBass
          next: Dummy Output

        - name: Dummy Output
          type: Output
    ").unwrap();

    loop { thread::park(); }
}


