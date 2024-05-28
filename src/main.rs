use std::io::{Read, stdin};
use std::sync::Arc;
use std::time::Duration;

use crate::instruments::{DrumBot, MechBass};
use crate::midi::{Input, Output};
use crate::node::{DelayNode, Node};

mod node;
mod data;
mod midi;
mod instruments;


const  DRUMBOT_DELAY_MS: u64 = 2000;

// mechbass is trivially fast at the moment
const MECHBASS_DELAY_MS: u64 = 0;

const MAX_DELAY: u64 = [MECHBASS_DELAY_MS, DRUMBOT_DELAY_MS][(MECHBASS_DELAY_MS < DRUMBOT_DELAY_MS) as usize];

fn main() {
    // EXAMPLE NETWORK, WILL LATER IMPLEMENT CONFIG LOADING
    // nodes
    let mech_in = Input::new("MechBass Input").unwrap();
    let drum_in = Input::new("DrumBot Input").unwrap();

    let mech_handle: Arc<dyn Node> = Arc::new(MechBass::new(Duration::from_millis(MAX_DELAY - MECHBASS_DELAY_MS)));
    let drum_handle: Arc<dyn Node> = Arc::new(DrumBot::new());

    // let mech_delay: Arc<dyn Node> = Arc::new(DelayNode::new(Duration::from_millis(max_delay - MECHBASS_DELAY_MS)));
    let drum_delay: Arc<dyn Node> = Arc::new(DelayNode::new(Duration::from_millis(MAX_DELAY - DRUMBOT_DELAY_MS)));

    let mech_output: Arc<dyn Node> = Arc::new(Output::new("MechBass Output").unwrap());
    let drum_output: Arc<dyn Node> = Arc::new(Output::new("DrumBot Output").unwrap());

    //linking
    mech_in.bind(Arc::downgrade(&mech_handle));
    drum_in.bind(Arc::downgrade(&drum_handle));
    mech_handle.bind(Arc::downgrade(&mech_output));
    drum_handle.bind(Arc::downgrade(&drum_delay));
    // mech_delay.bind(Arc::downgrade(&mech_output));
    drum_delay.bind(Arc::downgrade(&drum_output));

    // for now we just block with stdin. later, we'll implement some frontend, maybe...
    let mut buffer = [];
    stdin().read(&mut buffer).unwrap();
}


