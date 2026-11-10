use super::engine::{self, TransitionOutcome, TutorialEvent, TutorialSnapshot};
use crate::models::{GameEvent, Identity};
use crate::unlock_command_key;
use curriculum::FINAL_STEP;
use spacetimedb::{ReducerContext, Table};

/// Evaluates a domain event through the pure tutorial engine and applies any resulting transitions
/// to the SpacetimeDB database tables (`TutorialState` and `Player`).
pub fn apply_event(
    ctx: &ReducerContext,
    player_id: Identity,
    event: &TutorialEvent,
    snapshot: &TutorialSnapshot,
) -> bool {
    let Some(mut state) = ctx.db.tutorial_state().player().find(&player_id) else {
        return false;
    };
    if state.current_step >= FINAL_STEP {
        return false;
    }

    let outcome = engine::evaluate_transition(state.current_step, event, snapshot);
    match outcome {
        TransitionOutcome::Advance {
            from_step,
            to_step,
            is_completed,
            free_play_unlocked,
        } => {
            state.current_step = to_step;
            state.step_started_at = ctx.timestamp;
            state.completed_at = is_completed.then_some(ctx.timestamp);
            ctx.db.tutorial_state().player().update(state);

            if let Some(mut player) = ctx.db.player().identity().find(&player_id) {
                player.tutorial_step = to_step;
                ctx.db.player().identity().update(player);
            }

            if free_play_unlocked {
                for key in curriculum::free_play_commands() {
                    unlock_command_key(ctx, player_id, key);
                }
            }

            ctx.db.game_event().insert(GameEvent {
                event_id: 0,
                player: player_id,
                event_type: "TUTORIAL_STEP_ADVANCED".to_string(),
                message: format!("Tutorial-Stufe {} erreicht!", to_step),
                timestamp: ctx.timestamp,
            });

            log::info!(
                "Player {:?} advanced tutorial {} -> {}",
                player_id,
                from_step,
                to_step
            );
            true
        }
        TransitionOutcome::Rejected {
            error_code,
            message,
        } => {
            ctx.db.game_event().insert(GameEvent {
                event_id: 0,
                player: player_id,
                event_type: error_code.to_string(),
                message: message.to_string(),
                timestamp: ctx.timestamp,
            });
            false
        }
        TransitionOutcome::NoChange => false,
    }
}

/// Explicitly advances a player's tutorial progress to a target step, updating both tables
/// and unlocking all prerequisite loot box and free-play command cards.
pub fn set_step_explicit(
    ctx: &ReducerContext,
    player_id: Identity,
    target_step: u8,
) -> Result<(), String> {
    let target = curriculum::step(target_step)
        .ok_or_else(|| format!("Target step must be between 0 and {}", FINAL_STEP))?;

    if let Some(mut state) = ctx.db.tutorial_state().player().find(&player_id) {
        let from_step = state.current_step;
        state.current_step = target_step;
        state.step_started_at = ctx.timestamp;
        state.completed_at = target.is_free_play().then_some(ctx.timestamp);
        ctx.db.tutorial_state().player().update(state);

        if let Some(mut player) = ctx.db.player().identity().find(&player_id) {
            player.tutorial_step = target_step;
            ctx.db.player().identity().update(player);
        }

        for key in target.catch_up_command_keys() {
            unlock_command_key(ctx, player_id, key);
        }

        ctx.db.game_event().insert(GameEvent {
            event_id: 0,
            player: player_id,
            event_type: "TUTORIAL_STEP_SET".to_string(),
            message: format!("Tutorial-Stufe {} gesetzt!", target_step),
            timestamp: ctx.timestamp,
        });

        log::info!(
            "Player {:?} tutorial explicitly set {} -> {}",
            player_id,
            from_step,
            target_step
        );
    }
    Ok(())
}
