use crate::game_tick;
use crate::math::Vec2;
use spacetimedb::{Identity, ScheduleAt, SpacetimeType, Timestamp, table};

#[derive(SpacetimeType, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceType {
    GenericOre = 0,
    Iron = 1,
    Ice = 2,
    Titanium = 3,
}

#[derive(SpacetimeType, Debug, Clone, PartialEq)]
pub enum ShipCommand {
    FlyTo { target: Vec2 },
    Stop,
    Mine { asteroid_id: u32 },
    Dock,
}

#[table(accessor = config, public)]
#[derive(Debug, Clone)]
pub struct Config {
    #[primary_key]
    pub id: u32, // singleton row with id = 0
    pub world_size: Vec2,
    pub megastation_pos: Vec2,
    pub tick_interval_ms: u32,
    pub level: u8,
    pub paused: bool,
}

#[table(accessor = player, public)]
#[derive(Debug, Clone)]
pub struct Player {
    #[primary_key]
    pub identity: Identity,
    pub name: String,
    pub color_hue: u16, // 0..360 for color wheel
    pub score: u64,
    pub created_at: Timestamp,
}

#[table(accessor = base_station, public)]
#[derive(Debug, Clone)]
pub struct BaseStation {
    #[primary_key]
    pub owner: Identity,
    #[auto_inc]
    #[unique]
    pub base_id: u32,
    pub position: Vec2,
    pub signal_radius: f32,
    pub stored_ore: u32,
    pub stored_energy: u32,
    pub level: u8,
}

#[table(accessor = ship, public)]
#[derive(Debug, Clone)]
pub struct Ship {
    #[primary_key]
    #[auto_inc]
    pub ship_id: u32,
    #[index(btree)]
    pub owner: Identity,
    pub ship_tier: u8, // 0: Builder, 1: Basic Miner/Drone, 2: Advanced Explorer/Hauler
    pub position: Vec2,
    pub velocity: Vec2,
    pub target_position: Option<Vec2>,
    pub heading_rad: f32,
    pub speed: f32,
    pub max_speed: f32,
    pub energy: f32,
    pub max_energy: f32,
    pub cargo_ore: u32,
    pub max_cargo: u32,
    pub target_asteroid_id: Option<u32>,
    pub is_mining: bool,
    pub is_stranded: bool,
    pub signal_countdown: u16,
}

#[table(accessor = asteroid, public)]
#[derive(Debug, Clone)]
pub struct Asteroid {
    #[primary_key]
    #[auto_inc]
    pub asteroid_id: u32,
    pub position: Vec2,
    pub resource_type: u8,
    pub remaining_ore: u32,
    pub radius: f32,
}

#[table(accessor = game_event, public)]
#[derive(Debug, Clone)]
pub struct GameEvent {
    #[primary_key]
    #[auto_inc]
    pub event_id: u64,
    pub player: Identity,
    pub event_type: String,
    pub message: String,
    pub timestamp: Timestamp,
}

#[table(accessor = tick_schedule, scheduled(game_tick))]
#[derive(Debug, Clone)]
pub struct TickSchedule {
    #[primary_key]
    #[auto_inc]
    pub scheduled_id: u64,
    pub scheduled_at: ScheduleAt,
}
