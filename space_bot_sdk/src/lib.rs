pub mod action;
pub mod config;
pub mod module_bindings;
pub mod radar;
pub mod research;
pub mod runtime;
pub mod world;

pub use action::{Action, Actions};
pub use config::BotConfig;
pub use module_bindings::build_ship_reducer::build_ship;
pub use module_bindings::{DbConnection, PublicRiddle};
pub use radar::{
    BaseStation, PerceivedAsteroid, PerceivedBase, PerceivedDrone, Radar, Ship,
    UnidentifiedContact, Vec2,
};
pub use research::{
    DenoisePayload, DenoiseSolution, DistancePayload, DistanceSolution, RiddleSolution, RiddleTask,
    SortPayload, SortSolution, UpgradeTrack,
};
pub use runtime::run;
pub use world::{BaseController, World};

/// Baukosten für eine Stufe-1-Drohne (50 Erz; kostenlos, wenn die Flotte 0 Schiffe hat).
pub const DRONE_TIER_1_COST: u32 = 50;

/// Baukosten für eine Stufe-2-Drohne (150 Erz).
pub const DRONE_TIER_2_COST: u32 = 150;

/// Hilfsfunktion zum Bauen einer neuen Drohne an der Heimatbasis.
pub fn build_drone(conn: &DbConnection, tier: u8) -> spacetimedb_sdk::Result<()> {
    conn.reducers.build_ship(tier)
}
