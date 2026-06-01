# lau-shell-kernel

**The bare shell kernel — the hermes-construct in Rust form. An empty shell with no rooms, no ensigns, no APIs. The construct that can be cloned and decomposed by Hermes.**

`lau-shell-kernel` is the foundational runtime for agent shells in the PLATO ecosystem. A `ShellKernel` is a self-contained unit of agency: it has an identity, a filesystem-backed universe, a tile-based memory store, communication ports, resource allowances, and the ability to spawn child shells. Everything an agent *does* becomes a `Tile`, everything it *communicates through* is a `Port`, and everything it *spends* is tracked as conservation budget.

## Key Idea

The shell kernel is modeled on the metaphor of a starship or station:

| Concept | Implementation | Description |
|---|---|---|
| **Shell** | `ShellKernel` | A self-contained agent runtime |
| **Universe** | `Universe` | The filesystem directory that holds everything |
| **Tiles** | `Tile`, `TileStore` | The fundamental unit — every thought, observation, action, artifact |
| **Rooms** | Directories in `Universe` | Organizational containers for tiles |
| **Ports** | `Port` | Communication channels (Telegram, HTTP, MQTT, etc.) |
| **Allowances** | `Allowance` | Budgeted API access with rate limits and permissions |
| **Ensigns** | Directories in `Universe` | Sub-agent workers |
| **Children** | `ShellKernel::spawn_child` | Nested shells with parent/child relationships |
| **Conservation** | `conservation_budget` | Energy/budget tracking per tile creation |

The kernel is intentionally minimal — it provides the *structure* (tiles, ports, indexing, persistence) without prescribing *behavior* (that's higher layers).

## Install

Add to your `Cargo.toml`:

```toml
[dependencies]
lau-shell-kernel = "0.1.0"
```

Requires **Rust 2021 edition**. Dependencies:

- [`serde`](https://crates.io/crates/serde) 1 + [`serde_json`](https://crates.io/crates/serde_json) 1 — serialization
- [`tempfile`](https://crates.io/crates/tempfile) 3 (dev only) — test directories

## Quick Start

### Create and bootstrap a shell

```rust
use lau_shell_kernel::{ShellKernel, ShellKind};

let mut kernel = ShellKernel::new(ShellKind::Hermes, "/tmp/my-shell");
kernel.bootstrap().unwrap();

println!("Shell ID: {}", kernel.id);
println!("Kind: {}", kernel.kind);
println!("Tick: {}", kernel.tick);
```

### Create tiles (the fundamental unit)

```rust
use lau_shell_kernel::{TileType, TileFilter, TileStatus};

// Every thought, observation, and action is a tile
let observation = kernel.tile(TileType::Observation, "Saw an anomaly in sector 7").unwrap();
let thought = kernel.tile(TileType::Thought, "This could be a sensor malfunction").unwrap();
let action = kernel.tile(TileType::Action, "Run diagnostic on sensor array").unwrap();

// Create parent-child tile relationships
let sub_action = thought.child(TileType::Action, "Check sensor calibration data");

// Lifecycle
let mut urgent = kernel.tile(TileType::Action, "Respond to alert").unwrap();
urgent.escalate("captain override");
assert_eq!(urgent.status, TileStatus::Escalated);
```

### Query and filter tiles

```rust
// Find all observations
let obs = kernel.query(TileFilter::new().tile_type(TileType::Observation));

// Find tiles in a specific room
let room_tiles = kernel.query(TileFilter::new().room("bridge"));

// Find active tiles since tick 50
let recent = kernel.query(
    TileFilter::new()
        .status(TileStatus::Active)
        .since_tick(50)
);

// Find children of a specific tile
let children = kernel.query(TileFilter::new().parent(&observation.id));
```

### Configure communication ports

```rust
use lau_shell_kernel::{Port, PortProtocol, PortDirection, PortDeadband};

let mut http_port = Port::new("api-in", PortProtocol::Http, PortDirection::Inbound);
http_port.target = "0.0.0.0:8080".into();
http_port.grant_permission("read");
http_port.grant_permission("write");

let mut tg_port = Port::new("telegram", PortProtocol::Telegram, PortDirection::Bidirectional);
tg_port.deadband = Some(PortDeadband::new(0.0, 100.0, 5000));

kernel.add_port(http_port).unwrap();
kernel.add_port(tg_port).unwrap();

// Disable a port
kernel.remove_port("api-in").unwrap();
```

### Manage API allowances

```rust
use lau_shell_kernel::Allowance;

let mut allowance = Allowance::new(&kernel.id.to_string(), "openai");
allowance.budget = 50.0;
allowance.rate_limit = 60;
allowance.grant("chat-completions");
allowance.grant("embeddings");

kernel.grant_allowance(allowance);

// Use the API (tracks budget)
let allow = kernel.allowances.get_mut(&format!("allow-{}-openai", kernel.id)).unwrap();
allow.use_api(0.03).unwrap();
println!("Remaining: ${:.2}", allow.remaining_budget());
```

### Spawn child shells

```rust
// Spawn a sandboxed worker
let child_id = kernel.spawn_child(ShellKind::ZeroClaw, "worker-1").unwrap();
println!("Spawned child: {}", child_id);

// Spawn a GPU worker
let gpu_child = kernel.spawn_child(ShellKind::CUDAClaw, "gpu-worker").unwrap();

// List all children
for child in kernel.list_children() {
    println!("Child shell: {}", child);
}
```

### Tick, status, and shutdown

```rust
// Advance the clock
let result = kernel.advance_tick();
println!("Tick {}: created {} tiles, {} completed",
    kernel.tick, result.tiles_created, result.tiles_completed);

// Get status
let status = kernel.status();
println!("Shell {} ({}) @ tick {}",
    status.shell_id, status.kind, status.uptime_ticks);
println!("Tiles: {}, Rooms: {}, Ports: {} active",
    status.tiles, status.rooms, status.ports_active);
println!("Conservation: {:.1}/{:.1} remaining",
    status.conservation_remaining, kernel.conservation_budget);

// Graceful shutdown (flushes tiles, disables ports)
kernel.shutdown().unwrap();
```

### Persistence

```rust
// Tiles are persisted as JSON in the universe
kernel.tile_store.flush().unwrap();

// Load tiles back
let mut kernel2 = ShellKernel::new(ShellKind::Hermes, "/tmp/my-shell");
kernel2.tile_store.load().unwrap();
println!("Loaded {} tiles", kernel2.tile_store.count());
```

## API Reference

### Shell Kernel

| Method | Description |
|---|---|
| `ShellKernel::new(kind, path)` | Create empty shell |
| `.bootstrap()` | Create universe directories and write config |
| `.tile(type, content)` | Create a tile (costs conservation budget) |
| `.query(filter)` | Query tiles with filter |
| `.add_port(port)` | Add communication port |
| `.remove_port(id)` | Remove port |
| `.grant_allowance(allowance)` | Grant API budget |
| `.spawn_child(kind, name)` | Create nested child shell |
| `.list_children()` | List child shell IDs |
| `.advance_tick()` | Advance clock, return tick result |
| `.status()` | Get shell status snapshot |
| `.shutdown()` | Flush tiles, disable ports |
| `.is_sandboxed()` | Check if shell is sandboxed (ZeroClaw/CUDAClaw) |
| `.can_access(path)` | Check filesystem access within universe |

### Tile

| Field | Description |
|---|---|
| `id` | Auto-generated unique ID |
| `tile_type` | Observation, Action, Thought, Delegation, Escalation, Artifact, System |
| `status` | Active, Complete, Deadband, Escalated, Archived |
| `content` | Text + optional binary data with MIME type |
| `room_id` | Which room this tile belongs to |
| `parent_id` | Parent tile (for hierarchical reasoning) |
| `children` | Child tile IDs |
| `created_tick` / `updated_tick` | When created/updated |
| `deadband` | Optional deadband with trend tracking |
| `ensign_id` | Which ensign produced this tile |
| `model_used` / `tokens_used` | Model usage tracking |
| `conservation_delta` | Energy cost of this tile |
| `metadata` | Arbitrary key-value metadata |

| Method | Description |
|---|---|
| `Tile::new(type, content)` | Create at tick 0 |
| `Tile::with_tick(type, content, tick)` | Create at specific tick |
| `.child(type, content)` | Create child tile |
| `.complete()` | Mark as Complete |
| `.archive()` | Mark as Archived |
| `.escalate(reason)` | Mark as Escalated with reason |
| `.is_in_deadband()` | Check deadband status |
| `.describe()` | One-line description |

### TileFilter

Builder pattern for querying tiles:

```rust
TileFilter::new()
    .room("bridge")
    .tile_type(TileType::Observation)
    .status(TileStatus::Active)
    .ensign("worker-1")
    .since_tick(100)
    .until_tick(200)
    .parent("tile-42")
```

### TileStore

| Method | Description |
|---|---|
| `.store(tile)` | Add tile to store + index |
| `.get(id)` | Lookup by ID |
| `.get_mut(id)` | Mutable lookup |
| `.query(filter)` | Filter with TileFilter |
| `.children_of(id)` | Get child tiles |
| `.room_tiles(room)` | Get tiles in a room |
| `.recent(n)` | N most recent tiles |
| `.count()` | Total tile count |
| `.flush()` | Persist all tiles to disk as JSON |
| `.load()` | Load tiles from disk |

### Port

| Field | Description |
|---|---|
| `id` | Port identifier |
| `protocol` | Telegram, Web, WebSocket, HTTP, MQTT, Serial, GPIO, Custom |
| `direction` | Inbound, Outbound, Bidirectional |
| `target` | Connection target address |
| `permissions` | Granted permissions |
| `enabled` | Active status |
| `deadband` | Optional deadband monitoring |

### Allowance

| Field | Description |
|---|---|
| `api` | API name (e.g., "openai") |
| `rate_limit` | Max requests per interval |
| `budget` / `budget_used` | Spending tracking |
| `permissions` | Granted API capabilities |
| `expires` | Optional expiration tick |

### ShellConfig

| Field | Default | Minimal | Hermes |
|---|---|---|---|
| `max_tiles` | 100,000 | 1,000 | 1,000,000 |
| `max_rooms` | 100 | 5 | 1,000 |
| `max_ensigns` | 50 | 3 | 500 |
| `max_child_shells` | 20 | 3 | 100 |
| `conservation_limit` | 1,000 | 100 | 10,000 |
| `log_level` | info | warn | debug |

### ShellKind

| Variant | Sandboxed | Description |
|---|---|---|
| `Hermes` | No | Full privileged shell |
| `ZeroClaw` | Yes | Sandboxed compute worker |
| `CUDAClaw` | Yes | GPU-accelerated sandboxed worker |
| `Ensign` | No | Sub-agent worker |
| `Custom(name)` | No | User-defined type |

### Universe (Filesystem Layout)

```
<universe_path>/
├── shell.json          # Shell configuration
├── state.json          # Runtime state
├── rooms/              # Room directories
│   ├── bridge/
│   └── engineering/
├── tiles/              # Tile JSON files
│   ├── tile-1.json
│   └── tile-2.json
├── ensigns/            # Ensign (sub-agent) directories
│   └── worker-1/
├── ports/              # Port configuration JSON
│   └── telegram.json
└── children/           # Child shells
    └── worker-1/
```

## How It Works

### Tile-Based Memory

Everything is a tile. Observations, thoughts, actions, delegations, escalations, artifacts, and system events are all represented as `Tile` structs. Tiles form a DAG via `parent_id`/`children` relationships. This gives:

- **Provenance**: Every action traces back to the observation that triggered it
- **Auditability**: The complete reasoning chain is preserved
- **Queryability**: Multi-index queries via `TileIndex` (by room, type, status, parent, ensign, tick)

### Tile Index

`TileIndex` maintains multiple hash map indices plus a sorted tick-index:

```
by_room:    HashMap<room_id, Vec<tile_id>>
by_type:    HashMap<tile_type, Vec<tile_id>>
by_status:  HashMap<status, Vec<tile_id>>
by_parent:  HashMap<parent_id, Vec<tile_id>>
by_ensign:  HashMap<ensign_id, Vec<tile_id>>
by_tick:    Vec<(tick, tile_id)>  // sorted by tick
```

`TileFilter.matches()` checks each constraint; combined with the index, this enables efficient multi-dimensional queries.

### Conservation Budget

Tile creation costs conservation budget (default 0.01 per tile). This prevents runaway tile creation and forces the agent to be mindful of resource usage. When the budget is exhausted, `kernel.tile()` returns an error.

### Child Shells

`spawn_child` creates a new `ShellKernel` in a subdirectory with the parent's tick as the starting tick. Children inherit the parent's shell ID as `parent_shell`. The parent tracks children in `child_shells`. This enables hierarchical agent architectures:

```
Hermes (root)
├── ZeroClaw worker-1 (sandboxed)
├── ZeroClaw worker-2 (sandboxed)
└── CUDAClaw gpu-worker (sandboxed, GPU)
```

### Filesystem Persistence

`TileStore.flush()` writes each tile as a JSON file in `tiles/`. `TileStore.load()` reads them back. The universe directory structure is created by `Universe::create()` and validated by `Universe::validate()`.

### Deadband Monitoring

Tiles and ports support deadband monitoring via `TileDeadband` and `PortDeadband`:

```rust
pub struct TileDeadband {
    lower: f64,
    upper: f64,
    current: f64,
    trend: DeadbandTrend,  // Stable, Drifting(rate), Oscillating(amplitude), Diverging
}
```

A tile in deadband (current within [lower, upper]) doesn't need attention. The trend field tracks whether the value is stable, drifting, oscillating, or diverging.

## The Math

This is primarily a systems/data-structure crate. The quantitative elements:

### Tile ID Generation

Atomic counter with relaxed ordering:

```
tile_id = "tile-{N}"  where N = fetch_add(1, Relaxed)
```

### Conservation Budget

```
conservation_used += cost_per_tile  (default 0.01)
reject if conservation_used + cost > conservation_budget
```

### Disk Usage

Recursive directory size computation:

```
size(dir) = Σ file_size(file) + Σ size(subdir)
```

### Betti Curve (in persistence)

Tracks how dimensions of H⁰ and H¹ change across filtration values — not directly in this crate but supported by the tick-based temporal structure.

## Test Suite

**55+ tests** covering:

- ShellId: creation, display, clone/eq/hash, uniqueness
- ShellKind: hermes/sandboxed checks, labels, display
- Universe: create/exists, paths, validate, list operations, disk usage
- Port: creation, enable/disable, permissions, describe, deadband
- Tile: creation, children, lifecycle (complete/archive/escalate), deadband, describe
- TileType and TileStatus display
- TileFilter: type/room/status/tick/parent/ensign matching, combined filters
- TileIndex: index, remove, rebuild
- TileStore: store/get, query, children_of, recent, flush/load persistence
- Allowance: creation, use/exhaust, permissions, expiration
- ShellConfig: default, minimal, hermes presets
- ShellKernel: new/empty, bootstrap, ports, allowances, children, tiles, queries, ticks, status, sandbox, access control, shutdown, max tiles enforcement, conservation budget, serialization

Run with:

```bash
cargo test
```

## License

MIT
