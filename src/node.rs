use std::sync::Weak;
use may::sync::RwLock;
use crate::data::MidiData;

pub(crate) type OptNode = RwLock<Option<Weak<dyn Node>>>;

pub(crate) trait Node: Sync + Send {
    fn call(&self, data: MidiData) -> ();

    //FIXME: This doesn't seem appropriate. Not all nodes support binding children
    fn bind(&self, node: Weak<dyn Node>) -> ();
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
        println!("{} Received {:?} at {}ns", self.name, data.data, data.ts);
        self.next.call(data);
    }

    fn bind(&self, node: Weak<dyn Node>) {
        self.next.bind(node);
    }
}

