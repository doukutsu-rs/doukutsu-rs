# doukutsu-rs

A re-implementation of Cave Story (Doukutsu Monogatari) engine written in [Rust](https://www.rust-lang.org/), aiming for behavior accuracy and cleaner code.
Later plans might involve turning it into a fully-featured modding tool with live debugging and stuff.

**The project is still in a very early state and nowhere near being playable. Expect lots of breaking changes and bugs**

[Join the Discord server](https://discord.gg/fbRsNNB)

#### Data files

doukutsu-rs project does not re-distribute any copyrighted files. 

The engine should work fine with [CSE2-Enhanced](https://github.com/Clownacy/CSE2) or [NXEngine(-evo)](https://github.com/nxengine/nxengine-evo) modified freeware data files and [Cave Story+](https://www.nicalis.com/games/cavestory+) (Nicalis commercial release, loading is supported but note we're not going to reverse engineer it or support it's features) data files.

Vanilla Cave Story does not work yet because some important data files are embedded inside executable and we don't have an extractor yet.

##### Where to get them?

**Freeware**

- https://github.com/Clownacy/CSE2/archive/enhanced.zip - copy `game_english/data` from archive to the runtime directory (place you run the executable from, usually project root)
- https://github.com/nxengine/nxengine-evo/releases/download/v2.6.4/NXEngine-v2.6.4-Win32.zip - copy `NXEngine-evo-2.6.4-xxx/data` from the archive to runtime directory

**Cave Story+**

- PC release - Copy `data` folder from installation directory ([guide for Steam](https://steamcommunity.com/sharedfiles/filedetails/?id=760447682)) to the runtime directory.
- Switch release - While some support is implemented, hacking consoles and extracting cartridge content is a kind of gray legal area so I will leave it to you... 

#### Roadmap

- [ ] Rendering
  - [x] Backdrops
  - [x] Tilemap
  - [x] Player and it's animations
  - [x] Carets
  - [ ] Bullets
  - [x] NPCs
  - [x] Text
  - [ ] HUD
- [ ] Text scripts (TSC)
  - [x] Initial implementation
  - [x] Execution of basic subset of opcodes and game conversations
  - [ ] Full implementation of opcodes
  - [ ] Shift-JIS encoding support
- [ ] Audio
  - [x] Organya BGM playback
  - [x] Text script bindings
  - [ ] CS+ style .ogg BGM playback
  - [ ] PixTone SFX
- [ ] NPCs/entities
  - [x] Initial implementation
  - [ ] Miscellaneous entities
  - [x] First Cave
  - [ ] Mimiga Village
  - [ ] Egg Corridor
  - [ ] Grasstown
- [ ] Weapons
  - [x] Initial implementation
  - [ ] Snake
  - [x] Polar Star
  - [ ] Fireball
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
- [ ] Optional enhanced graphics effects

*(tbd)*

#### Mandatory screenshots

**Freeware data files:**

![freeware](https://i.imgur.com/ZvOtpaI.png)

**Cave Story+ data files:**

![cs+](https://i.imgur.com/DIlz4eo.jpg)

#### Credits

- Studio Pixel for Cave Story 
- [Cave Story Tribute Site](https://cavestory.org) - for LOTS of useful resources related to the game. 
- [Clownacy/Cucky for CSE2](https://github.com/Clownacy/CSE2) - some game logic reference / mutual help in reverse engineering bitfields and other shit.
- [LunarLambda for organism](https://gitdab.com/LunarLambda/organism) - which is being used by us as `.org` playback engine.
