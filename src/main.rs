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

//! touhou-rpc — Native Linux Discord Rich Presence for the Touhou series.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::Result;

mod common;
mod discord;
mod games;
mod memory;
mod process;

use common::{format_score, GameId, Phase, Snapshot, StageState, TouhouGame};
use discord::{Activity, Assets, DiscordIpc, Timestamps};
use memory::MemoryReader;

const POLL_INTERVAL: Duration = Duration::from_secs(5);
const WAIT_INTERVAL: Duration = Duration::from_secs(3);

fn main() -> Result<()> {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || r.store(false, Ordering::SeqCst))?;

    log("touhou-rpc started. Waiting for a Touhou process under Wine/Proton…");

    let mut current_pid: Option<i32> = None;
    let mut current_game_id: Option<GameId> = None;
    let mut current_game: Option<Box<dyn TouhouGame>> = None;
    let mut reader: Option<MemoryReader> = None;
    let mut ipc: Option<DiscordIpc> = None;
    let mut session_start: Option<u64> = None;
    let mut last_phase: Option<Phase> = None;

    while running.load(Ordering::SeqCst) {
        // If we have a pid, make sure it's still alive AND still the same game
        // (guards against pid recycling).
        if let Some(pid) = current_pid {
            if !still_running_as(pid, current_game_id) {
                log(&format!("Game process {} ended.", pid));
                current_pid = None;
                current_game_id = None;
                current_game = None;
                reader = None;
                session_start = None;
                last_phase = None;
                if let Some(ipc) = ipc.as_mut() { let _ = ipc.clear_activity(); }
                ipc = None; // must reconnect with new client_id anyway
            }
        }

        // Find a game if we don't have one.
        if current_pid.is_none() {
            match process::find_any() {
                Some(det) => {
                    log(&format!("Detected {} at pid {}.", det.game.short_name(), det.pid));
                    current_pid = Some(det.pid);
                    current_game_id = Some(det.game);
                    current_game = Some(games::make(det.game));
                    reader = Some(MemoryReader::new(det.pid));
                    thread::sleep(Duration::from_millis(500)); // let the game finish loading
                }
                None => {
                    sleep_interruptible(WAIT_INTERVAL, &running);
                    continue;
                }
            }
        }

        // Read game state.
        let snap = match current_game.as_mut().unwrap().read(reader.as_mut().unwrap()) {
            Ok(s) => s,
            Err(e) => {
                log(&format!("memory read failed: {}", e));
                current_pid = None; current_game_id = None; current_game = None;
                reader = None; ipc = None;
                sleep_interruptible(WAIT_INTERVAL, &running);
                continue;
            }
        };

        // Reset session timer on phase changes.
        let phase = snap.phase;
        if last_phase != Some(phase) {
            session_start = Some(discord::now_secs());
            last_phase = Some(phase);
        }

        // Ensure Discord is connected with the *right* client_id for this game.
        if ipc.is_none() {
            let cid = current_game_id.unwrap().client_id();
            match DiscordIpc::connect(cid) {
                Ok(c) => { log("Connected to Discord IPC."); ipc = Some(c); }
                Err(e) => {
                    log(&format!("Discord IPC unavailable ({}). Retrying…", e));
                    sleep_interruptible(WAIT_INTERVAL, &running);
                    continue;
                }
            }
        }

        // Push activity.
        let activity = build_activity(current_game_id.unwrap(), &snap, session_start);
        if let Some(c) = ipc.as_mut() {
            if let Err(e) = c.set_activity(&activity) {
                log(&format!("set_activity failed ({}). Reconnecting.", e));
                ipc = None;
                continue;
            }
        }

        sleep_interruptible(POLL_INTERVAL, &running);
    }

    if let Some(c) = ipc.as_mut() { let _ = c.clear_activity(); }
    log("Bye.");
    Ok(())
}

fn build_activity(game: GameId, snap: &Snapshot, start: Option<u64>) -> Activity {
    // Details line: what the player is doing right now.
    let details = if let Some(o) = &snap.details_override {
        o.clone()
    } else {
        match snap.phase {
            Phase::MainMenu => "In the menus".into(),
            Phase::StagePractice => format!("Practicing Stage {}", snap.stage.max(1)),
            Phase::SpellPractice => "Spell Practice".into(),
            Phase::WatchingReplay => format!("Watching Stage {}", snap.stage.max(1)),
            Phase::GameOver => "Game Over".into(),
            Phase::Ending => "Watching ending".into(),
            Phase::StaffRoll => "Staff roll".into(),
            Phase::SceneComplete => "Scene complete".into(),
            Phase::SceneFail => "Scene failed".into(),
            Phase::Playing => match (snap.stage_state, &snap.boss_name) {
                (StageState::Boss, Some(b))    => format!("Stage {}: vs {}", snap.stage, b),
                (StageState::Midboss, Some(b)) => format!("Stage {}: vs {} (mid)", snap.stage, b),
                _ => format!("Stage {}", snap.stage.max(1)),
            },
        }
    };

    // State line: character, difficulty, resources, score.
    let state = build_state_line(snap);

    Activity {
        details: Some(details),
        state: if state.is_empty() { None } else { Some(state) },
        assets: Some(Assets {
            large_image: Some(game.asset_key().to_string()),
            large_text: Some(format!("Touhou {} — {}", short_num(game), game.full_name())),
            small_image: None,
            small_text: None,
        }),
        timestamps: start.map(|s| Timestamps { start: Some(s), end: None }),
    }
}

fn build_state_line(snap: &Snapshot) -> String {
    let mut parts: Vec<String> = Vec::new();

    let diff = snap.difficulty.label();
    if !diff.is_empty() { parts.push(diff.into()); }
    if !snap.character.is_empty() { parts.push(snap.character.clone()); }

    // Resources — lives + bombs, or the game-specific custom resource.
    match (snap.lives, snap.bombs) {
        (Some(l), Some(b)) => parts.push(format!("{}L {}B", l, b)),
        (Some(l), None) => parts.push(format!("{}L", l)),
        _ => {}
    }
    if let Some((label, v)) = &snap.custom_resource {
        parts.push(format!("{}{}", label, format_score(*v)));
    }
    if let Some(s) = snap.score {
        parts.push(format_score(s));
    }

    parts.join(" — ")
}

/// Short number like "6", "12.8" for a GameId. For the presence tooltip.
fn short_num(id: GameId) -> &'static str {
    match id {
        GameId::Th06 => "6", GameId::Th07 => "7", GameId::Th08 => "8",
        GameId::Th09 => "9", GameId::Th095 => "9.5",
        GameId::Th10 => "10", GameId::Th11 => "11", GameId::Th12 => "12",
        GameId::Th125 => "12.5", GameId::Th128 => "12.8",
        GameId::Th13 => "13", GameId::Th14 => "14", GameId::Th143 => "14.3",
        GameId::Th15 => "15", GameId::Th16 => "16",
        GameId::Th17 => "17", GameId::Th18 => "18",
    }
}

/// Check if pid still refers to a live process of the same game.
fn still_running_as(pid: i32, expected: Option<GameId>) -> bool {
    if !std::path::Path::new(&format!("/proc/{}", pid)).exists() { return false; }
    let Some(expected) = expected else { return true; };
    let Ok(comm) = std::fs::read_to_string(format!("/proc/{}/comm", pid)) else { return false; };
    let comm = comm.trim();
    expected.exe_names().iter().any(|n| comm.eq_ignore_ascii_case(n))
}

fn sleep_interruptible(total: Duration, running: &AtomicBool) {
    let step = Duration::from_millis(200);
    let start = Instant::now();
    while start.elapsed() < total {
        if !running.load(Ordering::SeqCst) { break; }
        thread::sleep(step);
    }
}

fn log(msg: &str) {
    let secs = discord::now_secs();
    let h = (secs / 3600) % 24;
    let m = (secs / 60) % 60;
    let s = secs % 60;
    eprintln!("[{:02}:{:02}:{:02}Z] {}", h, m, s, msg);
}
