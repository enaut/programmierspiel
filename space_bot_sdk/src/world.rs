use crate::action::Actions;
use crate::module_bindings::{
    BaseStation, PerceivedAsteroid, PerceivedBase, PerceivedDrone, Ship, Vec2,
};
use crate::research::RiddleTask;

/// Repräsentiert die gesamte wahrgenommene Spielwelt in diesem Tick.
pub struct World<'a> {
    /// Die eigene Heimatbasis. Garantiert vorhanden, sobald `tick` aufgerufen wird.
    pub base: &'a BaseStation,
    /// Alle aktuell aktiven Schiffe/Drohnen der eigenen Flotte.
    pub fleet: &'a [Ship],
    /// Sichtbare Asteroiden im Sensorbereich.
    pub asteroids: &'a [PerceivedAsteroid],
    /// Sichtbare fremde Drohnen.
    pub other_drones: &'a [PerceivedDrone],
    /// Sichtbare fremde Raumstationen.
    pub other_bases: &'a [PerceivedBase],
    /// Ein aktuell aktives Forschungsrätsel, falls angefordert.
    pub active_riddle: Option<&'a RiddleTask>,
    /// Fortlaufende Nummer des aktuellen Ticks (10 Ticks = 1 Sekunde).
    pub tick: u64,
}

impl<'a> World<'a> {
    /// Sucht ein eigenes Schiff anhand seiner ID.
    pub fn ship(&self, ship_id: u32) -> Option<&Ship> {
        self.fleet.iter().find(|s| s.ship_id == ship_id)
    }

    /// Hilfsmethode für den Einstieg: Liefert die erste Drohne der Flotte.
    pub fn first_drone(&self) -> Option<&Ship> {
        self.fleet.first()
    }

    /// Findet den geometrisch am nächsten gelegenen Asteroiden zu einer Position.
    pub fn closest_asteroid(&self, from: Vec2) -> Option<&PerceivedAsteroid> {
        self.asteroids.iter().min_by(|a, b| {
            a.position
                .distance(&from)
                .total_cmp(&b.position.distance(&from))
        })
    }

    /// Findet den nächsten Asteroiden, der noch Erz enthält (`remaining_ore > 0`).
    pub fn closest_asteroid_with_ore(&self, from: Vec2) -> Option<&PerceivedAsteroid> {
        self.asteroids
            .iter()
            .filter(|a| a.remaining_ore > 0)
            .min_by(|a, b| {
                a.position
                    .distance(&from)
                    .total_cmp(&b.position.distance(&from))
            })
    }
}

/// Trait für die zentrale Steuerung der Heimatbasis und der gesamten Drohnenflotte.
pub trait BaseController {
    /// Wird 10-mal pro Sekunde (alle 100ms) aufgerufen.
    ///
    /// - `world`: Reiner Lese-Zugriff auf Basis, Flotte, Radar und Rätsel.
    /// - `actions`: Sammelbehälter für Befehle, die in diesem Tick abgesetzt werden.
    fn tick(&mut self, world: &World, actions: &mut Actions);
}

/// Repräsentiert die wahrgenommene Spielwelt aus der Perspektive eines einzelnen Schiffes oder Kriegsschiffes.
pub struct ShipWorld<'a> {
    /// Das eigene gesteuerte Schiff.
    pub ship: &'a Ship,
    /// Alle eigenen Schiffe/Drohnen im Kommunikationsbereich (z.B. Sub-Drohnen eines Träger-/Kriegsschiffes).
    pub fleet: &'a [Ship],
    /// Sichtbare Asteroiden im Sensorbereich.
    pub asteroids: &'a [PerceivedAsteroid],
    /// Sichtbare fremde Drohnen.
    pub other_drones: &'a [PerceivedDrone],
    /// Sichtbare fremde Raumstationen.
    pub other_bases: &'a [PerceivedBase],
    /// Die Heimatbasis, falls in Signalreichweite.
    pub base: Option<&'a BaseStation>,
    /// Fortlaufende Nummer des aktuellen Ticks (10 Ticks = 1 Sekunde).
    pub tick: u64,
}

impl<'a> ShipWorld<'a> {
    /// Findet den geometrisch am nächsten gelegenen Asteroiden zu einer Position.
    pub fn closest_asteroid(&self, from: Vec2) -> Option<&PerceivedAsteroid> {
        self.asteroids.iter().min_by(|a, b| {
            a.position
                .distance(&from)
                .total_cmp(&b.position.distance(&from))
        })
    }

    /// Findet den nächsten Asteroiden, der noch Erz enthält (`remaining_ore > 0`).
    pub fn closest_asteroid_with_ore(&self, from: Vec2) -> Option<&PerceivedAsteroid> {
        self.asteroids
            .iter()
            .filter(|a| a.remaining_ore > 0)
            .min_by(|a, b| {
                a.position
                    .distance(&from)
                    .total_cmp(&b.position.distance(&from))
            })
    }
}

/// Trait für die Steuerung eines einzelnen Schiffes (z.B. Kriegsschiff, Erkunder, Drohne).
pub trait ShipController {
    /// Wird 10-mal pro Sekunde (alle 100ms) aufgerufen.
    ///
    /// - `world`: Reiner Lese-Zugriff auf das Schiff und seine Sensorumgebung.
    /// - `actions`: Sammelbehälter für Befehle, die in diesem Tick abgesetzt werden.
    fn tick(&mut self, world: &ShipWorld, actions: &mut Actions);
}
