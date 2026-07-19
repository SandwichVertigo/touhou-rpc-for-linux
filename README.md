**touhou-rpc-for-linux**  
**Native Linux Discord Rich Presence for the Touhou series**, running under Wine or Proton.  
No Windows dependencies. No Wine-side bridge. No named-pipe symlinks. One 603 KB binary that reads game state directly from the Wine process and speaks Discord's IPC protocol over the Linux Unix socket. First tool of its kind for Linux.  
***This project is a Linux-native port and rewrite of ***[ ***TouhouRPC by TheBakaRem*** ***.*** * Memory addresses, boss and stage tables, state-machine logic, and Discord application IDs (which supply the cover art) all come from their reverse-engineering work. This project would not exist without theirs. As a derivative work under GPL v3, * *touhou-rpc-for-linux* * is also GPL v3. See *](https://github.com/TheBakaRem/TouhouRPC "https://github.com/TheBakaRem/TouhouRPC")[ *NOTICE* * for a precise breakdown of what was derived and what is original.*](NOTICE "NOTICE")  
**Supported games**  
Detection and presence -all games below show up on Discord with cover art:  
| | | |  
|-|-|-|  
| **#** | **Title** | **Depth** |   
| 6 | Embodiment of Scarlet Devil | Full: character, difficulty, stage, mid-boss/boss, lives, bombs, score |   
| 7 | Perfect Cherry Blossom | Full |   
| 8 | Imperishable Night | Full (incl. team detection, stage 4A/4B/6A/6B branches) |   
| 9 | Phantasmagoria of Flower View | Detection + character |   
| 9.5 | Shoot the Bullet | Detection only |   
| 10 | Mountain of Faith | Detection only (needs BGM-string parsing - TODO) |   
| 11 | Subterranean Animism | Detection only (same reason) |   
| 12 | Undefined Fantastic Object | Full |   
| 12.5 | Double Spoiler | Detection only |   
| 12.8 | Fairy Wars | Detection only |   
| 13 | Ten Desires | Full |   
| 14 | Double Dealing Character | Full |   
| 14.3 | Impossible Spell Card | Detection only |   
| 15 | Legacy of Lunatic Kingdom | Full |   
| 16 | Hidden Star in Four Seasons | Full (incl. season) |   
| 17 | Wily Beast and Weakest Creature | Full (incl. beast) |   
| 18 | Unconnected Marketeers | Full (incl. money counter) |   
   
**Not supported:** the fighters (10.5, 12.3, 13.5, 14.5, 15.5, 17.5), Violet Detector (16.5), and TH19 -no publicly-reverse-engineered memory maps exist for these.  
**Full = ** character, difficulty, stage, lives, bombs, score plus a boss name when you're fighting one.  
**Detection only = ** shows "Playing Touhou N Title" with cover art but no gameplay details. A v0.2 pass will fill these in.  
**Build & run**  
sudo apt install rustc cargo      
 cargo build --release  
 ./target/release/touhou-rpc-for-linux  
   
Launch order doesn't matter. Start the daemon, start Discord, launch any Touhou game under Wine or Proton. Presence updates within 5 seconds. Ctrl-C clears it.  
**How it works**  
1. **Process scan.**/proc/<pid>/comm and /cmdline are checked against every known Touhou exe basename (ASCII *and* the original Japanese release names - 東方紅魔郷.exe, 東方妖々夢.exe, etc.). First match wins.  
2. **Memory reads.**process_vm_readv(2) first, falling back to /proc/<pid>/mem if Yama denies the syscall. Touhou is 32-bit; Wine maps the PE at its preferred base, so the Windows-documented static addresses are valid inside the Wine process.  
3. **Per-game state extraction.** Each game has its own reader in src/games.rs with the correct address table and boss/stage tables (ported from TouhouRPC).  
4. **Discord IPC.** Length-prefixed frames over $XDG_RUNTIME_DIR/discord-ipc-{0..9} (plus Flatpak and Snap paths). One Discord app per game, so the daemon reconnects with a new client_id whenever you switch games.  
Poll cadence: 5 s during play, 3 s while waiting for a game.  
**Cover art**  
The images shown on Discord live on Discord's servers, uploaded to the 17 Discord applications belonging to TheBakaRem for TouhouRPC. This project reuses those app IDs, so you get working presence with zero setup but that also means the Discord activity is technically running under **TouhouRPC's Discord apps**, which is worth knowing if you'd rather own the whole chain yourself. To use your own:  
1. Create an app at [https://discord.com/developers/applications.](https://discord.com/developers/applications "https://discord.com/developers/applications")  
2. Under **Rich Presence → Art Assets**, upload a cover with the key th06, th07, etc.  
3. Replace the ID in GameId::client_id (src/common.rs) with yours.  
**Common issues**  
**"memory read failed: open /proc/N/mem"**  
   
 Yama's ptrace_scope is blocking the daemon. Cleanest fix:  
sudo setcap cap_sys_ptrace=eip ./target/release/touhou-rpc-for-linux  
   
Alternative (session-wide, resets on reboot):  
echo 0 | sudo tee /proc/sys/kernel/yama/ptrace_scope  
   
*"no discord-ipc-* * socket (is Discord running?)"**  
   
 Discord isn't running, or it's a sandbox variant with a socket path this project doesn't know. ls $XDG_RUNTIME_DIR/discord-ipc-* and check the Flatpak / Snap paths in src/discord.rs::find_socket.  
**Presence shows the right game but the wrong stage/boss**  
   
 Addresses are for the standard official releases (typically v1.00 letter-suffix builds). Modded or patched builds may have shifted layouts.  
**Systemd user service**  
# ~/.config/systemd/user/touhou-rpc-for-linux.service  
 [Unit]  
 Description=Touhou Discord Rich Presence  
 After=graphical-session.target  
   
 [Service]  
 ExecStart=%h/bin/touhou-rpc-for-linux  
 Restart=on-failure  
 RestartSec=5  
   
 [Install]  
 WantedBy=default.target  
   
Then systemctl --user enable --now touhou-rpc-for-linux.  
**Roadmap**  
- v0.2 - fill in TH10/11 (BGM-string parsing), scene detection for TH9.5/12.5/14.3  
- v0.3 - spell card names (needs porting the ~600-entry per-game tables)  
- v0.4 - Music-room BGM display  
- Someday - fighters, VD, TH19 (needs someone to reverse-engineer the addresses; contributions welcome)  
**License and credits**  
touhou-rpc-for-linux is licensed under the **GNU General Public License version 3 or later** (GPL-3.0-or-later). See [LICENSE for the full text.](LICENSE "LICENSE")  
This project is a derivative work of [TouhouRPC by TheBakaRem, which is also licensed under GPL v3. See ](https://github.com/TheBakaRem/TouhouRPC "https://github.com/TheBakaRem/TouhouRPC")[NOTICE for a precise breakdown of what code and data comes from where.](NOTICE "NOTICE")  
**Not affiliated with or endorsed by TheBakaRem, ZUN, or Team Shanghai Alice.**  
