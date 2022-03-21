# doukutsu-rs

A fully playable re-implementation of Cave Story (Doukutsu Monogatari) engine written
in [Rust](https://www.rust-lang.org/).

[Join the Discord server](https://discord.gg/fbRsNNB)

![https://ci.appveyor.com/api/projects/status/github/doukutsu-rs/doukutsu-rs](https://ci.appveyor.com/api/projects/status/github/doukutsu-rs/doukutsu-rs)

- [Get nightly builds from AppVeyor](https://ci.appveyor.com/project/alula/doukutsu-rs) (recommended for now, has latest fixes and improvements, select platform -> Artifacts ->
  download the .zip)
- [Get stable/beta builds from GitHub Releases](https://github.com/doukutsu-rs/doukutsu-rs/releases) (executables only,
  no data files bundled, see below for instructions)

#### Data files

In order to work doukutsu-rs needs to be paired with supported data files. This repository does not contain any data
files.

doukutsu-rs works fine with pre-extracted freeware data from [this repository](https://github.com/doukutsu-rs/game-data)
builds or [NXEngine(-evo)](https://github.com/nxengine/nxengine-evo) or from a supported copy
of [Cave Story+](https://www.nicalis.com/games/cavestory+).

#### Supported game editions and data file acquisition guides

**Freeware**

- Vanilla freeware can't be just used without additional work, because some important data files are embedded inside the
  executable. An automatic extractor might be available in future.
  
<details>
<summary>Manual extraction guide</summary>

Tools required:
- Windows version of the game (1.0.0.6), original Japanese or with Aeon Genesis patch.  
- [Resource Hacker](http://www.angusj.com/resourcehacker/#download)
- [Booster's Lab](https://www.cavestory.org/download/editors.php)
    
1. Open Doukutsu.exe in Resource Hacker.
2. Click on `ORG` group, select `Action` -> `Save [ORG] group to an .RC file`.
3. Navigate to `data` folder and create a folder named `Org` and save the .RC file there.
4. Click on `BITMAP` group, select `Action` -> `Save [BITMAP] group to an .RC file`.
5. Save them in `data` folder (**NOT** in `Org` folder).
6. Go to file explorer and navigate to `data` folder.
7. Delete Bitmap.rc
8. Go to `Org` folder.
9. Delete Org.rc
10. Rename extension of all files from `.bin` to `.org` - you won't have music if you don't do that!
11. Close Resource Hacker.
12. Open Booster's Lab
13. Load `Doukutsu.exe` in Booster's Lab - you can ignore the fact it tries to apply any patches or renames .pbm to .bmp, d-rs doesn't care.
14. Select `File` -> `Export mapdata` -> `stage.tbl`
15. Close Booster's Lab, saving isn't necessary.
16. Optionally delete leftover files and folders - `.boostlab`, `ScriptSource`, `tsc_def.txt`
17. That's all, you have everything to use it with doukutsu-rs now.

If you followed the above steps, the directory structure should look like this:

`data/`:

![files in /data/](https://media.discordapp.net/attachments/745322954660905103/947915770376102008/unknown.png?width=844&height=629)

`data/Org`:
![files in /data/Org/](https://media.discordapp.net/attachments/745322954660905103/947915770690687016/unknown.png)

</details>
    
- https://github.com/doukutsu-rs/game-data - Pre-extracted freeware game data, graphics converted to .png, already
  distributed with AppVeyor builds for your convenience. (recommended)
- https://github.com/nxengine/nxengine-evo/releases/download/v2.6.5-1/NXEngine-Evo-v2.6.5-1-Win64.zip -
  copy `NXEngine-evo-2.6.5-1-xxx/data` from the archive to runtime directory

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

**Remastered version (first released in 2017 on Switch)**

Note that this version is **incompatible** with saves from the original version.

Interchanging the save files may result in spawning in wrong locations, softlocks, graphical glitches, or other issues.

<details>
<summary>Nintendo Switch</summary>

(Only 1.2+ has been tested, earlier versions may not work properly due to lack of 2P/Original Graphics support.)

Your interest is only in `data` directory placed in romfs.

Requires a hackable/modchipped console. If you got your Switch early, it's likely that it's hackable so give it a shot -
just be very careful to not get your console banned. There's tons of guides you can easily find online so we won't cover
it there.

You can dump the ROM (or just dump the romfs directly but it's just a bit slow so we recommend doing it on PC instead)
using [nxdumptool](https://github.com/DarkMatterCore/nxdumptool).

Once you got the keys and ROM dumped you can use the romfs extraction feature in Ryujinx or yuzu emulators to grab the
data files.

</details>

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
