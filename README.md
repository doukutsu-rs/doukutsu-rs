# doukutsu-rs

![Release](https://github.com/doukutsu-rs/doukutsu-rs/workflows/Release/badge.svg)

[Download latest Nightly builds](https://github.com/doukutsu-rs/doukutsu-rs/actions) (Requires being logged in to GitHub)

A fully playable re-implementation of Cave Story (Doukutsu Monogatari) engine written in [Rust](https://www.rust-lang.org/).

[Join the Discord server](https://discord.gg/fbRsNNB)

#### Data files

This repository does not contain any copyrighted files. 

For better user experience, AppVeyor builds are distributed with slightly modified freeware game files.

*doukutsu-rs* should work fine with pre-extracted and tweaked data files from [this repository](https://github.com/doukutsu-rs/game-data), [NXEngine(-evo)](https://github.com/nxengine/nxengine-evo) extracted freeware data files and [Cave Story+](https://www.nicalis.com/games/cavestory+) data files.

Vanilla Cave Story does not work yet because some important data files have been embedded inside the executable. and we don't have a loader/extractor implemented yet.

##### Where to get them?

**Freeware**

- https://github.com/doukutsu-rs/game-data - Freeware game data distributed with CI builds, based on those two below.
- https://github.com/nxengine/nxengine-evo/releases/download/v2.6.4/NXEngine-v2.6.4-Win32.zip - copy `NXEngine-evo-2.6.4-xxx/data` from the archive to runtime directory

**Cave Story+ (no full support)**

- SDL version (first released in 2011 on Steam)
  - PC release (Steam) - Copy `data` folder from installation directory ([guide for Steam](https://steamcommunity.com/sharedfiles/filedetails/?id=760447682)) to the runtime directory.
  - PC release (Epic Games Store) - Essentially the same thing as with Steam version.
  - PC release (Humble Bundle) - Essentially the same thing as with Steam version.
- KAGE version (first released in 2017 on Switch)
  - Switch release - (tested only with eShop version) Extract `data` directory from romfs. Requires a hacked console and a recent and legal copy of the game. If you don't know how, look in Google how to exactly do that because the methods really differ.

#### Mandatory screenshots

**Freeware data files:**

![Japanese Freeware](https://i.imgur.com/eZ0V5rK.png)

**Cave Story+ data files:**

![CS+ with enhanced graphics](https://i.imgur.com/YaPAs70.png)

#### Credits

- Studio Pixel/Nicalis for Cave Story 
- - [@uselesscalcium](https://twitter.com/uselesscalcium) - Android port icon artwork
- Daedily - banner / server icon artwork
- [ggez](https://github.com/ggez/ggez) - parts of it are used in `crate::framework`, notably the VFS code.
- [@ClayHanson_](https://twitter.com/ClayHanson_) - for letting us use his .pxchar skin format from Cave Story Multiplayer mod. 
- [Clownacy](https://github.com/Clownacy) - widescreen camera code.
- [LunarLambda for organism](https://gitdab.com/LunarLambda/organism) - used as basis for our Organya playback engine.
