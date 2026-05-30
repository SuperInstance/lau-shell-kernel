//! lau-shell-kernel — the bare shell kernel, the hermes-construct in Rust form.
//!
//! An empty shell with no rooms, no ensigns, no APIs.
//! The construct that can be cloned and decomposed by Hermes.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

// ---------------------------------------------------------------------------
// ShellId
// ---------------------------------------------------------------------------

/// Unique identifier for a shell.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ShellId(pub String);

impl ShellId {
    pub fn new(id: &str) -> Self {
        Self(id.to_string())
    }

    pub fn generate() -> Self {
        Self(format!(
            "shell-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ShellId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ---------------------------------------------------------------------------
// ShellKind
// ---------------------------------------------------------------------------

/// What kind of shell this is.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShellKind {
    Hermes,
    ZeroClaw,
    CUDAClaw,
    Ensign,
    Custom(String),
}

impl ShellKind {
    pub fn is_hermes(&self) -> bool {
        matches!(self, ShellKind::Hermes)
    }

    pub fn is_sandboxed(&self) -> bool {
        matches!(self, ShellKind::ZeroClaw | ShellKind::CUDAClaw)
    }

    pub fn label(&self) -> String {
        match self {
            ShellKind::Hermes => "Hermes".to_string(),
            ShellKind::ZeroClaw => "ZeroClaw".to_string(),
            ShellKind::CUDAClaw => "CUDAClaw".to_string(),
            ShellKind::Ensign => "Ensign".to_string(),
            ShellKind::Custom(s) => s.clone(),
        }
    }
}

impl std::fmt::Display for ShellKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

// ---------------------------------------------------------------------------
// Universe
// ---------------------------------------------------------------------------

/// The folder on disk that holds everything for a shell.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Universe {
    pub path: String,
    pub shell_id: ShellId,
    pub rooms_dir: String,
    pub tiles_dir: String,
    pub ensigns_dir: String,
    pub ports_dir: String,
    pub config_path: String,
    pub state_path: String,
}

impl Universe {
    pub fn new(path: &str, shell_id: &ShellId) -> Self {
        let base = path.to_string();
        Self {
            rooms_dir: format!("{}/rooms", base),
            tiles_dir: format!("{}/tiles", base),
            ensigns_dir: format!("{}/ensigns", base),
            ports_dir: format!("{}/ports", base),
            config_path: format!("{}/shell.json", base),
            state_path: format!("{}/state.json", base),
            path: base,
            shell_id: shell_id.clone(),
        }
    }

    pub fn create(&self) -> Result<(), String> {
        for dir in &[
            &self.path,
            &self.rooms_dir,
            &self.tiles_dir,
            &self.ensigns_dir,
            &self.ports_dir,
        ] {
            fs::create_dir_all(dir).map_err(|e| format!("mkdir {}: {}", dir, e))?;
        }
        Ok(())
    }

    pub fn exists(&self) -> bool {
        Path::new(&self.path).is_dir()
    }

    pub fn room_path(&self, room_id: &str) -> String {
        format!("{}/{}", self.rooms_dir, room_id)
    }

    pub fn tile_path(&self, tile_id: &str) -> String {
        format!("{}/{}.json", self.tiles_dir, tile_id)
    }

    pub fn ensign_path(&self, ensign_id: &str) -> String {
        format!("{}/{}", self.ensigns_dir, ensign_id)
    }

    pub fn port_path(&self, port_id: &str) -> String {
        format!("{}/{}.json", self.ports_dir, port_id)
    }

    pub fn list_rooms(&self) -> Vec<String> {
        list_dir_names(&self.rooms_dir)
    }

    pub fn list_ensigns(&self) -> Vec<String> {
        list_dir_names(&self.ensigns_dir)
    }

    pub fn list_ports(&self) -> Vec<String> {
        list_dir_json_names(&self.ports_dir)
    }

    pub fn disk_usage(&self) -> u64 {
        dir_size(Path::new(&self.path))
    }

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        for (name, dir) in &[
            ("root", &self.path),
            ("rooms", &self.rooms_dir),
            ("tiles", &self.tiles_dir),
            ("ensigns", &self.ensigns_dir),
            ("ports", &self.ports_dir),
        ] {
            if !Path::new(dir).is_dir() {
                errors.push(format!("missing {} directory: {}", name, dir));
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

fn list_dir_names(dir: &str) -> Vec<String> {
    fs::read_dir(dir)
        .ok()
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir() || e.path().extension().is_none())
                .filter_map(|e| e.file_name().to_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

fn list_dir_json_names(dir: &str) -> Vec<String> {
    fs::read_dir(dir)
        .ok()
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter_map(|e| {
                    let name = e.file_name().to_str()?.to_string();
                    if name.ends_with(".json") {
                        Some(name.trim_end_matches(".json").to_string())
                    } else {
                        Some(name)
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}

fn dir_size(path: &Path) -> u64 {
    if !path.exists() {
        return 0;
    }
    let mut total = 0u64;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                total += dir_size(&p);
            } else if let Ok(meta) = p.metadata() {
                total += meta.len();
            }
        }
    }
    total
}

// ---------------------------------------------------------------------------
// Port
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PortDirection {
    Inbound,
    Outbound,
    Bidirectional,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PortProtocol {
    Telegram,
    Web,
    WebSocket,
    Http,
    Mqtt,
    Serial,
    Gpio,
    Custom(String),
}

impl std::fmt::Display for PortProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PortProtocol::Telegram => write!(f, "Telegram"),
            PortProtocol::Web => write!(f, "Web"),
            PortProtocol::WebSocket => write!(f, "WebSocket"),
            PortProtocol::Http => write!(f, "Http"),
            PortProtocol::Mqtt => write!(f, "Mqtt"),
            PortProtocol::Serial => write!(f, "Serial"),
            PortProtocol::Gpio => write!(f, "Gpio"),
            PortProtocol::Custom(s) => write!(f, "Custom({})", s),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortDeadband {
    pub lower: f64,
    pub upper: f64,
    pub current: f64,
    pub check_interval_ms: u64,
}

impl PortDeadband {
    pub fn new(lower: f64, upper: f64, check_interval_ms: u64) -> Self {
        Self {
            lower,
            upper,
            current: lower,
            check_interval_ms,
        }
    }

    pub fn is_in_band(&self) -> bool {
        self.current >= self.lower && self.current <= self.upper
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Port {
    pub id: String,
    pub direction: PortDirection,
    pub protocol: PortProtocol,
    pub target: String,
    pub permissions: Vec<String>,
    pub enabled: bool,
    pub deadband: Option<PortDeadband>,
}

impl Port {
    pub fn new(id: &str, protocol: PortProtocol, direction: PortDirection) -> Self {
        Self {
            id: id.to_string(),
            direction,
            protocol,
            target: String::new(),
            permissions: Vec::new(),
            enabled: true,
            deadband: None,
        }
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn grant_permission(&mut self, perm: &str) {
        if !self.permissions.contains(&perm.to_string()) {
            self.permissions.push(perm.to_string());
        }
    }

    pub fn revoke_permission(&mut self, perm: &str) {
        self.permissions.retain(|p| p != perm);
    }

    pub fn has_permission(&self, perm: &str) -> bool {
        self.permissions.contains(&perm.to_string())
    }

    pub fn is_active(&self) -> bool {
        self.enabled
    }

    pub fn describe(&self) -> String {
        let dir = match &self.direction {
            PortDirection::Inbound => "in",
            PortDirection::Outbound => "out",
            PortDirection::Bidirectional => "in/out",
        };
        let state = if self.enabled { "active" } else { "disabled" };
        format!(
            "Port {} [{}] {} → {} ({})",
            self.id, dir, self.protocol, self.target, state
        )
    }
}

// ---------------------------------------------------------------------------
// Tile
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TileType {
    Observation,
    Action,
    Thought,
    Delegation,
    Escalation,
    Artifact,
    System,
}

impl std::fmt::Display for TileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TileType::Observation => write!(f, "Observation"),
            TileType::Action => write!(f, "Action"),
            TileType::Thought => write!(f, "Thought"),
            TileType::Delegation => write!(f, "Delegation"),
            TileType::Escalation => write!(f, "Escalation"),
            TileType::Artifact => write!(f, "Artifact"),
            TileType::System => write!(f, "System"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TileStatus {
    Active,
    Complete,
    Deadband,
    Escalated,
    Archived,
}

impl std::fmt::Display for TileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TileStatus::Active => write!(f, "Active"),
            TileStatus::Complete => write!(f, "Complete"),
            TileStatus::Deadband => write!(f, "Deadband"),
            TileStatus::Escalated => write!(f, "Escalated"),
            TileStatus::Archived => write!(f, "Archived"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileContent {
    pub text: String,
    pub data: Option<Vec<u8>>,
    pub mime_type: Option<String>,
}

impl TileContent {
    pub fn text(text: &str) -> Self {
        Self {
            text: text.to_string(),
            data: None,
            mime_type: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DeadbandTrend {
    Stable,
    Drifting(f64),
    Oscillating(f64),
    Diverging,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileDeadband {
    pub lower: f64,
    pub upper: f64,
    pub current: f64,
    pub trend: DeadbandTrend,
}

impl TileDeadband {
    pub fn new(lower: f64, upper: f64, current: f64) -> Self {
        Self {
            lower,
            upper,
            current,
            trend: DeadbandTrend::Stable,
        }
    }

    pub fn is_in_band(&self) -> bool {
        self.current >= self.lower && self.current <= self.upper
    }
}

/// The fundamental unit — everything is tiles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tile {
    pub id: String,
    pub room_id: Option<String>,
    pub tile_type: TileType,
    pub parent_id: Option<String>,
    pub children: Vec<String>,
    pub created_tick: u64,
    pub updated_tick: u64,
    pub status: TileStatus,
    pub content: TileContent,
    pub deadband: Option<TileDeadband>,
    pub ensign_id: Option<String>,
    pub model_used: Option<String>,
    pub tokens_used: u32,
    pub conservation_delta: f64,
    pub metadata: HashMap<String, String>,
}

static TILE_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

fn next_tile_id() -> String {
    let n = TILE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    format!("tile-{}", n)
}

impl Tile {
    pub fn new(tile_type: TileType, content: &str) -> Self {
        Self::with_tick(tile_type, content, 0)
    }

    pub fn with_tick(tile_type: TileType, content: &str, tick: u64) -> Self {
        Self {
            id: next_tile_id(),
            room_id: None,
            tile_type,
            parent_id: None,
            children: Vec::new(),
            created_tick: tick,
            updated_tick: tick,
            status: TileStatus::Active,
            content: TileContent::text(content),
            deadband: None,
            ensign_id: None,
            model_used: None,
            tokens_used: 0,
            conservation_delta: 0.0,
            metadata: HashMap::new(),
        }
    }

    pub fn child(&self, tile_type: TileType, content: &str) -> Tile {
        let mut child = Tile::with_tick(tile_type, content, self.updated_tick);
        child.parent_id = Some(self.id.clone());
        child.room_id = self.room_id.clone();
        child
    }

    pub fn complete(&mut self) {
        self.status = TileStatus::Complete;
    }

    pub fn archive(&mut self) {
        self.status = TileStatus::Archived;
    }

    pub fn escalate(&mut self, reason: &str) {
        self.status = TileStatus::Escalated;
        self.metadata
            .insert("escalation_reason".to_string(), reason.to_string());
    }

    pub fn is_in_deadband(&self) -> bool {
        self.deadband
            .as_ref()
            .map(|db| db.is_in_band())
            .unwrap_or(false)
    }

    pub fn describe(&self) -> String {
        let room = self
            .room_id
            .as_deref()
            .unwrap_or("no-room");
        let parent = self
            .parent_id
            .as_deref()
            .unwrap_or("none");
        format!(
            "[{}] {} ({}) room={} parent={} children={} status={} \"{}\"",
            self.id,
            self.tile_type,
            self.tile_type,
            room,
            parent,
            self.children.len(),
            self.status,
            if self.content.text.len() > 60 {
                format!("{}...", &self.content.text[..60])
            } else {
                self.content.text.clone()
            }
        )
    }
}

// ---------------------------------------------------------------------------
// TileFilter
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TileFilter {
    pub room_id: Option<String>,
    pub tile_type: Option<TileType>,
    pub status: Option<TileStatus>,
    pub ensign_id: Option<String>,
    pub since_tick: Option<u64>,
    pub until_tick: Option<u64>,
    pub parent_id: Option<String>,
}

impl TileFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn room(mut self, room_id: &str) -> Self {
        self.room_id = Some(room_id.to_string());
        self
    }

    pub fn tile_type(mut self, tile_type: TileType) -> Self {
        self.tile_type = Some(tile_type);
        self
    }

    pub fn status(mut self, status: TileStatus) -> Self {
        self.status = Some(status);
        self
    }

    pub fn ensign(mut self, ensign_id: &str) -> Self {
        self.ensign_id = Some(ensign_id.to_string());
        self
    }

    pub fn since_tick(mut self, tick: u64) -> Self {
        self.since_tick = Some(tick);
        self
    }

    pub fn until_tick(mut self, tick: u64) -> Self {
        self.until_tick = Some(tick);
        self
    }

    pub fn parent(mut self, parent_id: &str) -> Self {
        self.parent_id = Some(parent_id.to_string());
        self
    }

    pub fn matches(&self, tile: &Tile) -> bool {
        if let Some(ref room_id) = self.room_id {
            if tile.room_id.as_deref() != Some(room_id.as_str()) {
                return false;
            }
        }
        if let Some(ref tt) = self.tile_type {
            if tile.tile_type != *tt {
                return false;
            }
        }
        if let Some(ref st) = self.status {
            if tile.status != *st {
                return false;
            }
        }
        if let Some(ref eid) = self.ensign_id {
            if tile.ensign_id.as_deref() != Some(eid.as_str()) {
                return false;
            }
        }
        if let Some(tick) = self.since_tick {
            if tile.created_tick < tick {
                return false;
            }
        }
        if let Some(tick) = self.until_tick {
            if tile.created_tick > tick {
                return false;
            }
        }
        if let Some(ref pid) = self.parent_id {
            if tile.parent_id.as_deref() != Some(pid.as_str()) {
                return false;
            }
        }
        true
    }
}

// ---------------------------------------------------------------------------
// TileIndex
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TileIndex {
    pub by_room: HashMap<String, Vec<String>>,
    pub by_type: HashMap<String, Vec<String>>,
    pub by_status: HashMap<String, Vec<String>>,
    pub by_parent: HashMap<String, Vec<String>>,
    pub by_ensign: HashMap<String, Vec<String>>,
    pub by_tick: Vec<(u64, String)>,
}

impl TileIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn index(&mut self, tile: &Tile) {
        // by_room
        if let Some(ref room_id) = tile.room_id {
            self.by_room
                .entry(room_id.clone())
                .or_default()
                .push(tile.id.clone());
        }
        // by_type
        self.by_type
            .entry(tile.tile_type.to_string())
            .or_default()
            .push(tile.id.clone());
        // by_status
        self.by_status
            .entry(tile.status.to_string())
            .or_default()
            .push(tile.id.clone());
        // by_parent
        if let Some(ref parent_id) = tile.parent_id {
            self.by_parent
                .entry(parent_id.clone())
                .or_default()
                .push(tile.id.clone());
        }
        // by_ensign
        if let Some(ref ensign_id) = tile.ensign_id {
            self.by_ensign
                .entry(ensign_id.clone())
                .or_default()
                .push(tile.id.clone());
        }
        // by_tick — sorted insert
        let entry = (tile.created_tick, tile.id.clone());
        let pos = self.by_tick.binary_search_by(|t| t.0.cmp(&tile.created_tick)).unwrap_or_else(|e| e);
        self.by_tick.insert(pos, entry);
    }

    pub fn remove(&mut self, tile: &Tile) {
        if let Some(ref room_id) = tile.room_id {
            if let Some(v) = self.by_room.get_mut(room_id) {
                v.retain(|id| id != &tile.id);
            }
        }
        if let Some(v) = self.by_type.get_mut(&tile.tile_type.to_string()) {
            v.retain(|id| id != &tile.id);
        }
        if let Some(v) = self.by_status.get_mut(&tile.status.to_string()) {
            v.retain(|id| id != &tile.id);
        }
        if let Some(ref parent_id) = tile.parent_id {
            if let Some(v) = self.by_parent.get_mut(parent_id) {
                v.retain(|id| id != &tile.id);
            }
        }
        if let Some(ref ensign_id) = tile.ensign_id {
            if let Some(v) = self.by_ensign.get_mut(ensign_id) {
                v.retain(|id| id != &tile.id);
            }
        }
        self.by_tick.retain(|(_, id)| id != &tile.id);
    }

    pub fn rebuild(&mut self, tiles: &HashMap<String, Tile>) {
        *self = Self::new();
        for tile in tiles.values() {
            self.index(tile);
        }
    }
}

// ---------------------------------------------------------------------------
// TileStore
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileStore {
    pub universe: Universe,
    pub tiles: HashMap<String, Tile>,
    pub index: TileIndex,
}

impl TileStore {
    pub fn new(universe: &Universe) -> Self {
        Self {
            universe: universe.clone(),
            tiles: HashMap::new(),
            index: TileIndex::new(),
        }
    }

    pub fn store(&mut self, tile: Tile) -> Result<(), String> {
        self.index.index(&tile);
        self.tiles.insert(tile.id.clone(), tile);
        Ok(())
    }

    pub fn get(&self, tile_id: &str) -> Option<&Tile> {
        self.tiles.get(tile_id)
    }

    pub fn get_mut(&mut self, tile_id: &str) -> Option<&mut Tile> {
        self.tiles.get_mut(tile_id)
    }

    pub fn query(&self, filter: TileFilter) -> Vec<&Tile> {
        self.tiles
            .values()
            .filter(|t| filter.matches(t))
            .collect()
    }

    pub fn children_of(&self, tile_id: &str) -> Vec<&Tile> {
        self.query(TileFilter::new().parent(tile_id))
    }

    pub fn room_tiles(&self, room_id: &str) -> Vec<&Tile> {
        self.query(TileFilter::new().room(room_id))
    }

    pub fn recent(&self, n: usize) -> Vec<&Tile> {
        let mut tiles: Vec<&Tile> = self.tiles.values().collect();
        tiles.sort_by_key(|t| std::cmp::Reverse(t.created_tick));
        tiles.truncate(n);
        tiles
    }

    pub fn count(&self) -> usize {
        self.tiles.len()
    }

    pub fn flush(&self) -> Result<(), String> {
        fs::create_dir_all(&self.universe.tiles_dir)
            .map_err(|e| format!("create tiles dir: {}", e))?;
        for tile in self.tiles.values() {
            let path = self.universe.tile_path(&tile.id);
            let json = serde_json::to_string_pretty(tile)
                .map_err(|e| format!("serialize tile {}: {}", tile.id, e))?;
            fs::write(&path, json)
                .map_err(|e| format!("write tile {}: {}", tile.id, e))?;
        }
        Ok(())
    }

    pub fn load(&mut self) -> Result<(), String> {
        let dir = Path::new(&self.universe.tiles_dir);
        if !dir.is_dir() {
            return Ok(());
        }
        self.tiles.clear();
        self.index = TileIndex::new();
        for entry in fs::read_dir(dir).map_err(|e| format!("read tiles dir: {}", e))? {
            let entry = entry.map_err(|e| format!("read entry: {}", e))?;
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                let data = fs::read_to_string(&path)
                    .map_err(|e| format!("read {}: {}", path.display(), e))?;
                let tile: Tile = serde_json::from_str(&data)
                    .map_err(|e| format!("parse {}: {}", path.display(), e))?;
                self.index.index(&tile);
                self.tiles.insert(tile.id.clone(), tile);
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Allowance
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Allowance {
    pub id: String,
    pub shell_id: String,
    pub api: String,
    pub rate_limit: u32,
    pub budget: f64,
    pub budget_used: f64,
    pub permissions: Vec<String>,
    pub expires: Option<u64>,
}

impl Allowance {
    pub fn new(shell_id: &str, api: &str) -> Self {
        Self {
            id: format!("allow-{}-{}", shell_id, api),
            shell_id: shell_id.to_string(),
            api: api.to_string(),
            rate_limit: 100,
            budget: 100.0,
            budget_used: 0.0,
            permissions: Vec::new(),
            expires: None,
        }
    }

    pub fn use_api(&mut self, cost: f64) -> Result<(), String> {
        if self.remaining_budget() < cost {
            return Err(format!(
                "budget exceeded: remaining {} < cost {}",
                self.remaining_budget(),
                cost
            ));
        }
        self.budget_used += cost;
        Ok(())
    }

    pub fn grant(&mut self, perm: &str) {
        if !self.permissions.contains(&perm.to_string()) {
            self.permissions.push(perm.to_string());
        }
    }

    pub fn revoke(&mut self, perm: &str) {
        self.permissions.retain(|p| p != perm);
    }

    pub fn is_expired(&self, current_tick: u64) -> bool {
        self.expires.map(|e| current_tick >= e).unwrap_or(false)
    }

    pub fn remaining_budget(&self) -> f64 {
        self.budget - self.budget_used
    }
}

// ---------------------------------------------------------------------------
// ShellConfig
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellConfig {
    pub max_tiles: u32,
    pub max_rooms: u32,
    pub max_ensigns: u32,
    pub max_child_shells: u32,
    pub default_deadband_tolerance: f64,
    pub conservation_limit: f64,
    pub log_level: String,
}

impl ShellConfig {
    pub fn default_config() -> Self {
        Self {
            max_tiles: 100_000,
            max_rooms: 100,
            max_ensigns: 50,
            max_child_shells: 20,
            default_deadband_tolerance: 0.05,
            conservation_limit: 1000.0,
            log_level: "info".to_string(),
        }
    }

    pub fn minimal() -> Self {
        Self {
            max_tiles: 1_000,
            max_rooms: 5,
            max_ensigns: 3,
            max_child_shells: 3,
            default_deadband_tolerance: 0.1,
            conservation_limit: 100.0,
            log_level: "warn".to_string(),
        }
    }

    pub fn hermes() -> Self {
        Self {
            max_tiles: 1_000_000,
            max_rooms: 1000,
            max_ensigns: 500,
            max_child_shells: 100,
            default_deadband_tolerance: 0.01,
            conservation_limit: 10000.0,
            log_level: "debug".to_string(),
        }
    }
}

impl Default for ShellConfig {
    fn default() -> Self {
        Self::default_config()
    }
}

// ---------------------------------------------------------------------------
// ShellTickResult
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellTickResult {
    pub tiles_created: u32,
    pub tiles_completed: u32,
    pub conservation_delta: f64,
    pub active_ports: u32,
    pub children_active: u32,
}

// ---------------------------------------------------------------------------
// ShellStatus
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellStatus {
    pub shell_id: String,
    pub kind: String,
    pub autonomy_level: u32,
    pub rooms: u32,
    pub ensigns: u32,
    pub tiles: u32,
    pub ports_active: u32,
    pub children: u32,
    pub conservation_remaining: f64,
    pub disk_usage: u64,
    pub uptime_ticks: u64,
}

// ---------------------------------------------------------------------------
// ShellKernel
// ---------------------------------------------------------------------------

/// THE kernel — the hermes-construct. Empty shell, no rooms, no ensigns, no APIs.
#[derive(Debug, Serialize, Deserialize)]
pub struct ShellKernel {
    pub id: ShellId,
    pub kind: ShellKind,
    pub universe: Universe,
    pub tile_store: TileStore,
    pub ports: HashMap<String, Port>,
    pub allowances: HashMap<String, Allowance>,
    pub conservation_budget: f64,
    pub conservation_used: f64,
    pub autonomy_level: u32,
    pub parent_shell: Option<ShellId>,
    pub child_shells: Vec<ShellId>,
    pub tick: u64,
    pub config: ShellConfig,
}

impl ShellKernel {
    /// Create a new empty shell. No rooms, no ensigns, no APIs.
    pub fn new(kind: ShellKind, path: &str) -> Self {
        let id = ShellId::generate();
        let universe = Universe::new(path, &id);
        Self {
            tile_store: TileStore::new(&universe),
            id,
            kind,
            universe,
            ports: HashMap::new(),
            allowances: HashMap::new(),
            conservation_budget: 1000.0,
            conservation_used: 0.0,
            autonomy_level: 1,
            parent_shell: None,
            child_shells: Vec::new(),
            tick: 0,
            config: ShellConfig::default_config(),
        }
    }

    /// Create universe dirs, initialize.
    pub fn bootstrap(&mut self) -> Result<(), String> {
        self.universe.create()?;
        // Write config
        let config_json = serde_json::to_string_pretty(&self.config)
            .map_err(|e| format!("serialize config: {}", e))?;
        fs::write(&self.universe.config_path, config_json)
            .map_err(|e| format!("write config: {}", e))?;
        Ok(())
    }

    pub fn add_port(&mut self, port: Port) -> Result<(), String> {
        if self.ports.contains_key(&port.id) {
            return Err(format!("port {} already exists", port.id));
        }
        self.ports.insert(port.id.clone(), port);
        Ok(())
    }

    pub fn remove_port(&mut self, port_id: &str) -> Result<(), String> {
        self.ports
            .remove(port_id)
            .ok_or_else(|| format!("port {} not found", port_id))?;
        Ok(())
    }

    pub fn grant_allowance(&mut self, allowance: Allowance) {
        self.allowances.insert(allowance.id.clone(), allowance);
    }

    /// Create a child shell in a subfolder.
    pub fn spawn_child(&mut self, kind: ShellKind, name: &str) -> Result<ShellId, String> {
        if self.child_shells.len() as u32 >= self.config.max_child_shells {
            return Err("max child shells reached".to_string());
        }
        let child_path = format!("{}/children/{}", self.universe.path, name);
        let mut child = ShellKernel::new(kind, &child_path);
        child.parent_shell = Some(self.id.clone());
        child.tick = self.tick;
        child.bootstrap()?;
        let child_id = child.id.clone();
        self.child_shells.push(child_id.clone());
        Ok(child_id)
    }

    pub fn list_children(&self) -> Vec<ShellId> {
        self.child_shells.clone()
    }

    /// Create a tile in this shell.
    pub fn tile(&mut self, tile_type: TileType, content: &str) -> Result<Tile, String> {
        if self.tile_store.count() as u32 >= self.config.max_tiles {
            return Err("max tiles reached".to_string());
        }
        let mut tile = Tile::with_tick(tile_type, content, self.tick);
        let cost = 0.01; // base cost per tile
        if self.conservation_used + cost > self.conservation_budget {
            return Err("conservation budget exceeded".to_string());
        }
        self.conservation_used += cost;
        tile.conservation_delta = cost;
        self.tile_store.store(tile.clone())?;
        Ok(tile)
    }

    pub fn query(&self, filter: TileFilter) -> Vec<&Tile> {
        self.tile_store.query(filter)
    }

    /// Advance one tick.
    pub fn advance_tick(&mut self) -> ShellTickResult {
        let tiles_before = self.tile_store.count();
        self.tick += 1;

        // Count completed tiles
        let completed = self
            .tile_store
            .tiles
            .values()
            .filter(|t| t.status == TileStatus::Complete)
            .count() as u32;

        let created = self.tile_store.count() as u32 - tiles_before as u32;
        let active_ports = self.ports.values().filter(|p| p.enabled).count() as u32;

        ShellTickResult {
            tiles_created: created,
            tiles_completed: completed,
            conservation_delta: self.conservation_used,
            active_ports,
            children_active: self.child_shells.len() as u32,
        }
    }

    pub fn status(&self) -> ShellStatus {
        ShellStatus {
            shell_id: self.id.to_string(),
            kind: self.kind.label(),
            autonomy_level: self.autonomy_level,
            rooms: self.universe.list_rooms().len() as u32,
            ensigns: self.universe.list_ensigns().len() as u32,
            tiles: self.tile_store.count() as u32,
            ports_active: self.ports.values().filter(|p| p.enabled).count() as u32,
            children: self.child_shells.len() as u32,
            conservation_remaining: self.conservation_budget - self.conservation_used,
            disk_usage: self.universe.disk_usage(),
            uptime_ticks: self.tick,
        }
    }

    pub fn is_sandboxed(&self) -> bool {
        self.kind.is_sandboxed()
    }

    pub fn can_access(&self, path: &str) -> bool {
        let canonical_universe = std::path::Path::new(&self.universe.path)
            .canonicalize()
            .ok();
        let canonical_path = std::path::Path::new(path).canonicalize().ok();

        match (canonical_universe, canonical_path) {
            (Some(u), Some(p)) => p.starts_with(&u),
            _ => {
                // Fallback: string prefix check
                path.starts_with(&self.universe.path)
            }
        }
    }

    pub fn shutdown(&mut self) -> Result<(), String> {
        self.tile_store.flush()?;
        // Disable all ports
        for port in self.ports.values_mut() {
            port.disable();
        }
        Ok(())
    }
}

// ===========================================================================
// TESTS
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn tmp() -> TempDir {
        tempfile::tempdir().unwrap()
    }

    fn kernel_path(tmp: &TempDir, name: &str) -> String {
        format!("{}/{}", tmp.path().display(), name)
    }

    // --- ShellId ---

    #[test]
    fn shell_id_new() {
        let id = ShellId::new("test-1");
        assert_eq!(id.0, "test-1");
        assert_eq!(id.as_str(), "test-1");
    }

    #[test]
    fn shell_id_display() {
        let id = ShellId::new("abc");
        assert_eq!(format!("{}", id), "abc");
    }

    #[test]
    fn shell_id_clone_eq_hash() {
        let a = ShellId::new("x");
        let b = a.clone();
        assert_eq!(a, b);
        let mut hm = std::collections::HashSet::new();
        hm.insert(a.clone());
        assert!(hm.contains(&b));
    }

    #[test]
    fn shell_id_generate_unique() {
        let a = ShellId::generate();
        let b = ShellId::generate();
        assert_ne!(a, b);
    }

    // --- ShellKind ---

    #[test]
    fn shell_kind_hermes() {
        assert!(ShellKind::Hermes.is_hermes());
        assert!(!ShellKind::Hermes.is_sandboxed());
        assert_eq!(ShellKind::Hermes.label(), "Hermes");
    }

    #[test]
    fn shell_kind_zeroclaw() {
        assert!(!ShellKind::ZeroClaw.is_hermes());
        assert!(ShellKind::ZeroClaw.is_sandboxed());
        assert_eq!(ShellKind::ZeroClaw.label(), "ZeroClaw");
    }

    #[test]
    fn shell_kind_cudaclaw() {
        assert!(ShellKind::CUDAClaw.is_sandboxed());
        assert_eq!(ShellKind::CUDAClaw.label(), "CUDAClaw");
    }

    #[test]
    fn shell_kind_ensign() {
        assert!(!ShellKind::Ensign.is_hermes());
        assert!(!ShellKind::Ensign.is_sandboxed());
        assert_eq!(ShellKind::Ensign.label(), "Ensign");
    }

    #[test]
    fn shell_kind_custom() {
        let c = ShellKind::Custom("MyKind".to_string());
        assert!(!c.is_hermes());
        assert!(!c.is_sandboxed());
        assert_eq!(c.label(), "MyKind");
    }

    // --- Universe ---

    #[test]
    fn universe_create_and_exists() {
        let tmp = tmp();
        let sid = ShellId::new("u1");
        let u = Universe::new(&kernel_path(&tmp, "u1"), &sid);
        assert!(!u.exists());
        u.create().unwrap();
        assert!(u.exists());
    }

    #[test]
    fn universe_paths() {
        let tmp = tmp();
        let sid = ShellId::new("u2");
        let path = kernel_path(&tmp, "u2");
        let u = Universe::new(&path, &sid);
        assert_eq!(u.room_path("r1"), format!("{}/rooms/r1", path));
        assert_eq!(u.tile_path("t1"), format!("{}/tiles/t1.json", path));
        assert_eq!(u.ensign_path("e1"), format!("{}/ensigns/e1", path));
        assert_eq!(u.port_path("p1"), format!("{}/ports/p1.json", path));
    }

    #[test]
    fn universe_validate() {
        let tmp = tmp();
        let sid = ShellId::new("u3");
        let u = Universe::new(&kernel_path(&tmp, "u3"), &sid);
        assert!(u.validate().is_err());
        u.create().unwrap();
        assert!(u.validate().is_ok());
    }

    #[test]
    fn universe_list_empty() {
        let tmp = tmp();
        let sid = ShellId::new("u4");
        let u = Universe::new(&kernel_path(&tmp, "u4"), &sid);
        u.create().unwrap();
        assert!(u.list_rooms().is_empty());
        assert!(u.list_ensigns().is_empty());
        assert!(u.list_ports().is_empty());
    }

    #[test]
    fn universe_disk_usage() {
        let tmp = tmp();
        let sid = ShellId::new("u5");
        let u = Universe::new(&kernel_path(&tmp, "u5"), &sid);
        assert_eq!(u.disk_usage(), 0);
        u.create().unwrap();
        // Empty dirs have 0 bytes, so write a file to test
        fs::write(format!("{}/test.txt", u.path), "hello").unwrap();
        assert!(u.disk_usage() > 0);
    }

    // --- Port ---

    #[test]
    fn port_new_is_active() {
        let p = Port::new("p1", PortProtocol::Http, PortDirection::Inbound);
        assert!(p.is_active());
        assert_eq!(p.id, "p1");
    }

    #[test]
    fn port_enable_disable() {
        let mut p = Port::new("p1", PortProtocol::WebSocket, PortDirection::Bidirectional);
        p.disable();
        assert!(!p.is_active());
        p.enable();
        assert!(p.is_active());
    }

    #[test]
    fn port_permissions() {
        let mut p = Port::new("p1", PortProtocol::Mqtt, PortDirection::Outbound);
        assert!(!p.has_permission("read"));
        p.grant_permission("read");
        assert!(p.has_permission("read"));
        // duplicate grant is no-op
        p.grant_permission("read");
        assert_eq!(p.permissions.len(), 1);
        p.revoke_permission("read");
        assert!(!p.has_permission("read"));
    }

    #[test]
    fn port_describe() {
        let p = Port::new("web-1", PortProtocol::Http, PortDirection::Inbound);
        let desc = p.describe();
        assert!(desc.contains("web-1"));
        assert!(desc.contains("Http"));
        assert!(desc.contains("active"));
    }

    #[test]
    fn port_deadband() {
        let db = PortDeadband::new(0.0, 100.0, 50);
        assert!(db.is_in_band());
        let db2 = PortDeadband::new(0.0, 100.0, 50);
        assert_eq!(db2.check_interval_ms, 50);
    }

    // --- Tile ---

    #[test]
    fn tile_new() {
        let t = Tile::new(TileType::Observation, "saw something");
        assert_eq!(t.tile_type, TileType::Observation);
        assert_eq!(t.content.text, "saw something");
        assert_eq!(t.status, TileStatus::Active);
        assert!(t.room_id.is_none());
        assert!(t.parent_id.is_none());
        assert!(t.children.is_empty());
    }

    #[test]
    fn tile_child() {
        let parent = Tile::new(TileType::Thought, "thinking...");
        let child = parent.child(TileType::Action, "do it");
        assert_eq!(child.parent_id.as_deref(), Some(parent.id.as_str()));
        assert_eq!(child.content.text, "do it");
    }

    #[test]
    fn tile_lifecycle() {
        let mut t = Tile::new(TileType::Action, "act");
        assert_eq!(t.status, TileStatus::Active);
        t.complete();
        assert_eq!(t.status, TileStatus::Complete);
        t.archive();
        assert_eq!(t.status, TileStatus::Archived);
    }

    #[test]
    fn tile_escalate() {
        let mut t = Tile::new(TileType::Action, "help");
        t.escalate("stuck");
        assert_eq!(t.status, TileStatus::Escalated);
        assert_eq!(
            t.metadata.get("escalation_reason"),
            Some(&"stuck".to_string())
        );
    }

    #[test]
    fn tile_deadband() {
        let mut t = Tile::new(TileType::Observation, "temp");
        assert!(!t.is_in_deadband());
        t.deadband = Some(TileDeadband::new(10.0, 20.0, 15.0));
        assert!(t.is_in_deadband());
        t.deadband = Some(TileDeadband::new(10.0, 20.0, 25.0));
        assert!(!t.is_in_deadband());
    }

    #[test]
    fn tile_describe() {
        let t = Tile::new(TileType::System, "boot");
        let desc = t.describe();
        assert!(desc.contains(&t.id));
        assert!(desc.contains("System"));
    }

    #[test]
    fn tile_types_display() {
        assert_eq!(TileType::Observation.to_string(), "Observation");
        assert_eq!(TileType::Action.to_string(), "Action");
        assert_eq!(TileType::Thought.to_string(), "Thought");
        assert_eq!(TileType::Delegation.to_string(), "Delegation");
        assert_eq!(TileType::Escalation.to_string(), "Escalation");
        assert_eq!(TileType::Artifact.to_string(), "Artifact");
        assert_eq!(TileType::System.to_string(), "System");
    }

    #[test]
    fn tile_status_display() {
        assert_eq!(TileStatus::Active.to_string(), "Active");
        assert_eq!(TileStatus::Complete.to_string(), "Complete");
        assert_eq!(TileStatus::Deadband.to_string(), "Deadband");
        assert_eq!(TileStatus::Escalated.to_string(), "Escalated");
        assert_eq!(TileStatus::Archived.to_string(), "Archived");
    }

    // --- TileFilter ---

    #[test]
    fn filter_matches_type() {
        let t = Tile::new(TileType::Observation, "x");
        assert!(TileFilter::new().tile_type(TileType::Observation).matches(&t));
        assert!(!TileFilter::new().tile_type(TileType::Action).matches(&t));
    }

    #[test]
    fn filter_matches_room() {
        let mut t = Tile::new(TileType::Observation, "x");
        t.room_id = Some("room-1".to_string());
        assert!(TileFilter::new().room("room-1").matches(&t));
        assert!(!TileFilter::new().room("room-2").matches(&t));
    }

    #[test]
    fn filter_matches_status() {
        let mut t = Tile::new(TileType::Observation, "x");
        t.complete();
        assert!(TileFilter::new().status(TileStatus::Complete).matches(&t));
        assert!(!TileFilter::new().status(TileStatus::Active).matches(&t));
    }

    #[test]
    fn filter_matches_tick_range() {
        let t = Tile::with_tick(TileType::System, "x", 50);
        assert!(TileFilter::new().since_tick(40).matches(&t));
        assert!(!TileFilter::new().since_tick(60).matches(&t));
        assert!(TileFilter::new().until_tick(60).matches(&t));
        assert!(!TileFilter::new().until_tick(40).matches(&t));
    }

    #[test]
    fn filter_matches_parent() {
        let parent = Tile::new(TileType::Thought, "p");
        let child = parent.child(TileType::Action, "c");
        assert!(TileFilter::new().parent(&parent.id).matches(&child));
        assert!(!TileFilter::new().parent("nonexistent").matches(&child));
    }

    #[test]
    fn filter_matches_ensign() {
        let mut t = Tile::new(TileType::Observation, "x");
        t.ensign_id = Some("ensign-1".to_string());
        assert!(TileFilter::new().ensign("ensign-1").matches(&t));
        assert!(!TileFilter::new().ensign("ensign-2").matches(&t));
    }

    #[test]
    fn filter_combined() {
        let mut t = Tile::new(TileType::Observation, "x");
        t.room_id = Some("r1".to_string());
        t.complete();
        let f = TileFilter::new().room("r1").status(TileStatus::Complete);
        assert!(f.matches(&t));
    }

    // --- TileIndex ---

    #[test]
    fn tile_index_and_query() {
        let mut idx = TileIndex::new();
        let t1 = Tile::new(TileType::Observation, "a");
        let t2 = Tile::new(TileType::Action, "b");
        idx.index(&t1);
        idx.index(&t2);
        assert!(idx.by_type.contains_key(&"Observation".to_string()));
        assert!(idx.by_type.contains_key(&"Action".to_string()));
    }

    #[test]
    fn tile_index_remove() {
        let mut idx = TileIndex::new();
        let t = Tile::new(TileType::Observation, "x");
        idx.index(&t);
        assert_eq!(idx.by_type["Observation"].len(), 1);
        idx.remove(&t);
        assert!(idx.by_type["Observation"].is_empty());
    }

    #[test]
    fn tile_index_rebuild() {
        let mut idx = TileIndex::new();
        let mut tiles = HashMap::new();
        for i in 0..5 {
            let mut t = Tile::new(TileType::Observation, &format!("t{}", i));
            t.room_id = Some("r1".to_string());
            tiles.insert(t.id.clone(), t);
        }
        idx.rebuild(&tiles);
        assert_eq!(idx.by_room["r1"].len(), 5);
    }

    // --- TileStore ---

    #[test]
    fn tile_store_store_and_get() {
        let tmp = tmp();
        let sid = ShellId::new("ts1");
        let u = Universe::new(&kernel_path(&tmp, "ts1"), &sid);
        let mut store = TileStore::new(&u);
        let t = Tile::new(TileType::Observation, "hello");
        let id = t.id.clone();
        store.store(t).unwrap();
        assert!(store.get(&id).is_some());
        assert_eq!(store.count(), 1);
    }

    #[test]
    fn tile_store_query() {
        let tmp = tmp();
        let sid = ShellId::new("ts2");
        let u = Universe::new(&kernel_path(&tmp, "ts2"), &sid);
        let mut store = TileStore::new(&u);
        let mut t1 = Tile::new(TileType::Observation, "obs1");
        t1.room_id = Some("r1".to_string());
        let t2 = Tile::new(TileType::Action, "act1");
        store.store(t1).unwrap();
        store.store(t2).unwrap();
        let results = store.query(TileFilter::new().room("r1"));
        assert_eq!(results.len(), 1);
        let results = store.query(TileFilter::new().tile_type(TileType::Action));
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn tile_store_children_of() {
        let tmp = tmp();
        let sid = ShellId::new("ts3");
        let u = Universe::new(&kernel_path(&tmp, "ts3"), &sid);
        let mut store = TileStore::new(&u);
        let parent = Tile::new(TileType::Thought, "think");
        let child = parent.child(TileType::Action, "act");
        let parent_id = parent.id.clone();
        store.store(parent).unwrap();
        store.store(child).unwrap();
        let children = store.children_of(&parent_id);
        assert_eq!(children.len(), 1);
    }

    #[test]
    fn tile_store_recent() {
        let tmp = tmp();
        let sid = ShellId::new("ts4");
        let u = Universe::new(&kernel_path(&tmp, "ts4"), &sid);
        let mut store = TileStore::new(&u);
        for i in 0..10 {
            let mut t = Tile::with_tick(TileType::System, &format!("{}", i), i);
            t.created_tick = i as u64;
            store.store(t).unwrap();
        }
        let recent = store.recent(3);
        assert_eq!(recent.len(), 3);
    }

    #[test]
    fn tile_store_flush_and_load() {
        let tmp = tmp();
        let sid = ShellId::new("ts5");
        let u = Universe::new(&kernel_path(&tmp, "ts5"), &sid);
        fs::create_dir_all(&u.tiles_dir).unwrap();
        let mut store = TileStore::new(&u);
        let t = Tile::new(TileType::Observation, "persist me");
        let id = t.id.clone();
        store.store(t).unwrap();
        store.flush().unwrap();

        let mut store2 = TileStore::new(&u);
        store2.load().unwrap();
        assert!(store2.get(&id).is_some());
        assert_eq!(store2.count(), 1);
    }

    // --- Allowance ---

    #[test]
    fn allowance_new() {
        let a = Allowance::new("shell-1", "openai");
        assert_eq!(a.shell_id, "shell-1");
        assert_eq!(a.api, "openai");
        assert_eq!(a.remaining_budget(), 100.0);
    }

    #[test]
    fn allowance_use_and_exhaust() {
        let mut a = Allowance::new("s1", "api");
        a.use_api(30.0).unwrap();
        assert_eq!(a.remaining_budget(), 70.0);
        a.use_api(70.0).unwrap();
        assert!(a.use_api(1.0).is_err());
    }

    #[test]
    fn allowance_permissions() {
        let mut a = Allowance::new("s1", "api");
        a.grant("read");
        a.grant("write");
        assert!(a.permissions.contains(&"read".to_string()));
        a.revoke("read");
        assert!(!a.permissions.contains(&"read".to_string()));
    }

    #[test]
    fn allowance_expires() {
        let mut a = Allowance::new("s1", "api");
        a.expires = Some(100);
        assert!(!a.is_expired(99));
        assert!(a.is_expired(100));
        assert!(a.is_expired(101));
    }

    // --- ShellConfig ---

    #[test]
    fn config_defaults() {
        let c = ShellConfig::default_config();
        assert_eq!(c.max_tiles, 100_000);
        assert_eq!(c.log_level, "info");
    }

    #[test]
    fn config_minimal() {
        let c = ShellConfig::minimal();
        assert_eq!(c.max_tiles, 1_000);
        assert_eq!(c.log_level, "warn");
    }

    #[test]
    fn config_hermes() {
        let c = ShellConfig::hermes();
        assert_eq!(c.max_tiles, 1_000_000);
        assert_eq!(c.log_level, "debug");
    }

    // --- ShellKernel ---

    #[test]
    fn kernel_new_is_empty() {
        let tmp = tmp();
        let mut k = ShellKernel::new(ShellKind::Hermes, &kernel_path(&tmp, "k1"));
        k.bootstrap().unwrap();
        assert!(k.ports.is_empty());
        assert!(k.child_shells.is_empty());
        assert_eq!(k.tile_store.count(), 0);
    }

    #[test]
    fn kernel_bootstrap_creates_dirs() {
        let tmp = tmp();
        let mut k = ShellKernel::new(ShellKind::Hermes, &kernel_path(&tmp, "k2"));
        k.bootstrap().unwrap();
        assert!(k.universe.exists());
        assert!(k.universe.validate().is_ok());
    }

    #[test]
    fn kernel_add_remove_port() {
        let tmp = tmp();
        let mut k = ShellKernel::new(ShellKind::Hermes, &kernel_path(&tmp, "k3"));
        k.bootstrap().unwrap();
        let p = Port::new("p1", PortProtocol::Http, PortDirection::Inbound);
        k.add_port(p).unwrap();
        assert_eq!(k.ports.len(), 1);
        k.remove_port("p1").unwrap();
        assert!(k.ports.is_empty());
    }

    #[test]
    fn kernel_add_duplicate_port_fails() {
        let tmp = tmp();
        let mut k = ShellKernel::new(ShellKind::Hermes, &kernel_path(&tmp, "k4"));
        k.bootstrap().unwrap();
        k.add_port(Port::new("p1", PortProtocol::Http, PortDirection::Inbound))
            .unwrap();
        assert!(k
            .add_port(Port::new("p1", PortProtocol::Http, PortDirection::Inbound))
            .is_err());
    }

    #[test]
    fn kernel_grant_allowance() {
        let tmp = tmp();
        let mut k = ShellKernel::new(ShellKind::Hermes, &kernel_path(&tmp, "k5"));
        k.bootstrap().unwrap();
        let a = Allowance::new(&k.id.to_string(), "openai");
        k.grant_allowance(a);
        assert_eq!(k.allowances.len(), 1);
    }

    #[test]
    fn kernel_spawn_child() {
        let tmp = tmp();
        let mut k = ShellKernel::new(ShellKind::Hermes, &kernel_path(&tmp, "k6"));
        k.bootstrap().unwrap();
        let child_id = k.spawn_child(ShellKind::ZeroClaw, "worker-1").unwrap();
        assert_eq!(k.child_shells.len(), 1);
        assert_eq!(k.child_shells[0], child_id);
    }

    #[test]
    fn kernel_list_children() {
        let tmp = tmp();
        let mut k = ShellKernel::new(ShellKind::Hermes, &kernel_path(&tmp, "k7"));
        k.bootstrap().unwrap();
        k.spawn_child(ShellKind::ZeroClaw, "w1").unwrap();
        k.spawn_child(ShellKind::CUDAClaw, "w2").unwrap();
        assert_eq!(k.list_children().len(), 2);
    }

    #[test]
    fn kernel_create_tile() {
        let tmp = tmp();
        let mut k = ShellKernel::new(ShellKind::Hermes, &kernel_path(&tmp, "k8"));
        k.bootstrap().unwrap();
        let t = k.tile(TileType::Observation, "saw it").unwrap();
        assert_eq!(t.content.text, "saw it");
        assert_eq!(k.tile_store.count(), 1);
    }

    #[test]
    fn kernel_query_tiles() {
        let tmp = tmp();
        let mut k = ShellKernel::new(ShellKind::Hermes, &kernel_path(&tmp, "k9"));
        k.bootstrap().unwrap();
        k.tile(TileType::Observation, "obs").unwrap();
        k.tile(TileType::Action, "act").unwrap();
        let results = k.query(TileFilter::new().tile_type(TileType::Observation));
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn kernel_advance_tick() {
        let tmp = tmp();
        let mut k = ShellKernel::new(ShellKind::Hermes, &kernel_path(&tmp, "k10"));
        k.bootstrap().unwrap();
        let result = k.advance_tick();
        assert_eq!(result.tiles_created, 0);
        assert_eq!(k.tick, 1);
    }

    #[test]
    fn kernel_status() {
        let tmp = tmp();
        let mut k = ShellKernel::new(ShellKind::Hermes, &kernel_path(&tmp, "k11"));
        k.bootstrap().unwrap();
        k.tile(TileType::System, "boot").unwrap();
        k.add_port(Port::new("p1", PortProtocol::Http, PortDirection::Inbound))
            .unwrap();
        let status = k.status();
        assert_eq!(status.kind, "Hermes");
        assert_eq!(status.tiles, 1);
        assert_eq!(status.ports_active, 1);
    }

    #[test]
    fn kernel_is_sandboxed() {
        let tmp = tmp();
        let k1 = ShellKernel::new(ShellKind::Hermes, &kernel_path(&tmp, "k12"));
        assert!(!k1.is_sandboxed());
        let k2 = ShellKernel::new(ShellKind::ZeroClaw, &kernel_path(&tmp, "k13"));
        assert!(k2.is_sandboxed());
        let k3 = ShellKernel::new(ShellKind::CUDAClaw, &kernel_path(&tmp, "k14"));
        assert!(k3.is_sandboxed());
    }

    #[test]
    fn kernel_can_access() {
        let tmp = tmp();
        let path = kernel_path(&tmp, "k15");
        let mut k = ShellKernel::new(ShellKind::Hermes, &path);
        k.bootstrap().unwrap();
        assert!(k.can_access(&format!("{}/tiles", path)));
        assert!(!k.can_access("/etc/passwd"));
    }

    #[test]
    fn kernel_shutdown() {
        let tmp = tmp();
        let mut k = ShellKernel::new(ShellKind::Hermes, &kernel_path(&tmp, "k16"));
        k.bootstrap().unwrap();
        k.tile(TileType::System, "hello").unwrap();
        k.add_port(Port::new("p1", PortProtocol::Http, PortDirection::Inbound))
            .unwrap();
        k.shutdown().unwrap();
        // ports disabled
        assert!(!k.ports["p1"].is_active());
    }

    #[test]
    fn kernel_spawn_child_creates_universe() {
        let tmp = tmp();
        let mut k = ShellKernel::new(ShellKind::Hermes, &kernel_path(&tmp, "k17"));
        k.bootstrap().unwrap();
        let _child_id = k.spawn_child(ShellKind::ZeroClaw, "child1").unwrap();
        let child_path = format!(
            "{}/children/child1",
            kernel_path(&tmp, "k17")
        );
        assert!(Path::new(&child_path).is_dir());
    }

    #[test]
    fn kernel_max_tiles_respected() {
        let tmp = tmp();
        let mut k = ShellKernel::new(ShellKind::Hermes, &kernel_path(&tmp, "k18"));
        k.config.max_tiles = 2;
        k.bootstrap().unwrap();
        k.tile(TileType::System, "t1").unwrap();
        k.tile(TileType::System, "t2").unwrap();
        assert!(k.tile(TileType::System, "t3").is_err());
    }

    #[test]
    fn deadband_trend_variants() {
        let stable = DeadbandTrend::Stable;
        let drifting = DeadbandTrend::Drifting(0.5);
        let oscillating = DeadbandTrend::Oscillating(1.0);
        let diverging = DeadbandTrend::Diverging;
        // Just ensure they exist and are usable
        assert_eq!(stable, DeadbandTrend::Stable);
        assert_eq!(drifting, DeadbandTrend::Drifting(0.5));
        assert_eq!(oscillating, DeadbandTrend::Oscillating(1.0));
        assert_eq!(diverging, DeadbandTrend::Diverging);
    }

    #[test]
    fn tile_with_room_preserved() {
        let tmp = tmp();
        let sid = ShellId::new("r1");
        let u = Universe::new(&kernel_path(&tmp, "r1"), &sid);
        let mut store = TileStore::new(&u);
        let mut t = Tile::new(TileType::Observation, "in room");
        t.room_id = Some("room-42".to_string());
        let id = t.id.clone();
        store.store(t).unwrap();
        let results = store.room_tiles("room-42");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, id);
    }

    #[test]
    fn tile_content_text() {
        let c = TileContent::text("hello");
        assert_eq!(c.text, "hello");
        assert!(c.data.is_none());
    }

    #[test]
    fn port_protocol_display() {
        assert_eq!(PortProtocol::Telegram.to_string(), "Telegram");
        assert_eq!(PortProtocol::Custom("foo".into()).to_string(), "Custom(foo)");
    }

    #[test]
    fn shell_kind_display() {
        assert_eq!(format!("{}", ShellKind::Hermes), "Hermes");
        assert_eq!(format!("{}", ShellKind::Custom("X".into())), "X");
    }

    #[test]
    fn serde_roundtrip_kernel() {
        let tmp = tmp();
        let mut k = ShellKernel::new(ShellKind::Hermes, &kernel_path(&tmp, "s1"));
        k.bootstrap().unwrap();
        k.tile(TileType::System, "boot").unwrap();
        let json = serde_json::to_string(&k).unwrap();
        let _: ShellKernel = serde_json::from_str(&json).unwrap();
    }

    #[test]
    fn serde_roundtrip_tile() {
        let t = Tile::new(TileType::Artifact, "some artifact");
        let json = serde_json::to_string(&t).unwrap();
        let t2: Tile = serde_json::from_str(&json).unwrap();
        assert_eq!(t.id, t2.id);
        assert_eq!(t.content.text, t2.content.text);
    }

    #[test]
    fn kernel_conservation_budget() {
        let tmp = tmp();
        let mut k = ShellKernel::new(ShellKind::Hermes, &kernel_path(&tmp, "k19"));
        k.conservation_budget = 0.05; // only enough for ~5 tiles at 0.01 each
        k.bootstrap().unwrap();
        k.tile(TileType::System, "t1").unwrap();
        k.tile(TileType::System, "t2").unwrap();
        k.tile(TileType::System, "t3").unwrap();
        k.tile(TileType::System, "t4").unwrap();
        k.tile(TileType::System, "t5").unwrap();
        assert!(k.tile(TileType::System, "t6").is_err());
    }
}
