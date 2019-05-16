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
    println!("{:?}", config);
    let link = config.link.clone(); // config could have two parts to be consumed independently
    remote_exec::execute(config, Command::Start);
    println!("{}", "done");
    print!("{}", link);
    opener::open(link).expect("Can't open browser");
}