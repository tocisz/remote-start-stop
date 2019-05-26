#[macro_use] extern crate log;
extern crate env_logger;

use std::env;
use config::OpenerConfig;

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
    OpenError(opener::OpenError),
    UnsupportedCommands(Vec<String>)
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

fn check_commands(commands: &Vec<String>, mods: &Vec<&Runner>) -> Result<(),TopLevelError> {
    let mut unsupported = Vec::new();
    for cmd in commands.iter().skip(1) {
        let mut executed = false;
        for m in mods {
            if m.has_command(&cmd) {
                executed = true;
            }
        }
        if !executed {
            unsupported.push((*cmd).clone());
        }
    }
    if !unsupported.is_empty() {
        Err(TopLevelError::UnsupportedCommands(unsupported))
    } else {
        Ok(())
    }
}

fn execute_commands(commands: &Vec<String>, mods: &Vec<&Runner>) -> Result<(),TopLevelError> {
    for cmd in commands.iter().skip(1) {
        for m in mods {
            if m.has_command(&cmd) {
                info!("Executing {}... ", cmd);
                m.run(&cmd)?;
                info!("done");
            }
        }
    }
    Ok(())
}

fn execute_and_open(commands: Vec<String>) -> Result<(),TopLevelError> {
    let config = config::Config::from_file()?;

    info!("Creating SSH client.");
    let client = remote_exec::Client::new(config.ssh)?;
    info!("Creating Opener.");
    let opener = Opener{config: config.opener};

    let mods: Vec<&Runner> = vec![&client, &opener];

    check_commands(&commands, &mods)?;
    execute_commands(&commands, &mods)?;

    Ok(())
}

fn main() {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        error!("Please give command to execute");
        return;
    }

    match execute_and_open(args) {
        Ok(()) => (),
        Err(TopLevelError::ConfigError(e)) =>
            error!("Invalid configuration file: {:?}", e),
        Err(TopLevelError::ClientError(e)) =>
            error!("SSH client error: {:?}", e),
        Err(TopLevelError::OpenError(e)) =>
            error!("Error opening browser: {:?}", e),
        Err(TopLevelError::UnsupportedCommands(list)) => {
            for cmd in &list {
                error!("Command {} is not supported!", cmd);
            }
        }
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
