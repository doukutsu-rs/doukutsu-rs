# Custom Music Table Documentation

This document explains how to use the new customizable music table system in doukutsu-rs.

## Overview

The music table system allows mods to customize which songs loop and which don't, replacing hardcoded behavior that was causing issues with fanfares and sound effects.

## Usage

Create a `music_table.json` file in your mod directory with this structure:

```json
{
  "version": 1,
  "songs": [
    {"name": "wanpaku", "id": 1},
    {"name": "gameover", "id": 3, "noLooping": true},
    {"name": "fanfale1", "id": 10, "noLooping": true}
  ]
}
```

## Schema Reference

- `version`: Must be `1`
- `songs`: Array of song definitions
  - `name`: Song filename without extension
  - `id`: Song ID used in `<CMU` commands
  - `noLooping`: Optional boolean (default: false)

## Recommended Settings

These songs typically should not loop:

- `gameover` (ID 3): Game over jingle
- `fanfale1` (ID 10): Victory fanfare  
- `fanfale2` (ID 16): Item get fanfare
- `fanfale3` (ID 15): Health/life capsule fanfare

See `example_music_table.json` for a complete working example.