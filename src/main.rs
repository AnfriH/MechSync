use std::error::Error;
use std::fs::read_to_string;
use std::path::Path;
use std::process::exit;
use std::{env, thread};
use std::io::Write;
use clap::Parser;
use log::{error, info};
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
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info")
    }
    env_logger::builder().format(|fmt, record|
        writeln!(fmt, "[{}@{}]:\n{}", record.level(), record.target(), record.args())
    ).init();
    run().unwrap_or_else(|err| {
        error!(target: "Startup", "{}", err);
        exit(1);
    });
}

fn run() -> Result<(), Box<dyn Error>> {
    let args = Args::try_parse()?;
    let yaml = read_to_string(Path::new(&args.config_file))?;
    info!(target: "Startup", "Loading config");
    let _graph = Graph::from_yaml(&yaml)?;
    info!(target: "Startup", "Config loaded!");
    loop {
        thread::park();
    }
}


