use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Action {
    Work,
    WorkDone,
    Break,
    Stop,
    GetState,
}

pub const SOCKET_ADDRESS: &'static str = "/tmp/pomodoro.sock";
