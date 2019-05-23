use std::env;

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

fn execute_and_open(commands: Vec<String>) -> Result<(),TopLevelError> {
    let config = config::Config::from_file()?;

    println!("Create SSH client.");
    let client = remote_exec::Client::new(config.ssh)?;
    for cmd in commands.iter().skip(1) {
        println!("Executing {}... ", cmd);
        client.run(&cmd)?;
        println!("done");
    }

    let cnf = config.opener;
    print!("Opening {}", cnf.link);
    opener::open(cnf.link)?;
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
