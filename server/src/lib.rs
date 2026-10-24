pub mod math;
pub mod models;

use math::Vec2;
pub use models::*;
use spacetimedb::{
    AnonymousViewContext, ReducerContext, ScheduleAt, SpacetimeType, Table, ViewContext, reducer,
    view,
};
use std::time::Duration;

const DEFAULT_WORLD_WIDTH: f32 = 3200.0;
const DEFAULT_WORLD_HEIGHT: f32 = 2000.0;
const DEFAULT_MEGASTATION_POS: Vec2 = Vec2::new(1600.0, 1000.0);
const DEFAULT_TICK_INTERVAL_MS: u32 = 50; // 20 Ticks/s (50ms interval)
const DEFAULT_SIGNAL_RADIUS: f32 = 1200.0;
const MIN_DISTANCE_FROM_MEGASTATION_TO_DEPLOY: f32 = 250.0;

// ==========================================
// Lifecycle Reducers
// ==========================================

#[reducer(init)]
pub fn init(ctx: &ReducerContext) -> Result<(), String> {
    log::info!("Initializing Space Programming Game Server (v2.13)...");

    // 1. Singleton Config Table
    if ctx.db.config().id().find(&0).is_none() {
        ctx.db.config().insert(Config {
            id: 0,
            world_size: Vec2::new(DEFAULT_WORLD_WIDTH, DEFAULT_WORLD_HEIGHT),
            megastation_pos: DEFAULT_MEGASTATION_POS,
            tick_interval_ms: DEFAULT_TICK_INTERVAL_MS,
            level: 1,
            paused: false,
        });
    }

    // 2. Spawn initial Asteroids
    seed_asteroids(ctx, 30);

    // 3. Schedule periodic Game Tick (every 50ms / 20 Ticks/s)
    ctx.db.tick_schedule().insert(TickSchedule {
        scheduled_id: 0,
        scheduled_at: ScheduleAt::Interval(
            Duration::from_millis(DEFAULT_TICK_INTERVAL_MS as u64).into(),
        ),
    });

    log::info!("Game Server Initialized with 30 asteroids and 50ms tick loop.");
    Ok(())
}

#[reducer(client_connected)]
pub fn on_client_connected(ctx: &ReducerContext) {
    log::info!("Client connected: {:?}", ctx.sender());
}

#[reducer(client_disconnected)]
pub fn on_client_disconnected(ctx: &ReducerContext) {
    log::info!("Client disconnected: {:?}", ctx.sender());
}

// ==========================================
// Player & Station Reducers
// ==========================================

#[reducer]
pub fn register_player(ctx: &ReducerContext, name: String, color_hue: u16) -> Result<(), String> {
    let sender = ctx.sender();
    if ctx.db.player().identity().find(&sender).is_some() {
        return Err("Player is already registered".to_string());
    }

    let clean_name = name.trim().to_string();
    if clean_name.is_empty() {
        return Err("Player name cannot be empty".to_string());
    }

    let config = ctx.db.config().id().find(&0).ok_or("Config missing")?;

    // 1. Insert Player
    ctx.db.player().insert(Player {
        identity: sender,
        name: clean_name.clone(),
        color_hue: color_hue % 360,
        score: 0,
        created_at: ctx.timestamp,
    });

    // 2. Spawn Tier-0 Builder Ship at Megastation
    // Offset slightly based on hue to prevent perfect overlap
    let angle = (color_hue as f32).to_radians();
    let spawn_offset = Vec2::new(angle.cos() * 80.0, angle.sin() * 80.0);
    let spawn_pos = config.megastation_pos + spawn_offset;

    ctx.db.ship().insert(Ship {
        ship_id: 0,
        universe: 0,
        owner: sender,
        ship_tier: 0, // Builder ship
        position: spawn_pos,
        velocity: Vec2::ZERO,
        target_position: None,
        heading_rad: angle,
        speed: 0.0,
        max_speed: 150.0,
        energy: 100.0,
        max_energy: 100.0,
        cargo_ore: 0,
        max_cargo: 100,
        target_asteroid_id: None,
        is_mining: false,
        is_stranded: false,
        signal_countdown: 30, // 30 seconds buffer
    });

    // 3. Emit Event
    ctx.db.game_event().insert(GameEvent {
        event_id: 0,
        player: sender,
        event_type: "PLAYER_REGISTERED".to_string(),
        message: format!("Player '{}' entered the sector at Mega-Station", clean_name),
        timestamp: ctx.timestamp,
    });

    log::info!(
        "Registered player '{}' with Tier-0 Builder Ship.",
        clean_name
    );
    Ok(())
}

#[reducer]
pub fn deploy_base_station(ctx: &ReducerContext, ship_id: u32) -> Result<(), String> {
    let sender = ctx.sender();

    // Verify player does not already have a base
    if ctx.db.base_station().owner().find(&sender).is_some() {
        return Err("Player already deployed a Base Station".to_string());
    }

    // Verify ship belongs to sender and is Tier 0
    let ship = ctx
        .db
        .ship()
        .ship_id()
        .find(&ship_id)
        .ok_or("Ship not found")?;

    if ship.owner != sender {
        return Err("You do not own this ship".to_string());
    }

    if ship.ship_tier != 0 {
        return Err("Only a Tier-0 Builder Ship can be deployed into a Base Station".to_string());
    }

    let config = ctx.db.config().id().find(&0).ok_or("Config missing")?;

    let dist_to_megastation = ship.position.distance(&config.megastation_pos);
    if dist_to_megastation < MIN_DISTANCE_FROM_MEGASTATION_TO_DEPLOY {
        return Err(format!(
            "Must be at least {:.0} units away from Mega-Station to deploy base (current: {:.0})",
            MIN_DISTANCE_FROM_MEGASTATION_TO_DEPLOY, dist_to_megastation
        ));
    }

    // Deploy Base Station at ship position
    ctx.db.base_station().insert(BaseStation {
        base_id: 0,
        universe: 0,
        owner: sender,
        position: ship.position,
        signal_radius: DEFAULT_SIGNAL_RADIUS,
        stored_ore: 0,
        stored_energy: 500,
        level: 1,
    });

    // Consume the builder ship and replace with first Tier-1 Miner
    ctx.db.ship().ship_id().delete(&ship_id);

    ctx.db.ship().insert(Ship {
        ship_id: 0,
        universe: 0,
        owner: sender,
        ship_tier: 1, // First programmable miner drone!
        position: ship.position + Vec2::new(30.0, 0.0),
        velocity: Vec2::ZERO,
        target_position: None,
        heading_rad: 0.0,
        speed: 0.0,
        max_speed: 120.0,
        energy: 100.0,
        max_energy: 100.0,
        cargo_ore: 0,
        max_cargo: 50,
        target_asteroid_id: None,
        is_mining: false,
        is_stranded: false,
        signal_countdown: 20, // 20 seconds buffer
    });

    ctx.db.game_event().insert(GameEvent {
        event_id: 0,
        player: sender,
        event_type: "BASE_DEPLOYED".to_string(),
        message: "Home Base Station established! Programmable Drone Online.".to_string(),
        timestamp: ctx.timestamp,
    });

    log::info!(
        "Player {:?} established Base Station at {:?}.",
        sender,
        ship.position
    );
    Ok(())
}

#[reducer]
pub fn build_ship(ctx: &ReducerContext, tier: u8) -> Result<(), String> {
    let sender = ctx.sender();
    let mut base = ctx
        .db
        .base_station()
        .owner()
        .find(&sender)
        .ok_or("Deploy a Base Station first before building ships")?;

    let (cost, max_speed, max_energy, max_cargo) = match tier {
        1 => (50, 120.0, 100.0, 50),
        2 => (150, 150.0, 200.0, 150),
        _ => return Err("Invalid ship tier".to_string()),
    };

    if base.stored_ore < cost {
        return Err(format!(
            "Insufficient ore: need {} but have {}",
            cost, base.stored_ore
        ));
    }

    base.stored_ore -= cost;
    ctx.db.base_station().owner().update(base.clone());

    ctx.db.ship().insert(Ship {
        ship_id: 0,
        universe: 0,
        owner: sender,
        ship_tier: tier,
        position: base.position + Vec2::new(25.0, 25.0),
        velocity: Vec2::ZERO,
        target_position: None,
        heading_rad: 0.0,
        speed: 0.0,
        max_speed,
        energy: max_energy,
        max_energy,
        cargo_ore: 0,
        max_cargo,
        target_asteroid_id: None,
        is_mining: false,
        is_stranded: false,
        signal_countdown: 20, // 20 seconds buffer
    });

    ctx.db.game_event().insert(GameEvent {
        event_id: 0,
        player: sender,
        event_type: "SHIP_BUILT".to_string(),
        message: format!("Built Tier-{} ship", tier),
        timestamp: ctx.timestamp,
    });

    Ok(())
}

// ==========================================
// Ship Command Reducer (Called by Student Code / UI)
// ==========================================

#[reducer]
pub fn command_ship(ctx: &ReducerContext, ship_id: u32, cmd: ShipCommand) -> Result<(), String> {
    let sender = ctx.sender();
    let mut ship = ctx
        .db
        .ship()
        .ship_id()
        .find(&ship_id)
        .ok_or("Ship not found")?;

    if ship.owner != sender {
        return Err("You do not own this ship".to_string());
    }

    if ship.is_stranded {
        return Err(
            "Ship is stranded without signal/energy and cannot accept commands".to_string(),
        );
    }

    match cmd {
        ShipCommand::FlyTo { target } => {
            ship.target_position = Some(target);
            ship.is_mining = false;
            ship.target_asteroid_id = None;
        }
        ShipCommand::Stop => {
            ship.target_position = None;
            ship.velocity = Vec2::ZERO;
            ship.speed = 0.0;
            ship.is_mining = false;
            ship.target_asteroid_id = None;
        }
        ShipCommand::Mine { asteroid_id } => {
            let asteroid = ctx
                .db
                .asteroid()
                .asteroid_id()
                .find(&asteroid_id)
                .ok_or("Target asteroid not found")?;

            ship.target_asteroid_id = Some(asteroid_id);
            ship.target_position = Some(asteroid.position);
            ship.is_mining = true;
        }
        ShipCommand::Dock => {
            let base = ctx
                .db
                .base_station()
                .owner()
                .find(&sender)
                .ok_or("Base station not found to dock at")?;

            ship.target_position = Some(base.position);
            ship.is_mining = false;
            ship.target_asteroid_id = None;
        }
    }

    ctx.db.ship().ship_id().update(ship);
    Ok(())
}

// ==========================================
// Admin Reducers (Teacher Controls)
// ==========================================

#[reducer]
pub fn admin_set_phase(ctx: &ReducerContext, level: u8, paused: bool) -> Result<(), String> {
    let mut config = ctx.db.config().id().find(&0).ok_or("Config missing")?;

    config.level = level;
    config.paused = paused;
    ctx.db.config().id().update(config);

    log::info!(
        "Admin set game phase to Level {} (paused: {})",
        level,
        paused
    );
    Ok(())
}

#[reducer]
pub fn authenticate_admin(ctx: &ReducerContext, password: String) -> Result<(), String> {
    if password.trim() != "admin" {
        return Err("Invalid admin password".to_string());
    }
    let sender = ctx.sender();
    if ctx.db.admin_user().identity().find(&sender).is_none() {
        ctx.db.admin_user().insert(AdminUser {
            identity: sender,
            authenticated_at: ctx.timestamp,
        });
    }
    log::info!("Admin authenticated successfully: {:?}", sender);
    Ok(())
}

// ==========================================
// Scheduled Game Tick Reducer
// ==========================================

#[reducer]
pub fn game_tick(ctx: &ReducerContext, _schedule: TickSchedule) -> Result<(), String> {
    let mut config = match ctx.db.config().id().find(&0) {
        Some(c) => c,
        None => return Ok(()),
    };

    // Auto-migrate config to 50ms tick interval if still set to 300ms
    if config.tick_interval_ms != DEFAULT_TICK_INTERVAL_MS {
        config.tick_interval_ms = DEFAULT_TICK_INTERVAL_MS;
        ctx.db.config().id().update(config.clone());
    }

    // Auto-migrate tick schedule interval if it doesn't match config
    let target_interval =
        ScheduleAt::Interval(Duration::from_millis(config.tick_interval_ms as u64).into());
    if _schedule.scheduled_at != target_interval {
        ctx.db
            .tick_schedule()
            .scheduled_id()
            .delete(&_schedule.scheduled_id);
        ctx.db.tick_schedule().insert(TickSchedule {
            scheduled_id: 0,
            scheduled_at: target_interval,
        });
        log::info!("Upgraded tick schedule to 50ms interval");
    }

    if config.paused {
        return Ok(());
    }

    let dt = (config.tick_interval_ms as f32) / 1000.0;
    let tick_interval_micros = (config.tick_interval_ms as i64 * 1000).max(1);
    let tick_count = ctx.timestamp.to_micros_since_unix_epoch() / tick_interval_micros;
    let ticks_per_sec = (1000 / config.tick_interval_ms).max(1) as i64;

    // 1. Update Ships
    let mut ships_to_update = Vec::new();
    let mut ships_to_delete = Vec::new();

    for mut ship in ctx.db.ship().iter() {
        if ship.is_stranded {
            continue;
        }

        // Movement logic towards target
        if let Some(target) = ship.target_position {
            let diff = target - ship.position;
            let dist = diff.magnitude();

            if dist < 8.0 {
                // Arrived at destination
                ship.position = target;
                ship.velocity = Vec2::ZERO;
                ship.speed = 0.0;

                // Check if docking at base
                if let Some(mut base) = ctx.db.base_station().owner().find(&ship.owner) {
                    if base.position.distance(&ship.position) < 40.0 {
                        // Offload cargo
                        if ship.cargo_ore > 0 {
                            base.stored_ore += ship.cargo_ore;
                            if let Some(mut player) = ctx.db.player().identity().find(&ship.owner) {
                                player.score += ship.cargo_ore as u64 * 10;
                                ctx.db.player().identity().update(player);
                            }
                            ship.cargo_ore = 0;
                        }
                        // Recharge energy
                        ship.energy = ship.max_energy;
                        ctx.db.base_station().owner().update(base);
                        ship.target_position = None;
                    }
                }

                // If not mining and arrived, clear target
                if !ship.is_mining {
                    ship.target_position = None;
                }
            } else {
                // Fly towards target
                let dir = diff.normalized();
                ship.heading_rad = dir.heading();
                ship.speed = ship.max_speed;
                ship.velocity = dir * ship.speed;
                ship.position += ship.velocity * dt;

                // Burn energy while flying
                let energy_cost = 0.8 * dt;
                ship.energy = (ship.energy - energy_cost).max(0.0);
                if ship.energy <= 0.0 {
                    ship.is_stranded = true;
                    ship.velocity = Vec2::ZERO;
                    ship.speed = 0.0;
                }
            }
        }

        // Mining logic (mines 1 ore every 200ms = 4 ticks, approx 5 ore/sec)
        let is_mining_tick = tick_count % 4 == 0;
        if is_mining_tick && ship.is_mining && ship.cargo_ore < ship.max_cargo {
            if let Some(ast_id) = ship.target_asteroid_id {
                if let Some(mut asteroid) = ctx.db.asteroid().asteroid_id().find(&ast_id) {
                    if ship.position.distance(&asteroid.position) <= asteroid.radius + 35.0 {
                        let amount = 1.min(asteroid.remaining_ore);
                        if amount > 0 && ship.cargo_ore + amount <= ship.max_cargo {
                            ship.cargo_ore += amount;
                            asteroid.remaining_ore = asteroid.remaining_ore.saturating_sub(amount);

                            if asteroid.remaining_ore == 0 {
                                ctx.db.asteroid().asteroid_id().delete(&ast_id);
                                ship.target_asteroid_id = None;
                                ship.is_mining = false;
                            } else {
                                ctx.db.asteroid().asteroid_id().update(asteroid);
                            }
                        }
                    }
                }
            }
        }

        // Signal range check (countdown decrements once per second)
        if let Some(base) = ctx.db.base_station().owner().find(&ship.owner) {
            let dist_to_base = ship.position.distance(&base.position);
            if dist_to_base > base.signal_radius {
                // Outside signal!
                if ship.ship_tier <= 1 {
                    let is_second_tick = tick_count % ticks_per_sec == 0;
                    if is_second_tick {
                        if ship.signal_countdown > 0 {
                            ship.signal_countdown -= 1;
                        } else {
                            // Tier 0/1 ship destroyed
                            ships_to_delete.push(ship.ship_id);
                            ctx.db.game_event().insert(GameEvent {
                                event_id: 0,
                                player: ship.owner,
                                event_type: "SHIP_LOST".to_string(),
                                message: format!(
                                    "Ship #{} was lost outside signal range",
                                    ship.ship_id
                                ),
                                timestamp: ctx.timestamp,
                            });
                            continue;
                        }
                    }
                } else {
                    // Tier 2+ becomes stranded (rescueable)
                    ship.is_stranded = true;
                    ship.velocity = Vec2::ZERO;
                    ship.speed = 0.0;
                    ship.target_position = None;
                }
            } else {
                // Inside signal: reset countdown if healthy
                if !ship.is_stranded {
                    ship.signal_countdown = if ship.ship_tier == 0 { 30 } else { 20 };
                }
            }
        }

        ships_to_update.push(ship);
    }

    // Apply ship updates
    for s in ships_to_update {
        ctx.db.ship().ship_id().update(s);
    }
    for id in ships_to_delete {
        ctx.db.ship().ship_id().delete(&id);
    }

    // Respawn asteroids if count drops below 20
    let asteroid_count = ctx.db.asteroid().count();
    if asteroid_count < 20 {
        seed_asteroids(ctx, (25 - asteroid_count) as usize);
    }

    // 250ms periodic sensor sweep (every 5 ticks of 50ms)
    let is_sensor_tick = tick_count % 5 == 0;
    if is_sensor_tick {
        update_radar_sensors(ctx);
    }

    Ok(())
}

fn update_radar_sensors(ctx: &ReducerContext) {
    let all_bases: Vec<BaseStation> = ctx.db.base_station().iter().collect();
    let all_ships: Vec<Ship> = ctx.db.ship().iter().collect();
    let all_asteroids: Vec<Asteroid> = ctx.db.asteroid().iter().collect();

    for player in ctx.db.player().iter() {
        let p_id = player.identity;

        // 1. Clear existing perceived rows for this viewer
        let drone_ids: Vec<u64> = ctx
            .db
            .perceived_drone()
            .viewer()
            .filter(&p_id)
            .map(|r| r.id)
            .collect();
        for id in drone_ids {
            ctx.db.perceived_drone().id().delete(&id);
        }

        let base_ids: Vec<u64> = ctx
            .db
            .perceived_base()
            .viewer()
            .filter(&p_id)
            .map(|r| r.id)
            .collect();
        for id in base_ids {
            ctx.db.perceived_base().id().delete(&id);
        }

        let asteroid_ids: Vec<u64> = ctx
            .db
            .perceived_asteroid()
            .viewer()
            .filter(&p_id)
            .map(|r| r.id)
            .collect();
        for id in asteroid_ids {
            ctx.db.perceived_asteroid().id().delete(&id);
        }

        let contact_ids: Vec<u64> = ctx
            .db
            .unidentified_contact()
            .viewer()
            .filter(&p_id)
            .map(|r| r.id)
            .collect();
        for id in contact_ids {
            ctx.db.unidentified_contact().id().delete(&id);
        }

        // 2. Identify Primary Hub
        let hub_pos = ctx
            .db
            .base_station()
            .owner()
            .find(&p_id)
            .map(|b| b.position)
            .or_else(|| {
                all_ships
                    .iter()
                    .find(|s| s.owner == p_id && s.ship_tier == 0)
                    .map(|s| s.position)
            });

        // 3. Identify Own Drones
        let own_drones: Vec<&Ship> = all_ships.iter().filter(|s| s.owner == p_id).collect();

        // If no sensors are available, skip
        if hub_pos.is_none() && own_drones.is_empty() {
            continue;
        }

        let get_dist_to_hub = |pos: &Vec2| hub_pos.as_ref().map(|h| pos.distance(h));
        let get_dist_to_drones = |pos: &Vec2| {
            own_drones
                .iter()
                .map(|d| pos.distance(&d.position))
                .fold(f32::INFINITY, f32::min)
        };

        // A. Drones
        for s in &all_ships {
            let dist_hub = get_dist_to_hub(&s.position);
            let dist_drone = get_dist_to_drones(&s.position);

            let is_in_drone_id_range =
                dist_hub.map_or(false, |d| d <= 150.0) || dist_drone <= 150.0;

            if is_in_drone_id_range {
                ctx.db.perceived_drone().insert(PerceivedDrone {
                    id: 0,
                    viewer: p_id,
                    ship_id: s.ship_id,
                    owner: s.owner,
                    ship_tier: s.ship_tier,
                    position: s.position,
                    heading_rad: s.heading_rad,
                    speed: s.speed,
                    is_mining: s.is_mining,
                });
            } else if s.owner != p_id && dist_hub.map_or(false, |d| d <= 600.0) {
                ctx.db.unidentified_contact().insert(UnidentifiedContact {
                    id: 0,
                    viewer: p_id,
                    contact_id: s.ship_id,
                    position: s.position,
                });
            }
        }

        // B. Bases
        for b in &all_bases {
            let dist_hub = get_dist_to_hub(&b.position);
            let is_in_base_id_range = dist_hub.map_or(false, |d| d <= 300.0);

            if is_in_base_id_range {
                ctx.db.perceived_base().insert(PerceivedBase {
                    id: 0,
                    viewer: p_id,
                    base_id: b.base_id,
                    owner: b.owner,
                    position: b.position,
                    signal_radius: b.signal_radius,
                    level: b.level,
                });
            } else if b.owner != p_id && dist_hub.map_or(false, |d| d <= 600.0) {
                ctx.db.unidentified_contact().insert(UnidentifiedContact {
                    id: 0,
                    viewer: p_id,
                    contact_id: 100_000 + b.base_id,
                    position: b.position,
                });
            }
        }

        // C. Asteroids
        for ast in &all_asteroids {
            let dist_hub = get_dist_to_hub(&ast.position);
            let dist_drone = get_dist_to_drones(&ast.position);

            let is_in_asteroid_id_range =
                dist_hub.map_or(false, |d| d <= 300.0) || dist_drone <= 150.0;

            if is_in_asteroid_id_range {
                ctx.db.perceived_asteroid().insert(PerceivedAsteroid {
                    id: 0,
                    viewer: p_id,
                    asteroid_id: ast.asteroid_id,
                    position: ast.position,
                    resource_type: ast.resource_type,
                    remaining_ore: ast.remaining_ore,
                    radius: ast.radius,
                });
            } else if dist_hub.map_or(false, |d| d <= 600.0) {
                ctx.db.unidentified_contact().insert(UnidentifiedContact {
                    id: 0,
                    viewer: p_id,
                    contact_id: 200_000 + ast.asteroid_id,
                    position: ast.position,
                });
            }
        }
    }
}

// ==========================================
// Views
// ==========================================

#[derive(SpacetimeType)]
pub struct CountRow {
    pub count: u64,
}

#[view(accessor = total_players, public)]
pub fn total_players(ctx: &AnonymousViewContext) -> Vec<CountRow> {
    vec![CountRow {
        count: ctx.db.player().count(),
    }]
}

#[view(accessor = my_ships, public)]
pub fn my_ships(ctx: &ViewContext) -> Vec<Ship> {
    ctx.db.ship().owner().filter(&ctx.sender()).collect()
}

#[view(accessor = my_base, public)]
pub fn my_base(ctx: &ViewContext) -> Option<BaseStation> {
    ctx.db.base_station().owner().find(&ctx.sender())
}

#[view(accessor = visible_drones, public)]
pub fn visible_drones(ctx: &ViewContext) -> Vec<PerceivedDrone> {
    ctx.db
        .perceived_drone()
        .viewer()
        .filter(&ctx.sender())
        .collect()
}

#[view(accessor = visible_bases, public)]
pub fn visible_bases(ctx: &ViewContext) -> Vec<PerceivedBase> {
    ctx.db
        .perceived_base()
        .viewer()
        .filter(&ctx.sender())
        .collect()
}

#[view(accessor = visible_asteroids, public)]
pub fn visible_asteroids(ctx: &ViewContext) -> Vec<PerceivedAsteroid> {
    ctx.db
        .perceived_asteroid()
        .viewer()
        .filter(&ctx.sender())
        .collect()
}

#[view(accessor = unidentified_objects, public)]
pub fn unidentified_objects(ctx: &ViewContext) -> Vec<UnidentifiedContact> {
    ctx.db
        .unidentified_contact()
        .viewer()
        .filter(&ctx.sender())
        .collect()
}

#[view(accessor = admin_all_ships, public)]
pub fn admin_all_ships(ctx: &ViewContext) -> Vec<Ship> {
    if ctx.db.admin_user().identity().find(&ctx.sender()).is_none() {
        return Vec::new();
    }
    ctx.db.ship().universe().filter(&0u8).collect()
}

#[view(accessor = admin_all_bases, public)]
pub fn admin_all_bases(ctx: &ViewContext) -> Vec<BaseStation> {
    if ctx.db.admin_user().identity().find(&ctx.sender()).is_none() {
        return Vec::new();
    }
    ctx.db.base_station().universe().filter(&0u8).collect()
}

#[view(accessor = admin_all_asteroids, public)]
pub fn admin_all_asteroids(ctx: &ViewContext) -> Vec<Asteroid> {
    if ctx.db.admin_user().identity().find(&ctx.sender()).is_none() {
        return Vec::new();
    }
    ctx.db.asteroid().universe().filter(&0u8).collect()
}

// ==========================================
// Helper Functions
// ==========================================

fn seed_asteroids(ctx: &ReducerContext, count: usize) {
    let base_x = 300.0;
    let base_y = 200.0;
    let width = DEFAULT_WORLD_WIDTH - 600.0;
    let height = DEFAULT_WORLD_HEIGHT - 400.0;

    for i in 0..count {
        // Deterministic pseudo-random distribution based on timestamp & index
        let seed = ctx.timestamp.to_micros_since_unix_epoch() as f32 + (i as f32 * 137.5);
        let rx = (seed * 1.618).sin().abs();
        let ry = (seed * 2.718).cos().abs();

        let pos = Vec2::new(base_x + rx * width, base_y + ry * height);

        // Keep asteroids away from Megastation center
        if pos.distance(&DEFAULT_MEGASTATION_POS) < 300.0 {
            continue;
        }

        let res_type = (i % 4) as u8;
        let ore = match res_type {
            3 => 150, // Titanium
            2 => 100, // Ice
            1 => 80,  // Iron
            _ => 50,  // Generic
        };

        ctx.db.asteroid().insert(Asteroid {
            asteroid_id: 0,
            universe: 0,
            position: pos,
            resource_type: res_type,
            remaining_ore: ore,
            radius: 20.0 + (i % 3) as f32 * 8.0,
        });
    }
}
