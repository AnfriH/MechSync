use std::array;
use std::sync::Weak;
use std::time::{Duration, Instant};
use log::{info};
use may::sync::RwLock;
use crate::data::MidiData;
use crate::node::{Node, OptNode};

const DRUMBOT_DELAY: Duration = Duration::from_millis(2000);

// FIXME: This needs a proper constructor

struct Arm {
    mapping: Vec<(u8, u8)>, // likely cheaper to just use a vec
    // with linear search instead of a hash
    last_played: u8,
    ts: Instant
}

impl Arm {
    fn new(mapping: Vec<(u8, u8)>) -> Arm {
        let last_played = mapping.get(0).map(|e| e.0).unwrap_or_default();
        Arm {
            mapping,
            last_played,
            ts: Instant::now(),
        }
    }

    fn get(&self, key: u8) -> Option<u8> {
        self.mapping.iter()
            .filter(|(k, _v)| *k == key)
            .map(|(_k, v)| *v)
            .next()
    }
}

pub struct DrumBot {
    arms: Vec<RwLock<Arm>>,
    next: OptNode
}

impl DrumBot {
    pub(crate) fn new() -> Self {
        let mappings: [Vec<(u8, u8)>; 3] = array::from_fn(|_| vec![]);
        DrumBot {
            arms: mappings.iter().map(|data| RwLock::new(Arm::new(data.clone()))).collect(),
            next: RwLock::new(None)
        }
    }
}

impl Node for DrumBot {
    fn call(&self, data: MidiData) -> () {
        // FIXME: this is cloned from MechBass, should we modularize?

        // only allow note-ons (might be changed later)
        if data.instruction != 0b1001 {
            return;
        }

        // simple check that an arm isn't already there
        for (index, arm) in self.arms.iter().enumerate() {
            let arm_lock = arm.read().unwrap();
            if arm_lock.last_played == data.note {
                info!(target: "DrumBot", "▩{} on arm {}", data.note, index);
                self.next.call(MidiData {
                    instruction: data.instruction,
                    channel: data.channel,
                    note: arm_lock.get(data.note).unwrap(),
                    velocity: data.velocity,
                });
                return;
            }
        }

        // if no arms are at the drum, we want to use the least-used arm (whichever has been idle for the longest)
        let mut arms: Vec<(usize, &RwLock<Arm>)> = self.arms.iter()
            .enumerate()
            .filter(|(_i, arm)| arm.read().unwrap().get(data.note).is_some())
            .collect();
        arms.sort_unstable_by_key(|(_i, arm)| arm.read().unwrap().ts);
        if let Some((index, arm)) = arms.get(0) {
            let note: u8;
            {
                let mut arm_lock = arm.write().unwrap();
                arm_lock.ts = Instant::now();
                arm_lock.last_played = data.note;
                note = arm_lock.get(data.note).unwrap();
            }
            info!(target: "MechBass", "▩{} on arm {}", data.note, index);
            self.next.call(MidiData {
                instruction: data.instruction,
                channel: data.channel,
                note,
                velocity: data.velocity,
            });

            return;
        }

        // if no arms were assigned, it's likely we want to passthrough
        self.next.call(data);
    }

    fn bind(&self, node: Weak<dyn Node>) -> () {
        self.next.bind(node);
    }

    fn delay(&self) -> Duration {
        DRUMBOT_DELAY
    }
}