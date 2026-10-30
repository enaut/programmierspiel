pub use crate::module_bindings::{
    BaseStation, PerceivedAsteroid, PerceivedBase, PerceivedDrone, Ship, UnidentifiedContact, Vec2,
};

impl Copy for Vec2 {}

impl Vec2 {
    /// Null-Vektor (0.0, 0.0)
    pub const ZERO: Vec2 = Vec2 { x: 0.0, y: 0.0 };

    /// Erzeugt einen neuen 2D-Vektor mit X- und Y-Koordinaten.
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Berechnet die euklidische Distanz zu einem anderen Punkt.
    pub fn distance(&self, other: &Vec2) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

impl From<(f32, f32)> for Vec2 {
    fn from((x, y): (f32, f32)) -> Self {
        Vec2 { x, y }
    }
}

impl From<[f32; 2]> for Vec2 {
    fn from([x, y]: [f32; 2]) -> Self {
        Vec2 { x, y }
    }
}

impl Ship {
    /// Berechnet die Distanz zwischen diesem Schiff und einer Position.
    pub fn distance_to(&self, target: &Vec2) -> f32 {
        self.position.distance(target)
    }

    /// Gibt zurück, ob der Laderaum voll ist (`cargo_ore >= max_cargo`).
    pub fn cargo_is_full(&self) -> bool {
        self.cargo_ore >= self.max_cargo
    }

    /// Gibt den verbleibenden freien Frachtplatz zurück.
    pub fn cargo_free_space(&self) -> u32 {
        self.max_cargo.saturating_sub(self.cargo_ore)
    }

    /// Gibt den aktuellen Batteriestand als Prozentwert zwischen 0.0 und 1.0 zurück.
    pub fn energy_percent(&self) -> f32 {
        if self.max_energy > 0.0 {
            (self.energy / self.max_energy).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }
}

impl BaseStation {
    /// Berechnet die Distanz zwischen der Basis und einer Position.
    pub fn distance_to(&self, target: &Vec2) -> f32 {
        self.position.distance(target)
    }

    /// Prüft, ob sich die gegebene Position innerhalb des Funk- und Schutzradius der Basis befindet.
    pub fn is_in_range(&self, target: &Vec2) -> bool {
        self.distance_to(target) <= self.signal_radius
    }

    /// Gibt die aktuelle Sensorreichweite für Drohnen-Scans zurück (Stufe 1 = 150m, +50m pro Stufe).
    pub fn drone_scanner_range(&self) -> f32 {
        150.0 + (self.drone_scanner_level.saturating_sub(1) as f32 * 50.0)
    }

    /// Gibt die aktuelle Sensorreichweite für Asteroiden & fremde Basen zurück (Stufe 1 = 300m, +100m pro Stufe).
    pub fn base_scanner_range(&self) -> f32 {
        300.0 + (self.base_scanner_level.saturating_sub(1) as f32 * 100.0)
    }

    /// Gibt den aktuellen Radar-Horizont für unbekannte Kontakte zurück (Stufe 1 = 600m, +150m pro Stufe).
    pub fn radar_horizon_range(&self) -> f32 {
        600.0 + (self.radar_horizon_level.saturating_sub(1) as f32 * 150.0)
    }
}

impl PerceivedAsteroid {
    /// Berechnet die Distanz zwischen diesem Asteroiden und einer Position.
    pub fn distance_to(&self, target: &Vec2) -> f32 {
        self.position.distance(target)
    }

    /// Prüft, ob das Erz des Asteroiden bereits vollständig abgebaut wurde.
    pub fn is_depleted(&self) -> bool {
        self.remaining_ore == 0
    }

    /// Gibt den lesbaren Namen des Rohstofftyps zurück (z.B. "Iron", "Ice", "Titanium").
    pub fn resource_name(&self) -> &'static str {
        match self.resource_type {
            1 => "Iron",
            2 => "Ice",
            3 => "Titanium",
            _ => "Generic",
        }
    }
}

/// Das Radar liefert dem Bot in jedem Tick alle relevanten Informationen über seine Umgebung.
#[derive(Debug, Clone)]
pub struct Radar {
    /// Die eigene Heimatbasis des Spielers (falls bereits errichtet).
    pub base_station: Option<BaseStation>,
    /// Alle Schiffe im Besitz des Spielers.
    pub my_ships: Vec<Ship>,
    /// Alle sichtbaren Asteroiden im Scanbereich der eigenen Basen und Schiffe.
    pub asteroids: Vec<PerceivedAsteroid>,
    /// Fremde Drohnen und Arbeitsschiffe im Sensorbereich.
    pub other_drones: Vec<PerceivedDrone>,
    /// Sichtbare fremde Raumstationen im Sensorbereich.
    pub other_bases: Vec<PerceivedBase>,
    /// Unbekannte Radarkontakte außerhalb der direkten Scanner-Auflösung.
    pub unidentified_contacts: Vec<UnidentifiedContact>,
}

impl Radar {
    /// Findet den geometrisch am nächsten gelegenen Asteroiden relativ zu einer Startposition.
    pub fn closest_asteroid(&self, from: &Vec2) -> Option<&PerceivedAsteroid> {
        self.asteroids.iter().min_by(|a, b| {
            let dist_a = a.distance_to(from);
            let dist_b = b.distance_to(from);
            dist_a.total_cmp(&dist_b)
        })
    }

    /// Findet den nächsten Asteroiden, der noch Erz enthält (`remaining_ore > 0`).
    pub fn closest_asteroid_with_ore(&self, from: &Vec2) -> Option<&PerceivedAsteroid> {
        self.asteroids
            .iter()
            .filter(|a| !a.is_depleted())
            .min_by(|a, b| {
                let dist_a = a.distance_to(from);
                let dist_b = b.distance_to(from);
                dist_a.total_cmp(&dist_b)
            })
    }

    /// Berechnet die Distanz zur eigenen Heimatbasis, falls vorhanden.
    pub fn distance_to_base(&self, from: &Vec2) -> Option<f32> {
        self.base_station
            .as_ref()
            .map(|base| base.distance_to(from))
    }

    /// Prüft, ob sich die gegebene Position innerhalb des Funkbereichs der Heimatbasis befindet.
    pub fn is_in_base_range(&self, from: &Vec2) -> bool {
        self.base_station
            .as_ref()
            .map(|base| base.is_in_range(from))
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spacetimedb_sdk::Identity;

    #[test]
    fn test_vec2_distance() {
        let a = Vec2::new(0.0, 0.0);
        let b = Vec2::new(3.0, 4.0);
        assert!((a.distance(&b) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_ship_cargo_and_energy() {
        let ship = Ship {
            ship_id: 1,
            universe: 0,
            owner: Identity::from_byte_array([0u8; 32]),
            ship_tier: 1,
            position: Vec2::new(10.0, 20.0),
            velocity: Vec2::ZERO,
            target_position: None,
            heading_rad: 0.0,
            speed: 0.0,
            max_speed: 100.0,
            energy: 50.0,
            max_energy: 100.0,
            cargo_ore: 100,
            max_cargo: 100,
            target_asteroid_id: None,
            is_mining: false,
            is_stranded: false,
            signal_countdown: 30,
        };

        assert!(ship.cargo_is_full());
        assert_eq!(ship.cargo_free_space(), 0);
        assert!((ship.energy_percent() - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_radar_closest_asteroid() {
        let radar = Radar {
            base_station: None,
            my_ships: Vec::new(),
            asteroids: vec![
                PerceivedAsteroid {
                    id: 1,
                    viewer: Identity::from_byte_array([0u8; 32]),
                    asteroid_id: 10,
                    position: Vec2::new(100.0, 100.0),
                    velocity: Vec2::ZERO,
                    resource_type: 1,
                    remaining_ore: 0,
                    radius: 20.0,
                },
                PerceivedAsteroid {
                    id: 2,
                    viewer: Identity::from_byte_array([0u8; 32]),
                    asteroid_id: 20,
                    position: Vec2::new(200.0, 200.0),
                    velocity: Vec2::ZERO,
                    resource_type: 2,
                    remaining_ore: 50,
                    radius: 25.0,
                },
            ],
            other_drones: Vec::new(),
            other_bases: Vec::new(),
            unidentified_contacts: Vec::new(),
        };

        let origin = Vec2::ZERO;
        let closest_any = radar.closest_asteroid(&origin).unwrap();
        assert_eq!(closest_any.asteroid_id, 10);

        let closest_with_ore = radar.closest_asteroid_with_ore(&origin).unwrap();
        assert_eq!(closest_with_ore.asteroid_id, 20);
    }
}
