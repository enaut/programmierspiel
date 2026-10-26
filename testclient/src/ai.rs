#![allow(unused_imports, unused_variables)]

use space_bot_sdk::{Action, Radar, Ship, Vec2};

/// 🤖 HIER PROGRAMMIERST DU DEINEN BOT!
///
/// Diese Funktion wird in jedem Spiel-Tick (ca. 10-mal pro Sekunde) für jedes
/// deiner Schiffe aufgerufen.
///
/// Du erhältst:
///   - `ship`: Informationen über das aktuelle Schiff (Position, Energie, Frachtladung, ...)
///   - `radar`: Die Umgebungssicht deines Radars (Heimatbasis, Asteroiden, andere Schiffe, ...)
///
/// Du gibst zurück:
///   - eine `Action`: Was soll das Schiff tun?
///     * `Action::fly_to(ziel)`      -> Zu einer Position fliegen
///     * `Action::mine(asteroid_id)` -> Zu einem Asteroiden fliegen und Erz abbauen
///     * `Action::dock()`            -> Zur Heimatbasis fliegen (Auftanken & Entladen)
///     * `Action::stop()`            -> Sofort anhalten
pub fn decide(ship: &Ship, radar: &Radar) -> Action {
    // =========================================================================
    // START-ZUSTAND:
    // Standardmäßig stoppt das Schiff. Entferne oder passe den folgenden Code an!
    // =========================================================================

    // -------------------------------------------------------------------------
    // 💡 BEISPIEL STUFE 1: Der erste Flug zum nächsten Asteroiden
    // -------------------------------------------------------------------------
    // Wenn ein Asteroid in Sicht ist, fliege hin und baue Erz ab:
    //
    // if let Some(asteroid) = radar.closest_asteroid(&ship.position) {
    //     return Action::mine(asteroid.asteroid_id);
    // }

    // -------------------------------------------------------------------------
    // 💡 BEISPIEL STUFE 2: Fracht- & Batteriemanagement (if / else)
    // -------------------------------------------------------------------------
    // Wenn der Frachtraum voll ist ODER die Energie knapp wird (< 20%),
    // fliege zurück zur Basis zum Entladen und Auftanken:
    //
    if ship.cargo_is_full() || ship.energy_percent() < 0.1 {
        println!("Full - docking");
        return Action::dock();
    }

    println!("{:?}", radar.unidentified_contacts);

    if let Some(asteroid) = radar.closest_asteroid_with_ore(&ship.position) {
        println!("mining");
        return Action::mine(asteroid.asteroid_id);
    }
    if let Some(x) = radar.unidentified_contacts.iter().next() {
        println!("searching");
        return Action::fly_to(x.position.clone());
    }

    if radar.closest_asteroid_with_ore(&ship.position).is_none() {
        println!("nothing of interest");
        return Action::dock();
    }
    // -------------------------------------------------------------------------
    // 💡 BEISPIEL STUFE 3: Eigene Asteroidensuche mit Schleifen (for / if)
    // -------------------------------------------------------------------------
    // Durchsuche alle sichtbaren Asteroiden nach demjenigen mit dem meisten Erz:
    //
    // let mut best_asteroid = None;
    // let mut max_ore = 0;
    // for ast in &radar.asteroids {
    //     if ast.remaining_ore > max_ore {
    //         max_ore = ast.remaining_ore;
    //         best_asteroid = Some(ast);
    //     }
    // }
    // if let Some(target) = best_asteroid {
    //     return Action::mine(target.asteroid_id);
    // }

    // -------------------------------------------------------------------------
    // 💡 BEISPIEL STUFE 4: Rohstoffe filtern mit Pattern Matching (match)
    // -------------------------------------------------------------------------
    // Bevorzuge Titan (3) oder Eisen (1) gegenüber einfachem Gestein:
    //
    // for ast in &radar.asteroids {
    //     match ast.resource_type {
    //         3 => return Action::mine(ast.asteroid_id), // Titan
    //         1 => return Action::mine(ast.asteroid_id), // Eisen
    //         _ => {}
    //     }
    // }

    Action::fly_to(Vec2::new(
        ship.ship_id as f32 * 800.,
        -1.0 * ship.ship_id as f32 * 800.,
    ))
}
