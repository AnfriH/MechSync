use std::collections::HashMap;
// mutex should be fine here, as we only bind from a single thread. sorry may :(
use std::sync::{Mutex, Weak};

use may::go;
use may::sync::RwLock;
use midir::{ConnectError, MidiInput, MidiInputConnection};
use midir::os::unix::VirtualInput;
use once_cell::sync::Lazy;

use crate::data::MidiData;
use crate::node::Node;

static MIDI_INPUTS: Lazy<Mutex<HashMap<String, &'static Input>>> = Lazy::new(|| Mutex::new(HashMap::new()));
static HANDLES: Lazy<Mutex<Vec<MidiInputConnection<()>>>> = Lazy::new(|| Mutex::new(vec![]));

struct Input {
    callback: RwLock<Option<Weak<dyn Node>>>
}

impl Input {
    fn new() -> Self {
        Input {
            callback: RwLock::new(None),
        }
    }

    fn bind(&self, node: Weak<dyn Node>) {
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

// FIXME: this API should return a Result<(), Error>
pub(crate) fn bind_input(name: &str, node: Weak<dyn Node>) -> Result<(), ()> {
    let inputs = MIDI_INPUTS.lock().unwrap();
    let input = inputs.get(name.into());
    match input {
        None => Err(()),
        Some(i) => {
            i.bind(node);
            Ok(())
        }
    }
}

// TODO: we should warn that this is a resource leak of input
pub(crate) fn virtual_input(name: &str) -> Result<(), ConnectError<MidiInput>> {
    let backing = MidiInput::new(name).unwrap();
    //NOTE: resource leak occurs here
    let input = Box::leak(Box::new(Input::new()));
    MIDI_INPUTS.lock().unwrap().insert(String::from(name), input);
    backing.create_virtual(name, |ts, data, _| {
        let md = MidiData::from_slice(ts, data);
        go!(|| {
            input.call(md);
        });
    }, ()
    ).map(|context| {
        HANDLES.lock().unwrap().push(context);
    })
}