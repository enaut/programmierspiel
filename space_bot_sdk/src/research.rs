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
