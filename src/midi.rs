use std::mem::ManuallyDrop;
// mutex should be fine here, as we only bind from a single thread. sorry may :(
use std::sync::{Arc, Mutex, Weak};

use may::go;
use may::sync::RwLock;
use midir::{ConnectError, MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};
use midir::os::unix::{VirtualInput, VirtualOutput};

use crate::data::MidiData;
use crate::node::Node;

struct InputImpl {
    callback: RwLock<Option<Weak<dyn Node>>>
}

impl InputImpl {
    fn new() -> Self {
        InputImpl {
            callback: RwLock::new(None),
        }
    }

    fn bind(&self, node: Weak<dyn Node>) {
        let mut callback = self.callback.write().unwrap();
        let _prev = callback.insert(node);
    }

    fn call(&self, data: MidiData) {
        let callback = self.callback.read().unwrap();
        if let Some(cb) = callback.as_ref().and_then(|w| w.upgrade()) {
            cb.call(data);
        }
    }
}

pub(crate) struct Input {
    connection: ManuallyDrop<Mutex<MidiInputConnection<()>>>,

    // NOTE: binding must never be leaked, otherwise reference to freed memory will occur
    binding: &'static InputImpl
}

impl Input {
    pub(crate) fn new(name: &str) -> Result<Self, ConnectError<MidiInput>> {
        let backing = MidiInput::new(name).unwrap();

        let binding = Box::leak(Box::new(InputImpl::new()));

        let input = Input {
            connection: ManuallyDrop::new(Mutex::new(backing.create_virtual(name, |ts, data, _| {
                let md = MidiData::from_slice(ts, data);
                go!(|| {
                    binding.call(md);
                });
            }, ())?)),
            binding
        };

        Ok(input)
    }

    pub(crate) fn bind(&self, node: Weak<dyn Node>) {
        self.binding.bind(node)
    }
}

impl Node for Input {
    fn call(&self, data: MidiData) -> () {
        self.binding.call(data)
    }
}

impl Drop for Input {
    fn drop(&mut self) {
        unsafe {
            // we must explicitly drop the midi connection first
            ManuallyDrop::drop(&mut self.connection);

            // then we must turn our &'static into a *mut to properly deconstruct
            let _ = Box::from_raw(self.binding as *const InputImpl as *mut InputImpl);
        }
    }
}

pub(crate) struct Output {
    output: Mutex<MidiOutputConnection>
}

impl Output {
    pub(crate) fn new(name: &str) -> Result<Self, ConnectError<MidiOutput>> {
        let backing = MidiOutput::new(name).unwrap();
        Ok(Output { output: Mutex::new(backing.create_virtual(name)?), })
    }
}

impl Node for Output {
    fn call(&self, data: MidiData) -> () {
        self.output.lock().unwrap().send(data.data.as_slice()).unwrap();
    }
}