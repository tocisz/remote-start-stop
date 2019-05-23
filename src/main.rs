extern crate core;

use std::env;
use config::OpenerConfig;
use std::rc::Rc;

mod remote_exec;
mod config;

#[derive(Debug)]
pub struct CommandData {
    pub command: String,
    pub expected: Option<String>
}

#[derive(Debug)]
enum TopLevelError {
    ConfigError(config::ConfigError),
    ClientError(remote_exec::ClientError),
    OpenError(opener::OpenError)
}

trait Runner {
    fn has_command(&self, cmd: &String) -> bool;
    fn run(&self, cmd: &String) -> Result<(), TopLevelError>;
}

struct Opener {
    config: OpenerConfig
}

impl Runner for Opener {
    fn has_command(&self, cmd: &String) -> bool {
        cmd == "open"
    }
    fn run(&self, _cmd: &String) -> Result<(), TopLevelError> {
        opener::open(&self.config.link)
            .map_err(|e| TopLevelError::OpenError(e))
    }
}


fn execute_and_open(commands: Vec<String>) -> Result<(),TopLevelError> {
    let config = config::Config::from_file()?;

    println!("Create SSH client.");
    let client = remote_exec::Client::new(config.ssh)?;
    let opener = Opener{config: config.opener};

    let mods = vec![
        Rc::new(&client as &Runner),
        Rc::new(&opener as &Runner)
    ];

    for cmd in commands.iter().skip(1) {
        for m in &mods {
            if m.has_command(&cmd) {
                println!("Executing {}... ", cmd);
                m.run(&cmd)?;
                println!("done");
            }
        }
    }

    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Please give command to execute");
        return;
    }
    env_logger::init();
    match execute_and_open(args) {
        Ok(()) => (),
        Err(TopLevelError::ConfigError(e)) =>
            eprintln!("Invalid configuration file: {:?}", e),
        Err(TopLevelError::ClientError(e)) =>
            eprintln!("SSH client error: {:?}", e),
        Err(TopLevelError::OpenError(e)) =>
            eprintln!("Error opening browser: {:?}", e),
    }
}

impl From<config::ConfigError> for TopLevelError {
    fn from(error: config::ConfigError) -> Self {
        TopLevelError::ConfigError(error)
    }
}

impl From<remote_exec::ClientError> for TopLevelError {
    fn from(error: remote_exec::ClientError) -> Self {
        TopLevelError::ClientError(error)
    }
}

impl From<opener::OpenError> for TopLevelError {
    fn from(error: opener::OpenError) -> Self {
        TopLevelError::OpenError(error)
    }
}
