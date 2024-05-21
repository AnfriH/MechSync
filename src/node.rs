use crate::data::MidiData;

pub(crate) trait Node {
    fn call(&self, data: MidiData) -> ();
}