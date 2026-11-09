//! Discord Rich Presence as a plugin. Hestia knows nothing about Discord: the
//! plugin opens Discord's local socket through `ipc:connect`, speaks its RPC
//! framing (a little-endian opcode and length, then JSON), and decides what to
//! show from the running instances whenever a process event arrives.
//!
//! Every call runs on a fresh instance, so what must last lives elsewhere: the
//! connection is kept by the host (found again with `ipc::connections`), and
//! the last card sent is kept in the data directory, so an unchanged card is
//! not sent again.

use std::time::Duration;

use hestia::ipc::{self, Connection};
use hestia::proto::instance::{InstanceInfo, InstanceList};
use hestia::proto::process::ProcessState;
use hestia::proto::Empty;
use hestia::{json, Error, Plugin, Value};

const HANDSHAKE: u32 = 0;
const FRAME: u32 = 1;
const ENDPOINTS: [&str; 3] = ["", "app/com.discordapp.Discord/", "snap.discord/"];
const SENT: &str = "sent";

struct Presence;

impl Plugin for Presence {
    fn on_event(_topic: &str, _payload: Value) {
        if let Err(error) = refresh() {
            hestia::log::debug(&format!("presence not updated: {error}"));
        }
    }

    fn on_settings_changed(_key: &str) {
        for connection in ipc::connections().unwrap_or_default() {
            let _ = connection.close();
        }
        let _ = hestia::data::save(SENT, &Value::Null);
        let _ = refresh();
    }
}

/// What is being played: the newest running session of any instance.
fn playing() -> Result<Option<(InstanceInfo, u32, i64)>, Error> {
    let instances = hestia::call::<InstanceList>(&Empty {})?;
    Ok(instances
        .into_iter()
        .filter_map(|instance| {
            let session = instance
                .sessions
                .iter()
                .filter(|session| session.state == ProcessState::Running)
                .max_by_key(|session| session.started_unix)?;
            let (pid, started) = (session.pid, session.started_unix);
            Some((instance, pid, started))
        })
        .max_by_key(|(_, _, started)| *started))
}

fn refresh() -> Result<(), Error> {
    let Some(client_id) = hestia::settings::get::<String>("client-id")?.filter(|id| !id.is_empty())
    else {
        return Ok(());
    };
    let card = match playing()? {
        Some((instance, pid, started)) => json!({
            "pid": pid,
            "activity": {
                "details": format!("Playing {}", instance.name),
                "state": format!("{} {}", instance.flavor, instance.game_version),
                "timestamps": { "start": started },
            },
        }),
        None => json!({ "pid": 0, "activity": null }),
    };
    if hestia::data::load::<Value>(SENT)?.as_ref() == Some(&card) {
        return Ok(());
    }
    let discord = connect(&client_id)?;
    let command = json!({ "cmd": "SET_ACTIVITY", "args": card, "nonce": nonce() });
    if let Err(error) = send(&discord, FRAME, &command) {
        let _ = discord.close();
        return Err(error);
    }
    drain(&discord);
    hestia::data::save(SENT, &card)
}

/// The open connection, or a new one with the handshake done.
fn connect(client_id: &str) -> Result<Connection, Error> {
    if let Some(open) = ipc::connections()?.pop() {
        return Ok(open);
    }
    let _ = hestia::data::save(SENT, &Value::Null);
    for prefix in ENDPOINTS {
        for n in 0..10 {
            let Ok(discord) = ipc::open(&format!("{prefix}discord-ipc-{n}")) else {
                continue;
            };
            send(&discord, HANDSHAKE, &json!({ "v": 1, "client_id": client_id }))?;
            drain(&discord);
            return Ok(discord);
        }
    }
    Err(Error::NotFound("Discord is not running".into()))
}

fn send(discord: &Connection, opcode: u32, body: &Value) -> Result<(), Error> {
    let body = body.to_string();
    let mut frame = Vec::with_capacity(8 + body.len());
    frame.extend_from_slice(&opcode.to_le_bytes());
    frame.extend_from_slice(&(body.len() as u32).to_le_bytes());
    frame.extend_from_slice(body.as_bytes());
    discord.send(&frame)
}

/// Reads Discord's replies so they never pile up; their content is not needed.
fn drain(discord: &Connection) {
    let mut wait = Duration::from_millis(500);
    while matches!(discord.receive(64 * 1024, wait), Ok(bytes) if !bytes.is_empty()) {
        wait = Duration::from_millis(20);
    }
}

fn nonce() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", now.as_nanos())
}

hestia::plugin!(Presence);
