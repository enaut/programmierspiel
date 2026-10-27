use crate::module_bindings::{PublicRiddle, Vec2};
use serde::{Deserialize, Serialize};

/// Die drei verfügbaren Upgrade-Zweige für die Heimatbasis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpgradeTrack {
    /// 1: Erhöht die Drohnenscan-Reichweite (150m bis 350m). Rätsel: Listen sortieren.
    DroneScanner = 1,
    /// 2: Erhöht den Scanbereich für Asteroiden & Basen (300m bis 700m). Rätsel: Distanzberechnung.
    BaseAsteroidScanner = 2,
    /// 3: Erhöht den Radar-Horizont für Kontakte (600m bis 1200m). Rätsel: Spikes filtern (Denoising).
    RadarHorizon = 3,
}

impl UpgradeTrack {
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }

    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            1 => Some(Self::DroneScanner),
            2 => Some(Self::BaseAsteroidScanner),
            3 => Some(Self::RadarHorizon),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::DroneScanner => "Drohnen-Scanner",
            Self::BaseAsteroidScanner => "Basen- & Asteroidenscan",
            Self::RadarHorizon => "Radar-Horizont",
        }
    }
}

// ==========================================
// Rätsel-Datenstrukturen
// ==========================================

/// Aufgabenstellung für Track 1: Telemetrie-Zahlen sortieren
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SortPayload {
    pub numbers: Vec<i64>,
}

/// Lösung für Track 1
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SortSolution {
    pub sorted: Vec<i64>,
}

/// Aufgabenstellung für Track 2: Distanzen von der Basis zu Zielpunkten berechnen
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DistancePayload {
    pub base: [f32; 2],
    pub targets: Vec<[f32; 2]>,
}

/// Lösung für Track 2
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DistanceSolution {
    pub distances: Vec<f32>,
}

/// Aufgabenstellung für Track 3: Störsignale (Spikes) aus einer Messreihe entfernen
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DenoisePayload {
    pub readings: Vec<f32>,
    pub threshold: f32,
}

/// Lösung für Track 3
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DenoiseSolution {
    pub cleaned: Vec<f32>,
}

/// Stark typisierte Aufgabenstellung eines Forschungsrätsels.
#[derive(Debug, Clone)]
pub enum RiddleTask {
    /// Track 1: Drohnen-Scanner (Zahlenreihe aufsteigend sortieren)
    Sort { numbers: Vec<i64> },
    /// Track 2: Basen- und Asteroidenscanner (Distanzen von Basis zu Zielkoordinaten berechnen)
    Distance { base: Vec2, targets: Vec<Vec2> },
    /// Track 3: Radar-Horizont (Störsignale über Schwellenwert entfernen)
    Denoise { readings: Vec<f32>, threshold: f32 },
}

impl RiddleTask {
    pub fn from_public_riddle(riddle: &PublicRiddle) -> Option<Self> {
        match riddle.track {
            1 => {
                let payload: SortPayload = serde_json::from_str(&riddle.payload).ok()?;
                Some(Self::Sort {
                    numbers: payload.numbers,
                })
            }
            2 => {
                let payload: DistancePayload = serde_json::from_str(&riddle.payload).ok()?;
                Some(Self::Distance {
                    base: Vec2::new(payload.base[0], payload.base[1]),
                    targets: payload
                        .targets
                        .into_iter()
                        .map(|t| Vec2::new(t[0], t[1]))
                        .collect(),
                })
            }
            3 => {
                let payload: DenoisePayload = serde_json::from_str(&riddle.payload).ok()?;
                Some(Self::Denoise {
                    readings: payload.readings,
                    threshold: payload.threshold,
                })
            }
            _ => None,
        }
    }
}

/// Stark typisierte Lösung für ein Forschungsrätsel.
#[derive(Debug, Clone)]
pub enum RiddleSolution {
    Sort(Vec<i64>),
    Distance(Vec<f32>),
    Denoise(Vec<f32>),
}

impl RiddleSolution {
    pub fn to_json_string(&self) -> String {
        match self {
            Self::Sort(sorted) => serde_json::to_string(&SortSolution {
                sorted: sorted.clone(),
            })
            .unwrap_or_default(),
            Self::Distance(distances) => serde_json::to_string(&DistanceSolution {
                distances: distances.clone(),
            })
            .unwrap_or_default(),
            Self::Denoise(cleaned) => serde_json::to_string(&DenoiseSolution {
                cleaned: cleaned.clone(),
            })
            .unwrap_or_default(),
        }
    }

    pub fn track(&self) -> u8 {
        match self {
            Self::Sort(_) => 1,
            Self::Distance(_) => 2,
            Self::Denoise(_) => 3,
        }
    }
}
