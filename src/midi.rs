use std::mem::ManuallyDrop;
// mutex should be fine here, as we only bind from a single thread. sorry may :(
use std::sync::{Mutex, Weak};

use may::go;
use may::sync::RwLock;
use midir::{ConnectError, MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};
use midir::os::unix::{VirtualInput, VirtualOutput};

use crate::data::MidiData;
use crate::node::Node;

#[derive(Copy, Clone)]
struct InputCallback {
    ptr: *mut RwLock<Option<Weak<dyn Node>>>
}

unsafe impl Sync for InputCallback {}
unsafe impl Send for InputCallback {}

impl InputCallback {
    unsafe fn call(&self, data: MidiData) {
        if let Some(callback) = self.ptr.as_ref().unwrap().read().unwrap().as_ref() {
            callback.upgrade().unwrap().call(data);
        }
    }

    unsafe fn bind(&self, node: Weak<dyn Node>) {
        let _ = self.ptr.as_ref().unwrap().write().unwrap().insert(node);
    }
}

pub(crate) struct Input {
    connection: ManuallyDrop<Mutex<MidiInputConnection<()>>>,
    ptr: InputCallback
}

impl Input {
    pub(crate) fn new(name: &str) -> Result<Self, ConnectError<MidiInput>> {
        let backing = MidiInput::new(name).unwrap();

        let binding = InputCallback { ptr: Box::into_raw(Box::new(RwLock::new(None)))};
        let binding_cpy = binding;

        let input = Input {
            connection: ManuallyDrop::new(Mutex::new(backing.create_virtual(name, move |ts, data, _| {
                let md = MidiData::from_slice(ts, data);
                let binding_cpy = binding_cpy;
                go!(move || unsafe{
                    binding_cpy.call(md);
                });
            }, ())?)),
            ptr: binding,
        };

        Ok(input)
    }

    pub(crate) fn bind(&self, node: Weak<dyn Node>) {
        unsafe {
            self.ptr.bind(node);
        }
    }
}

impl Node for Input {
    // NOTE: you probably didn't want to call this
    fn call(&self, _data: MidiData) -> () {
        //TODO: what should we do here?
    }
}

impl Drop for Input {
    fn drop(&mut self) {
        unsafe {
            // we must explicitly drop the midi connection first
            ManuallyDrop::drop(&mut self.connection);

            // then destroy the ptr
            let _ = Box::from_raw(self.ptr.ptr);
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