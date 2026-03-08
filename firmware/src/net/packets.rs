use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum SendPacket {
    HelloClient,
    DataRead(i32),
    Ping,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum RecvPacket {
    WelcomeServer,
    CommandToggleLed(bool),
    Pong,
}
