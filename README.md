# dokutsu-rs  
A fully playable re-implementation of Cave Story's (Doukutsu Monogatari) engine written in [Rust](https://www.rust-lang.org/).

[Download latest Nightly builds](https://github.com/doukutsu-rs/doukutsu-rs/actions) (Requires being logged in to GitHub)

[Join our Discord server!](https://discord.gg/fbRsNNB)

---

## Data files

*This repository does not contain any copyrighted files. Doukutsu-rs should not be redistributed bundled with data files taken from any commercial port released by Nicalis Inc. without their explicit permission.*

### Compatibility Graph

Port | Support | Versions | Notes
:------------ | :-------------| :-------------| :-------------
Freeware | Full |  All | Fully Playable
CS+ Steam/Epic/Humble | Full | All | Missing Features
CS+ Switch/ GOG | Full | All | Missing Features
WiiWare | Semi | All | Missing Files
WiiWare Demo | No | N/A | Imcompatible Files
DsiWare | No | All | Incompatible Sprites
3DS Eshop | No  | All | Incompatible Sprites
CS3D | No  | All | Incompatible Files

### Where to get Data? / Where do i put Doukutsu-rs
Every port of Cave Story has a **Data** folder which contains game assets and files required for Doukutsu-rs to run, place either besides eachother.

**Freeware**  
Vanilla Freeware embeds important files into the executable so until an extractor is implemented, any of the options below should give you a working Freeware Data.

- https://ci.appveyor.com/project/alula/doukutsu-rs AppVeyor builds are distributed with slightly modified freeware game files.
- https://github.com/doukutsu-rs/game-data - Freeware game data distributed with CI builds, based on those two below.
- https://github.com/nxengine/nxengine-evo/releases/download/v2.6.4/NXEngine-v2.6.4-Win32.zip - copy `NXEngine-evo-2.6.4-xxx/data` from the archive to runtime directory.

**CS+ PC**  
No extra steps should be required.

_Where can i find my CS+ Folder?_
<details>
  <summary>Steam</summary>
Follow these instructions.
  
  ![image](https://user-images.githubusercontent.com/53099651/155904982-eb6032d8-7a4d-4af7-ae6f-b69041ecfaa4.png)

  
</details>

<details>
  <summary>Epic</summary>
  Check your default installation directory.

![image](https://user-images.githubusercontent.com/53099651/155905035-0080eace-bd98-4cf5-9628-c98334ea768c.png)


</details>

<details>
  <summary>GOG</summary>
  Check your default installation directory.
  
  ![image](https://user-images.githubusercontent.com/53099651/155906494-1e53f174-f12f-41be-ab53-8745cdf735b5.png)

</details>

**CS+ Switch**
Extract data folder from romfs, Requires a hacked console and a recent and legal copy of the game, search for a guide to walk you through this as instructions vary.

## Mandatory screenshots


<details>
  <summary>Freeware Data:</summary>

  ![Japanese Freeware](https://i.imgur.com/eZ0V5rK.png)
  
  ![Toroko_Fight Freeware](https://i.imgur.com/m2xMQ7f.png)
  
  ![No lighting Freeware](https://i.imgur.com/MnMDPgA.png)
  
</details>

<details>
  <summary>CS+ PC Data:</summary>

  ![CS+ with enhanced graphics](https://i.imgur.com/YaPAs70.png)
  
  ![CS+ Showoff OuterWall](https://i.imgur.com/7XSW0tO.png)
  
  ![CS+ Challenge support](https://i.imgur.com/Vv8i2sv.png)
  
</details>

<details>
  <summary>CS+ Switch Data:</summary>
  
  ![Balcony Switch](https://i.imgur.com/xsl2a4C.png)
  
  ![Almond Switch](https://i.imgur.com/32ZB3Pt.png)
  
  ![Hell Switch](https://i.imgur.com/R6T4y6w.png)
  
  </details
  
---

## Credits

- Studio Pixel/Nicalis for Cave Story 
- [@Daedily](https://twitter.com/Daedliy) - brand artwork (Icon / Banner / Server)
- [ggez](https://github.com/ggez/ggez) - parts of it are used in `crate::framework`, notably the VFS code. 
- [Clownacy](https://github.com/Clownacy) - widescreen camera code.
- [LunarLambda for organism](https://gitdab.com/LunarLambda/organism) - used as basis for our Organya playback engine.
