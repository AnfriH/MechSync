use std::io::Read;
use std::thread;
use crate::config::config::Config;
use crate::node::Node;

mod node;
mod data;
mod midi;
mod instruments;
mod config;

const  DRUMBOT_DELAY_MS: u64 = 2000;

// mechbass is trivially fast at the moment
const MECHBASS_DELAY_MS: u64 = 0;

const MAX_DELAY: u64 = [MECHBASS_DELAY_MS, DRUMBOT_DELAY_MS][(MECHBASS_DELAY_MS < DRUMBOT_DELAY_MS) as usize];

fn main() {
    let graph = serde_yml::from_str::<Config>("
    - name: Dummy Input
      type: Input
      next: Dummy Output

    - name: Dummy Output
      type: Output
    ").expect("EXPLODE!").build().expect("It blew up");

    loop { thread::park(); }
}


