# lau-shell-kernel

The bare shell kernel — the hermes-construct in Rust form.

An empty shell with no rooms, no ensigns, no APIs. The construct that can be cloned and decomposed by Hermes.

## Architecture

- **ShellKernel** — THE kernel, the hermes-construct
- **ShellId** — unique shell identifier
- **ShellKind** — Hermes, ZeroClaw, CUDAClaw, Ensign, Custom
- **Universe** — the folder on disk
- **Port** — connection to outside (Telegram, HTTP, MQTT, etc.)
- **TileStore** — where all tiles live (persisted to disk)
- **Tile** — the fundamental unit (everything is tiles)
- **TileFilter** — query tiles by room, type, status, etc.
- **TileIndex** — fast lookup indices
- **Allowance** — API rate limits and budgets
- **ShellConfig** — shell configuration

## Key Insight

This is the bare construct. No rooms, no ensigns, no APIs. Just the kernel. Hermes bootstraps by adding rooms, deploying ensigns, connecting ports. ZeroClaws are child shells in their own sandboxes. The shell can be cloned, decomposed, and recomposed. Everything is tiles. Everything is logged. The universe is just a folder.
