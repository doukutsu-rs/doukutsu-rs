# doukutsu-rs

![Release](https://github.com/doukutsu-rs/doukutsu-rs/workflows/Release/badge.svg)

[Download latest Nightly builds](https://github.com/doukutsu-rs/doukutsu-rs/actions) (Requires being logged in to GitHub)

A re-implementation of Cave Story (Doukutsu Monogatari) engine written in [Rust](https://www.rust-lang.org/).

**The project is still incomplete and not fully playable yet.**

[Join the Discord server](https://discord.gg/fbRsNNB)

#### Data files

This repository does not contain any copyrighted files. 

For better user experience, pre-built binaries are distributed with slightly modified freeware game files. 

*doukutsu-rs* should work fine with [CSE2-Enhanced](https://github.com/Clownacy/CSE2) or [NXEngine(-evo)](https://github.com/nxengine/nxengine-evo) freeware data files and [Cave Story+](https://www.nicalis.com/games/cavestory+) data files.

Vanilla Cave Story does not work yet because some important data files have been embedded inside the executable. and we don't have a loader/extractor implemented yet.

##### Where to get them?

**Freeware**

- https://github.com/doukutsu-rs/game-data - Freeware game data distributed with CI builds, based on those two below.
- https://github.com/Clownacy/CSE2/archive/enhanced.zip - copy `game_english/data` from archive to the runtime directory (place you run the executable from, usually project root)
- https://github.com/nxengine/nxengine-evo/releases/download/v2.6.4/NXEngine-v2.6.4-Win32.zip - copy `NXEngine-evo-2.6.4-xxx/data` from the archive to runtime directory

**Cave Story+**

- PC release - (Tested only with Steam version, both Windows and Linux builds) Copy `data` folder from installation directory ([guide for Steam](https://steamcommunity.com/sharedfiles/filedetails/?id=760447682)) to the runtime directory.
- Switch release - **Not supported or actively tested.** Some of release-specific opcodes have been implemented (no code 
decompilation was involved, just pure data file analysis), so you should be able to play it without any major issues. 
Because methods used to extract game data from cartridge vary, you have to find that out on your own.

#### Gameplay support roadmap

- [x] Checkmarked things = fully implemented
- [ ] Unmarked things = partially or not implemented yet.

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
  - [ ] Miscellaneous entities (~30% done)
  - [ ] Bosses (~20% done)
  - [x] First Cave
  - [x] Mimiga Village
  - [x] Egg Corridor
  - [x] Grasstown
  - [ ] Sand Zone (~10% done)
  - [ ] Labirynth (~10% done)
  - [ ] Outer Wall
  - [ ] Plantation
  - [ ] Last Cave
  - [ ] Balcony
  - [ ] Hell
  - [ ] Cave Story+ specific NPCs
    - [x] Dashing Gaudis (361)
    - [ ] ??? (362)
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
  - [ ] Cave Story+
    - [x] Base mod
    - [ ] Mod loading
    - [ ] Curly Story
    - [ ] Wind Fortress (~40%)
    - [ ] Boss Run
    - [x] Seasonal graphics
    - [ ] Co-op gameplay (~70%)
    - [ ] Remastered soundtrack

*(tbd)*

#### Mandatory screenshots

**Freeware data files:**

![Japanese Freeware](https://i.imgur.com/eZ0V5rK.png)

**Cave Story+ data files:**

![CS+ with enhanced graphics](https://i.imgur.com/YaPAs70.png)

#### Legal note

This project includes reverse engineered implementations of NPC and game physics algorithms, derived from freeware Cave Story and PC Cave Story+ executables.

Since the game's (non-existent, even for CS+) EULA does not prohibit reverse engineering, 
[according to Secion 103(f)](https://www.law.cornell.edu/uscode/text/17/1201) we could legally revese engineer those parts 
to achieve interoperability.   

#### Credits

- Studio Pixel/Nicalis for Cave Story 
- [Cave Story Tribute Site](https://cavestory.org) - has lots of useful resources related to the game. 
- [CSE2](https://github.com/Clownacy/CSE2) - widescreen fixes, more readable reference for game logic, mutual help in various things.
- [LunarLambda for organism](https://gitdab.com/LunarLambda/organism) - which is being used by us as `.org` playback engine.
