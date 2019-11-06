#[macro_use]
extern crate log;

mod error;
mod jwt;
mod encrypt;
mod establish_connection;
mod subscriber_list;

use log::SetLoggerError;
use simplelog::{CombinedLogger, Config, LevelFilter, SharedLogger, SimpleLogger, TermLogger, WriteLogger, TerminalMode};
use structopt::*;

use error::Error;

use std::{
    collections::hash_map::{Entry, HashMap},
    sync::Arc,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    io::BufReader,
};

use tokio::sync::mpsc;
use tokio::net::TcpListener;

#[derive(Debug)]
enum Void {}

#[derive(StructOpt, Debug)]
#[structopt(name = "OHX", about = "High performance asynchronous connection message broker")]
pub struct CommandLine {
    /// The connection port
    #[structopt(short, long, default_value = "8080")]
    pub port: u16,
    /// The timeout in seconds is used for: session establishing and sending to a socket
    #[structopt(short, long, default_value = "5")]
    pub timeout_secs: u32,
    /// The maximum number of clients. Further client connections will be refused.
    #[structopt(short, long, default_value = "1000000")]
    pub max_clients: u32,
    /// The log level
    #[structopt(short, long, default_value = "info")]
    pub level: String,
    /// Optional log file
    #[structopt(short, long)]
    pub file: Option<String>,
    /// Public key url to download a JWK set. Used for token verification.
    #[structopt(short, long, default_value = "https://oauth.openhabx.com/.well-known/jwks.json")]
    pub jwk_url: String,
    /// An auth service url with /token and /authorize endpoints to get a jwt access token.
    /// That token is expected to have a private claim "cloud_key".
    #[structopt(short, long, default_value = "https://oauth.openhabx.com")]
    pub oauth_token_url: String,
}

fn setup_loggers(conf: &CommandLine) -> Result<(), SetLoggerError> {
    let log_level = match &conf.level[..] {
        "error" => LevelFilter::Error,
        "warn" => LevelFilter::Warn,
        "info" => LevelFilter::Info,
        "debug" => LevelFilter::Debug,
        "trace" => LevelFilter::Trace,
        _ => LevelFilter::Info,
    };

    // The terminal logger will not work if in daemon mode
    let mut loggers: Vec<Box<dyn SharedLogger>> = vec![];
    match TermLogger::new(log_level, Config::default(), TerminalMode::Mixed) {
        Some(logger) => {
            loggers.push(logger);
        }
        None => loggers.push(SimpleLogger::new(log_level, Config::default())),
    }
    if let Some(filename) = &conf.file {
        match File::create(filename) {
            Err(e) => {
                warn!("Log file {} could not be used: {}", filename, e);
            }
            Ok(file) => loggers.push(WriteLogger::new(log_level, Config::default(), file)),
        }
    }
    CombinedLogger::init(loggers)
}

/// Start to accept connections. Maximum of 10 unprocessed connection attempts at the same time.
/// From there on new connections are only accepted when other new ones have been processed.
#[tokio::main]
async fn main() -> Result<(), Error> {
    let cl: CommandLine = CommandLine::from_args();
    setup_loggers(&cl)?;

    let listener = TcpListener::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), cl.port)).await?;

    let (mut broker_sender, broker_receiver) = tokio::sync::mpsc::channel(100);
    let broker = tokio::spawn(init_connections(broker_receiver, cl.jwk_url));
    let mut incoming = listener.incoming();
    while let Some(stream) = incoming.next().await {
        let stream = stream?;
        // We want to write a packet buffer content first and then call flush() when it's time.
        stream.set_nodelay(false)?;
        broker_sender.send(stream).await;
    }
    drop(broker_sender);

    Ok(())
}
