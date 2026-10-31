use crate::action::Actions;
use crate::config::BotConfig;
use crate::module_bindings::api_heartbeat_reducer::api_heartbeat;
use crate::module_bindings::build_ship_reducer::build_ship;
use crate::module_bindings::command_ship_reducer::command_ship;
use crate::module_bindings::my_active_riddle_table::MyActiveRiddleTableAccess;
use crate::module_bindings::my_base_table::MyBaseTableAccess;
use crate::module_bindings::my_ships_table::MyShipsTableAccess;
use crate::module_bindings::request_riddle_reducer::request_riddle;
use crate::module_bindings::submit_riddle_solution_reducer::submit_riddle_solution;
use crate::module_bindings::unidentified_objects_table::UnidentifiedObjectsTableAccess;
use crate::module_bindings::visible_asteroids_table::VisibleAsteroidsTableAccess;
use crate::module_bindings::visible_bases_table::VisibleBasesTableAccess;
use crate::module_bindings::visible_drones_table::VisibleDronesTableAccess;
use crate::module_bindings::{
    BaseStation, DbConnection, PerceivedAsteroid, PerceivedBase, PerceivedDrone, Ship, ShipCommand,
    UnidentifiedContact, Vec2,
};
use crate::research::RiddleTask;
use crate::world::{BaseController, ShipController, ShipWorld, World};
use spacetimedb_sdk::{DbContext, Table};
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

/// Number of 100ms ticks to wait before re-sending a command whose server state has not taken effect.
const COMMAND_RESEND_COOLDOWN_TICKS: u64 = 10;

/// Ensures that starting a new bot instance (e.g. clicking Zed's `▶` button again)
/// causes any previously started bot instance using the same config to exit cleanly
/// instead of competing for control over the same fleet.
struct InstanceLock {
    lock_path: PathBuf,
    pid: u32,
}

impl InstanceLock {
    fn acquire(config_path: &Path) -> Self {
        let canonical =
            std::fs::canonicalize(config_path).unwrap_or_else(|_| config_path.to_path_buf());
        let mut hasher = DefaultHasher::new();
        canonical.hash(&mut hasher);
        let hash = hasher.finish();
        let lock_path = std::env::temp_dir().join(format!("space_bot_{:016x}.pid", hash));
        let pid = std::process::id();
        let _ = std::fs::write(&lock_path, pid.to_string());
        Self { lock_path, pid }
    }

    fn is_superseded(&self) -> bool {
        match std::fs::read_to_string(&self.lock_path) {
            Ok(content) => match content.trim().parse::<u32>() {
                Ok(active_pid) => active_pid != self.pid,
                Err(_) => false,
            },
            Err(_) => false,
        }
    }
}

/// Checks whether a ship's replicated server state matches the given `ShipCommand`.
pub(crate) fn is_ship_executing_command(
    ship: &Ship,
    base_pos: Option<Vec2>,
    cmd: &ShipCommand,
) -> bool {
    match cmd {
        ShipCommand::Mine(ast_id) => ship.is_mining && ship.target_asteroid_id == Some(*ast_id),
        ShipCommand::Dock => {
            if ship.is_mining || ship.target_asteroid_id.is_some() {
                return false;
            }
            match base_pos {
                Some(bpos) => {
                    let heading_to_base = ship
                        .target_position
                        .as_ref()
                        .is_some_and(|t| t.distance(&bpos) <= 1.0);
                    let already_at_base = ship.position.distance(&bpos) <= 45.0;
                    heading_to_base || already_at_base
                }
                None => ship.target_position.is_some(),
            }
        }
        ShipCommand::FlyTo(target) => {
            if ship.is_mining || ship.target_asteroid_id.is_some() {
                return false;
            }
            let heading_to_target = ship
                .target_position
                .as_ref()
                .is_some_and(|t| t.distance(target) <= 1.0);
            let already_at_target = ship.position.distance(target) <= 10.0;
            heading_to_target || already_at_target
        }
        ShipCommand::Stop => {
            !ship.is_mining && ship.target_asteroid_id.is_none() && ship.target_position.is_none()
        }
    }
}

/// Decides whether `cmd` should be sent to the server on `current_tick`.
/// Sends immediately when `cmd` differs from the last sent command for `ship_id`,
/// and retries after `COMMAND_RESEND_COOLDOWN_TICKS` if the server ship state
/// does not reflect `cmd` (e.g. if a previous reducer call was rejected or overwritten).
pub(crate) fn should_send_ship_command(
    last_sent: Option<&(ShipCommand, u64)>,
    cmd: &ShipCommand,
    ship: Option<&Ship>,
    base_pos: Option<Vec2>,
    current_tick: u64,
) -> bool {
    match last_sent {
        None => true,
        Some((last_cmd, sent_tick)) => {
            if last_cmd != cmd {
                return true;
            }
            if current_tick < sent_tick.saturating_add(COMMAND_RESEND_COOLDOWN_TICKS) {
                return false;
            }
            match ship {
                Some(s) => !is_ship_executing_command(s, base_pos, cmd),
                None => false,
            }
        }
    }
}

/// Startet den Bot-Laufzeitzyklus mit einem zustandsbehafteten `BaseController`.
pub fn run<C: BaseController>(mut controller: C) {
    let (config, config_path) = match BotConfig::load_with_path() {
        Ok(cfg) => cfg,
        Err(_) => {
            std::process::exit(1);
        }
    };

    let instance_lock = InstanceLock::acquire(&config_path);

    println!("================================================================================");
    println!("🚀 [SPACE BOT] Starte Heimatbasis- und Flottensteuerung...");
    println!("   Server:   {}", config.server_url);
    println!("   Modul:    {}", config.module_name);
    println!("================================================================================");

    let is_connected = Arc::new(AtomicBool::new(false));
    let is_subscribed = Arc::new(AtomicBool::new(false));

    let conn_flag = Arc::clone(&is_connected);
    let sub_flag = Arc::clone(&is_subscribed);

    let config_path_clone = config_path.clone();
    let auth_token_empty = config.auth_token.is_empty();
    let token_arg = if auth_token_empty {
        None
    } else {
        Some(config.auth_token.clone())
    };

    let conn_builder = DbConnection::builder()
        .with_uri(&config.server_url)
        .with_database_name(&config.module_name)
        .with_token(token_arg)
        .on_connect(move |_conn, identity, token| {
            conn_flag.store(true, Ordering::SeqCst);
            println!("✅ [SPACE BOT] Verbindung erfolgreich!");
            println!("   Identität: {}", identity.to_hex());

            if auth_token_empty {
                if let Err(e) = BotConfig::save_token(&config_path_clone, token) {
                    eprintln!(
                        "⚠️ [SPACE BOT] Konnte Token nicht in {:?} speichern: {}",
                        config_path_clone, e
                    );
                } else {
                    println!(
                        "💾 [SPACE BOT] Neuer Verbindungs-Token wurde in {:?} gespeichert.",
                        config_path_clone
                    );
                }
            }
        })
        .on_disconnect(move |_ctx, err| {
            if let Some(e) = err {
                eprintln!("⚠️ [SPACE BOT] Verbindung unterbrochen: {}", e);
            } else {
                println!("ℹ️ [SPACE BOT] Verbindung getrennt.");
            }
        });

    let conn = match conn_builder.build() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ [SPACE BOT] Fehler beim Verbindungsaufbau: {}", e);
            eprintln!(
                "   Läuft die SpacetimeDB-Instanz auf {}?",
                config.server_url
            );
            std::process::exit(1);
        }
    };

    let _net_handle = conn.run_threaded();

    conn.subscription_builder()
        .on_applied(move |_ctx| {
            sub_flag.store(true, Ordering::SeqCst);
            println!(
                "📡 [SPACE BOT] Sensor-Abonnement aktiv. Starte Basis- und Flotten-Loop (10 Ticks/s)..."
            );
        })
        .on_error(|_ctx, err| {
            eprintln!("❌ [SPACE BOT] Abonnement-Fehler: {}", err);
        })
        .subscribe([
            "SELECT * FROM config",
            "SELECT * FROM player",
            "SELECT * FROM my_base",
            "SELECT * FROM my_ships",
            "SELECT * FROM visible_bases",
            "SELECT * FROM visible_drones",
            "SELECT * FROM visible_asteroids",
            "SELECT * FROM unidentified_objects",
            "SELECT * FROM my_active_riddle",
            "SELECT * FROM my_tutorial_state",
            "SELECT * FROM game_event",
        ]);

    let mut tick_counter: u64 = 0;
    let mut last_status_tick: u64 = 0;
    let mut last_heartbeat = std::time::Instant::now()
        .checked_sub(Duration::from_secs(60))
        .unwrap_or_else(std::time::Instant::now);
    let mut last_emergency_build = std::time::Instant::now()
        .checked_sub(Duration::from_secs(60))
        .unwrap_or_else(std::time::Instant::now);

    let mut actions = Actions::new();
    let mut last_commands: HashMap<u32, (ShipCommand, u64)> = HashMap::new();

    loop {
        thread::sleep(Duration::from_millis(100));
        tick_counter += 1;

        if instance_lock.is_superseded() {
            println!(
                "ℹ️ [SPACE BOT] Neue Bot-Instanz wurde gestartet – beende diese ältere Instanz (PID {}).",
                instance_lock.pid
            );
            break;
        }

        if !is_subscribed.load(Ordering::SeqCst) {
            continue;
        }

        // Heartbeat alle 5 Sekunden senden
        if last_heartbeat.elapsed() >= Duration::from_secs(5) {
            last_heartbeat = std::time::Instant::now();
            let _ = conn.reducers.api_heartbeat();
        }

        let my_base: Option<BaseStation> = conn.db.my_base().iter().next();

        // Wenn noch keine Basis existiert: warten
        let base = match my_base {
            Some(b) => b,
            None => {
                if tick_counter >= last_status_tick + 30 {
                    last_status_tick = tick_counter;
                    println!(
                        "⏳ [SPACE BOT] Keine Heimatbasis sichtbar. Warte auf Zuweisung im Web-Dashboard..."
                    );
                    println!(
                        "   👉 Trage die oben angezeigte Identität im Web-Cockpit unter 'Heimatbasis-AI' ein."
                    );
                }
                continue;
            }
        };

        let mut my_ships: Vec<Ship> = conn.db.my_ships().iter().collect();
        my_ships.sort_by_key(|s| s.ship_id);
        let asteroids: Vec<PerceivedAsteroid> = conn.db.visible_asteroids().iter().collect();
        let drones: Vec<PerceivedDrone> = conn.db.visible_drones().iter().collect();
        let bases: Vec<PerceivedBase> = conn.db.visible_bases().iter().collect();
        let _unidentified: Vec<UnidentifiedContact> =
            conn.db.unidentified_objects().iter().collect();

        // Notfall-Drohne anfordern, falls alle Schiffe zerstört oder verloren sind
        if my_ships.is_empty() && last_emergency_build.elapsed() >= Duration::from_secs(5) {
            last_emergency_build = std::time::Instant::now();
            println!(
                "🛸 [SPACE BOT] Keine aktiven Schiffe vorhanden – fordere kostenlose Notfall-Drohne (Tier 1) an..."
            );
            if let Err(e) = conn.reducers.build_ship(1) {
                eprintln!("⚠️ [SPACE BOT] Notfall-Drohnenbau fehlgeschlagen: {}", e);
            }
        }

        let raw_riddle = conn.db.my_active_riddle().iter().next();
        let active_riddle = raw_riddle.as_ref().and_then(RiddleTask::from_public_riddle);

        let world = World {
            base: &base,
            fleet: &my_ships,
            asteroids: &asteroids,
            other_drones: &drones,
            other_bases: &bases,
            active_riddle: active_riddle.as_ref(),
            tick: tick_counter,
        };

        actions.clear();

        // Schüler-Tick aufrufen
        controller.tick(&world, &mut actions);

        // 1. Schiffsbefehle abarbeiten (bei Befehlsänderung oder Server-Abweichung senden & loggen)
        for (ship_id, cmd) in actions.ship_commands.drain(..) {
            let ship_state = my_ships.iter().find(|s| s.ship_id == ship_id);
            let should_send = should_send_ship_command(
                last_commands.get(&ship_id),
                &cmd,
                ship_state,
                Some(base.position),
                tick_counter,
            );

            if should_send {
                log_command(ship_id, &cmd);
                if let Err(e) = conn.reducers.command_ship(ship_id, cmd.clone()) {
                    eprintln!(
                        "⚠️ [SPACE BOT] Fehler beim Senden des Befehls für Drohne #{}: {}",
                        ship_id, e
                    );
                }
                last_commands.insert(ship_id, (cmd, tick_counter));
            }
        }

        // 2. Bau-Aufträge
        for tier in actions.build_requests.drain(..) {
            println!("🏗️ [SPACE BOT] Baue neue Drohne (Tier {})...", tier);
            if let Err(e) = conn.reducers.build_ship(tier) {
                eprintln!("⚠️ [SPACE BOT] Drohnenbau fehlgeschlagen: {}", e);
            }
        }

        // 3. Forschungs-Anfragen
        for track in actions.research_requests.drain(..) {
            println!(
                "🧪 [SPACE BOT] Fordere Forschungsrätsel an für: {}...",
                track.name()
            );
            if let Err(e) = conn.reducers.request_riddle(track.as_u8()) {
                eprintln!("⚠️ [SPACE BOT] Forschungs-Anfrage fehlgeschlagen: {}", e);
            }
        }

        // 4. Rätsel-Lösungen einreichen
        for solution in actions.riddle_solutions.drain(..) {
            println!(
                "💡 [SPACE BOT] Reiche Lösung für Forschungs-Track {} ein...",
                solution.track()
            );
            if let Err(e) = conn
                .reducers
                .submit_riddle_solution(solution.track(), solution.to_json_string())
            {
                eprintln!("⚠️ [SPACE BOT] Einreichen der Lösung fehlgeschlagen: {}", e);
            }
        }
    }
}

/// Startet den Bot-Laufzeitzyklus mit einem `ShipController` für ein einzelnes Schiff (oder Kriegsschiff).
pub fn run_ship<C: ShipController>(mut controller: C) {
    let (config, config_path) = match BotConfig::load_with_path() {
        Ok(cfg) => cfg,
        Err(_) => {
            std::process::exit(1);
        }
    };

    let instance_lock = InstanceLock::acquire(&config_path);

    println!("================================================================================");
    println!("🚀 [SPACE BOT] Starte Einzelschiff-/Kriegsschiffsteuerung...");
    println!("   Server:   {}", config.server_url);
    println!("   Modul:    {}", config.module_name);
    println!("================================================================================");

    let is_connected = Arc::new(AtomicBool::new(false));
    let is_subscribed = Arc::new(AtomicBool::new(false));

    let conn_flag = Arc::clone(&is_connected);
    let sub_flag = Arc::clone(&is_subscribed);

    let config_path_clone = config_path.clone();
    let auth_token_empty = config.auth_token.is_empty();
    let token_arg = if auth_token_empty {
        None
    } else {
        Some(config.auth_token.clone())
    };

    let conn_builder = DbConnection::builder()
        .with_uri(&config.server_url)
        .with_database_name(&config.module_name)
        .with_token(token_arg)
        .on_connect(move |_conn, identity, token| {
            conn_flag.store(true, Ordering::SeqCst);
            println!("✅ [SPACE BOT] Verbindung erfolgreich!");
            println!("   Identität: {}", identity.to_hex());

            if auth_token_empty {
                if let Err(e) = BotConfig::save_token(&config_path_clone, token) {
                    eprintln!(
                        "⚠️ [SPACE BOT] Konnte Token nicht in {:?} speichern: {}",
                        config_path_clone, e
                    );
                } else {
                    println!(
                        "💾 [SPACE BOT] Neuer Verbindungs-Token wurde in {:?} gespeichert.",
                        config_path_clone
                    );
                }
            }
        })
        .on_disconnect(move |_ctx, err| {
            if let Some(e) = err {
                eprintln!("⚠️ [SPACE BOT] Verbindung unterbrochen: {}", e);
            } else {
                println!("ℹ️ [SPACE BOT] Verbindung getrennt.");
            }
        });

    let conn = match conn_builder.build() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ [SPACE BOT] Fehler beim Verbindungsaufbau: {}", e);
            eprintln!(
                "   Läuft die SpacetimeDB-Instanz auf {}?",
                config.server_url
            );
            std::process::exit(1);
        }
    };

    let _net_handle = conn.run_threaded();

    conn.subscription_builder()
        .on_applied(move |_ctx| {
            sub_flag.store(true, Ordering::SeqCst);
            println!("📡 [SPACE BOT] Sensor-Abonnement aktiv. Starte Schiff-Loop (10 Ticks/s)...");
        })
        .on_error(|_ctx, err| {
            eprintln!("❌ [SPACE BOT] Abonnement-Fehler: {}", err);
        })
        .subscribe([
            "SELECT * FROM config",
            "SELECT * FROM player",
            "SELECT * FROM my_base",
            "SELECT * FROM my_ships",
            "SELECT * FROM visible_bases",
            "SELECT * FROM visible_drones",
            "SELECT * FROM visible_asteroids",
            "SELECT * FROM unidentified_objects",
            "SELECT * FROM game_event",
        ]);

    let mut tick_counter: u64 = 0;
    let mut last_status_tick: u64 = 0;
    let mut last_heartbeat = std::time::Instant::now()
        .checked_sub(Duration::from_secs(60))
        .unwrap_or_else(std::time::Instant::now);

    let mut actions = Actions::new();
    let mut last_commands: HashMap<u32, (ShipCommand, u64)> = HashMap::new();

    loop {
        thread::sleep(Duration::from_millis(100));
        tick_counter += 1;

        if instance_lock.is_superseded() {
            println!(
                "ℹ️ [SPACE BOT] Neue Bot-Instanz wurde gestartet – beende diese ältere Instanz (PID {}).",
                instance_lock.pid
            );
            break;
        }

        if !is_subscribed.load(Ordering::SeqCst) {
            continue;
        }

        // Heartbeat alle 5 Sekunden senden
        if last_heartbeat.elapsed() >= Duration::from_secs(5) {
            last_heartbeat = std::time::Instant::now();
            let _ = conn.reducers.api_heartbeat();
        }

        let mut my_ships: Vec<Ship> = conn.db.my_ships().iter().collect();
        my_ships.sort_by_key(|s| s.ship_id);

        // Wenn noch kein Schiff sichtbar/zugewiesen ist: warten
        let ship = match my_ships.first() {
            Some(s) => s.clone(),
            None => {
                if tick_counter >= last_status_tick + 30 {
                    last_status_tick = tick_counter;
                    println!(
                        "⏳ [SPACE BOT] Kein Schiff sichtbar. Warte auf Zuweisung im Web-Dashboard..."
                    );
                    println!(
                        "   👉 Klicke ein Schiff im Web-Radar an und binde die Identität im Inspektor-HUD ein."
                    );
                }
                continue;
            }
        };

        let my_base: Option<BaseStation> = conn.db.my_base().iter().next();
        let asteroids: Vec<PerceivedAsteroid> = conn.db.visible_asteroids().iter().collect();
        let drones: Vec<PerceivedDrone> = conn.db.visible_drones().iter().collect();
        let bases: Vec<PerceivedBase> = conn.db.visible_bases().iter().collect();

        let ship_world = ShipWorld {
            ship: &ship,
            fleet: &my_ships,
            asteroids: &asteroids,
            other_drones: &drones,
            other_bases: &bases,
            base: my_base.as_ref(),
            tick: tick_counter,
        };

        actions.clear();

        controller.tick(&ship_world, &mut actions);

        let base_pos = my_base.as_ref().map(|b| b.position);
        for (ship_id, cmd) in actions.ship_commands.drain(..) {
            let ship_state = my_ships.iter().find(|s| s.ship_id == ship_id);
            let should_send = should_send_ship_command(
                last_commands.get(&ship_id),
                &cmd,
                ship_state,
                base_pos,
                tick_counter,
            );

            if should_send {
                log_command(ship_id, &cmd);
                if let Err(e) = conn.reducers.command_ship(ship_id, cmd.clone()) {
                    eprintln!(
                        "⚠️ [SPACE BOT] Fehler beim Senden des Befehls für Drohne #{}: {}",
                        ship_id, e
                    );
                }
                last_commands.insert(ship_id, (cmd, tick_counter));
            }
        }
    }
}

fn log_command(ship_id: u32, cmd: &ShipCommand) {
    match cmd {
        ShipCommand::Mine(ast_id) => {
            println!(
                "🛸 [DROHNE #{}] Neuer Befehl: Erz abbauen an Asteroid #{}",
                ship_id, ast_id
            );
        }
        ShipCommand::Dock => {
            println!(
                "🛸 [DROHNE #{}] Neuer Befehl: Zur Heimatbasis zurückkehren (Dock)",
                ship_id
            );
        }
        ShipCommand::FlyTo(target) => {
            println!(
                "🛸 [DROHNE #{}] Neuer Befehl: Fliege zu Ziel ({:.0}, {:.0})",
                ship_id, target.x, target.y
            );
        }
        ShipCommand::Stop => {
            println!("🛸 [DROHNE #{}] Neuer Befehl: Stopp", ship_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spacetimedb_sdk::Identity;

    fn sample_ship(
        ship_id: u32,
        position: Vec2,
        target_position: Option<Vec2>,
        target_asteroid_id: Option<u32>,
        is_mining: bool,
    ) -> Ship {
        Ship {
            ship_id,
            owner: Identity::ZERO,
            universe: 0,
            ship_tier: 1,
            position,
            velocity: Vec2::new(0.0, 0.0),
            target_position,
            heading_rad: 0.0,
            speed: 0.0,
            max_speed: 140.0,
            energy: 100.0,
            max_energy: 100.0,
            cargo_ore: 50,
            max_cargo: 50,
            target_asteroid_id,
            is_mining,
            is_stranded: false,
            signal_countdown: 20,
        }
    }

    #[test]
    fn resends_dock_when_ship_still_mining_on_server_after_cooldown() {
        let base_pos = Vec2::new(1529.0, -1248.55);
        // Drone #2 is still sitting on Asteroid #1490 with is_mining = true
        let stuck_drone = sample_ship(2, Vec2::new(1606.17, -1199.31), None, Some(1490), true);
        let last_sent = (ShipCommand::Dock, 10);

        // Within cooldown window (ticks 11..19): do not spam
        assert!(!should_send_ship_command(
            Some(&last_sent),
            &ShipCommand::Dock,
            Some(&stuck_drone),
            Some(base_pos),
            15,
        ));

        // Once cooldown expires (tick 20): ship is still mining Asteroid #1490, so Dock must be re-sent!
        assert!(should_send_ship_command(
            Some(&last_sent),
            &ShipCommand::Dock,
            Some(&stuck_drone),
            Some(base_pos),
            20,
        ));
    }

    #[test]
    fn does_not_resend_dock_when_ship_is_already_flying_to_base_or_docked() {
        let base_pos = Vec2::new(1529.0, -1248.55);
        let returning_drone =
            sample_ship(2, Vec2::new(1580.0, -1210.0), Some(base_pos), None, false);
        let last_sent = (ShipCommand::Dock, 10);

        assert!(!should_send_ship_command(
            Some(&last_sent),
            &ShipCommand::Dock,
            Some(&returning_drone),
            Some(base_pos),
            25,
        ));

        let docked_drone = sample_ship(2, base_pos, None, None, false);
        assert!(!should_send_ship_command(
            Some(&last_sent),
            &ShipCommand::Dock,
            Some(&docked_drone),
            Some(base_pos),
            25,
        ));
    }

    #[test]
    fn actions_keep_latest_command_per_ship_within_tick() {
        let mut actions = Actions::new();
        actions.mine(2, 1490);
        actions.dock(2);
        assert_eq!(actions.ship_commands, vec![(2, ShipCommand::Dock)]);
    }
}
