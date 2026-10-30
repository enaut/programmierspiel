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
    UnidentifiedContact,
};
use crate::research::RiddleTask;
use crate::world::{BaseController, ShipController, ShipWorld, World};
use spacetimedb_sdk::{DbContext, Table};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

/// Startet den Bot-Laufzeitzyklus mit einem zustandsbehafteten `BaseController`.
pub fn run<C: BaseController>(mut controller: C) {
    let (config, config_path) = match BotConfig::load_with_path() {
        Ok(cfg) => cfg,
        Err(_) => {
            std::process::exit(1);
        }
    };

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
    let mut last_commands: HashMap<u32, ShipCommand> = HashMap::new();

    loop {
        thread::sleep(Duration::from_millis(100));
        tick_counter += 1;

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

        let my_ships: Vec<Ship> = conn.db.my_ships().iter().collect();
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

        // 1. Schiffsbefehle abarbeiten (nur bei Befehlsänderung senden & loggen)
        for (ship_id, cmd) in actions.ship_commands.drain(..) {
            let has_changed = last_commands
                .get(&ship_id)
                .map_or(true, |last_cmd| *last_cmd != cmd);

            if has_changed {
                log_command(ship_id, &cmd);
                if let Err(e) = conn.reducers.command_ship(ship_id, cmd.clone()) {
                    eprintln!(
                        "⚠️ [SPACE BOT] Fehler beim Senden des Befehls für Drohne #{}: {}",
                        ship_id, e
                    );
                }
                last_commands.insert(ship_id, cmd);
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
    let mut last_commands: HashMap<u32, ShipCommand> = HashMap::new();

    loop {
        thread::sleep(Duration::from_millis(100));
        tick_counter += 1;

        if !is_subscribed.load(Ordering::SeqCst) {
            continue;
        }

        // Heartbeat alle 5 Sekunden senden
        if last_heartbeat.elapsed() >= Duration::from_secs(5) {
            last_heartbeat = std::time::Instant::now();
            let _ = conn.reducers.api_heartbeat();
        }

        let my_ships: Vec<Ship> = conn.db.my_ships().iter().collect();

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

        for (ship_id, cmd) in actions.ship_commands.drain(..) {
            let has_changed = last_commands
                .get(&ship_id)
                .map_or(true, |last_cmd| *last_cmd != cmd);

            if has_changed {
                log_command(ship_id, &cmd);
                if let Err(e) = conn.reducers.command_ship(ship_id, cmd.clone()) {
                    eprintln!(
                        "⚠️ [SPACE BOT] Fehler beim Senden des Befehls für Drohne #{}: {}",
                        ship_id, e
                    );
                }
                last_commands.insert(ship_id, cmd);
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
