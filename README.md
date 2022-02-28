# doukutsu-rs  
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

  ![JP Freeware 2](https://user-images.githubusercontent.com/53099651/155924461-c63afc93-a41f-4cfd-ac9f-8f021cebcb04.png)
  
  ![Toroko Fight Freeware](https://user-images.githubusercontent.com/53099651/155924215-d492907a-ed0e-4323-bd46-61745b8fb32a.png)
  
  ![No Lighting Freeware](https://user-images.githubusercontent.com/53099651/155923814-621cf29e-bb20-4680-a96d-f049aaef1f88.png)
  
</details>

<details>
  <summary>CS+ PC Data:</summary>

  ![CS+ Sand Zone](https://user-images.githubusercontent.com/53099651/155923620-db230077-0df5-4de4-b086-be6b4dcbc6df.png)
  
  ![CS+ Showoff Outer Wall](https://user-images.githubusercontent.com/53099651/155920013-3967cd03-8d69-4fc5-8f1d-fe659ff2e953.png)
  
  ![CS+ Challenge](https://user-images.githubusercontent.com/53099651/155919381-7e8159a0-a7cf-461a-8be2-2ce864631299.png)
  
</details>

<details>
  <summary>CS+ Switch Data:</summary>
  
  ![Balcony Switch](https://user-images.githubusercontent.com/53099651/155918810-063c0f06-2d48-485f-8367-6337525deab7.png)
  
  ![Almond Switch](https://user-images.githubusercontent.com/53099651/155918761-7b97dcd7-884c-4929-ad6c-9694e42bd5ff.png)
  
  ![Hell Switch](https://user-images.githubusercontent.com/53099651/155918602-62268274-c529-41c2-a87e-0c31e7874b94.png)

  
  </details
  
---

## Credits

- Studio Pixel/Nicalis for Cave Story 
- [@Daedily](https://twitter.com/Daedliy) - brand artwork (Icon / Banner / Server)
- [ggez](https://github.com/ggez/ggez) - parts of it are used in `crate::framework`, notably the VFS code. 
- [Clownacy](https://github.com/Clownacy) - widescreen camera code.
- [LunarLambda for organism](https://gitdab.com/LunarLambda/organism) - used as basis for our Organya playback engine.
