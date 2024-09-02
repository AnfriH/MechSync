use std::mem::ManuallyDrop;
// mutex should be fine here, as we only bind from a single thread. sorry may :(
use std::sync::{Mutex, Weak};

use may::go;
use may::sync::RwLock;
use midir::{ConnectError, MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};
use midir::os::unix::{VirtualInput, VirtualOutput};

use crate::data::MidiData;
use crate::node::{Node, OptNode};

#[derive(Copy, Clone)]
struct InputCallback {
    ptr: *mut OptNode
}

unsafe impl Sync for InputCallback {}
unsafe impl Send for InputCallback {}

impl InputCallback {
    unsafe fn call(&self, data: MidiData) {
        self.ptr.as_ref().unwrap().call(data);
    }

    unsafe fn bind(&self, node: Weak<dyn Node>) {
        self.ptr.as_ref().unwrap().bind(node);
    }
}

pub(crate) struct Input {
    connection: ManuallyDrop<Mutex<MidiInputConnection<()>>>,
    binding: InputCallback
}

impl Input {
    pub(crate) fn new(name: &str) -> Result<Self, ConnectError<MidiInput>> {
        let backing = MidiInput::new("MechSync").unwrap();

        let binding = InputCallback { ptr: Box::into_raw(Box::new(RwLock::new(None)))};
        let binding_cpy = binding;

        let input = Input {
            connection: ManuallyDrop::new(Mutex::new(backing.create_virtual(name, move |_ts, data, _| {
                let md = MidiData::from_slice(data);
                let binding_cpy = binding_cpy;
                go!(move || unsafe{
                    binding_cpy.call(md);
                });
            }, ())?)),
            binding,
        };

        Ok(input)
    }
}

impl Node for Input {
    // NOTE: you probably didn't want to call this
    fn call(&self, _data: MidiData) -> () {
        //TODO: what should we do here?
    }

    fn bind(&self, node: Weak<dyn Node>) {
        unsafe {
            self.binding.bind(node);
        }
    }
}

impl Drop for Input {
    fn drop(&mut self) {
        unsafe {
            // we must explicitly drop the midi connection first
            ManuallyDrop::drop(&mut self.connection);

            // then destroy the ptr
            let _ = Box::from_raw(self.binding.ptr);
        }
    }
}

pub(crate) struct Output {
    output: Mutex<MidiOutputConnection>
}

impl Output {
    pub(crate) fn new(name: &str) -> Result<Self, ConnectError<MidiOutput>> {
        let backing = MidiOutput::new("MechSync").unwrap();
        Ok(Output { output: Mutex::new(backing.create_virtual(name)?), })
    }
}

impl Node for Output {
    fn call(&self, data: MidiData) -> () {
        self.output.lock().unwrap().send(data.to_array().as_slice()).unwrap();
    }

    // NOTE: you probably didn't want to call this
    fn bind(&self, _node: Weak<dyn Node>) -> () {
        //TODO: what should we do here?
    }
}