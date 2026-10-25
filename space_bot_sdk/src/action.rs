use crate::module_bindings::{ShipCommand, Vec2};

/// Repräsentiert eine Aktion, die ein Schiff ausführen soll.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// Fliege zu den angegebenen 2D-Koordinaten im Weltraum.
    FlyTo(Vec2),
    /// Stoppe das Schiff sofort und setze Geschwindigkeit auf 0.
    Stop,
    /// Fliege zum angegebenen Asteroiden und baue dort Erz ab.
    Mine(u32),
    /// Fliege zur eigenen Heimatbasis zurück und docke an (lädt Batterie auf & liefert Erz ab).
    Dock,
}

impl Action {
    /// Hilfsmethode zum Erzeugen einer Flugaktion.
    pub fn fly_to(target: impl Into<Vec2>) -> Self {
        Action::FlyTo(target.into())
    }

    /// Hilfsmethode zum Stoppen des Schiffes.
    pub fn stop() -> Self {
        Action::Stop
    }

    /// Hilfsmethode zum Starten des Erzabbaus an einem Asteroiden (über dessen `asteroid_id`).
    pub fn mine(asteroid_id: u32) -> Self {
        Action::Mine(asteroid_id)
    }

    /// Hilfsmethode zum Andocken an der Heimatbasis.
    pub fn dock() -> Self {
        Action::Dock
    }

    /// Konvertiert diese Aktion in das interne SpacetimeDB-Kommando.
    pub fn into_ship_command(self) -> ShipCommand {
        match self {
            Action::FlyTo(target) => ShipCommand::FlyTo(target),
            Action::Stop => ShipCommand::Stop,
            Action::Mine(asteroid_id) => ShipCommand::Mine(asteroid_id),
            Action::Dock => ShipCommand::Dock,
        }
    }
}
