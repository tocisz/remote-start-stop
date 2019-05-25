extern crate thrussh;
extern crate thrussh_keys;
extern crate tokio;

use std::sync::Arc;
use self::thrussh::*;
use self::thrussh::client::{Connection, Config};
use self::thrussh_keys::key;
use self::tokio::prelude::*;
use self::tokio::net::TcpStream;
use self::tokio::runtime::Runtime;

use super::config as config;
use std::result::Result;
use TopLevelError;

#[derive(Clone)]
pub struct Client {
    key: Arc<thrussh_keys::key::KeyPair>,
    client_conf: Arc<Config>,
    config: Arc<config::SshConfig>,
}

impl client::Handler for Client {
    type Error = ();
    type FutureBool = futures::Finished<(Self, bool), Self::Error>;
    type FutureUnit = futures::Finished<Self, Self::Error>;
    type FutureSign = futures::Finished<(Self, thrussh::CryptoVec), Self::Error>;
    type SessionUnit = futures::Finished<(Self, client::Session), Self::Error>;
    fn auth_banner(self, banner: &str) -> Self::FutureUnit {
        debug!("banner {}", banner);
        futures::finished(self)
    }
    fn check_server_key(self, server_public_key: &key::PublicKey) -> Self::FutureBool {
        debug!("check_server_key: {:?}", server_public_key);
        futures::finished((self, true))
    }
    fn channel_open_confirmation(self, channel: ChannelId, session: client::Session) -> Self::SessionUnit {
        debug!("channel_open_confirmation: {:?}", channel);
        futures::finished((self, session))
    }
    fn channel_eof(self, channel: ChannelId, session: client::Session) -> Self::SessionUnit {
        debug!("received EOF on {:?}", channel);
        futures::finished((self, session))
    }
    fn data(self, channel: ChannelId, ext: Option<u32>, data: &[u8], session: client::Session) -> Self::SessionUnit {
        let str = std::str::from_utf8(data);
        match str {
            Ok(s) => debug!("data on channel {:?} {:?}:\n{}", ext, channel, s),
            Err(e) => warn!("invalid data on channel {:?} {:?}: {:?}", ext, channel, e)
        }
        futures::finished((self, session))
    }
}

impl super::Runner for Client {
    fn has_command(&self, cmd: &String) -> bool {
        self.config.commands.contains_key(cmd)
    }

    fn run(&self, cmd: &String) -> Result<(), TopLevelError> {
        self.clone().run_and_consume(cmd)
            .map_err(|e| TopLevelError::ClientError(e))
    }
}

impl Client {
    fn run_and_consume(self, cmd: &String) -> Result<(),ClientError> {
        let cmd_str;
        {
            let cmd = &self.config.commands[cmd];
            cmd_str = cmd.command.clone();
        }
        let username = self.config.username.clone();
        let key = self.key.clone();
        let host = self.config.host.clone();
        let config = self.client_conf.clone();

        let connect_future =
            client::connect_future(host, config, None, self, move |connection: Connection<TcpStream, Client>| {
                connection.authenticate_key(&username, key) // what if authentication fails?
                    .and_then(|connection| {
                        // we could check connection.is_authenticated()
                        connection.channel_open_session().and_then(move |(mut connection, chan)| {
                            // we could check connection.is_channel_open()
                            connection.exec(
                                chan,
                                true,
                                &cmd_str,
                            );
                            connection.flush().unwrap(); // How to handle?
                            connection.wait(move |connection| {
                                !connection.is_channel_open(chan)
                            }).and_then(|mut connection| {
                                connection.disconnect(Disconnect::ByApplication, "Ciao", "");
                                connection
                            })
                        })
                    })
            })?;

        let mut runtime : Runtime = Runtime::new().unwrap();
        if let Err(e) = runtime.block_on(connect_future) {
            return Err(ClientError::ConnectionProblem(format!("{:?}", e)));
        }
        Ok(())
    }

    pub fn new(config: config::SshConfig) -> Result<Client, ClientError> {
        let client_key = thrussh_keys::decode_secret_key(&config.key, None)?;
        let mut client_config = thrussh::client::Config::default();
        client_config.connection_timeout = Some(std::time::Duration::from_secs(6));
        Ok(Client {
            key: Arc::new(client_key),
            client_conf: Arc::new(client_config),
            config: Arc::new(config),
        })
    }
}

#[derive(Debug)]
pub enum ClientError {
    DecodeKey(thrussh_keys::Error),
    ConnectionProblem(String),
    FutureProblem(thrussh::Error)
}

impl From<thrussh_keys::Error> for ClientError {
    fn from(error: thrussh_keys::Error) -> Self {
        ClientError::DecodeKey(error)
    }
}

impl From<thrussh::Error> for ClientError {
    fn from(error: thrussh::Error) -> Self {
        ClientError::FutureProblem(error)
    }
}
