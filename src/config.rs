extern crate yaml_rust;

use self::yaml_rust as yaml;
use self::yaml_rust::YamlLoader;
use std::fs;
use std::io;
use std::collections::HashMap;
use super::CommandData;

#[derive(Debug)]
pub struct Config {
    pub username: String,
    pub key: String,
    pub host: String,
    pub link: String,
    pub commands: HashMap<String, CommandData>,
}

#[derive(Debug)]
pub enum ConfigError {
    IoError(io::Error),
    YamlError(yaml_rust::ScanError),
    YamlContents(String),
    Missing(String),
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

fn read_string(yaml: &yaml::Yaml, key: &str) -> Result<String, ConfigError>
{
    read_string_p(yaml, key, &None)
}

fn read_string_p(yaml: &yaml::Yaml, key: &str,
               prefix: &Option<String>) -> Result<String, ConfigError>
{
    let s;
    if let Some(r) = yaml[key].as_str() {
        s = String::from(r);
    } else {
        return Err(ConfigError::Missing(maybe_concat_with_dot(prefix, key)));
    }

    Ok(s)
}

fn maybe_concat_with_dot(a: &Option<String>, b: &str) -> String {
    match a {
        None => String::from(b),
        Some(a) => concat_with_dot(a, b)
    }
}

fn concat_with_dot(a: &str, b: &str) -> String {
    let mut name = String::from(a);
    name.push('.');
    name.push_str(b);
    name
}

fn read_commands(yaml: &yaml::Yaml) -> Result<HashMap<String,CommandData>, ConfigError>
{
    let prefix = "commands";
    if let Some(yaml_map) = yaml.as_hash() {
        let mut map = HashMap::new();
        for (key, value) in yaml_map {
            if let Some(key) = key.as_str() {
                let prefix = Some(concat_with_dot(prefix,key));
                let command = read_string_p(value, "command", &prefix)?;
                let expected = read_string_p(value, "expected", &prefix).ok();
                map.insert(String::from(key),CommandData {command, expected});
            } else {
                return Err(ConfigError::YamlContents(String::from("Command keys must be strings")))
            }
        }
        Ok(map)
    } else {
        Err(ConfigError::YamlContents(String::from("Invalid commands map")))
    }
}

impl Config {
    pub fn from_file() -> Result<Config, ConfigError> {
        let contents = fs::read_to_string("config.yml")?;
        let yaml = YamlLoader::load_from_str(&contents)?;
        let yaml = yaml.get(0).unwrap();

        let username = read_string(yaml, "username")?;
        let host = read_string(yaml, "host")?;
        let key = read_string(yaml, "key")?;
        let link = read_string(yaml, "link")?;
        let commands = read_commands(&yaml["commands"])?;
        Ok(Config { username, key, host, link, commands })
    }
}