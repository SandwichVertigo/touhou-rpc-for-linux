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

//! Common vocabulary shared across all game modules.

use anyhow::Result;

use crate::memory::MemoryReader;

/// Every game we support.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameId {
    Th06, Th07, Th08, Th09, Th095,
    Th10, Th11, Th12, Th125, Th128,
    Th13, Th14, Th143, Th15, Th16, Th17, Th18,
}

impl GameId {
    pub fn all() -> &'static [GameId] {
        &[
            GameId::Th06, GameId::Th07, GameId::Th08, GameId::Th09, GameId::Th095,
            GameId::Th10, GameId::Th11, GameId::Th12, GameId::Th125, GameId::Th128,
            GameId::Th13, GameId::Th14, GameId::Th143, GameId::Th15, GameId::Th16,
            GameId::Th17, GameId::Th18,
        ]
    }

    /// Windows executable basenames that identify this game (ASCII and JP).
    pub fn exe_names(self) -> &'static [&'static str] {
        match self {
            GameId::Th06 => &["th06.exe", "\u{6771}\u{65b9}\u{7d05}\u{9b54}\u{90f7}.exe"],
            GameId::Th07 => &["th07.exe", "\u{6771}\u{65b9}\u{5996}\u{3005}\u{5922}.exe"],
            GameId::Th08 => &["th08.exe", "\u{6771}\u{65b9}\u{6c38}\u{591c}\u{6284}.exe"],
            GameId::Th09 => &["th09.exe", "\u{6771}\u{65b9}\u{82b1}\u{6620}\u{585a}.exe"],
            GameId::Th095 => &["th095.exe", "\u{6771}\u{65b9}\u{6587}\u{82b1}\u{5e16}.exe"],
            GameId::Th10 => &["th10.exe", "\u{6771}\u{65b9}\u{98a8}\u{795e}\u{9332}.exe"],
            GameId::Th11 => &["th11.exe", "\u{6771}\u{65b9}\u{5730}\u{970a}\u{6bbf}.exe"],
            GameId::Th12 => &["th12.exe", "\u{6771}\u{65b9}\u{661f}\u{84ee}\u{8239}.exe"],
            GameId::Th125 => &["th125.exe", "\u{30c0}\u{30d6}\u{30eb}\u{30b9}\u{30dd}\u{30a4}\u{30e9}\u{30fc}.exe"],
            GameId::Th128 => &["th128.exe", "\u{5996}\u{7cbe}\u{5927}\u{6226}\u{4e89}.exe"],
            GameId::Th13 => &["th13.exe", "\u{6771}\u{65b9}\u{795e}\u{970a}\u{5edf}.exe"],
            GameId::Th14 => &["th14.exe", "\u{6771}\u{65b9}\u{8f1d}\u{91dd}\u{57ce}.exe"],
            GameId::Th143 => &["th143.exe"],
            GameId::Th15 => &["th15.exe", "\u{6771}\u{65b9}\u{7d05}\u{6cea}\u{6f5c}.exe"],
            GameId::Th16 => &["th16.exe", "\u{6771}\u{65b9}\u{5929}\u{7a7a}\u{748b}.exe"],
            GameId::Th17 => &["th17.exe", "\u{6771}\u{65b9}\u{9b3c}\u{5f62}\u{7378}.exe"],
            GameId::Th18 => &["th18.exe", "\u{6771}\u{65b9}\u{8679}\u{9f8d}\u{6d1e}.exe"],
        }
    }

    /// TouhouRPC's Discord application ID for this game. These apps have the
    /// cover art already uploaded — reusing them means presence Just Works.
    pub fn client_id(self) -> &'static str {
        match self {
            GameId::Th06  => "712067805398171658",
            GameId::Th07  => "711300438867312692",
            GameId::Th08  => "712068017172905984",
            GameId::Th09  => "717460728990139023",
            GameId::Th095 => "725634084050829352",
            GameId::Th10  => "716759035571077171",
            GameId::Th11  => "712067875757752442",
            GameId::Th12  => "716678778755219508",
            GameId::Th125 => "896809286456467456",
            GameId::Th128 => "717045124076405239",
            GameId::Th13  => "712836601407078410",
            GameId::Th14  => "709074475789844602",
            GameId::Th143 => "791038671322480681",
            GameId::Th15  => "712067916862062633",
            GameId::Th16  => "712067956481458197",
            GameId::Th17  => "712071166143234109",
            GameId::Th18  => "823266075659599892",
        }
    }

    pub fn full_name(self) -> &'static str {
        match self {
            GameId::Th06  => "Embodiment of Scarlet Devil",
            GameId::Th07  => "Perfect Cherry Blossom",
            GameId::Th08  => "Imperishable Night",
            GameId::Th09  => "Phantasmagoria of Flower View",
            GameId::Th095 => "Shoot the Bullet",
            GameId::Th10  => "Mountain of Faith",
            GameId::Th11  => "Subterranean Animism",
            GameId::Th12  => "Undefined Fantastic Object",
            GameId::Th125 => "Double Spoiler",
            GameId::Th128 => "Fairy Wars",
            GameId::Th13  => "Ten Desires",
            GameId::Th14  => "Double Dealing Character",
            GameId::Th143 => "Impossible Spell Card",
            GameId::Th15  => "Legacy of Lunatic Kingdom",
            GameId::Th16  => "Hidden Star in Four Seasons",
            GameId::Th17  => "Wily Beast and Weakest Creature",
            GameId::Th18  => "Unconnected Marketeers",
        }
    }

    pub fn short_name(self) -> &'static str {
        match self {
            GameId::Th06  => "Touhou 6",
            GameId::Th07  => "Touhou 7",
            GameId::Th08  => "Touhou 8",
            GameId::Th09  => "Touhou 9",
            GameId::Th095 => "Touhou 9.5",
            GameId::Th10  => "Touhou 10",
            GameId::Th11  => "Touhou 11",
            GameId::Th12  => "Touhou 12",
            GameId::Th125 => "Touhou 12.5",
            GameId::Th128 => "Touhou 12.8",
            GameId::Th13  => "Touhou 13",
            GameId::Th14  => "Touhou 14",
            GameId::Th143 => "Touhou 14.3",
            GameId::Th15  => "Touhou 15",
            GameId::Th16  => "Touhou 16",
            GameId::Th17  => "Touhou 17",
            GameId::Th18  => "Touhou 18",
        }
    }

    /// The Discord `large_image` asset key. TouhouRPC uploads every game's
    /// cover art under the key "cover" in each app.
    pub fn asset_key(self) -> &'static str {
        "cover"
    }
}

// --- Shared enums (adapted from TouhouRPC's Games-Enums.ixx) ----------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    None, Easy, Normal, Hard, Lunatic, Extra, Phantasm, Overdrive,
}
impl Difficulty {
    pub fn label(self) -> &'static str {
        match self {
            Difficulty::None => "",
            Difficulty::Easy => "Easy",
            Difficulty::Normal => "Normal",
            Difficulty::Hard => "Hard",
            Difficulty::Lunatic => "Lunatic",
            Difficulty::Extra => "Extra",
            Difficulty::Phantasm => "Phantasm",
            Difficulty::Overdrive => "Overdrive",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    MainMenu,
    Playing,
    StagePractice,
    SpellPractice,
    WatchingReplay,
    GameOver,
    Ending,
    StaffRoll,
    SceneComplete,
    SceneFail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageState { Stage, Midboss, Boss }

#[derive(Debug, Clone)]
pub struct Snapshot {
    pub phase: Phase,
    pub difficulty: Difficulty,
    /// Character/shot-type label as a string — every game has different sets,
    /// so we normalize downstream to text.
    pub character: String,
    /// Stage number (1-based); 0 if not applicable.
    pub stage: u8,
    pub stage_state: StageState,
    /// Human-readable "we're fighting X" name (mid-boss or boss).
    pub boss_name: Option<String>,
    /// Currently-playing BGM/spell-card name for the details line, if known.
    pub extra: Option<String>,
    pub lives: Option<u32>,
    pub bombs: Option<u32>,
    pub score: Option<u64>,
    /// Game-specific extra numeric (money in TH18, graze in fighter/scene games, etc.)
    pub custom_resource: Option<(String, u64)>,
    /// If Some, the daemon renders this as the "details" line verbatim,
    /// overriding the derived stage/boss text (for menu screens, endings, etc.).
    pub details_override: Option<String>,
}

impl Snapshot {
    pub fn empty(phase: Phase) -> Self {
        Snapshot {
            phase, difficulty: Difficulty::None, character: String::new(),
            stage: 0, stage_state: StageState::Stage, boss_name: None,
            extra: None, lives: None, bombs: None, score: None,
            custom_resource: None, details_override: None,
        }
    }
}

/// Every game module implements this.
pub trait TouhouGame: Send {
    fn id(&self) -> GameId;
    fn read(&mut self, mem: &mut MemoryReader) -> Result<Snapshot>;
}

// --- Helpers used by many game modules --------------------------------------

pub fn diff_from_u32(v: u32) -> Difficulty {
    match v {
        0 => Difficulty::Easy,
        1 => Difficulty::Normal,
        2 => Difficulty::Hard,
        3 => Difficulty::Lunatic,
        4 => Difficulty::Extra,
        5 => Difficulty::Phantasm,
        _ => Difficulty::Normal,
    }
}

pub fn format_score(s: u64) -> String {
    let n = s.to_string();
    let mut out = String::with_capacity(n.len() + n.len() / 3);
    for (i, ch) in n.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 { out.push(','); }
        out.push(ch);
    }
    out.chars().rev().collect()
}
