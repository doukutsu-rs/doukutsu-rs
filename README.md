# doukutsu-rs

A re-implementation of Cave Story (Doukutsu Monogatari) engine written in [Rust](https://www.rust-lang.org/), aiming for behavior accuracy and cleaner code.
Later plans might involve turning it into a fully-featured modding tool with live debugging and stuff.

**The project is still in a very early state and nowhere near being playable. Expect lots of breaking changes and bugs**

[Join the Discord server](https://discord.gg/fbRsNNB)

#### Data files

doukutsu-rs project does not re-distribute any copyrighted files. 

The engine should work fine with [CSE2-Enhanced](https://github.com/Clownacy/CSE2) or [NXEngine(-evo)](https://github.com/nxengine/nxengine-evo) modified freeware data files and [Cave Story+](https://www.nicalis.com/games/cavestory+) (Nicalis commercial release, loading is supported but note we're not going to reverse engineer it or support it's features) data files.

#### Roadmap

- [ ] Rendering
  - [x] Backdrops
  - [x] Tilemap
  - [x] Player and it's animations
  - [x] Carets
  - [ ] Bullets
  - [ ] NPCs
  - [ ] Text
  - [ ] HUD
- [ ] Text scripts (TSC)
  - [x] Initial implementation
  - [x] Execution of basic subset of opcodes and game conversations
  - [ ] Full implementation
- [ ] NPCs/entities
- [ ] Weapons
- [ ] Modding enhancements and built-in tools
  - [x] Debugger
  - [ ] Level editor
  - [ ] Texture auto-reload mode for spriters
- [ ] Optional enhanced graphics effects

*(tbd)*

#### Mandatory screenshots

**Freeware data files:**

![freeware](https://i.imgur.com/oCduSsy.png)

**Cave Story+ data files:**

![cs+](https://i.imgur.com/8qbfkeb.png)

#### why rust, it's a hipster language lol

The project is a result of me wanting to build something in a new programming language for memes.

I had an idea of writing my own CS engine long time before and I would've very likely picked C++17/20 and SDL2, but after 
all I've picked Rust instead because it seemed quite interesting for me.

Would 90% of end-users running this thing care about the programming language software was written in? After all who tf cares if the performance is the same (and maybe a slightly better), but you also get a lot of various benefits?

#### Credits

- Studio Pixel for Cave Story 
- [Cave Story Tribute Site](https://cavestory.org) - for LOTS of useful resources related to the game. 
- [Clownacy for CSE2](https://github.com/Clownacy/CSE2) - a great and very accurate reference for game's logic used in this project
- [CSMC](https://discord.gg/xRsWpz6) - a helpful Cave Story modding community
- [NXEngine](https://github.com/nxengine/nxengine-evo) - an another OSS rewrite of Cave Story engine I took some inspiration from.
