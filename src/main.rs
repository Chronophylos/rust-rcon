use anyhow::{Context, Result};
use clap::{App, Arg};
use log::{debug, info};
use std::{
    borrow::Cow,
    io::{self, BufRead},
};
use tungstenite::{connect, Message};

#[derive(Debug, Clone)]
struct Package<'a> {
    identifier: i32,
    message: Cow<'a, str>,
    name: Cow<'a, str>,
}

impl<'a> Package<'a> {
    pub fn new_command<C>(command: C) -> Self
    where
        C: Into<Cow<'a, str>>,
    {
        Self {
            identifier: -1,
            message: command.into(),
            name: Cow::from("WebRcon"),
        }
    }
}

impl Package<'_> {
    pub fn to_text(&self) -> String {
        format!(
            r#"{{Identifier:"{}",Message:"{}",Name:{}}}"#,
            self.identifier,
            self.message.escape_debug(),
            self.name.escape_debug()
        )
    }

    pub fn to_message(&self) -> Message {
        Message::Text(self.to_text())
    }
}

fn send_packages(url: &str, packages: Vec<Package>) -> Result<()> {
    let (mut socket, response) = connect(url).context("Could not connect to RCON")?;

    info!("Connected to RCON");
    debug!("Response HTTP code: {}", response.status());
    debug!("Response Headers: {:#?}", response.headers());

    for package in packages {
        info!("Sending: {:?}", &package);

        socket
            .write_message(package.to_message())
            .context("Could not send message to RCON")?;
    }

    socket.close(None).context("Could not close socket")?;

    Ok(())
}

fn run(server: &str, port: u16, password: &str, packages: Vec<Package>) -> Result<()> {
    send_packages(&format!("ws://{}:{}/{}", server, port, password), packages)
}

fn main() -> Result<()> {
    env_logger::init();

    let matches = App::new("Rust RCON Tool")
        .about("written in Rust")
        .after_help(
            "Each command need to be in hyphens to differenciante between them.

Example: myrustserver.com s3cur3 \"say Setting time to 0900\" \"env.time 9\"",
        )
        .arg(
            Arg::with_name("port")
                .help("RCON Port")
                .short("-p")
                .long("--port")
                .default_value("28016"),
        )
        .arg(
            Arg::with_name("server")
                .help("Rust Server name or IP")
                .required(true),
        )
        .arg(
            Arg::with_name("password")
                .help("RCON Password")
                .required(true),
        )
        .arg(
            Arg::with_name("commands")
                .help("Commands to execute on server. Pass '-' to read from STDIN")
                .multiple(true)
                .required(true),
        )
        .get_matches();

    let mut packages = Vec::new();

    for command in matches
        .values_of("commands")
        .context("Missing argument 'commands'")?
    {
        if command != "-" {
            packages.push(Package::new_command(command));
            continue;
        }

        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            packages.push(Package::new_command(
                line.context("Could not read line from STDIN")?,
            ));
        }
    }

    run(
        matches
            .value_of("server")
            .context("Missing argument 'server'")?,
        matches
            .value_of("port")
            .context("Missing argument 'port'")?
            .parse()
            .context("Could not parse port")?,
        matches
            .value_of("password")
            .context("Missing argument 'password'")?,
        packages,
    )?;

    Ok(())
}
