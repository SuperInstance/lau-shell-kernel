# lau-shell-kernel

**THE bare shell kernel.** Clone it, get an empty construct. No rooms, no ensigns, no APIs — just the shell.

```rust
use lau_shell_kernel::*;

let mut shell = ShellKernel::new(ShellKind::Hermes, "/tmp/my-shell");
shell.bootstrap()?;                          // create universe dirs
let tile = shell.tile(TileType::Observation, "I exist")?;
let child = shell.spawn_child(ShellKind::ZeroClaw, "worker-1")?;
```

## 30-Second Quickstart

```
1. new       → ShellKernel::new(kind, path)    // empty shell
2. bootstrap → shell.bootstrap()               // create universe on disk
3. spawn     → shell.spawn_child(kind, "name") // sandboxed child
```

That's it. You now have a hermes-construct with a child sandbox.

## The Sandbox Model

**Folder = Universe.** Every shell lives in one directory. Children are subfolders.

```
/tmp/my-shell/          ← Hermes shell
├── rooms/
├── tiles/
├── ensigns/
├── ports/
├── children/
│   └── worker-1/       ← ZeroClaw (sandboxed)
│       ├── rooms/
│       └── ...
└── shell.json
```

Sandboxed shells (`ZeroClaw`, `CUDAClaw`) can only access their own directory. Hermes can access everything.

## Core Concepts

| Concept | What it is |
|---------|-----------|
| **Tile** | The fundamental unit. Observations, thoughts, actions, artifacts. Everything is tiles. |
| **Universe** | A folder on disk. One shell = one universe. |
| **Port** | A connection to the outside world (Telegram, HTTP, MQTT, GPIO). |
| **Allowance** | API budget and rate limits per service. |
| **ShellKind** | Hermes (full), ZeroClaw (sandboxed), CUDAClaw (GPU sandbox), Ensign, Custom. |

## Tile Lifecycle

```rust
let mut t = shell.tile(TileType::Action, "deploy model")?;
t.complete();     // done
t.archive();      // filed away
t.escalate("stuck"); // help needed
```

## Filtering & Queries

```rust
let recent = shell.query(
    TileFilter::new()
        .room("engineering")
        .tile_type(TileType::Observation)
        .since_tick(1000)
);
```

## Tests

**73 tests** — full coverage of shell lifecycle, tile CRUD, port management, child spawning, conservation budget, serialization roundtrips.

## Ecosystem

- **[lau-shell-kernel]** (this) — bare construct
- [lau-tile-store] — SQLite-backed tile persistence
- [lau-provider] — LLM provider abstraction with budget tracking
- [lau-git-agent] — repo-as-agent with git refs as state machine
- [lau-git-render] — multi-format rendering (terminal, dashboard, game engine, Telegram)

## License

MIT
