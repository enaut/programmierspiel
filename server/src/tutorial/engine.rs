use crate::math::Vec2;
use curriculum::FINAL_STEP;

pub const MIN_DISTANCE_FROM_MEGASTATION_TO_DEPLOY: f32 = 250.0;
pub const INITIAL_ASTEROID_SCAN_RADIUS: f32 = 300.0;
pub const MIN_ORE_IN_SCAN_RADIUS_TO_DEPLOY: u32 = 400;

#[derive(Debug, Clone, PartialEq)]
pub enum TutorialEvent {
    /// Builder ship moved / position checked (Step 1 -> 2)
    BuilderShipMoved {
        distance_to_megastation: f32,
        ore_in_scan_radius: u32,
    },
    /// Player deployed base station from builder ship (Step 1 or 2 -> 3)
    BaseDeployed,
    /// Player confirmed Zed connection in UI (Step 3 -> 4, or auto-chains to 5 if agent connected)
    BridgeConfirmed,
    /// AI agent bound to home base station (Step 4 -> 5)
    BaseAgentConnected,
    /// Command card claimed from a lootbox.
    LootBoxClaimed { remaining_in_box: usize },
    /// Student submitted drone coordinates (Step 6 -> 7)
    SubmitCoordinates { x: i32, y: i32 },
    /// Student submitted asteroid ID (Step 8 -> 9)
    SubmitAsteroidId { asteroid_id: u32 },
    /// Drone mined ore from asteroid (Step 10 -> 11)
    OreMined { amount: u32 },
    /// Ore delivered to home base station (Step 12 -> 13, Step 16 -> 17)
    OreDelivered {
        base_stored_ore: u32,
        fleet_size: usize,
    },
    /// New ship constructed at home base (Step 14 -> 15)
    ShipBuilt { tier: u8 },
    /// Drone battery recharged at base dock (Step 18 -> 19)
    DroneRecharged { energy: f32, max_energy: f32 },
    /// Telemetry research riddle solved (Step 20 -> 21)
    RiddleSolved,
}

#[derive(Debug, Clone, Default)]
pub struct TutorialSnapshot {
    pub primary_drone_position: Option<Vec2>,
    pub is_asteroid_visible: bool,
    pub base_agent_connected: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransitionOutcome {
    NoChange,
    Advance {
        from_step: u8,
        to_step: u8,
        is_completed: bool,
        free_play_unlocked: bool,
    },
    Rejected {
        error_code: &'static str,
        message: &'static str,
    },
}

impl TransitionOutcome {
    fn advance(from_step: u8, to_step: u8) -> Self {
        TransitionOutcome::Advance {
            from_step,
            to_step,
            is_completed: to_step >= FINAL_STEP,
            free_play_unlocked: to_step >= FINAL_STEP,
        }
    }
}

/// Checks if submitted coordinates match the drone position using Rust's round-ties-to-even semantics.
pub fn coordinates_match(position: Vec2, x: i32, y: i32) -> bool {
    f64::from(position.x).round_ties_even() == f64::from(x)
        && f64::from(position.y).round_ties_even() == f64::from(y)
}

/// Evaluates whether an in-game event triggers a tutorial transition for the given step.
pub fn evaluate_transition(
    current_step: u8,
    event: &TutorialEvent,
    snapshot: &TutorialSnapshot,
) -> TransitionOutcome {
    if current_step >= FINAL_STEP {
        return TransitionOutcome::NoChange;
    }

    match (current_step, event) {
        // Step 1 -> 2: Builder moved far enough and has ore in range
        (
            1,
            TutorialEvent::BuilderShipMoved {
                distance_to_megastation,
                ore_in_scan_radius,
            },
        ) => {
            if *distance_to_megastation >= MIN_DISTANCE_FROM_MEGASTATION_TO_DEPLOY
                && *ore_in_scan_radius >= MIN_ORE_IN_SCAN_RADIUS_TO_DEPLOY
            {
                TransitionOutcome::advance(1, 2)
            } else {
                TransitionOutcome::NoChange
            }
        }

        // Step 1 or 2 -> 3: Base deployed
        (1, TutorialEvent::BaseDeployed) => TransitionOutcome::advance(1, 3),
        (2, TutorialEvent::BaseDeployed) => TransitionOutcome::advance(2, 3),

        // Step 3 -> 4 (or auto-chains to 5 if agent is already bound)
        (3, TutorialEvent::BridgeConfirmed) => {
            if snapshot.base_agent_connected {
                TransitionOutcome::advance(3, 5)
            } else {
                TransitionOutcome::advance(3, 4)
            }
        }

        // Step 4 -> 5: AI agent connected to base
        (4, TutorialEvent::BaseAgentConnected) => TransitionOutcome::advance(4, 5),

        // Step 6 -> 7: Drone coordinates quiz
        (6, TutorialEvent::SubmitCoordinates { x, y }) => match snapshot.primary_drone_position {
            None => TransitionOutcome::Rejected {
                error_code: "TUTORIAL_COORDINATES_ERROR",
                message: "Keine eigene Drohne im Signalbereich der Heimatbasis sichtbar. Bitte erneut versuchen.",
            },
            Some(pos) if !coordinates_match(pos, *x, *y) => TransitionOutcome::Rejected {
                error_code: "TUTORIAL_COORDINATES_ERROR",
                message: "Die Koordinaten stimmen nicht. Gib die aktuellen, auf ganze Zahlen gerundeten Koordinaten der ersten Drohne ein.",
            },
            Some(_) => TransitionOutcome::advance(6, 7),
        },

        // Step 8 -> 9: Asteroid ID quiz
        (8, TutorialEvent::SubmitAsteroidId { .. }) => {
            if snapshot.is_asteroid_visible {
                TransitionOutcome::advance(8, 9)
            } else {
                TransitionOutcome::Rejected {
                    error_code: "TUTORIAL_ASTEROID_ERROR",
                    message: "Diese Asteroiden-ID ist nicht sichtbar. Gib die ID eines identifizierten Asteroiden im Scannerbereich ein.",
                }
            }
        }

        // Step 10 -> 11: First ore mined
        (10, TutorialEvent::OreMined { amount }) if *amount > 0 => {
            TransitionOutcome::advance(10, 11)
        }

        // Step 12 -> 13: Ore delivery to base >= 50
        (
            12,
            TutorialEvent::OreDelivered {
                base_stored_ore, ..
            },
        ) if *base_stored_ore >= 50 => TransitionOutcome::advance(12, 13),

        // Step 14 -> 15: Second drone built (tier 1)
        (14, TutorialEvent::ShipBuilt { tier }) if *tier == 1 => TransitionOutcome::advance(14, 15),

        // Step 16 -> 17: Multi-drone ore delivery with fleet >= 2
        (16, TutorialEvent::OreDelivered { fleet_size, .. }) if *fleet_size >= 2 => {
            TransitionOutcome::advance(16, 17)
        }

        // Step 18 -> 19: Drone recharged to full
        (18, TutorialEvent::DroneRecharged { energy, max_energy }) if *energy >= *max_energy => {
            TransitionOutcome::advance(18, 19)
        }

        // Step 20 -> 21: Riddle solved
        (20, TutorialEvent::RiddleSolved) => TransitionOutcome::advance(20, 21),

        // Lootbox claims advance after the final unclaimed card is collected.
        (step, TutorialEvent::LootBoxClaimed { remaining_in_box })
            if curriculum::is_loot_box_step(step) && *remaining_in_box <= 1 =>
        {
            TransitionOutcome::advance(step, step + 1)
        }

        _ => TransitionOutcome::NoChange,
    }
}

// ==========================================
// Unit Tests (100% In-Process, Zero DB Mocks)
// ==========================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_movement_triggers_step_2_when_conditions_met() {
        let snap = TutorialSnapshot::default();

        // Distance too short
        let evt_close = TutorialEvent::BuilderShipMoved {
            distance_to_megastation: 249.0,
            ore_in_scan_radius: 500,
        };
        assert_eq!(
            evaluate_transition(1, &evt_close, &snap),
            TransitionOutcome::NoChange
        );

        // Ore too low
        let evt_low_ore = TutorialEvent::BuilderShipMoved {
            distance_to_megastation: 300.0,
            ore_in_scan_radius: 399,
        };
        assert_eq!(
            evaluate_transition(1, &evt_low_ore, &snap),
            TransitionOutcome::NoChange
        );

        // Both conditions satisfied
        let evt_ok = TutorialEvent::BuilderShipMoved {
            distance_to_megastation: 250.0,
            ore_in_scan_radius: 400,
        };
        assert_eq!(
            evaluate_transition(1, &evt_ok, &snap),
            TransitionOutcome::Advance {
                from_step: 1,
                to_step: 2,
                is_completed: false,
                free_play_unlocked: false,
            }
        );
    }

    #[test]
    fn deploy_base_advances_from_1_or_2_to_3() {
        let snap = TutorialSnapshot::default();
        let evt = TutorialEvent::BaseDeployed;

        assert_eq!(
            evaluate_transition(1, &evt, &snap),
            TransitionOutcome::Advance {
                from_step: 1,
                to_step: 3,
                is_completed: false,
                free_play_unlocked: false,
            }
        );
        assert_eq!(
            evaluate_transition(2, &evt, &snap),
            TransitionOutcome::Advance {
                from_step: 2,
                to_step: 3,
                is_completed: false,
                free_play_unlocked: false,
            }
        );
    }

    #[test]
    fn bridge_confirmation_auto_cascades_if_agent_bound() {
        let snap_not_bound = TutorialSnapshot {
            base_agent_connected: false,
            ..Default::default()
        };
        assert_eq!(
            evaluate_transition(3, &TutorialEvent::BridgeConfirmed, &snap_not_bound),
            TransitionOutcome::Advance {
                from_step: 3,
                to_step: 4,
                is_completed: false,
                free_play_unlocked: false,
            }
        );

        let snap_bound = TutorialSnapshot {
            base_agent_connected: true,
            ..Default::default()
        };
        assert_eq!(
            evaluate_transition(3, &TutorialEvent::BridgeConfirmed, &snap_bound),
            TransitionOutcome::Advance {
                from_step: 3,
                to_step: 5,
                is_completed: false,
                free_play_unlocked: false,
            }
        );
    }

    #[test]
    fn coordinate_quiz_validation_and_error_codes() {
        let snap = TutorialSnapshot {
            primary_drone_position: Some(Vec2::new(12.51, -12.51)),
            ..Default::default()
        };

        // Rounding ties: (13, -13)
        assert_eq!(
            evaluate_transition(
                6,
                &TutorialEvent::SubmitCoordinates { x: 13, y: -13 },
                &snap
            ),
            TransitionOutcome::Advance {
                from_step: 6,
                to_step: 7,
                is_completed: false,
                free_play_unlocked: false,
            }
        );

        // Wrong input
        match evaluate_transition(
            6,
            &TutorialEvent::SubmitCoordinates { x: 12, y: -12 },
            &snap,
        ) {
            TransitionOutcome::Rejected { error_code, .. } => {
                assert_eq!(error_code, "TUTORIAL_COORDINATES_ERROR");
            }
            other => panic!("Expected rejection, got {:?}", other),
        }

        // Missing drone
        let snap_empty = TutorialSnapshot::default();
        match evaluate_transition(
            6,
            &TutorialEvent::SubmitCoordinates { x: 13, y: -13 },
            &snap_empty,
        ) {
            TransitionOutcome::Rejected { error_code, .. } => {
                assert_eq!(error_code, "TUTORIAL_COORDINATES_ERROR");
            }
            other => panic!("Expected rejection, got {:?}", other),
        }
    }

    #[test]
    fn asteroid_quiz_advances_on_visible() {
        let snap_visible = TutorialSnapshot {
            is_asteroid_visible: true,
            ..Default::default()
        };
        assert_eq!(
            evaluate_transition(
                8,
                &TutorialEvent::SubmitAsteroidId { asteroid_id: 42 },
                &snap_visible
            ),
            TransitionOutcome::Advance {
                from_step: 8,
                to_step: 9,
                is_completed: false,
                free_play_unlocked: false,
            }
        );

        let snap_hidden = TutorialSnapshot {
            is_asteroid_visible: false,
            ..Default::default()
        };
        match evaluate_transition(
            8,
            &TutorialEvent::SubmitAsteroidId { asteroid_id: 42 },
            &snap_hidden,
        ) {
            TransitionOutcome::Rejected { error_code, .. } => {
                assert_eq!(error_code, "TUTORIAL_ASTEROID_ERROR");
            }
            other => panic!("Expected rejection, got {:?}", other),
        }
    }

    #[test]
    fn ore_delivery_and_fleet_size_milestones() {
        let snap_low_ore = TutorialSnapshot::default();
        assert_eq!(
            evaluate_transition(
                12,
                &TutorialEvent::OreDelivered {
                    base_stored_ore: 49,
                    fleet_size: 1
                },
                &snap_low_ore
            ),
            TransitionOutcome::NoChange
        );

        assert_eq!(
            evaluate_transition(
                12,
                &TutorialEvent::OreDelivered {
                    base_stored_ore: 50,
                    fleet_size: 1
                },
                &snap_low_ore
            ),
            TransitionOutcome::Advance {
                from_step: 12,
                to_step: 13,
                is_completed: false,
                free_play_unlocked: false,
            }
        );

        // Step 16 requires fleet >= 2
        assert_eq!(
            evaluate_transition(
                16,
                &TutorialEvent::OreDelivered {
                    base_stored_ore: 100,
                    fleet_size: 1
                },
                &snap_low_ore
            ),
            TransitionOutcome::NoChange
        );

        assert_eq!(
            evaluate_transition(
                16,
                &TutorialEvent::OreDelivered {
                    base_stored_ore: 100,
                    fleet_size: 2
                },
                &snap_low_ore
            ),
            TransitionOutcome::Advance {
                from_step: 16,
                to_step: 17,
                is_completed: false,
                free_play_unlocked: false,
            }
        );
    }

    #[test]
    fn final_loot_box_26_unlocks_free_play() {
        let snap = TutorialSnapshot::default();
        assert_eq!(
            evaluate_transition(
                22,
                &TutorialEvent::LootBoxClaimed {
                    remaining_in_box: 1
                },
                &snap
            ),
            TransitionOutcome::Advance {
                from_step: 22,
                to_step: 23,
                is_completed: false,
                free_play_unlocked: false,
            }
        );
        assert_eq!(
            evaluate_transition(
                26,
                &TutorialEvent::LootBoxClaimed {
                    remaining_in_box: 1
                },
                &snap
            ),
            TransitionOutcome::Advance {
                from_step: 26,
                to_step: 27,
                is_completed: true,
                free_play_unlocked: true,
            }
        );

        // Step 27 is terminal
        assert_eq!(
            evaluate_transition(
                27,
                &TutorialEvent::LootBoxClaimed {
                    remaining_in_box: 0
                },
                &snap
            ),
            TransitionOutcome::NoChange
        );
    }
}
