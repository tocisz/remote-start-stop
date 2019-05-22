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
    match config::Config::from_file() {
        Ok(config) => {
            let link = config.link.clone(); // config could have two parts to be consumed independently
            println!("Create SSH client.");
            match remote_exec::Client::new(config) {
                Ok(client) => {
                    println!("Executing {:?}... ", Command::Start);
                    if let Err(err) = client.run(Command::Start){
                        eprintln!("{:?}", err);
                    } else {
                        println!("done");
                    }
                    //print!("Opening {}", link);
                    //opener::open(link).expect("Can't open browser");
                    //client.run(Command::Stop);
                },
                Err(e) => {
                    eprintln!("Can't create SSH client: {:?}", e)
                }
            }
        }
        Err(e) => {
            eprintln!("Invalid configuration file: {:?}", e)
        }
    }
}