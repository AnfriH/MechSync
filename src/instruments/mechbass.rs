use std::sync::Weak;
use std::time::Duration;

use may::coroutine::sleep;
use may::sync::RwLock;

use crate::data::{MidiData};
use crate::node::{Node, OptNode};
use crate::{rwlock_get, rwlock_get_mut};

// 12 notes in a scale
const TEMPERAMENT: f32 = 12f32;

// TODO: we've only derived using G2 -> (G2 .. G3), maybe worth testing some additional transitions?
// magic constants obtained via regression of Δt = linear * Δd ^ exponential
const LINEAR_COMP: f32 = 0.577963f32;
const EXPONENTIAL_COMP: f32 = 0.570981f32;
const TUNING: [u8; 4] = [43, 38, 33, 28];

#[derive(Copy, Clone, Debug)]
struct PlayedNode {
    pub(crate) note: u8,
    pub(crate) playing: bool,
    pub(crate) delay: Duration,
}

impl PlayedNode {
    fn default(note: u8) -> Self {
        PlayedNode {
            note,
            playing: false,
            delay: Duration::from_secs(0),
        }
    }

    fn play(note: u8, delay: Duration) -> Self {
        PlayedNode {
            note,
            playing: true,
            delay,
        }
    }
}

pub(crate) struct MechBass {
    // TODO: we need to encode prev_time into this
    prev_notes: [RwLock<PlayedNode>; 4],
    next: OptNode,
    delay: Duration
}

impl MechBass {
    pub(crate) fn new(delay: Duration) -> Self {
        MechBass {
            next: RwLock::new(None),
            prev_notes: TUNING.map(|n| RwLock::new(PlayedNode::default(n))),
            delay
        }
    }

    // distance in terms of string length, i.e 0.5 means half string length, etc
    fn note_distance(a: u8, b: u8) -> f32 {
        // convert into ratios based on equal temperament
        let a_ratio = 2f32.powf(-(a as f32 / TEMPERAMENT));
        let b_ratio = 2f32.powf(-(b as f32 / TEMPERAMENT));

        (a_ratio - b_ratio).abs()
    }

    fn panning_delay(&self, note: u8, channel: usize) -> Duration {
        let p = rwlock_get!(self.prev_notes[channel]).note;
        let prev_note: u8 = p - TUNING[channel];
        let cur_note: u8 = note - TUNING[channel];

        let dist = MechBass::note_distance(prev_note, cur_note);

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

                // TODO: a smarter dispatcher is needed for when a string is occupied
                // match depending on string
                let channel = match note_number {
                    0..=32 => 3u8,
                    33..=37 => 2u8,
                    38..=42 => 1u8,
                    _ => 0u8,
                };

                let function = (instruction << 4) | channel;

                let channel = channel as usize;
                let mut delay = Duration::from_secs(0);

                match instruction {
                    0b1001 => {
                        delay = self.panning_delay(note_number, channel);
                        rwlock_get_mut!(self.prev_notes[channel]) = PlayedNode::play(note_number, delay);
                    }
                    0b1000 => {
                        let pd = *rwlock_get!(self.prev_notes[channel]);
                        if pd.note == note_number {
                            delay = pd.delay;
                            rwlock_get_mut!(self.prev_notes[channel]).playing = false;
                        }
                    }
                    _ => {}
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