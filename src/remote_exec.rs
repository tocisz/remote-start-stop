extern crate ssh2;

use self::ssh2::Session;
use super::config as config;
use TopLevelError;
use std::result::Result;
use std::net::TcpStream;
use std::path::Path;
use std::io::Read;
use remote_exec::ClientError::{ConnectionProblem, SshProblem};

pub struct Client {
    config: Box<config::SshConfig>,
    #[allow(dead_code)]
    tcp_stream: Box<TcpStream>, // tcp connections needs to live as long and session
    session: Box<Session>
}

impl super::Runner for Client {
    fn has_command(&self, cmd: &String) -> bool {
        self.config.commands.contains_key(cmd)
    }

    fn run(&self, cmd: &String) -> Result<(), TopLevelError> {
        self.run_internal(cmd).map_err(|e| TopLevelError::ClientError(e))
    }
}

impl Client {

    pub fn new(config: config::SshConfig) -> Result<Client, ClientError> {
        let tcp = TcpStream::connect("odroid:22")?;
        let mut sess = Session::new().ok_or(ClientError::CreateSession)?;
        sess.handshake(&tcp)?;

        {
            let key = config.key.clone();
            let key_path = Path::new(&key);
            // Try to authenticate with the first identity in the agent.
            sess.userauth_pubkey_file(&config.username, None, key_path, None)?;
        }

        // Make sure we succeeded
        if sess.authenticated() {
            Ok(Client {
                config: Box::new(config),
                tcp_stream: Box::new(tcp),
                session: Box::new(sess)
            })
        } else {
            Err(ClientError::NotAuthenticated)
        }
    }

    fn run_internal(&self, cmd: &String) -> Result<(), ClientError> {
        let cmd = &self.config.commands[cmd].command;

        let mut channel = self.session.channel_session()?;
        channel.exec(cmd)?;
        let mut output_string = String::new();
        channel.read_to_string(&mut output_string)?;
        println!("Got:\n{}", output_string);
        // TODO check output_string for expected string
        channel.wait_close()?;
        println!("Exit status: {}", channel.exit_status()?);

        Ok(())
    }
}

#[derive(Debug)]
pub enum ClientError {
    ConnectionProblem(std::io::Error),
    CreateSession,
    NotAuthenticated,
    SshProblem(ssh2::Error)
}

impl From<std::io::Error> for ClientError {
    fn from(e: std::io::Error) -> Self {
        ConnectionProblem(e)
    }
}

impl From<ssh2::Error> for ClientError {
    fn from(e: ssh2::Error) -> Self {
        SshProblem(e)
    }
}