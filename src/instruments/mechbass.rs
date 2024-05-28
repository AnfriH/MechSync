use std::ops::{Deref, DerefMut};
use std::sync::Weak;
use std::time::Duration;

use may::coroutine::sleep;
use may::sync::RwLock;

use crate::data::MidiData;
use crate::node::{Node, OptNode};

const LINEAR_COMP: f32 = 0.577963f32;
const EXPONENTIAL_COMP: f32 = 0.570981f32;
const TUNING: [u8; 4] = [43, 38, 33, 28];

pub(crate) struct MechBass {
    // TODO: we need to encode prev_time into this
    prev_notes: [RwLock<(u8, Duration)>; 4],
    next: OptNode,
    delay: Duration
}

impl MechBass {
    pub(crate) fn new(delay: Duration) -> Self {
        MechBass {
            next: RwLock::new(None),
            prev_notes: TUNING.map(|e| RwLock::new((e, Duration::from_secs(0)))),
            delay
        }
    }

    fn panning_delay(&self, note: u8, channel: usize) -> Duration {
        let (p, d) = self.prev_notes[channel].read().unwrap().deref().clone();
        let prev_note: u8 = p - TUNING[channel];
        let cur_note: u8 = note - TUNING[channel];

        let prev_rat = 2f32.powf(-(prev_note as f32 / 12f32));
        let cur_rat = 2f32.powf(-(cur_note as f32 / 12f32));

        let dist = (prev_rat - cur_rat).abs();

        Duration::from_secs_f32(LINEAR_COMP * dist.powf(EXPONENTIAL_COMP))
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

                let channel = channel as usize;
                let mut delay = self.panning_delay(note_number, channel);

                if instruction == 0b1001 {
                    *self.prev_notes[channel].write().unwrap().deref_mut() = (note_number, delay);
                }
                if instruction == 0b1000 {
                    let (p, d) = self.prev_notes[channel].read().unwrap().deref().clone();
                    if p == note_number {
                        delay = d;
                    }
                }

                println!("{:?}", delay);
                sleep(self.delay - delay);

                self.next.call(MidiData {ts: data.ts, data: [function, data.data[1], data.data[2]] })
            }
            _ => {}
        };
    }

    fn bind(&self, node: Weak<dyn Node>) -> () {
        self.next.bind(node);
    }
}