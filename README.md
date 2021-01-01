# doukutsu-rs

![Release](https://github.com/doukutsu-rs/doukutsu-rs/workflows/Release/badge.svg)

[Download latest Nightly builds](https://github.com/doukutsu-rs/doukutsu-rs/actions) (Requires being logged in to GitHub)

A re-implementation of Cave Story (Doukutsu Monogatari) engine written in [Rust](https://www.rust-lang.org/).

**The project is still incomplete and not fully playable yet.**

[Join the Discord server](https://discord.gg/fbRsNNB)

#### Data files

This repository does not contain any copyrighted files. 

For better user experience, pre-built binaries are distributed with slightly modified freeware game files.

*doukutsu-rs* should work fine with pre-extracted and tweaked data files from [this repository](https://github.com/doukutsu-rs/game-data), [NXEngine(-evo)](https://github.com/nxengine/nxengine-evo) extracted freeware data files and [Cave Story+](https://www.nicalis.com/games/cavestory+) data files.

Vanilla Cave Story does not work yet because some important data files have been embedded inside the executable. and we don't have a loader/extractor implemented yet.

##### Where to get them?

**Freeware**

- https://github.com/doukutsu-rs/game-data - Freeware game data distributed with CI builds, based on those two below.
- ~~https://github.com/Clownacy/CSE2/archive/enhanced.zip - copy `game_english/data` from archive to the runtime directory (place you run the executable from, usually project root)~~
- https://github.com/nxengine/nxengine-evo/releases/download/v2.6.4/NXEngine-v2.6.4-Win32.zip - copy `NXEngine-evo-2.6.4-xxx/data` from the archive to runtime directory

**Cave Story+**

- PC release (Steam) - (Tested only with Steam version, both Windows and Linux builds) Copy `data` folder from installation directory ([guide for Steam](https://steamcommunity.com/sharedfiles/filedetails/?id=760447682)) to the runtime directory.
- PC release (EGS) - (Untested, but the game is essentially the same as Steam release) Same thing as with Steam version.
- Switch release - (Tested once, no guarantee to work) You need a hacked Switch and physical release. Google should help you.

#### Gameplay support roadmap

- [x] Checkmarked things = fully implemented
- [ ] Unmarked things = partially or not implemented yet.

- [ ] Text scripts (TSC)
  - [x] Initial implementation
  - [x] Full implementation of gameplay opcodes
  - [x] Shift-JIS encoding support
  - [ ] Credits opcodes
- [ ] Audio
  - [x] Organya BGM playback
  - [x] Text script bindings
  - [ ] CS+ style .ogg BGM playback
  - [x] PixTone SFX
- [ ] NPCs/entities
  - [x] Initial implementation
  - [ ] Miscellaneous entities (~40% done)
  - [ ] Bosses (~30% done)
    - [x] Omega
    - [x] Balfrog
    - [x] Monster X
  - [x] First Cave
  - [x] Mimiga Village
  - [x] Egg Corridor
  - [x] Grasstown
  - [ ] Sand Zone (~50% done)
  - [ ] Labirynth (~10% done)
  - [x] Waterway
  - [ ] Egg Corridor? (~20% done)
  - [ ] Outer Wall
  - [ ] Plantation
  - [ ] Last Cave
  - [ ] Balcony
  - [ ] Hell
- [ ] Weapons
  - [x] Leveling / XP system
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
- [x] Saving and loading game state
- [ ] Support for different game editions
  - [ ] Vanilla
  - [x] Modified vanilla
  - [ ] Cave Story+ (PC/Switch)
    - [x] Base mod
    - [ ] Mod loading
    - [x] Curly Story
    - [ ] Wind Fortress
    - [ ] Boss Run
    - [x] Seasonal graphics
    - [x] Co-op gameplay

*(tbd)*

#### Mandatory screenshots

**Freeware data files:**

![Japanese Freeware](https://i.imgur.com/eZ0V5rK.png)

**Cave Story+ data files:**

![CS+ with enhanced graphics](https://i.imgur.com/YaPAs70.png)

#### Credits

- Studio Pixel/Nicalis for Cave Story 
- [Cave Story Tribute Site](https://cavestory.org) - has lots of useful resources related to the game. 
- [CSE2](https://github.com/Clownacy/CSE2) - widescreen fixes, more readable reference for game logic, mutual help in various things.
- [LunarLambda for organism](https://gitdab.com/LunarLambda/organism) - which is being used by us as `.org` playback engine.
