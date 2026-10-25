use crate::action::Action;
use crate::config::BotConfig;
use crate::module_bindings::api_heartbeat_reducer::api_heartbeat;
use crate::module_bindings::build_ship_reducer::build_ship;
use crate::module_bindings::command_ship_reducer::command_ship;
use crate::module_bindings::my_base_table::MyBaseTableAccess;
use crate::module_bindings::my_ships_table::MyShipsTableAccess;
use crate::module_bindings::unidentified_objects_table::UnidentifiedObjectsTableAccess;
use crate::module_bindings::visible_asteroids_table::VisibleAsteroidsTableAccess;
use crate::module_bindings::visible_bases_table::VisibleBasesTableAccess;
use crate::module_bindings::visible_drones_table::VisibleDronesTableAccess;
use crate::module_bindings::{
    BaseStation, DbConnection, PerceivedAsteroid, PerceivedBase, PerceivedDrone, Ship,
    UnidentifiedContact,
};
use crate::radar::Radar;
use spacetimedb_sdk::{DbContext, Table};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

/// Startet den Bot-Laufzeitzyklus mit der vom Schüler bereitgestellten `decide`-Funktion.
pub fn run<F>(decide: F)
where
    F: Fn(&Ship, &Radar) -> Action + Send + Sync + 'static,
{
    // 1. Konfiguration laden
    let config = match BotConfig::load() {
        Ok(cfg) => cfg,
        Err(_) => {
            std::process::exit(1);
        }
    };

    println!("================================================================================");
    println!("🚀 [SPACE BOT] Starte Bot-Client...");
    println!("   Server:   {}", config.server_url);
    println!("   Modul:    {}", config.module_name);
    println!("================================================================================");

    let is_connected = Arc::new(AtomicBool::new(false));
    let is_subscribed = Arc::new(AtomicBool::new(false));

    let conn_flag = Arc::clone(&is_connected);
    let sub_flag = Arc::clone(&is_subscribed);

    // 2. Verbindung zu SpacetimeDB herstellen
    let conn_builder = DbConnection::builder()
        .with_uri(&config.server_url)
        .with_database_name(&config.module_name)
        .with_token(Some(config.auth_token.clone()))
        .on_connect(move |_conn, identity, _token| {
            conn_flag.store(true, Ordering::SeqCst);
            println!("✅ [SPACE BOT] Verbindung erfolgreich!");
            println!("   Identität: {:?}", identity);
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

    // 3. SpacetimeDB Background Network Worker starten
    let _net_handle = conn.run_threaded();

    // 4. Tabellen & Views abonnieren
    conn.subscription_builder()
        .on_applied(move |_ctx| {
            sub_flag.store(true, Ordering::SeqCst);
            println!(
                "📡 [SPACE BOT] Radar-Abonnement aktiv. Starte Entscheidungs-Loop (10 Ticks/s)..."
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

    // 5. Haupt-Tick-Loop (10 Hz = alle 100ms)
    loop {
        thread::sleep(Duration::from_millis(100));
        tick_counter += 1;

        if !is_subscribed.load(Ordering::SeqCst) {
            continue;
        }

        // Send API heartbeat every 5 seconds to keep connection timestamp fresh on server
        if last_heartbeat.elapsed() >= Duration::from_secs(5) {
            last_heartbeat = std::time::Instant::now();
            let _ = conn.reducers.api_heartbeat();
        }

        // Tabellendaten aus dem lokalen In-Memory-Cache lesen
        let my_ships: Vec<Ship> = conn.db.my_ships().iter().collect();
        let my_base: Option<BaseStation> = conn.db.my_base().iter().next();
        let asteroids: Vec<PerceivedAsteroid> = conn.db.visible_asteroids().iter().collect();
        let drones: Vec<PerceivedDrone> = conn.db.visible_drones().iter().collect();
        let bases: Vec<PerceivedBase> = conn.db.visible_bases().iter().collect();
        let unidentified: Vec<UnidentifiedContact> =
            conn.db.unidentified_objects().iter().collect();

        if my_ships.is_empty() {
            if let Some(_base) = &my_base {
                if last_emergency_build.elapsed() >= Duration::from_secs(5) {
                    last_emergency_build = std::time::Instant::now();
                    println!(
                        "🛸 [SPACE BOT] Keine aktiven Schiffe vorhanden – fordere automatische Notfall-Drohne (Tier 1) an..."
                    );
                    if let Err(e) = conn.reducers.build_ship(1) {
                        eprintln!(
                            "⚠️ [SPACE BOT] Fehler beim Anfordern der Notfall-Drohne: {}",
                            e
                        );
                    }
                }
            } else if tick_counter >= last_status_tick + 30 {
                last_status_tick = tick_counter;
                println!(
                    "⏳ [SPACE BOT] Noch keine Heimatbasis gefunden. Bitte platziere deine Basis im Web-Dashboard!"
                );
            }
            continue;
        }

        let radar = Radar {
            base_station: my_base,
            my_ships: my_ships.clone(),
            asteroids,
            other_drones: drones,
            other_bases: bases,
            unidentified_contacts: unidentified,
        };

        // Rufe die decide-Funktion des Schülers für jedes eigene Schiff auf
        for ship in &my_ships {
            let action = decide(ship, &radar);
            let cmd = action.into_ship_command();

            if let Err(e) = conn.reducers.command_ship(ship.ship_id, cmd) {
                eprintln!(
                    "⚠️ [SPACE BOT] Fehler beim Senden des Kommandos für Schiff {}: {}",
                    ship.ship_id, e
                );
            }
        }
    }
}
