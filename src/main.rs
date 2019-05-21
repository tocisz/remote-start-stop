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

fn main() {
    env_logger::init();
    let config = config::Config::from_file().unwrap();
    let link = config.link.clone(); // config could have two parts to be consumed independently
    println!("Create SSH client.");
    let client = remote_exec::Client::new(config);
    println!("Executing {:?}... ", Command::Start);
    client.run(Command::Start);
    println!("done");
    print!("Opening {}", link);
    opener::open(link).expect("Can't open browser");
    //client.run(Command::Stop);
}