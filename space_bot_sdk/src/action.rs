use crate::module_bindings::{ShipCommand, Vec2};
use crate::research::{RiddleSolution, UpgradeTrack};

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

/// Sammelbehälter für alle Befehle, die in einem Tick ausgeführt werden sollen.
#[derive(Debug, Clone, Default)]
pub struct Actions {
    pub(crate) ship_commands: Vec<(u32, ShipCommand)>,
    pub(crate) build_requests: Vec<u8>,
    pub(crate) research_requests: Vec<UpgradeTrack>,
    pub(crate) riddle_solutions: Vec<RiddleSolution>,
}

impl Actions {
    pub fn new() -> Self {
        Self::default()
    }

    fn set_ship_command(&mut self, ship_id: u32, cmd: ShipCommand) {
        if let Some(existing) = self
            .ship_commands
            .iter_mut()
            .find(|(existing_id, _)| *existing_id == ship_id)
        {
            existing.1 = cmd;
        } else {
            self.ship_commands.push((ship_id, cmd));
        }
    }

    /// Befiehlt einem Schiff, zum angegebenen Asteroiden zu fliegen und Erz abzubauen.
    pub fn mine(&mut self, ship_id: u32, asteroid_id: u32) {
        self.set_ship_command(ship_id, ShipCommand::Mine(asteroid_id));
    }

    /// Befiehlt einem Schiff, zur Heimatbasis zurückzukehren und anzudocken.
    pub fn dock(&mut self, ship_id: u32) {
        self.set_ship_command(ship_id, ShipCommand::Dock);
    }

    /// Lässt ein Schiff zu den angegebenen 2D-Koordinaten fliegen.
    pub fn fly_to(&mut self, ship_id: u32, target: Vec2) {
        self.set_ship_command(ship_id, ShipCommand::FlyTo(target));
    }

    /// Stoppt das Schiff sofort.
    pub fn stop(&mut self, ship_id: u32) {
        self.set_ship_command(ship_id, ShipCommand::Stop);
    }

    /// Gibt eine neue Drohne an der Basis in Auftrag (Tier 1: 50 Erz bzw. Notfall kostenlos; Tier 2: 150 Erz).
    pub fn build_drone(&mut self, tier: u8) {
        self.build_requests.push(tier);
    }

    /// Fordert eine Forschungsaufgabe für einen Upgrade-Zweig an.
    pub fn request_research(&mut self, track: UpgradeTrack) {
        self.research_requests.push(track);
    }

    /// Reicht die berechnete Lösung für das aktive Forschungsrätsel ein.
    pub fn solve_riddle(&mut self, solution: RiddleSolution) {
        self.riddle_solutions.push(solution);
    }

    pub(crate) fn clear(&mut self) {
        self.ship_commands.clear();
        self.build_requests.clear();
        self.research_requests.clear();
        self.riddle_solutions.clear();
    }
}
