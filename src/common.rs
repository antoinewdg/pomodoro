use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Action {
    Work,
    WorkDone,
    Break,
}

pub const SOCKET_ADDRESS: &'static str = "/tmp/pomodoro.sock";
