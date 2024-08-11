use std::sync::Weak;
use std::time::{Duration, Instant};

use may::coroutine::sleep;
use may::sync::RwLock;
use once_cell::sync::Lazy;
use crate::data::MidiData;
use crate::node::{Node, OptNode};

// 12 notes in a scale
const TEMPERAMENT: f32 = 12f32;

// TODO: we've only derived using G2 -> (G2 .. G3), maybe worth testing some additional transitions?
// magic constants obtained via regression of Δt = linear * Δd ^ exponential
const LINEAR_COMP: f32 = 0.577963f32;
const EXPONENTIAL_COMP: f32 = 0.570981f32;
const TUNING: [u8; 4] = [43, 38, 33, 28];
const FRETS: u8 = 13;

// derive the maximum panning duration based on the maximum distance travelled between frets
static MAX_PAN_TIME: Lazy<Duration> = Lazy::new(|| {
    Duration::from_secs_f32(
        LINEAR_COMP * MechBass::note_distance(0, FRETS).powf(EXPONENTIAL_COMP)
    )
});

#[derive(Copy, Clone, Debug)]
struct PlayedNote {
    playing: bool,
    note: u8,
    delay: Duration,
    ts: Instant
}

impl PlayedNote {
    fn default(note: u8) -> Self {
        PlayedNote {
            playing: false,
            note,
            delay: Duration::default(),
            ts: Instant::now()
        }
    }

    fn play(note: u8, delay: Duration) -> Self {
        PlayedNote {
            playing: true,
            note,
            delay,
            ts: Instant::now()
        }
    }
}

pub(crate) struct MechBass {
    // TODO: we need to encode prev_time into this
    prev_notes: [RwLock<PlayedNote>; 4],
    next: OptNode,
}

impl MechBass {
    pub(crate) fn new() -> Self {
        MechBass {
            next: RwLock::new(None),
            prev_notes: TUNING.map(|n| RwLock::new(PlayedNote::default(n))),
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

        *MAX_PAN_TIME - Duration::from_secs_f32(LINEAR_COMP * dist.powf(EXPONENTIAL_COMP))
    }

    fn dispatch_channel(&self, note: u8) -> (usize, Duration) {
        for channel in 0usize..4 {
            // TODO: We should also heuristically choose a different string if we cannot pan in time (rare)
            // play the highest string possible, if taken, use next highest and so on
            if TUNING[channel] <= note {
                let prev_note = self.prev_notes[channel].read().unwrap();
                let delay = self.panning_delay(note, channel);
                if !prev_note.playing && Instant::now() + delay > prev_note.ts + prev_note.delay {
                    return (channel, delay)
                }
            }
        }
        // TODO: should we consider stealing the channel which has played the longest?
        // if all usable channels are taken, we'll just steal a channel early
        for channel in 0usize..4 {
            if TUNING[channel] <= note {
                println!("WARNING: note {} overriden by {} - channel {}", self.prev_notes[channel].read().unwrap().note, note, channel);
                let delay = self.panning_delay(note, channel);
                return (channel, delay)
            }
        }
        return (0, self.panning_delay(note, 0))
    }

    fn find_playing(&self, note: u8) -> Option<(usize, Duration)> {
        for ch in 0usize..4 {
            let prev_note = self.prev_notes[ch].read().unwrap();
            if prev_note.playing && prev_note.note == note {
                let guard = self.prev_notes[ch].read().unwrap();

                return Some((ch, guard.delay));
            }
        }
        println!("WARNING: released note {}, but none were playing", note);
        return None
    }
}

impl Node for MechBass {
    fn call(&self, data: MidiData) -> () {
        let instruction = (data.data[0] & 0b1111_0000) >> 4;

        let (0b1000 | 0b1001) = instruction else {
            return;
        };
        let note = data.data[1];
        let velocity = data.data[2];
        let channel;
        let delay;

        // TODO: this behaviour still needs cleaning up a lot!
        if instruction == 0b1001 && velocity != 0 {
            (channel, delay) = self.dispatch_channel(note);
            {
                *(self.prev_notes[channel].write().unwrap()) = PlayedNote::play(note, delay);
            }
            println!("⬇ #{} - S{}", note, channel);
        } else {
            let Some(playing) = self.find_playing(note) else {
                return;
            };
            (channel, delay) = playing;

            {
                let mut prev_note = self.prev_notes[channel].write().unwrap();
                prev_note.playing = false;
                prev_note.ts = Instant::now();
            }

            println!("⬆ #{} - S{}", note, channel);
        }

        sleep(delay);
        let function = (instruction << 4) | channel as u8;
        self.next.call(MidiData {ts: data.ts, data: [function, data.data[1], data.data[2]] })
    }

    fn bind(&self, node: Weak<dyn Node>) -> () {
        self.next.bind(node);
    }
}