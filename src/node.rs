use std::sync::Weak;
use std::time::{Duration, Instant};
use log::debug;
use may::coroutine::sleep;
use may::sync::RwLock;
use crate::data::MidiData;

pub(crate) type OptNode = RwLock<Option<Weak<dyn Node>>>;

pub(crate) trait Node: Sync + Send {
    fn call(&self, data: MidiData) -> ();

    fn bind(&self, node: Weak<dyn Node>) -> ();

    fn delay(&self) -> Duration {
        Duration::from_secs(0)
    }
}

impl Node for OptNode {
    fn call(&self, data: MidiData) -> () {
        if let Some(wk_ref) = self.read().unwrap().as_ref().and_then(Weak::upgrade) {
            wk_ref.call(data);
        }
    }

    fn bind(&self, node: Weak<dyn Node>) -> () {
        let _ = self.write().unwrap().insert(node);
    }
}

pub(crate) struct DebugNode {
    name: String,
    next: OptNode
}

impl DebugNode {
    pub(crate) fn new(name: &str) -> Self {
        DebugNode {
            name: String::from(name),
            next: RwLock::new(None),
        }
    }
}

impl Node for DebugNode {
    fn call(&self, data: MidiData) -> () {
        debug!(target: &self.name, "Received {:?} at {:?}", data, Instant::now());
        self.next.call(data);
    }

    fn bind(&self, node: Weak<dyn Node>) {
        self.next.bind(node);
    }
}

pub(crate) struct DelayNode {
    duration: Duration,
    next: OptNode
}

impl DelayNode {
    pub(crate) fn new(duration: Duration) -> Self {
        DelayNode {
            duration,
            next: RwLock::new(None),
        }
    }
}

impl Node for DelayNode {
    fn call(&self, data: MidiData) -> () {
        //TODO: coroutine::sleep should be evaluated to see whether it may benefit from spin-locking
        sleep(self.duration);
        self.next.call(data);
    }

    fn bind(&self, node: Weak<dyn Node>) -> () {
        self.next.bind(node);
    }

    fn delay(&self) -> Duration {
        return self.duration
    }
}
