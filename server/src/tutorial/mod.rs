pub mod adapter;
pub mod engine;

pub use adapter::{apply_event, set_step_explicit};
pub use engine::{
    coordinates_match, evaluate_transition, TransitionOutcome, TutorialEvent, TutorialSnapshot,
    INITIAL_ASTEROID_SCAN_RADIUS, MIN_DISTANCE_FROM_MEGASTATION_TO_DEPLOY,
    MIN_ORE_IN_SCAN_RADIUS_TO_DEPLOY,
};
