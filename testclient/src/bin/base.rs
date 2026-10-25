use space_bot_sdk::module_bindings::api_heartbeat_reducer::api_heartbeat;
use space_bot_sdk::module_bindings::game_event_table::GameEventTableAccess;
use space_bot_sdk::module_bindings::my_active_riddle_table::MyActiveRiddleTableAccess;
use space_bot_sdk::module_bindings::my_base_table::MyBaseTableAccess;
use space_bot_sdk::module_bindings::request_riddle_reducer::request_riddle;
use space_bot_sdk::module_bindings::submit_riddle_solution_reducer::submit_riddle_solution;
use space_bot_sdk::module_bindings::{BaseStation, DbConnection, PublicRiddle};
use space_bot_sdk::{
    BotConfig, DenoisePayload, DenoiseSolution, DistancePayload, DistanceSolution, SortPayload,
    SortSolution,
};
use spacetimedb_sdk::{DbContext, Table};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use testclient::research::{self, UpgradeTrack};

/// Wähle hier gezielt einen Upgrade-Zweig aus, oder setze `None` für automatische Forschung:
/// - `Some(UpgradeTrack::DroneScanner)`       -> Track 1: Drohnenscan-Reichweite (Sortieren)
/// - `Some(UpgradeTrack::BaseAsteroidScanner)`-> Track 2: Basen- & Asteroidenscan (Distanzen)
/// - `Some(UpgradeTrack::RadarHorizon)`      -> Track 3: Radar-Horizont (Denoise)
/// - `None`                                  -> Erforscht automatisch den Zweig mit der niedrigsten Stufe
const TARGET_TRACK: Option<UpgradeTrack> = None;

fn main() {
    println!("================================================================================");
    println!("🏛️ [BASE STATION] Starte Heimatbasis- & Forschungs-Steuerung...");
    println!("================================================================================");

    // 1. Konfiguration laden
    let config = match BotConfig::load() {
        Ok(cfg) => cfg,
        Err(_) => std::process::exit(1),
    };

    println!("   Server:   {}", config.server_url);
    println!("   Modul:    {}", config.module_name);

    let is_connected = Arc::new(AtomicBool::new(false));
    let is_subscribed = Arc::new(AtomicBool::new(false));

    let conn_flag = Arc::clone(&is_connected);
    let sub_flag = Arc::clone(&is_subscribed);

    // 2. Verbindung herstellen
    let conn_builder = DbConnection::builder()
        .with_uri(&config.server_url)
        .with_database_name(&config.module_name)
        .with_token(Some(config.auth_token.clone()))
        .on_connect(move |_conn, identity, _token| {
            conn_flag.store(true, Ordering::SeqCst);
            println!(
                "✅ [BASE STATION] Verbindung erfolgreich! (Identität: {:?})",
                identity
            );
        })
        .on_disconnect(move |_ctx, err| {
            if let Some(e) = err {
                eprintln!("⚠️ [BASE STATION] Verbindung unterbrochen: {}", e);
            } else {
                println!("ℹ️ [BASE STATION] Verbindung getrennt.");
            }
        });

    let conn = match conn_builder.build() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ [BASE STATION] Fehler beim Verbindungsaufbau: {}", e);
            std::process::exit(1);
        }
    };

    let _net_handle = conn.run_threaded();

    // 3. Relevante Tabellen abonnieren
    conn.subscription_builder()
        .on_applied(move |_ctx| {
            sub_flag.store(true, Ordering::SeqCst);
            println!("📡 [BASE STATION] Stations-Abonnement aktiv. Starte Forschungs-Loop...");
        })
        .on_error(|_ctx, err| {
            eprintln!("❌ [BASE STATION] Abonnement-Fehler: {}", err);
        })
        .subscribe([
            "SELECT * FROM config",
            "SELECT * FROM player",
            "SELECT * FROM my_base",
            "SELECT * FROM my_active_riddle",
            "SELECT * FROM game_event",
        ]);

    let mut last_event_id: u64 = 0;
    let mut last_status_time = std::time::Instant::now();
    let mut last_ore_warning = std::time::Instant::now();
    let mut request_in_flight = false;
    let mut last_heartbeat = std::time::Instant::now()
        .checked_sub(Duration::from_secs(60))
        .unwrap_or_else(std::time::Instant::now);

    // 4. Haupt-Forschungszyklus
    loop {
        thread::sleep(Duration::from_millis(200));

        if !is_subscribed.load(Ordering::SeqCst) {
            continue;
        }

        // Send API heartbeat every 5 seconds to keep connection timestamp fresh on server
        if last_heartbeat.elapsed() >= Duration::from_secs(5) {
            last_heartbeat = std::time::Instant::now();
            let _ = conn.reducers.api_heartbeat();
        }

        // Neue Spielereignisse auf der Konsole ausgeben
        let events: Vec<_> = conn.db.game_event().iter().collect();
        for ev in events {
            if ev.event_id > last_event_id {
                last_event_id = ev.event_id;
                match ev.event_type.as_str() {
                    "UPGRADE_UNLOCKED" => {
                        println!("\n{}", "=".repeat(80));
                        println!("🎉 {}", ev.message);
                        println!("{}\n", "=".repeat(80));
                    }
                    "MODULE_BURNOUT" => {
                        println!("\n{}", "!".repeat(80));
                        println!("💥 {}", ev.message);
                        println!("{}\n", "!".repeat(80));
                    }
                    "RIDDLE_SOLVED" => {
                        println!("✔ [FORSCHUNG] {}", ev.message);
                    }
                    "PUZZLE_FAILED" => {
                        println!("❌ [FEHLER] {}", ev.message);
                    }
                    _ => {}
                }
            }
        }

        let my_base: Option<BaseStation> = conn.db.my_base().iter().next();
        let base = match my_base {
            Some(b) => b,
            None => {
                if last_status_time.elapsed().as_secs() >= 5 {
                    last_status_time = std::time::Instant::now();
                    println!(
                        "⏳ [BASE STATION] Noch keine Heimatbasis gefunden. Bitte platziere deine Basis im Web-Dashboard!"
                    );
                }
                continue;
            }
        };

        // Periodischer Statusbericht
        if last_status_time.elapsed().as_secs() >= 8 {
            last_status_time = std::time::Instant::now();
            println!(
                "🏠 [STATUS] Erz: {} | Drohnen-Scan: Lvl {} ({}/100) | Asteroiden: Lvl {} ({}/100) | Horizont: Lvl {} ({}/100)",
                base.stored_ore,
                base.drone_scanner_level,
                base.drone_scanner_progress,
                base.base_scanner_level,
                base.base_scanner_progress,
                base.radar_horizon_level,
                base.radar_horizon_progress
            );
        }

        // Ziel-Track auswählen
        let active_track = match TARGET_TRACK {
            Some(t) => t,
            None => select_next_track(&base),
        };

        let (current_level, current_progress) = match active_track {
            UpgradeTrack::DroneScanner => (base.drone_scanner_level, base.drone_scanner_progress),
            UpgradeTrack::BaseAsteroidScanner => {
                (base.base_scanner_level, base.base_scanner_progress)
            }
            UpgradeTrack::RadarHorizon => (base.radar_horizon_level, base.radar_horizon_progress),
        };

        if current_level >= 5 {
            // Alle Tracks auf Level 5?
            if base.drone_scanner_level >= 5
                && base.base_scanner_level >= 5
                && base.radar_horizon_level >= 5
            {
                if last_status_time.elapsed().as_secs() >= 10 {
                    println!(
                        "🏆 [BASE STATION] Alle Sensor-Upgrades haben Maximalstufe 5 erreicht!"
                    );
                }
            }
            continue;
        }

        // Benötigtes Erz für das nächste Rätsel berechnen
        // Formel: (Stufe - 1) * 100 + Fortschritt + 1
        let required_ore =
            (current_level.saturating_sub(1) as u32) * 100 + (current_progress as u32) + 1;

        // Prüfen, ob ausreichend Erz im Basislager vorhanden ist
        if base.stored_ore < required_ore {
            if last_ore_warning.elapsed().as_secs() >= 6 {
                last_ore_warning = std::time::Instant::now();
                println!(
                    "⏳ [BASE STATION] Warte auf Erz für Forschung ({:?} Stufe {}): {}/{} Erz vorhanden...",
                    active_track, current_level, base.stored_ore, required_ore
                );
            }
            continue;
        }

        // Aktives Rätsel prüfen
        let active_riddle: Option<PublicRiddle> = conn.db.my_active_riddle().iter().next();

        match active_riddle {
            None => {
                if !request_in_flight {
                    request_in_flight = true;
                    let track_u8 = active_track.as_u8();
                    let _ = conn.reducers.request_riddle(track_u8);
                }
            }
            Some(riddle) => {
                request_in_flight = false;
                // Löse das Rätsel programmatisch mit den Schüler-Funktionen in `src/research.rs`
                match solve_and_format_solution(&riddle) {
                    Ok(solution_json) => {
                        let _ = conn
                            .reducers
                            .submit_riddle_solution(riddle.track, solution_json);
                        // Kurze Pause, damit der Server das Ergebnis verarbeitet
                        thread::sleep(Duration::from_millis(150));
                    }
                    Err(err) => {
                        eprintln!(
                            "⚠️ [BASE STATION] Fehler beim Lösen des Rätsels (Track {}): {}",
                            riddle.track, err
                        );
                        thread::sleep(Duration::from_secs(1));
                    }
                }
            }
        }
    }
}

/// Wählt automatisch den Upgrade-Zweig mit der niedrigsten Stufe aus.
fn select_next_track(base: &BaseStation) -> UpgradeTrack {
    let mut tracks = [
        (
            UpgradeTrack::DroneScanner,
            base.drone_scanner_level,
            base.drone_scanner_progress,
        ),
        (
            UpgradeTrack::BaseAsteroidScanner,
            base.base_scanner_level,
            base.base_scanner_progress,
        ),
        (
            UpgradeTrack::RadarHorizon,
            base.radar_horizon_level,
            base.radar_horizon_progress,
        ),
    ];

    // Sortiere nach Stufe aufsteigend, dann nach Fortschritt absteigend (angefangenes beenden)
    tracks.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| b.2.cmp(&a.2)));

    // Nimm den ersten Track, der noch nicht Level 5 ist
    for (t, lvl, _) in tracks {
        if lvl < 5 {
            return t;
        }
    }

    UpgradeTrack::DroneScanner
}

/// Ruft die passende Schülerfunktion in `research.rs` auf und serialisiert die Antwort.
fn solve_and_format_solution(riddle: &PublicRiddle) -> Result<String, String> {
    match riddle.track {
        1 => {
            // Track 1: Sortieren
            let payload: SortPayload = serde_json::from_str(&riddle.payload)
                .map_err(|e| format!("Fehler beim Parsen von SortPayload: {}", e))?;

            let sorted = research::solve_sorting(&payload.numbers);
            let solution = SortSolution { sorted };
            serde_json::to_string(&solution).map_err(|e| e.to_string())
        }
        2 => {
            // Track 2: Distanzberechnung
            let payload: DistancePayload = serde_json::from_str(&riddle.payload)
                .map_err(|e| format!("Fehler beim Parsen von DistancePayload: {}", e))?;

            let distances = research::solve_distances(payload.base, &payload.targets);
            let solution = DistanceSolution { distances };
            serde_json::to_string(&solution).map_err(|e| e.to_string())
        }
        3 => {
            // Track 3: Denoise / Spikes filtern
            let payload: DenoisePayload = serde_json::from_str(&riddle.payload)
                .map_err(|e| format!("Fehler beim Parsen von DenoisePayload: {}", e))?;

            let cleaned = research::solve_denoise(&payload.readings, payload.threshold);
            let solution = DenoiseSolution { cleaned };
            serde_json::to_string(&solution).map_err(|e| e.to_string())
        }
        _ => Err(format!("Unbekannter UpgradeTrack: {}", riddle.track)),
    }
}
