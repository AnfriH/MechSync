use std::sync::Weak;
use may::sync::RwLock;
use crate::data::MidiData;
use crate::node::{Node, OptNode};

pub struct DrumBot {
    next: OptNode
}

impl DrumBot {
    pub(crate) fn new() -> Self {
        DrumBot {
            next: RwLock::new(None),
        }
    }
}

impl Node for DrumBot {
    fn call(&self, data: MidiData) -> () {
        self.next.call(data);
    }

    fn bind(&self, node: Weak<dyn Node>) -> () {
        self.next.bind(node);
    }
}