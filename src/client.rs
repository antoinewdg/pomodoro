use std::io;
use std::os::unix::net::UnixStream;

use thiserror::Error;

use crate::common::{Action, SOCKET_ADDRESS};

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Could not connect to daemon: {0}")]
    SocketConnect(#[from] io::Error),

    #[error("Error while serializing/deserializing data: {0}")]
    Serde(#[from] bincode::Error),

    #[error("Unknown action \"{0}\"")]
    UnknownAction(String),

    #[error("{0}")]
    DaemonResponse(String),
}

pub type ClientResult<T> = Result<T, ClientError>;

pub fn send_to_daemon(message: &Action) -> ClientResult<Result<String, String>> {
    let mut stream = UnixStream::connect(SOCKET_ADDRESS)?;
    bincode::serialize_into(&mut stream, message)?;
    Ok(bincode::deserialize_from(&mut stream)?)
}

pub fn main(arguments: &[String]) -> ClientResult<()> {
    let action = match arguments.get(0).map(String::as_str) {
        Some("work") => Ok(Action::Work),
        Some("break") => Ok(Action::Break),
        Some("stop") => Ok(Action::Stop),
        None => Ok(Action::GetState),
        Some(name) => Err(name.to_owned()),
    }
    .map_err(ClientError::UnknownAction)?;

    let response = send_to_daemon(&action)?.map_err(ClientError::DaemonResponse)?;
    println!("{}", response);
    Ok(())
}
