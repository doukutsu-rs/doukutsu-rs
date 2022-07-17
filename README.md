![doukutsu-rs](./res/sue_crab_banner_github.png)

A fully playable re-implementation of Cave Story (Doukutsu Monogatari) engine written
in [Rust](https://www.rust-lang.org/).

[Join the Discord server](https://discord.gg/fbRsNNB)

![https://ci.appveyor.com/api/projects/status/github/doukutsu-rs/doukutsu-rs](https://ci.appveyor.com/api/projects/status/github/doukutsu-rs/doukutsu-rs)

- [Get nightly builds from AppVeyor](https://ci.appveyor.com/project/alula/doukutsu-rs) (recommended for now, has latest fixes and improvements)

  Permalinks to latest builds from `master` branch:

  - [Windows (x86_64)](https://ci.appveyor.com/api/projects/alula/doukutsu-rs/artifacts/doukutsu-rs_win64.zip?branch=master&job=windows-x64)
  - [macOS (Intel, 64-bit, 10.14+)](https://ci.appveyor.com/api/projects/alula/doukutsu-rs/artifacts/doukutsu-rs_mac-intel.zip?branch=master&job=mac-x64)
  - [macOS (Apple M1, 11.0+)](https://ci.appveyor.com/api/projects/alula/doukutsu-rs/artifacts/doukutsu-rs_mac-m1.zip?branch=master&job=mac-arm64)
  - [Linux (x86_64)](https://ci.appveyor.com/api/projects/alula/doukutsu-rs/artifacts/doukutsu-rs_linux.zip?branch=master&job=linux-x64)

  **macOS note:** If you get a `"doukutsu-rs" can't be opened` message, right-click doukutsu-rs.app and click open.

- [Get stable/beta builds from GitHub Releases](https://github.com/doukutsu-rs/doukutsu-rs/releases) (Includes Android builds)

#### Data files

In order to work doukutsu-rs needs to be paired with supported data files. This repository does not contain any data
files.

doukutsu-rs works fine with freeware data files or [NXEngine(-evo)](https://github.com/nxengine/nxengine-evo) or from a
supported copy of [Cave Story+](https://www.nicalis.com/games/cavestory+).

#### Supported game editions and data file acquisition guides

**Freeware**

doukutsu-rs works out of the box when it's placed in the same directory as the original Doukutsu.exe executable. On the initial
startup, doukutsu-rs will automatically extract the additional resources that are embedded in the vanilla game into the `data`
directory. Until that is done, both doukutsu-rs and the vanilla executable have to exist in the directory.

<details>
<summary>Example root directory</summary>

![example root directory with doukutsu-rs and vanilla Cave Story](https://i.imgur.com/3dJ7WMB.png)

</details>

**Cave Story+**

doukutsu-rs can be used as drop-in replacement for `CaveStory+.exe`. No modifications to game files are needed.

**Original version (first released in 2011 on Steam)** - expand for instructions

<details>
<summary>Steam release (Win/Mac/Linux)</summary>

The `data` folder is in the same place across all platforms.

If you want to use doukutsu-rs as a substitute for Mac version of Cave Story+ (which at moment of writing doesn't work
on 10.15+ anymore), do the following:

1. Find the doukutsu-rs executable:
   - In AppVeyor builds, it's in `doukutsu-rs.app/Contents/MacOS/doukutsu-rs`
   - In your own builds, it's in `target/(release|debug)/doukutsu-rs`
2. Open Steam Library, select `Cave Story+`, press the `Manage` button (gear icon) and select `Properties...`
3. Select `Local Files` and press `Browse...`.
4. Open the `Cave Story+.app` bundle and navigate to `Contents/MacOS` directory.
5. Rename the `Cave Story+` executable to something else or delete it.
6. Copy the doukutsu-rs executable and rename it to `Cave Story+`.
7. Launch the game from Steam and enjoy!

![image](https://user-images.githubusercontent.com/53099651/155904982-eb6032d8-7a4d-4af7-ae6f-b69041ecfaa4.png)

</details>

<details>
<summary>Epic Games Store</summary>

Check your default installation directory.

![image](https://user-images.githubusercontent.com/53099651/155905035-0080eace-bd98-4cf5-9628-c98334ea768c.png)

</details>

<details>
<summary>GOG</summary>

Check your default installation directory.

![image](https://user-images.githubusercontent.com/53099651/155906494-1e53f174-f12f-41be-ab53-8745cdf735b5.png)

</details>

<details>
<summary>Humble Bundle</summary>

The archive from Humble Bundle contains the necessary `data` folder, in the same folder as `CaveStory+.exe`.

![image](https://user-images.githubusercontent.com/96957561/156861929-7fa03951-442b-4277-b673-474189411103.png)

</details>

<details>
<summary>WiiWare</summary>

1. [Dump Your WiiWare ``.wad``](https://wii.guide/dump-wads.html)  
2. [Extract and decompress the ``data`` folder](https://docs.google.com/document/d/1hDNDgNl0cUDlFOQ_BUOq3QCGb7S0xfUxRoob-hfM-DY)  
Example of a [valid uncompressed ``data`` folder](https://user-images.githubusercontent.com/53099651/159585593-43fead24-b041-48f4-8332-be50d712310d.png)

</details>

**Remastered version (first released in 2017 on Switch)**

Note that this version is **incompatible** with saves from the original version.

Interchanging the save files may result in spawning in wrong locations, softlocks, graphical glitches, or other issues.

<details>
<summary>Nintendo Switch</summary>

Extract the `data` folder directly from the ROM.

</details>

#### Controls

Same controls as the default for freeware and Cave Story+ keyboard.

To change, edit `doukutsu-rs\data\settings.json` within your user directory.

|              | P1        | P2        |
| ------------ | --------- | --------- |
| Movement     | `← ↑ ↓ →` | `, L . /` |
| Jump         | `Z`       | `B`       |
| Shoot        | `X`       | `N`       |
| Cycle Weapon | `A and S` | `G and H` |
| Inventory    | `Q`       | `T`       |
| Map          | `W`       | `Y`       |
| Strafe       | `LShift`  | `RShift`  |

- `Alt + Enter` - Toggle Fullscreen
- `F2` (While paused) - Quick Restart

#### Screenshots

<details>
<summary>Freeware</summary>

![JP Freeware 2](https://user-images.githubusercontent.com/53099651/155924461-c63afc93-a41f-4cfd-ac9f-8f021cebcb04.png)

![Toroko Fight Freeware](https://user-images.githubusercontent.com/53099651/155924215-d492907a-ed0e-4323-bd46-61745b8fb32a.png)

![No Lighting Freeware](https://user-images.githubusercontent.com/53099651/155923814-621cf29e-bb20-4680-a96d-f049aaef1f88.png)

</details>

<details>
<summary>Original CS+</summary>

![CS+ Sand Zone](https://user-images.githubusercontent.com/53099651/155923620-db230077-0df5-4de4-b086-be6b4dcbc6df.png)

![CS+ Showoff Outer Wall](https://user-images.githubusercontent.com/53099651/155920013-3967cd03-8d69-4fc5-8f1d-fe659ff2e953.png)

![CS+ Challenge](https://user-images.githubusercontent.com/53099651/155919381-7e8159a0-a7cf-461a-8be2-2ce864631299.png)

</details>

<details>
<summary>Remastered CS+</summary>

![Balcony Switch](https://user-images.githubusercontent.com/53099651/155918810-063c0f06-2d48-485f-8367-6337525deab7.png)

![Dogs Switch](https://media.discordapp.net/attachments/745322954660905103/947895408196202617/unknown.png)

![Almond Switch](https://media.discordapp.net/attachments/745322954660905103/947898268631826492/unknown.png)

![Hell Switch](https://user-images.githubusercontent.com/53099651/155918602-62268274-c529-41c2-a87e-0c31e7874b94.png)

</details>

#### Credits

- Studio Pixel/Nicalis for Cave Story
- [@Daedily](https://twitter.com/Daedliy) - brand artwork (Icon / Banner / Server), screenshots for this guide.
- [ggez](https://github.com/ggez/ggez) - parts of it are used in `crate::framework`, notably the VFS code.
- [Clownacy](https://github.com/Clownacy) - widescreen camera code.
- [LunarLambda for organism](https://gitdab.com/LunarLambda/organism) - used as basis for our Organya playback engine.
