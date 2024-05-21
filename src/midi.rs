use crate::node::Node;
use crate::data::MidiData;
use std::cell::Cell;
use std::collections::HashMap;

// mutex should be fine here, as we only bind from a single thread. sorry may :(
use std::sync::{Weak, Mutex};
use may::go;
use may::sync::RwLock;
use midir::{ConnectError, MidiInput, MidiInputConnection};
use midir::os::unix::VirtualInput;
use once_cell::sync::Lazy;

pub(crate) static MIDI_INPUTS: Lazy<Mutex<HashMap<String, &'static Input>>> = Lazy::new(|| Mutex::new(HashMap::new()));
static HANDLES: Lazy<Mutex<Vec<MidiInputConnection<()>>>> = Lazy::new(|| Mutex::new(vec![]));

struct Input {
    callback: RwLock<Option<Weak<dyn Node + Sync + Send>>>
}

impl Input {
    fn new() -> Self {
        Input {
            callback: RwLock::new(None),
        }
    }

    fn bind(&mut self, node: Weak<dyn Node + Sync + Send>) {
        let mut callback = self.callback.write().unwrap();
        let _ = callback.insert(node);
    }

    fn call(&self, data: MidiData) {
        let callback = self.callback.read().unwrap();
        if let Some(cb) = callback.as_ref().and_then(|w| w.upgrade()) {
            cb.call(data);
        }
    }
}

// TODO: we should warn that this is a resource leak of input
fn create_virtual(port_name: &str, midi_input: MidiInput) -> Result<(), ConnectError<MidiInput>> {
    //NOTE: resource leak occurs here
    let input = Box::leak(Box::new(Input::new()));
    MIDI_INPUTS.lock().unwrap().insert(String::from(port_name), input);
    midi_input.create_virtual(port_name, |ts, data, _| {
        let md = MidiData::from_slice(ts, data);
        go!(|| {
            input.call(md);
        });
    }, ()
    ).map(|context| {
        HANDLES.lock().unwrap().push(context);
    })
}