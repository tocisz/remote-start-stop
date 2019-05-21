extern crate thrussh;
extern crate thrussh_keys;
extern crate tokio;

use std::sync::Arc;
use self::thrussh::*;
use self::thrussh::client::{Connection, Config};
use self::thrussh_keys::*;
use self::tokio::prelude::*;
use self::tokio::net::TcpStream;

use super::config as config;
use Command;

#[derive(Clone)]
pub struct Client {
    key: Arc<thrussh_keys::key::KeyPair>,
    client_conf: Arc<Config>,
    config: Arc<config::Config>
}

impl client::Handler for Client {
    type Error = ();
    type FutureBool = futures::Finished<(Self, bool), Self::Error>;
    type FutureUnit = futures::Finished<Self, Self::Error>;
    type FutureSign = futures::Finished<(Self, thrussh::CryptoVec), Self::Error>;
    type SessionUnit = futures::Finished<(Self, client::Session), Self::Error>;
    fn check_server_key(self, server_public_key: &key::PublicKey) -> Self::FutureBool {
        println!("check_server_key: {:?}", server_public_key);
        futures::finished((self, true))
    }
    fn channel_open_confirmation(self, channel: ChannelId, session: client::Session) -> Self::SessionUnit {
        println!("channel_open_confirmation: {:?}", channel);
        futures::finished((self, session))
    }
    fn data(self, channel: ChannelId, ext: Option<u32>, data: &[u8], session: client::Session) -> Self::SessionUnit {
        let str = std::str::from_utf8(data);
        match str {
            Ok(s) => println!("data on channel {:?} {:?}:\n{}", ext, channel, s),
            Err(e) => println!("invalid data on channel {:?} {:?}: {:?}", ext, channel, e)
        }
        futures::finished((self, session))
    }
    fn auth_banner(self, banner: &str) -> Self::FutureUnit {
        println!("banner {}", banner);
        futures::finished(self)
    }
    fn channel_eof(self, channel: ChannelId, session: client::Session) -> Self::SessionUnit {
        println!("received EOF on {:?}", channel);
        futures::finished((self, session))
    }
}

impl Client {
    pub fn run(&self, cmd: Command) {
        self.clone().run_and_consume(cmd)
    }

    fn run_and_consume(self, cmd: Command) {
        let cmd_str;
        {
            let cmd = self.config.command[cmd].as_ref().unwrap();
            cmd_str = cmd.command.clone();
        }
        let username = self.config.username.clone();
        let key = self.key.clone();
        let host = self.config.host.clone();
        let config = self.client_conf.clone();

        let connect_future =
            client::connect_future(host, config, None, self, move |connection: Connection<TcpStream,Client>| {
                connection.authenticate_key(&username, key).and_then(|connection| {
                    connection.channel_open_session().and_then(move |(mut connection, chan)| {
                        connection.exec(
                            chan,
                            true,
                            &cmd_str,
                        );
                        connection.flush().unwrap();
                        connection.wait(move |connection| {
                            !connection.is_channel_open(chan)
                        }).and_then(|mut connection|{
                            connection.disconnect(Disconnect::ByApplication, "Ciao", "");
                            connection
                        })
                    })
                })
            }).unwrap().map_err(|_| {()});

        tokio::run(connect_future)
    }

    pub fn new(config: config::Config) -> Client {
        let client_key = thrussh_keys::decode_secret_key(&config.key, None).unwrap();
        let mut client_config = thrussh::client::Config::default();
        client_config.connection_timeout = Some(std::time::Duration::from_secs(6));
        Client {
            key: Arc::new(client_key),
            client_conf: Arc::new(client_config),
            config: Arc::new(config)
        }
    }


}