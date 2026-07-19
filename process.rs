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

//! Scan `/proc` for any known Touhou process.

use std::fs;
use std::path::Path;

use crate::common::GameId;

pub struct Detected {
    pub pid: i32,
    pub game: GameId,
}

pub fn find_any() -> Option<Detected> {
    let entries = fs::read_dir("/proc").ok()?;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = match name.to_str() { Some(s) => s, None => continue };
        let pid: i32 = match name.parse() { Ok(n) => n, Err(_) => continue };
        if let Some(game) = classify_pid(pid) {
            return Some(Detected { pid, game });
        }
    }
    None
}

fn classify_pid(pid: i32) -> Option<GameId> {
    // /proc/<pid>/comm — truncated to 15 bytes, but "th06.exe" fits.
    if let Ok(comm) = fs::read_to_string(format!("/proc/{}/comm", pid)) {
        let comm = comm.trim();
        if let Some(g) = match_name(comm) { return Some(g); }
    }
    // /proc/<pid>/cmdline — full argv[0], NUL-separated.
    if let Ok(bytes) = fs::read(format!("/proc/{}/cmdline", pid)) {
        if let Some(first) = bytes.split(|&b| b == 0).next() {
            if let Ok(s) = std::str::from_utf8(first) {
                let base = Path::new(s).file_name().and_then(|o| o.to_str()).unwrap_or(s);
                if let Some(g) = match_name(base) { return Some(g); }
            }
        }
    }
    None
}

fn match_name(candidate: &str) -> Option<GameId> {
    for g in GameId::all() {
        for name in g.exe_names() {
            if candidate.eq_ignore_ascii_case(name) {
                return Some(*g);
            }
        }
    }
    None
}
