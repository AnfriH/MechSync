use std::sync::Weak;
use std::time::{Duration, Instant};
use log::{error, info, warn};
use may::coroutine::sleep;
use may::sync::RwLock;
use pyo3::{intern, Py, PyErr, Python};
use pyo3::types::{PyAnyMethods, PyModule};
use crate::data::MidiData;
use crate::node::{Node, OptNode};

pub(crate) struct PyNode {
    duration: Duration,
    module: Py<PyModule>,
    next: OptNode,
}

impl PyNode {
    pub(crate) fn new(source: &str, duration: Duration) -> Result<Self, PyErr> {
        pyo3::prepare_freethreaded_python();
        let module: Py<PyModule> = Python::with_gil(|py| {
            PyModule::from_code_bound(py, source, "pynode.py", "pynode").and_then(|module_bound| {
                module_bound
                    .getattr(intern!(py, "call"))?
                    .getattr(intern!(py, "__call__"))?;

                Ok(module_bound.into())
            })
        })?;
        
        Ok(PyNode {
            duration,
            module,
            next: RwLock::new(None)
        })
    }
}

impl Node for PyNode {
    fn call(&self, data: MidiData) -> () {
        info!(target: "PyNode", "Recieved {:?}", data);
        let ts_start = Instant::now();
        let output = Python::with_gil(|py| {
            let module = self.module.bind(py);

            // Get the function and call it.
            module.getattr("call").unwrap().call1((
                data.instruction,
                data.channel,
                data.note,
                data.velocity
            )).and_then(|out| {
                out.extract::<(u8, u8, u8, u8, f32)>()
            })
        });
        if let Err(error) = output {
            error!(target: "PyNode", "Python failed due to error: {}", error);
            return;
        }
        let (instruction, channel, note, velocity, delay) = output.unwrap();
        let py_duration = Instant::now() - ts_start;
        let target_duration = self.duration + Duration::from_secs_f32(delay);
        if py_duration <= target_duration {
            sleep(target_duration - py_duration);
        } else {
            warn!(target: "PyNode", "Took longer than {:?} (was {:?})", target_duration, py_duration);
        }
        let out_data = MidiData {
            instruction,
            channel,
            note,
            velocity,
        };
        info!(target: "PyNode", "Sending {:?}", out_data);
        self.next.call(out_data);
    }

    fn bind(&self, node: Weak<dyn Node>) {
        self.next.bind(node);
    }

    fn delay(&self) -> Duration {
        self.duration
    }
}