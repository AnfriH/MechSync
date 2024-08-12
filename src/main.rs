use std::error::Error;
use std::fs::read_to_string;
use std::path::Path;
use std::thread;
use clap::Parser;
use crate::config::graph::Graph;

mod node;
mod data;
mod midi;
mod instruments;
mod config;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    config_file: String,
    // TODO: Implement debug logging to allow for better traceability within the graph
    // #[arg(short, long, default_value = "false")]
    // debug: bool
}

fn main() {
    run().unwrap_or_else(|err| println!("An error occured:\n{}", err));
}

fn run() -> Result<(), Box<dyn Error>> {
    let args = Args::try_parse()?;
    let yaml = read_to_string(Path::new(&args.config_file))?;
    let _graph = Graph::from_yaml(&yaml)?;

    loop {
        thread::park();
    }
}


