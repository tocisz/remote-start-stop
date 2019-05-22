#[macro_use]
extern crate enum_map;

mod remote_exec;
mod config;

#[derive(Enum,Debug)]
pub enum Command {
    Start,
    Stop,
    Status
}

#[derive(Debug)]
pub struct CommandData {
    pub command: String,
    pub expected: String
}

#[derive(Debug)]
enum TopLevelError {
    ConfigError(config::ConfigError),
    ClientError(remote_exec::ClientError),
    OpenError(opener::OpenError)
}

fn execute_and_open() -> Result<(),TopLevelError> {
    let config = config::Config::from_file()?;
    let link = config.link.clone(); // config could have two parts to be consumed independently
    println!("Create SSH client.");
    let client = remote_exec::Client::new(config)?;
    println!("Executing {:?}... ", Command::Start);
    client.run(Command::Start)?;
    println!("done");
    print!("Opening {}", link);
    opener::open(link)?;
    Ok(())
}

fn main() {
    env_logger::init();
    match execute_and_open() {
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
