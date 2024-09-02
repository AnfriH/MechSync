use std::cmp::Reverse;
use std::sync::Weak;
use std::time::{Duration, Instant};
use log::{info, warn};
use may::coroutine::sleep;
use may::sync::RwLock;
use once_cell::sync::Lazy;
use crate::data::MidiData;
use crate::node::{Node, OptNode};

// 12 notes in a scale
const TEMPERAMENT: f32 = 12f32;

// magic constants obtained via regression of Δt = linear * Δd ^ exponential + quad * Δd ^ 2
const LINEAR_COMP: f32 = 0.515936f32;
const EXPONENTIAL_COMP: f32 = 0.515920f32;
const QUADRATIC_COMP: f32 = 0.125675f32;

// TODO: We should consider turning these into instance-variables for configuration support
const TUNING: [u8; 4] = [43, 38, 33, 28];
const FRETS: u8 = 13;

#[inline]
fn time(dist: f32) -> f32 {
    LINEAR_COMP * dist.powf(EXPONENTIAL_COMP) + dist * dist * QUADRATIC_COMP
}

// derive the maximum panning duration based on the maximum distance travelled between frets
static MAX_PAN_TIME: Lazy<Duration> = Lazy::new(|| {
    Duration::from_secs_f32(
        time(MechBass::note_distance(0, FRETS))
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

        *MAX_PAN_TIME - Duration::from_secs_f32(time(dist))
    }

    fn dispatch_channel(&self, note: u8) -> (usize, Duration) {
        // collect all channels which the note can play on
        let mut channels: Vec<usize> = (0usize..4)
            .filter(|ch| TUNING[*ch] <= note && TUNING[*ch] + FRETS > note)
            .collect();
        // sort by the channel which is the closest to the note
        channels.sort_unstable_by_key(|ch| Reverse(self.panning_delay(note, *ch)));
        for channel in channels {
            let prev_note = self.prev_notes[channel].read().unwrap();
            let delay = self.panning_delay(note, channel);
            if !prev_note.playing && Instant::now() + delay > prev_note.ts + prev_note.delay {
                return (channel, delay)
            }
        }
        // TODO: should we consider stealing the channel which has played the longest?
        // if all usable channels are taken, we'll just steal a channel early
        for channel in 0usize..4 {
            if TUNING[channel] <= note {
                warn!(target: "MechBass", "Note {} overriden by {} - channel {}", self.prev_notes[channel].read().unwrap().note, note, channel);
                let delay = self.panning_delay(note, channel);
                return (channel, delay)
            }
        }
        (0, self.panning_delay(note, 0))
    }

    fn find_playing(&self, note: u8) -> Option<(usize, Duration)> {
        for ch in 0usize..4 {
            let prev_note = self.prev_notes[ch].read().unwrap();
            if prev_note.playing && prev_note.note == note {
                let guard = self.prev_notes[ch].read().unwrap();

                return Some((ch, guard.delay));
            }
        }
        warn!(target: "MechBass", "Released note {}, but none were playing", note);
        None
    }
}

impl Node for MechBass {
    fn call(&self, data: MidiData) -> () {
        let (0b1000 | 0b1001) = data.instruction else {
            return;
        };
        let channel;
        let delay;

        // TODO: this behaviour still needs cleaning up a lot!
        if data.instruction == 0b1001 && data.velocity != 0 {
            (channel, delay) = self.dispatch_channel(data.note);
            {
                *(self.prev_notes[channel].write().unwrap()) = PlayedNote::play(data.note, delay);
            }
            info!(target: "MechBass", "⬇{} on channel {}", data.note, channel);
        } else {
            let Some(playing) = self.find_playing(data.note) else {
                return;
            };
            (channel, delay) = playing;

            {
                let mut prev_note = self.prev_notes[channel].write().unwrap();
                prev_note.playing = false;
                prev_note.ts = Instant::now();
            }

            info!(target: "MechBass", "⬆{} on channel {}", data.note, channel);
        }

        sleep(delay);
        self.next.call(MidiData {
            instruction: data.instruction,
            channel: channel as u8,
            note: data.note,
            velocity: data.velocity,
        })
    }

    fn bind(&self, node: Weak<dyn Node>) -> () {
        self.next.bind(node);
    }

    fn delay(&self) -> Duration {
        *MAX_PAN_TIME
    }
}