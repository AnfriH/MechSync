use crate::data::MidiData;

pub(crate) trait Node: Sync + Send {
    fn call(&self, data: MidiData) -> ();
}

pub(crate) struct DummyNode {}

impl Node for DummyNode {
    fn call(&self, data: MidiData) -> () {
        println!("Recieved {:?} at {}ns", data.data, data.ts);
    }
}