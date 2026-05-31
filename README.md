# lau-shell-kernel

The bare shell kernel. Clone it, get an empty construct. No rooms, no ensigns, no APIs — just the shell.

This is the foundation everything else builds on: `ShellId`, `Universe`, `TileStore`, `Port`, `Allowance`. A shell is a container that holds tiles, exposes ports, manages an energy budget, and can spawn child shells.

## The concept in 60 seconds

A **shell** is an isolated compute boundary. It has an ID, a kind (Hermes, ZeroClaw, CUDAClaw, Ensign), a set of tiles (data), ports (I/O channels), and an energy allowance. Shells can spawn child shells, forming a tree.

The **Universe** is the top-level container that tracks all shells. Think of it as the process tree of your agent system — each shell is a process, tiles are its memory, ports are its file descriptors.

## Quick start

```rust
use lau_shell_kernel::*;

// Create a universe
let mut universe = Universe::new();

// Spawn a root shell
let root = universe.spawn(ShellKind::Hermes, "main-shell");

// Create a tile and store it
let tile = Tile::new(TileType::Memory, "configuration data")
    .with_room("bridge");
universe.tile_store.insert(tile);

// Open a port on the shell
let port = Port::new("sensor-feed", PortDirection::Inbound)
    .with_protocol(PortProtocol::Stream)
    .with_deadband(PortDeadband::new(0.1, 1.0));
universe.open_port(&root, port).unwrap();

// Check the energy allowance
let allowance = Allowance::new(1000.0);
assert!(allowance.can_spend(500.0));
allowance.spend(200.0);
assert!(allowance.is_conserved());

// Spawn a child shell
let child = universe.spawn(ShellKind::ZeroClaw, "worker-1");
universe.set_parent(&child, &root);
```

## Key types

| Type | What it does |
|------|-------------|
| `ShellId` | Unique identifier for a shell |
| `ShellKind` | Hermes, ZeroClaw, CUDAClaw, Ensign, Custom |
| `ShellKernel` | The shell itself: tiles, ports, energy, children |
| `ShellConfig` | Configuration for spawning shells |
| `Universe` | Top-level container tracking all shells |
| `Tile` | A data unit with type, status, room, metadata |
| `TileStore` | In-memory tile storage with indexing |
| `Port` | Named I/O channel with protocol and deadband |
| `PortDeadband` | Threshold configuration for port signaling |
| `Allowance` | Energy budget tracker with conservation |

## Tiles

Tiles are the memory units inside a shell:

```rust
let tile = Tile::new(TileType::Memory, "some content")
    .with_room("bridge")
    .with_status(TileStatus::Active);

// Filter and query
let filter = TileFilter::new()
    .with_type(TileType::Memory)
    .in_room("bridge");
let results = tile_store.query(&filter);
```

## Ports

Ports are the I/O boundary:

```rust
let port = Port::new("cmd", PortDirection::Outbound)
    .with_protocol(PortProtocol::Datagram)
    .with_deadband(PortDeadband::new(0.05, 0.5));
```

## Tick lifecycle

```rust
let tick_result = shell.tick();
// tick_result contains: processed ports, energy spent, tiles changed
```

## Contributing

PRs welcome. This crate is part of the [SuperInstance](https://github.com/SuperInstance) ecosystem. The kernel is intentionally minimal — higher-level behaviors belong in the crates that build on top of it.
