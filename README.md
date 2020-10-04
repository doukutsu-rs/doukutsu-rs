# doukutsu-rs

![Release](https://github.com/doukutsu-rs/doukutsu-rs/workflows/Release/badge.svg)

[Download latest Nightly builds](https://github.com/doukutsu-rs/doukutsu-rs/actions) (Requires being logged in to GitHub)

A re-implementation of Cave Story (Doukutsu Monogatari) engine written in [Rust](https://www.rust-lang.org/), aiming for behavior accuracy and cleaner code.
Later plans might involve turning it into a fully-featured modding tool with live debugging and stuff.

The engine also contains some (might be buggy and not accurate, everything was pure guess work on data files to avoid legal issues) implementation of Cave Story+ features from both PC and Switch versions.

Note you have to ship the data files yourself if you want to play with those features, but nothing is stopping you from creating a modification of freeware files that uses those new TSC opcodes and features. I'd actually would like to see something cool created using this engine.

**The project is still incomplete and might not be playable. Expect lots of breaking changes and bugs**

[Join the Discord server](https://discord.gg/fbRsNNB)

#### Data files

This repo does not redistribute any copyrighted files. 

The engine should work fine with [CSE2-Enhanced](https://github.com/Clownacy/CSE2) or [NXEngine(-evo)](https://github.com/nxengine/nxengine-evo) modified freeware data files and [Cave Story+](https://www.nicalis.com/games/cavestory+) data files.

Vanilla Cave Story does not work yet because some important data files are embedded inside executable and we don't have an extractor yet.

##### Where to get them?

**Freeware**

- https://github.com/doukutsu-rs/game-data - Freeware game data distributed with CI builds, based on those two below.
- https://github.com/Clownacy/CSE2/archive/enhanced.zip - copy `game_english/data` from archive to the runtime directory (place you run the executable from, usually project root)
- https://github.com/nxengine/nxengine-evo/releases/download/v2.6.4/NXEngine-v2.6.4-Win32.zip - copy `NXEngine-evo-2.6.4-xxx/data` from the archive to runtime directory

**Cave Story+**

- PC release - Copy `data` folder from installation directory ([guide for Steam](https://steamcommunity.com/sharedfiles/filedetails/?id=760447682)) to the runtime directory.
- Switch release - While some support is implemented, hacking consoles and extracting cartridge content is a kind of gray legal area so I will leave it to you... 

#### Roadmap

- [x] Rendering
  - [x] Backdrops
  - [x] Tilemap
  - [x] Player and it's animations
  - [x] Carets
  - [x] Bullets
  - [x] NPCs
  - [x] Text
  - [x] HUD
- [ ] Text scripts (TSC)
  - [x] Initial implementation
  - [ ] Full implementation of opcodes
  - [ ] Credits
  - [x] Shift-JIS encoding support
- [ ] Audio
  - [x] Organya BGM playback
  - [x] Text script bindings
  - [ ] CS+ style .ogg BGM playback
  - [x] PixTone SFX
- [ ] NPCs/entities
  - [x] Initial implementation
  - [ ] Miscellaneous entities
  - [ ] Bosses
  - [x] First Cave
  - [ ] Mimiga Village
  - [ ] Egg Corridor
  - [ ] Grasstown
  - [ ] Sand Zone
  - [ ] Labirynth
  - [ ] Outer Wall
  - [ ] Plantation
  - [ ] Last Cave
  - [ ] Balcony
  - [ ] Hell
  - [ ] Cave Story+ additions (no accuracy guaranteed)
    - [ ] Dashing Gaudis (361)
    - [ ] ??? (362)
- [ ] Weapons
  - [x] Initial implementation
  - [x] Snake
  - [x] Polar Star
  - [x] Fireball
  - [ ] Machine Gun
  - [ ] Missile Launcher
  - [ ] Bubbler
  - [ ] Blade
  - [ ] Super Missile Launcher
  - [ ] Nemesis
  - [ ] Spur
- [ ] Modding enhancements and built-in tools
  - [x] Debugger
  - [ ] Level editor
  - [ ] Texture auto-reload mode for spriters
- [x] Optional enhanced graphics effects

*(tbd)*

#### Mandatory screenshots

**Freeware data files:**

![freeware](https://i.imgur.com/ZvOtpaI.png)

**Cave Story+ data files:**

![CS+ with enhanced graphics](https://media.discordapp.net/attachments/745322954660905103/760599695009251328/unknown.png)

#### Credits

- Studio Pixel for Cave Story 
- [Cave Story Tribute Site](https://cavestory.org) - for LOTS of useful resources related to the game. 
- [Clownacy/Cucky for CSE2](https://github.com/Clownacy/CSE2) - some game logic reference / mutual help in reverse engineering bitfields and other shit.
- [LunarLambda for organism](https://gitdab.com/LunarLambda/organism) - which is being used by us as `.org` playback engine.
