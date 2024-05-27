use std::sync::Weak;
use may::sync::RwLock;
use crate::data::MidiData;
use crate::node::{Node, OptNode};

pub(crate) struct MechBass {
    next: OptNode
}

impl MechBass {
    pub(crate) fn new() -> Self {
        MechBass {
            next: RwLock::new(None),
        }
    }
}

impl Node for MechBass {
    fn call(&self, data: MidiData) -> () {
        let instruction = (data.data[0] & 0b1111_0000) >> 4;

        match instruction {
            // we only care about note playing
            0b1000 | 0b1001 | 0b1010 => {
                let note_number = data.data[1];

                // match depending on string
                let channel = match note_number {
                    0..=32 => 3u8,
                    33..=37 => 2u8,
                    38..=42 => 1u8,
                    _ => 0u8,
                };

                let function = (instruction << 4) | channel;

                self.next.call(MidiData {ts: data.ts, data: [function, data.data[1], data.data[2]] })
            }
            _ => {}
        };
    }

    fn bind(&self, node: Weak<dyn Node>) -> () {
        self.next.bind(node);
    }
}