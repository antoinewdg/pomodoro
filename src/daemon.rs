use std::fmt;
use std::fs::File;
use std::io;
use std::os::unix::net::UnixListener;
use std::path::Path;

use chrono::{DateTime, Duration, Local};
use rodio::Source;
use thiserror::Error;

use crate::common::{Action, SOCKET_ADDRESS};

#[derive(Debug, Error)]
pub enum DaemonError {
    #[error("Error while binding socket {0}")]
    SocketBind(io::Error),
    #[error("Could not open response socket {0}")]
    SocketConnect(io::Error),
    #[error("Error while serializing/deserializing data: {0}")]
    Serde(#[from] bincode::Error),
    #[error("Error while reading audio file: {0}")]
    AudioFile(io::Error),
    #[error("Audio-related error: {0}")]
    Audio(Box<dyn std::error::Error>),
    #[error("Invalid action \"{0}\"")]
    InvalidAction(String),
}

pub type DaemonResult<T> = Result<T, DaemonError>;

#[derive(Debug)]
enum State {
    Empty,
    Working(DateTime<Local>),
    WorkDone(OutputStreamWrapper),
}

pub fn main() -> DaemonResult<()> {
    let mut listener = Listener::new()?;
    let mut state = State::Empty;
    let session_duration = Duration::seconds(2);
    let audio_source = _load_source()?;

    fn invalid<S: Into<String>>(msg: S) -> DaemonResult<String> {
        Err(DaemonError::InvalidAction(msg.into()))
    }

    listener.listen(|action: Action| match action {
        Action::Work => match state {
            State::Empty => {
                let target_time = Local::now() + session_duration;
                state = State::Working(target_time);
                std::thread::spawn(move || ticker(target_time));
                Ok(format!("Starting a {} session!", session_duration))
            }
            State::Working(_) => invalid("Already working !"),
            State::WorkDone(_) => invalid("No way, you need a break"),
        },
        Action::WorkDone => match state {
            State::Empty => invalid("Not working !"),
            State::WorkDone(_) => invalid("Hmmm, already doing that"),
            State::Working(_) => {
                let (stream, handle) =
                    rodio::OutputStream::try_default().map_err(|e| DaemonError::Audio(e.into()))?;
                handle
                    .play_raw(audio_source.clone().repeat_infinite())
                    .map_err(|e| DaemonError::Audio(e.into()))?;
                state = State::WorkDone(OutputStreamWrapper(stream));
                Ok("Yayy".to_owned())
            }
        },
        Action::Break => match state {
            State::WorkDone(_) => {
                state = State::Empty;
                Ok("Starting break".to_owned())
            }
            State::Empty => invalid("Already on break"),
            State::Working(_) => invalid("You're working you lazy ****"),
        },
    })?;
    Ok(())
}

fn _load_source() -> DaemonResult<impl rodio::Source<Item = f32> + Clone + Send + 'static> {
    let path = Path::new(file!())
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("sound.mp3");
    dbg!(&path);
    let file = File::open(path).map_err(DaemonError::AudioFile)?;
    Ok(rodio::Decoder::new(std::io::BufReader::new(file))
        .map_err(|e| DaemonError::Audio(e.into()))?
        .convert_samples()
        .buffered())
}

// rodio::OutputStream does not implement Debug :(
struct OutputStreamWrapper(rodio::OutputStream);

impl fmt::Debug for OutputStreamWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OutputStreamWrapper").finish()
    }
}

struct Listener {
    inner_listener: UnixListener,
}

impl Listener {
    fn new() -> DaemonResult<Self> {
        match std::fs::remove_file(SOCKET_ADDRESS) {
            Ok(_) => Ok(()),
            Err(error) => {
                if error.kind() == io::ErrorKind::NotFound {
                    Ok(())
                } else {
                    Err(DaemonError::SocketBind(error))
                }
            }
        }?;
        let inner_listener = UnixListener::bind(SOCKET_ADDRESS).map_err(DaemonError::SocketBind)?;
        Ok(Listener { inner_listener })
    }

    fn listen<H>(&mut self, mut handler: H) -> DaemonResult<()>
    where
        H: FnMut(Action) -> DaemonResult<String>,
    {
        for stream in self.inner_listener.incoming() {
            let mut stream = stream.map_err(DaemonError::SocketConnect)?;
            let msg: Action = bincode::deserialize_from(&mut stream)?;
            let response = match handler(msg) {
                Ok(msg) => Ok(Ok(msg)),
                Err(DaemonError::InvalidAction(msg)) => Ok(Err(msg)),
                Err(error) => Err(error),
            }?;
            bincode::serialize_into(&mut stream, &response)?;
        }
        Ok(())
    }
}

fn ticker(target_time: DateTime<Local>) -> () {
    use crate::client::send_to_daemon;
    loop {
        std::thread::sleep(Duration::seconds(1).to_std().unwrap());
        if Local::now() >= target_time {
            let result = send_to_daemon(&Action::WorkDone).unwrap();
            match result {
                Ok(_) => (),
                Err(message) => {
                    eprintln!("{}", message);
                }
            };
            return;
        }
    }
}
