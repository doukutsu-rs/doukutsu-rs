# doukutsu-rs

![Release](https://github.com/doukutsu-rs/doukutsu-rs/workflows/Release/badge.svg)

[Download latest Nightly builds](https://github.com/doukutsu-rs/doukutsu-rs/actions) (Requires being logged in to GitHub)

A re-implementation of Cave Story (Doukutsu Monogatari) engine written in [Rust](https://www.rust-lang.org/).

**The project is still incomplete and might not be playable. Expect lots of breaking changes and bugs**

[Join the Discord server](https://discord.gg/fbRsNNB)

#### Data files

This repository does not contain any copyrighted files. 

For better user experience, binaries are being distributed with slightly modified freeware game files. 

*doukutsu-rs* should work fine with [CSE2-Enhanced](https://github.com/Clownacy/CSE2) or [NXEngine(-evo)](https://github.com/nxengine/nxengine-evo) modified freeware data files and [Cave Story+](https://www.nicalis.com/games/cavestory+) data files.

Vanilla Cave Story does not work yet because some important data files have been embedded inside the executable. and we don't have a loader/extractor implemented yet.

##### Where to get them?

**Freeware**

- https://github.com/doukutsu-rs/game-data - Freeware game data distributed with CI builds, based on those two below.
- https://github.com/Clownacy/CSE2/archive/enhanced.zip - copy `game_english/data` from archive to the runtime directory (place you run the executable from, usually project root)
- https://github.com/nxengine/nxengine-evo/releases/download/v2.6.4/NXEngine-v2.6.4-Win32.zip - copy `NXEngine-evo-2.6.4-xxx/data` from the archive to runtime directory

**Cave Story+**

- PC release - Copy `data` folder from installation directory ([guide for Steam](https://steamcommunity.com/sharedfiles/filedetails/?id=760447682)) to the runtime directory.
- Switch release - Not supported, because extracting the data files from the console is complicated and requires 
device-specific decryption keys. Some of release-specific features have been implemented, so you should be able to play 
it without any major issues. Google will likely help you if you really want to.

#### Roadmap

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
  - [ ] Full implementation of opcodes (~80% done)
  - [ ] Credits
  - [x] Shift-JIS encoding support
- [ ] Audio
  - [x] Organya BGM playback
  - [x] Text script bindings
  - [ ] CS+ style .ogg BGM playback
  - [x] PixTone SFX
- [ ] NPCs/entities
  - [x] Initial implementation
  - [ ] Miscellaneous entities (~30% done)
  - [ ] Bosses
  - [x] First Cave
  - [x] Mimiga Village
  - [ ] Egg Corridor (~70% done)
  - [ ] Grasstown (~10% done)
  - [ ] Sand Zone (~10% done)
  - [ ] Labirynth (~10% done)
  - [ ] Outer Wall
  - [ ] Plantation
  - [ ] Last Cave
  - [ ] Balcony
  - [ ] Hell
  - [ ] Cave Story+ specific NPCs
    - [ ] Dashing Gaudis (361)
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
- [ ] Modding enhancements and built-in tools
  - [x] Debugger
  - [ ] Level editor
  - [ ] Texture auto-reload mode for spriters
- [x] Saving and loading game state
- [ ] Data file support
  - [ ] Vanilla
  - [x] Modified vanilla
  - [ ] Cave Story+
    - [x] Base mod
    - [ ] Mod loading
    - [ ] Curly Story
    - [ ] Wind Fortress
    - [ ] Boss Run
    - [ ] Seasonal graphics
    - [ ] Remastered soundtrack
- [x] Optional enhanced graphics effects

*(tbd)*

#### Mandatory screenshots

**Freeware data files:**

![Japanese Freeware](https://i.imgur.com/eZ0V5rK.png)

**Cave Story+ data files:**

![CS+ with enhanced graphics](https://i.imgur.com/YaPAs70.png)

#### Credits

- Studio Pixel for Cave Story 
- [Cave Story Tribute Site](https://cavestory.org) - for LOTS of useful resources related to the game. 
- [Clownacy/Cucky for CSE2](https://github.com/Clownacy/CSE2) - some game logic reference / mutual help in reverse engineering bitfields and other shit.
- [LunarLambda for organism](https://gitdab.com/LunarLambda/organism) - which is being used by us as `.org` playback engine.
