extern crate yaml_rust;
use self::yaml_rust as yaml;
use self::yaml_rust::{Yaml, YamlLoader};
use std::fs;
use std::io;
use enum_map::EnumMap;
use super::Command;
use super::CommandData;

#[derive(Debug)]
pub struct Config {
    pub username: String,
    pub key: String,
    pub host: String,
    pub link: String,
    pub command: EnumMap<Command, Option<CommandData>>
}

#[derive(Debug)]
pub enum ConfigError {
    IoError(io::Error),
    YamlError(yaml_rust::ScanError),
    Missing(&'static str)
}

impl From<io::Error> for ConfigError {
    fn from(error: io::Error) -> Self {
        ConfigError::IoError(error)
    }
}

impl From<yaml_rust::ScanError> for ConfigError {
    fn from(error: yaml_rust::ScanError) -> Self {
        ConfigError::YamlError(error)
    }
}

fn read_enum(commands: &mut EnumMap<Command,Option<CommandData>>,
             yaml: &yaml::Yaml,
             com: Command,
             key: &str,
             e1: &'static str,
             e2: &'static str) -> Result<(),ConfigError>
{
    if let Some(start) = yaml[key].as_hash() {
        let c;
        if let Some(cc) = start[&Yaml::String(String::from("command"))].as_str() {
            c  = cc;
        } else {
            return Err(ConfigError::Missing(e1));
        }
        let ex;
        if let Some(e) = start[&Yaml::String(String::from("expected"))].as_str() {
            ex  = e;
        } else {
            return Err(ConfigError::Missing(e2));
        }
        commands[com] = Some(CommandData{
            command: String::from(c),
            expected: String::from(ex)
        });
    }

    Ok(())
}

impl Config {
    pub fn from_file() -> Result<Config,ConfigError> {
        let contents = fs::read_to_string("config.yml")?;
        let yaml = YamlLoader::load_from_str(&contents)?;
        let yaml: &yaml::Yaml = yaml.get(0).unwrap();

        let username;
        if let Some(h) = yaml["username"].as_str() {
            username = String::from(h);
        } else {
            return Err(ConfigError::Missing("username"));
        }

        let host;
        if let Some(h) = yaml["host"].as_str() {
            host = String::from(h);
        } else {
            return Err(ConfigError::Missing("host"));
        }

        let key;
        if let Some(k) = yaml["key"].as_str() {
            key = String::from(k);
        } else {
            return Err(ConfigError::Missing("key"));
        }

        let link;
        if let Some(k) = yaml["link"].as_str() {
            link = String::from(k);
        } else {
            return Err(ConfigError::Missing("link"));
        }

        let mut command: EnumMap<Command, Option<CommandData>> = EnumMap::new();
        read_enum(&mut command, yaml, Command::Start, "start", "start.command", "start.expected")?;
        read_enum(&mut command, yaml, Command::Stop, "stop", "stop.command", "stop.expected")?;
        read_enum(&mut command, yaml, Command::Status, "status", "status.command", "status.expected")?;

        Ok(Config {username, key, host, link, command})
    }

}