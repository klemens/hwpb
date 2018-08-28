use crate::errors::*;
use hyper_sse::Server;
use rocket::Config;
use std::net::SocketAddr;

lazy_static! {
    pub static ref SERVER: Server<i16> = Server::new();
}

pub struct Url(pub String);

pub fn parameters(config: &Config) -> Result<(Url, SocketAddr)> {
    let port = config.get_int("push_port")
        .chain_err(|| "push_port not specified")? as u16;
    let endpoint = if config.environment.is_dev() {
        format!("//{}:{}/push", config.address, port)
    } else {
        "/push".into()
    };

    Ok((Url(endpoint), ([127,0,0,1], port).into()))
}

#[derive(Serialize)]
pub struct Comment<'a> {
    pub group: i32,
    pub author: &'a str,
    pub comment: &'a str,
}

#[derive(Serialize)]
pub struct Completion {
    pub group: i32,
    pub task: i32,
    pub completed: bool,
}

#[derive(Serialize)]
pub struct Elaboration {
    pub group: i32,
    pub experiment: i32,
    pub handed_in: bool,
    pub rework: bool,
    pub accepted: bool,
}

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum Group {
    New { day: i32 },
    Change { group: i32 },
}

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum Student {
    Add { group: i32, student: i32, name: String },
    Remove { student: i32 },
    Instructed { student: i32, instructed: bool },
}
