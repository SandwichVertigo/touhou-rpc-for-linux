// touhou-rpc-for-linux — Native Linux Discord Rich Presence for the Touhou series.
// Copyright (C) 2026 SandwichVertigo
//
// This program is a derivative work of TouhouRPC by TheBakaRem (GPL-3.0-or-later).
// See NOTICE for details on what was derived and what is original.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! Minimal Discord IPC client. Length-prefixed frames over a Unix socket.

use std::env;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, bail, Context, Result};
use serde::Serialize;
use serde_json::json;

const OP_HANDSHAKE: u32 = 0;
const OP_FRAME: u32 = 1;

pub struct DiscordIpc {
    stream: UnixStream,
    client_id: String,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct Activity {
    #[serde(skip_serializing_if = "Option::is_none")] pub details: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")] pub state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")] pub assets: Option<Assets>,
    #[serde(skip_serializing_if = "Option::is_none")] pub timestamps: Option<Timestamps>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct Assets {
    #[serde(skip_serializing_if = "Option::is_none")] pub large_image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")] pub large_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")] pub small_image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")] pub small_text: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Default)]
pub struct Timestamps {
    #[serde(skip_serializing_if = "Option::is_none")] pub start: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")] pub end: Option<u64>,
}

impl DiscordIpc {
    pub fn connect(client_id: &str) -> Result<Self> {
        let path = find_socket().ok_or_else(|| anyhow!("no discord-ipc-* socket (is Discord running?)"))?;
        let stream = UnixStream::connect(&path).with_context(|| format!("connect {}", path.display()))?;
        let mut ipc = DiscordIpc { stream, client_id: client_id.to_string() };
        ipc.handshake()?;
        Ok(ipc)
    }

    fn handshake(&mut self) -> Result<()> {
        let p = json!({ "v": 1, "client_id": self.client_id });
        self.send_frame(OP_HANDSHAKE, &p.to_string())?;
        let (op, body) = self.recv_frame()?;
        if op == 2 { bail!("Discord rejected handshake: {}", body); }
        Ok(())
    }

    pub fn set_activity(&mut self, activity: &Activity) -> Result<()> {
        let nonce = next_nonce();
        let p = json!({
            "cmd": "SET_ACTIVITY", "nonce": nonce,
            "args": { "pid": std::process::id(), "activity": activity }
        });
        self.send_frame(OP_FRAME, &p.to_string())?;
        let _ = self.recv_frame()?;
        Ok(())
    }

    pub fn clear_activity(&mut self) -> Result<()> {
        let nonce = next_nonce();
        let p = json!({
            "cmd": "SET_ACTIVITY", "nonce": nonce,
            "args": { "pid": std::process::id() }
        });
        self.send_frame(OP_FRAME, &p.to_string())?;
        let _ = self.recv_frame()?;
        Ok(())
    }

    fn send_frame(&mut self, op: u32, payload: &str) -> Result<()> {
        let bytes = payload.as_bytes();
        let mut header = [0u8; 8];
        header[..4].copy_from_slice(&op.to_le_bytes());
        header[4..].copy_from_slice(&(bytes.len() as u32).to_le_bytes());
        self.stream.write_all(&header)?;
        self.stream.write_all(bytes)?;
        self.stream.flush()?;
        Ok(())
    }

    fn recv_frame(&mut self) -> Result<(u32, String)> {
        let mut header = [0u8; 8];
        self.stream.read_exact(&mut header)?;
        let op = u32::from_le_bytes(header[..4].try_into().unwrap());
        let len = u32::from_le_bytes(header[4..].try_into().unwrap()) as usize;
        let mut buf = vec![0u8; len];
        self.stream.read_exact(&mut buf)?;
        Ok((op, String::from_utf8_lossy(&buf).into_owned()))
    }
}

fn find_socket() -> Option<PathBuf> {
    let rd = env::var("XDG_RUNTIME_DIR").ok();
    let mut bases: Vec<PathBuf> = Vec::new();
    if let Some(r) = rd.as_ref() {
        bases.push(PathBuf::from(r));
        bases.push(PathBuf::from(r).join("app/com.discordapp.Discord"));
        bases.push(PathBuf::from(r).join("app/com.discordapp.DiscordCanary"));
        bases.push(PathBuf::from(r).join("snap.discord"));
        bases.push(PathBuf::from(r).join("snap.discord-canary"));
    }
    bases.push(PathBuf::from("/tmp"));
    for base in &bases {
        for n in 0..10 {
            let c = base.join(format!("discord-ipc-{}", n));
            if c.exists() { return Some(c); }
        }
    }
    None
}

pub fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn next_nonce() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{}-{}-{}", std::process::id(), now_secs(), n)
}
