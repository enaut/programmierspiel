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
    pub color_hue: u16,           // 0..360 for color wheel (primary)
    pub secondary_color_hue: u16, // 0..360 for color wheel (secondary)
    pub score: u64,
    pub created_at: Timestamp,
    pub last_api_activity: Option<Timestamp>,
    pub has_api_connected: bool,
    pub tutorial_step: u8,
}

#[table(accessor = base_station)]
#[derive(Debug, Clone)]
pub struct BaseStation {
    #[primary_key]
    pub owner: Identity,
    #[auto_inc]
    #[unique]
    pub base_id: u32,
    #[index(btree)]
    pub universe: u8,
    pub position: Vec2,
    pub signal_radius: f32,
    pub stored_ore: u32,
    pub stored_energy: u32,
    pub level: u8,
    pub drone_scanner_level: u8,
    pub drone_scanner_progress: u8,
    pub base_scanner_level: u8,
    pub base_scanner_progress: u8,
    pub radar_horizon_level: u8,
    pub radar_horizon_progress: u8,
    pub last_ship_built_at: Option<Timestamp>,
}

#[table(accessor = ship)]
#[derive(Debug, Clone)]
pub struct Ship {
    #[primary_key]
    #[auto_inc]
    pub ship_id: u32,
    #[index(btree)]
    pub universe: u8,
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

#[table(accessor = asteroid)]
#[derive(Debug, Clone)]
pub struct Asteroid {
    #[primary_key]
    #[auto_inc]
    pub asteroid_id: u32,
    #[index(btree)]
    pub universe: u8,
    #[index(btree)]
    pub chunk_key: u64,
    pub chunk_index: u16,
    pub position: Vec2,
    pub velocity: Vec2,
    pub resource_type: u8,
    pub remaining_ore: u32,
    pub radius: f32,
}

#[table(accessor = depleted_asteroid)]
#[derive(Debug, Clone)]
pub struct DepletedAsteroid {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    #[index(btree)]
    pub chunk_key: u64,
    pub chunk_index: u16,
    pub respawn_at: Timestamp,
}

#[table(accessor = active_chunk)]
#[derive(Debug, Clone)]
pub struct ActiveChunk {
    #[primary_key]
    pub chunk_key: u64,
    pub chunk_x: i32,
    pub chunk_y: i32,
    pub last_active_tick: u64,
}

#[table(accessor = admin_user)]
#[derive(Debug, Clone)]
pub struct AdminUser {
    #[primary_key]
    pub identity: Identity,
    pub authenticated_at: Timestamp,
}

#[table(accessor = ai_agent, public)]
#[derive(Debug, Clone)]
pub struct AiAgent {
    #[primary_key]
    pub agent_identity: Identity,
    #[index(btree)]
    pub owner: Identity,
    pub role: u8,             // 0: Base, 1: DroneShip, 2: Drone
    pub bound_entity_id: u32, // base_id or ship_id
    pub label: String,
    pub created_at: Timestamp,
    pub is_revoked: bool,
}

#[table(accessor = perceived_drone)]
#[derive(Debug, Clone)]
pub struct PerceivedDrone {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    #[index(btree)]
    pub viewer: Identity,
    pub ship_id: u32,
    pub owner: Identity,
    pub ship_tier: u8,
    pub position: Vec2,
    pub heading_rad: f32,
    pub speed: f32,
    pub is_mining: bool,
}

#[table(accessor = perceived_base)]
#[derive(Debug, Clone)]
pub struct PerceivedBase {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    #[index(btree)]
    pub viewer: Identity,
    pub base_id: u32,
    pub owner: Identity,
    pub position: Vec2,
    pub signal_radius: f32,
    pub level: u8,
    pub drone_scanner_level: u8,
    pub base_scanner_level: u8,
    pub radar_horizon_level: u8,
}

#[table(accessor = active_riddle)]
#[derive(Debug, Clone)]
pub struct ActiveRiddle {
    #[primary_key]
    pub owner: Identity,
    pub puzzle_id: u64,
    pub track: u8,
    pub target_level: u8,
    pub payload: String,
    pub expected_solution: String,
    pub created_at: Timestamp,
}

#[derive(SpacetimeType, Debug, Clone)]
pub struct PublicRiddle {
    pub puzzle_id: u64,
    pub track: u8,
    pub target_level: u8,
    pub payload: String,
}

#[table(accessor = perceived_asteroid)]
#[derive(Debug, Clone)]
pub struct PerceivedAsteroid {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    #[index(btree)]
    pub viewer: Identity,
    pub asteroid_id: u32,
    pub position: Vec2,
    pub velocity: Vec2,
    pub resource_type: u8,
    pub remaining_ore: u32,
    pub radius: f32,
}

#[table(accessor = unidentified_contact)]
#[derive(Debug, Clone)]
pub struct UnidentifiedContact {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    #[index(btree)]
    pub viewer: Identity,
    pub contact_id: u32,
    pub position: Vec2,
}

#[table(accessor = tutorial_state)]
#[derive(Debug, Clone)]
pub struct TutorialState {
    #[primary_key]
    pub player: Identity,
    pub current_step: u8,
    pub step_started_at: Timestamp,
    pub completed_at: Option<Timestamp>,
    pub attempts: u32,
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

#[table(accessor = player_achievement, public)]
#[derive(Debug, Clone)]
pub struct PlayerAchievement {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    #[index(btree)]
    pub player: Identity,
    pub achievement_key: String,
    pub title: String,
    pub description: String,
    pub icon: String,
    pub unlocked_at: Timestamp,
}

#[table(accessor = unlocked_command, public)]
#[derive(Debug, Clone)]
pub struct UnlockedCommand {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    #[index(btree)]
    pub player: Identity,
    pub command_key: String,
    pub unlocked_at: Timestamp,
}

#[table(accessor = tick_schedule, scheduled(game_tick))]
#[derive(Debug, Clone)]
pub struct TickSchedule {
    #[primary_key]
    #[auto_inc]
    pub scheduled_id: u64,
    pub scheduled_at: ScheduleAt,
}
