use std::sync::Weak;
use std::time::Duration;

use may::coroutine::sleep;
use may::sync::RwLock;

use crate::data::MidiData;
use crate::node::{Node, OptNode};

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
        let p = self.prev_notes[channel].read().unwrap().note;
        let prev_note: u8 = p - TUNING[channel];
        let cur_note: u8 = note - TUNING[channel];

        let dist = MechBass::note_distance(prev_note, cur_note);

        Duration::from_secs_f32(LINEAR_COMP * dist.powf(EXPONENTIAL_COMP))
    }

    fn dispatch_channel(&self, note: u8) -> usize {
        for channel in 0usize..4 {
            // TODO: We should also heuristically choose a different string if we cannot pan in time (rare)
            // play the highest string possible, if taken, use next highest and so on
            if TUNING[channel] <= note && !self.prev_notes[channel].read().unwrap().playing {
                return channel
            }
        }
        // TODO: should we consider stealing the channel which has played the longest?
        // if all usable channels are taken, we'll just steal a channel early
        for channel in 0usize..4 {
            if TUNING[channel] <= note {
                println!("WARNING: note {} overriden by {} - channel {}", self.prev_notes[channel].read().unwrap().note, note, channel);
                return channel
            }
        }
        0
    }
}

impl Node for MechBass {
    fn call(&self, data: MidiData) -> () {
        let instruction = (data.data[0] & 0b1111_0000) >> 4;
        if let 0b1000 | 0b1001 = instruction {
            let note = data.data[1];
            let velocity = data.data[2];
            let mut channel = 0;
            let mut delay = Duration::from_secs(0);

            if instruction == 0b1001 && velocity != 0 {
                channel = self.dispatch_channel(note);
                delay = self.panning_delay(note, channel);
                *(self.prev_notes[channel].write().unwrap()) = PlayedNode::play(note, delay);

                println!("⬇ #{} - S{}", note, channel);
                sleep(self.delay - delay);
            } else {
                for ch in 0usize..4 {
                    let n = self.prev_notes[ch].read().unwrap().note;
                    if n == note {
                        let guard = self.prev_notes[ch].read().unwrap();

                        channel = ch;
                        delay = guard.delay;
                    }
                }

                println!("⬆ #{} - S{}", note, channel);
                sleep(self.delay - delay);

                {
                    self.prev_notes[channel].write().unwrap().playing = false;
                }
            }

            let function = (instruction << 4) | channel as u8;
            self.next.call(MidiData {ts: data.ts, data: [function, data.data[1], data.data[2]] })
        }
    }

    fn bind(&self, node: Weak<dyn Node>) -> () {
        self.next.bind(node);
    }
}