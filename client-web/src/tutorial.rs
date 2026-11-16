use crate::audio::SoundEffects;
use crate::commands::{commands_for_coding_step, is_loot_box_step};
use crate::module_bindings::dioxus::{ConnectionState, use_spacetimedb_context};
use crate::module_bindings::game_event_table::GameEventTableAccess;
use crate::module_bindings::player_table::PlayerTableAccess;
use crate::module_bindings::submit_asteroid_id_reducer::submit_asteroid_id;
use crate::module_bindings::submit_drone_coordinates_reducer::submit_drone_coordinates;
use crate::module_bindings::{GameEvent, ReducerEventContext, Ship};

use dioxus::prelude::*;
use dioxus::signals::SyncSignal;
use spacetimedb_sdk::{Identity, Table, Timestamp};

pub use curriculum::{SubTask, step as get_step};

const TUTORIAL_FINAL_STEP: u8 = curriculum::FINAL_STEP;

pub fn get_step_info(step: u8) -> &'static curriculum::Step {
    curriculum::step(step).unwrap_or(&curriculum::STEPS[0])
}

enum TutorialAnswer {
    Coordinates(i32, i32),
    Asteroid(u32),
}

fn parse_coordinates(x: &str, y: &str) -> Result<(i32, i32), &'static str> {
    let x = x
        .trim()
        .parse::<i32>()
        .map_err(|_| "Bitte gib für X eine gültige ganze Zahl ein (32-Bit).")?;
    let y = y
        .trim()
        .parse::<i32>()
        .map_err(|_| "Bitte gib für Y eine gültige ganze Zahl ein (32-Bit).")?;
    Ok((x, y))
}

fn parse_asteroid_id(id: &str) -> Result<u32, &'static str> {
    id.trim().parse::<u32>().map_err(
        |_| "Bitte gib eine gültige nicht negative ganzzahlige Asteroiden-ID ein (32-Bit).",
    )
}

fn submission_error(
    events: impl Iterator<Item = GameEvent>,
    step: u8,
    after: Option<u64>,
    owner: Identity,
    timestamp: Timestamp,
) -> Option<String> {
    let event_type = match step {
        6 => "TUTORIAL_COORDINATES_ERROR",
        8 => "TUTORIAL_ASTEROID_ERROR",
        _ => return None,
    };
    events
        .filter(|event| {
            event.player == owner
                && event.event_type == event_type
                && event.timestamp == timestamp
                && after.is_none_or(|id| event.event_id > id)
        })
        .max_by_key(|event| event.event_id)
        .map(|event| event.message)
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum SubmissionStatus {
    Waiting,
    AwaitingProgress,
    Error(String),
}

#[derive(Clone, Debug)]
struct Submission {
    id: u64,
    step: u8,
    status: SubmissionStatus,
}

#[derive(Default)]
struct SubmissionState {
    next_id: u64,
    active: Option<Submission>,
}

impl SubmissionState {
    fn begin(&mut self, step: u8) -> u64 {
        self.next_id += 1;
        let id = self.next_id;
        self.active = Some(Submission {
            id,
            step,
            status: SubmissionStatus::Waiting,
        });
        id
    }

    fn update_pending(&mut self, id: u64, status: SubmissionStatus) {
        if let Some(active) = self.active.as_mut() {
            if active.id == id
                && matches!(
                    active.status,
                    SubmissionStatus::Waiting | SubmissionStatus::AwaitingProgress
                )
            {
                active.status = status;
            }
        }
    }

    fn fail_pending(&mut self, message: &str) {
        if let Some(active) = self.active.as_mut() {
            if matches!(
                active.status,
                SubmissionStatus::Waiting | SubmissionStatus::AwaitingProgress
            ) {
                active.status = SubmissionStatus::Error(message.to_string());
            }
        }
    }
}

fn finish_submission(
    mut submission: SyncSignal<SubmissionState>,
    id: u64,
    step: u8,
    after: Option<u64>,
    owner: Identity,
    ctx: &ReducerEventContext,
    result: Result<Result<(), String>, spacetimedb_sdk::__codegen::InternalError>,
) {
    let Ok(mut state) = submission.try_write() else {
        return;
    };
    if !state
        .active
        .as_ref()
        .is_some_and(|active| active.id == id && active.status == SubmissionStatus::Waiting)
    {
        return;
    }
    // A previous attempt may already have advanced the server while this request was in flight.
    let progressed = ctx
        .db
        .player()
        .identity()
        .find(&owner)
        .is_some_and(|player| player.tutorial_step != step);
    let status = if progressed {
        SubmissionStatus::AwaitingProgress
    } else {
        match result {
            Ok(Ok(())) => {
                // The callback sees the cache after this transaction's updates have been applied.
                match submission_error(
                    ctx.db.game_event().iter(),
                    step,
                    after,
                    owner,
                    ctx.event.timestamp,
                ) {
                    Some(message) => SubmissionStatus::Error(message),
                    None => SubmissionStatus::AwaitingProgress,
                }
            }
            Ok(Err(error)) => SubmissionStatus::Error(format!("Serverfehler: {error}")),
            Err(error) => SubmissionStatus::Error(format!("Verbindungsfehler: {error}")),
        }
    };
    state.update_pending(id, status);
}

async fn wait_for_submission_timeout() {
    #[cfg(target_arch = "wasm32")]
    {
        let promise = js_sys::Promise::new(&mut |resolve, _| {
            let scheduled = web_sys::window().is_some_and(|window| {
                window
                    .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 10_000)
                    .is_ok()
            });
            if !scheduled {
                let _ = resolve.call0(&wasm_bindgen::JsValue::UNDEFINED);
            }
        });
        let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let (tx, rx) = futures_channel::oneshot::channel();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(10));
            let _ = tx.send(());
        });
        let _ = rx.await;
    }
}

fn viewed_step_after_progress_change(previous: u8, current: u8, viewed: u8) -> u8 {
    if current > previous {
        current
    } else {
        viewed.min(current)
    }
}

#[cfg(test)]
mod navigation_tests {
    use super::{TUTORIAL_FINAL_STEP, viewed_step_after_progress_change};

    #[test]
    fn progress_shows_the_new_step_even_when_viewing_an_older_one() {
        assert_eq!(viewed_step_after_progress_change(8, 9, 3), 9);
        assert_eq!(viewed_step_after_progress_change(3, 5, 0), 5);
    }

    #[test]
    fn reset_keeps_the_view_if_it_is_still_available() {
        assert_eq!(viewed_step_after_progress_change(8, 5, 3), 3);
        assert_eq!(viewed_step_after_progress_change(8, 5, 5), 5);
        assert_eq!(viewed_step_after_progress_change(8, 5, 7), 5);
        assert_eq!(viewed_step_after_progress_change(8, 0, 3), 0);
    }

    #[test]
    fn progress_after_a_reset_shows_the_new_step_again() {
        let viewed = viewed_step_after_progress_change(8, 3, 7);
        assert_eq!(viewed, 3);
        assert_eq!(viewed_step_after_progress_change(3, 4, viewed), 4);
        assert_eq!(viewed_step_after_progress_change(16, 16, 0), 0);
    }

    #[test]
    fn new_final_step_respects_progress_and_reset() {
        assert_eq!(
            viewed_step_after_progress_change(36, TUTORIAL_FINAL_STEP, 6),
            TUTORIAL_FINAL_STEP
        );
        assert_eq!(
            viewed_step_after_progress_change(TUTORIAL_FINAL_STEP, 16, 36),
            16
        );
        assert_eq!(
            viewed_step_after_progress_change(TUTORIAL_FINAL_STEP, 8, 6),
            6
        );
        assert_eq!(viewed_step_after_progress_change(8, 9, 6), 9);
    }
}

#[cfg(target_arch = "wasm32")]
fn reset_quest_scroll() {
    if let Some(element) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id("quest-drawer-scroll"))
    {
        element.set_scroll_top(0);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn reset_quest_scroll() {}

#[component]
pub fn QuestDrawer(
    step: u8,
    mut is_collapsed: Signal<bool>,
    events: Vec<GameEvent>,
    ships: Vec<Ship>,
    on_advance: EventHandler<u8>,
    on_open_hall: EventHandler<Option<String>>,
    on_open_base: EventHandler<()>,
) -> Element {
    let ctx = use_spacetimedb_context();
    let connection = ctx.connection;
    let connection_state = ctx.state;
    let all_events = ctx.tables.game_event;
    let mut last_seen_step = use_signal(|| step);
    let mut viewed_step = use_signal(|| step);
    let mut show_celebration = use_signal(|| false);
    let mut coordinate_x = use_signal(String::new);
    let mut coordinate_y = use_signal(String::new);
    let mut asteroid_id = use_signal(String::new);
    let mut input_error = use_signal(|| Option::<String>::None);
    let mut submission = use_signal_sync(SubmissionState::default);

    use_effect(use_reactive((&step,), move |(current_step,)| {
        let previous_step = *last_seen_step.peek();
        if current_step != previous_step {
            last_seen_step.set(current_step);
            let viewed = *viewed_step.peek();
            let next_viewed =
                viewed_step_after_progress_change(previous_step, current_step, viewed);
            if next_viewed != viewed {
                viewed_step.set(next_viewed);
            }
            coordinate_x.set(String::new());
            coordinate_y.set(String::new());
            asteroid_id.set(String::new());
            input_error.set(None);
            submission.write().active = None;
            if current_step > previous_step {
                is_collapsed.set(false);
                show_celebration.set(true);
                SoundEffects::play_success_ping();
            }
        }
    }));

    use_effect(move || {
        if !matches!(connection_state(), ConnectionState::Connected(..)) {
            if let Ok(mut state) = submission.try_write() {
                state.fail_pending("Die Verbindung wurde unterbrochen. Bitte versuche es nach dem Wiederverbinden erneut.");
            }
        }
    });

    use_effect(move || {
        let _ = viewed_step();
        reset_quest_scroll();
    });

    let viewed = viewed_step().min(step);
    let is_current = viewed == step;
    let step_info = get_step_info(viewed);
    let coding_commands = commands_for_coding_step(viewed);
    let current_submission = submission
        .read()
        .active
        .clone()
        .filter(|active| active.step == step);
    let server_error = current_submission
        .as_ref()
        .and_then(|active| match &active.status {
            SubmissionStatus::Error(message) => Some(message.as_str()),
            _ => None,
        });
    let waiting_for_submission = current_submission.as_ref().is_some_and(|active| {
        matches!(
            active.status,
            SubmissionStatus::Waiting | SubmissionStatus::AwaitingProgress
        )
    });

    // Filter kritische Events der letzten 20 Sekunden
    let critical_alert = events
        .iter()
        .filter(|e| {
            e.event_type == "SIGNAL_LOST"
                || e.event_type == "ENERGY_CRITICAL"
                || e.event_type == "SHIP_STRANDED"
                || e.event_type == "PUZZLE_FAILED"
        })
        .last();

    let stranded_count = ships.iter().filter(|s| s.is_stranded).count();

    if is_collapsed() {
        return rsx! {
            button {
                style: "position: fixed; left: 16px; top: 76px; z-index: 50; background: rgba(15, 23, 42, 0.95); border: 1px solid rgba(0, 229, 255, 0.5); color: #00e5ff; padding: 8px 14px; border-radius: 20px; font-size: 12px; font-weight: 700; cursor: pointer; box-shadow: 0 4px 16px rgba(0, 229, 255, 0.2); backdrop-filter: blur(8px); display: flex; align-items: center; gap: 8px;",
                onclick: move |_| is_collapsed.set(false),
                span { "🛸 Mission: Stufe {step}/{TUTORIAL_FINAL_STEP}" }
                span { style: "font-size: 10px; color: #94a3b8;", "▶ Ausklappen" }
            }
        };
    }

    rsx! {
        div {
            style: "position: fixed; left: 16px; top: 76px; bottom: 20px; width: 400px; z-index: 50; background: rgba(11, 19, 38, 0.96); border: 1px solid rgba(0, 229, 255, 0.35); border-radius: 14px; box-shadow: 0 8px 32px rgba(0, 0, 0, 0.6), 0 0 16px rgba(0, 229, 255, 0.1); backdrop-filter: blur(12px); display: flex; flex-direction: column; overflow: hidden; color: #f1f5f9; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;",

            // Header mit Stufenbadge und Fortschritt
            div {
                style: "padding: 14px 18px; background: rgba(15, 23, 42, 0.9); border-bottom: 1px solid rgba(0, 229, 255, 0.2); display: flex; flex-direction: column; gap: 8px;",
                div {
                    style: "display: flex; justify-content: space-between; align-items: center; gap: 8px;",
                    div {
                        style: "display: flex; align-items: center; gap: 8px;",
                        if viewed > 0 {
                            button {
                                style: "background: rgba(255, 255, 255, 0.06); border: 1px solid rgba(255, 255, 255, 0.15); color: #f1f5f9; border-radius: 6px; padding: 5px 10px; cursor: pointer;",
                                aria_label: "Vorherige Stufe",
                                onclick: move |_| viewed_step.set(viewed.saturating_sub(1)),
                                "◀"
                            }
                        }
                        span {
                            style: "background: rgba(0, 229, 255, 0.15); color: #00e5ff; border: 1px solid rgba(0, 229, 255, 0.3); padding: 3px 8px; border-radius: 6px; font-size: 11px; font-weight: 800; letter-spacing: 0.5px; white-space: nowrap;",
                            "STUFE {viewed} / {TUTORIAL_FINAL_STEP}"
                        }
                    }
                    div {
                        style: "display: flex; align-items: center; gap: 8px;",
                        if !is_current {
                            button {
                                style: "background: rgba(255, 255, 255, 0.06); border: 1px solid rgba(255, 255, 255, 0.15); color: #f1f5f9; border-radius: 6px; padding: 5px 10px; cursor: pointer;",
                                aria_label: "Nächste Stufe",
                                onclick: move |_| viewed_step.set(viewed.saturating_add(1).min(step)),
                                "▶"
                            }
                            button {
                                style: "background: rgba(0, 229, 255, 0.12); border: 1px solid rgba(0, 229, 255, 0.3); color: #00e5ff; border-radius: 6px; padding: 5px 10px; font-size: 11px; font-weight: 700; cursor: pointer; white-space: nowrap;",
                                onclick: move |_| viewed_step.set(step),
                                "Zur aktuellen Mission"
                            }
                        }
                        button {
                            style: "background: rgba(255, 255, 255, 0.06); border: 1px solid rgba(255, 255, 255, 0.15); color: #94a3b8; border-radius: 6px; width: 28px; height: 28px; cursor: pointer; display: flex; align-items: center; justify-content: center; font-size: 12px; transition: all 0.2s;",
                            aria_label: "Mission einklappen",
                            title: "Mission einklappen",
                            onclick: move |_| is_collapsed.set(true),
                            "✗"
                        }
                    }
                }
                div {
                    style: "font-size: 11px; color: #94a3b8;",
                    "Aktueller Fortschritt: Stufe {step} / {TUTORIAL_FINAL_STEP}"
                    if step >= TUTORIAL_FINAL_STEP {
                        span { style: "margin-left: 8px; color: #ffd700; font-weight: 700;", "🏆 MEISTER" }
                    }
                }
                // Fortschrittsbalken
                div {
                    style: "width: 100%; height: 6px; background: #0f172a; border-radius: 3px; overflow: hidden;",
                    div {
                        style: "height: 100%; width: {(step.min(TUTORIAL_FINAL_STEP) as f32 / TUTORIAL_FINAL_STEP as f32) * 100.0}%; background: linear-gradient(90deg, #00b4d8, #00e5ff); border-radius: 3px; transition: width 0.5s ease-out;"
                    }
                }
            }

            // Feier-Banner bei Aufstieg
            if show_celebration() {
                div {
                    style: "background: linear-gradient(90deg, #059669, #10b981); color: #fff; padding: 10px 16px; font-size: 12px; font-weight: 700; display: flex; justify-content: space-between; align-items: center; animation: pulse 1s infinite;",
                    span { "🎉 ZIEL ERREICHT! Nächste Mission freigeschaltet!" }
                    button {
                        style: "background: none; border: none; color: #fff; cursor: pointer; font-size: 14px;",
                        onclick: move |_| show_celebration.set(false),
                        "✕"
                    }
                }
            }

            // Reaktive Warnkarte bei Server-Events oder gestrandeten Schiffen
            if stranded_count > 0 {
                div {
                    style: "margin: 12px 16px 0 16px; padding: 10px 14px; background: rgba(239, 68, 68, 0.15); border: 1px solid rgba(239, 68, 68, 0.4); border-radius: 8px; font-size: 11px; color: #fca5a5; display: flex; gap: 8px; align-items: center;",
                    span { style: "font-size: 16px;", "⚠️" }
                    div {
                        div { style: "font-weight: 700;", "Achtung: {stranded_count} Drohne(n) gestrandet!" }
                        div { "Funkreichweite überschritten oder Energie leer. Drohne nimmt keine Befehle an." }
                    }
                }
            } else if let Some(ref ev) = critical_alert {
                div {
                    style: "margin: 12px 16px 0 16px; padding: 10px 14px; background: rgba(245, 158, 11, 0.15); border: 1px solid rgba(245, 158, 11, 0.4); border-radius: 8px; font-size: 11px; color: #fde68a; display: flex; gap: 8px; align-items: center;",
                    span { style: "font-size: 16px;", "⚠️" }
                    div {
                        div { style: "font-weight: 700;", "Status-Warnung:" }
                        div { "{ev.message}" }
                    }
                }
            }

            // Scrollbarer Aufgabenbereich
            div {
                id: "quest-drawer-scroll",
                style: "flex: 1; overflow-y: auto; padding: 18px; display: flex; flex-direction: column; gap: 16px;",

                // Titel & Story
                div {
                    h3 { style: "margin: 0 0 8px 0; font-size: 15px; font-weight: 700; color: #ffffff;", "{step_info.title}" }
                    p { style: "margin: 0; font-size: 12px; line-height: 1.5; color: #94a3b8;", "{step_info.story}" }
                }

                // Nummerierte Teilaufgaben
                div {
                    style: "display: flex; flex-direction: column; gap: 10px;",
                    div { style: "font-size: 11px; font-weight: 700; color: #64748b; letter-spacing: 0.5px;", "DEINE AUFGABEN:" }
                    for sub in step_info.subtasks {
                        div {
                            style: "background: rgba(15, 23, 42, 0.6); border: 1px solid rgba(255, 255, 255, 0.08); border-radius: 8px; padding: 10px 12px; display: flex; gap: 10px;",
                            div {
                                style: "background: rgba(0, 229, 255, 0.2); color: #00e5ff; width: 22px; height: 22px; border-radius: 50%; display: flex; align-items: center; justify-content: center; font-size: 11px; font-weight: 800; flex-shrink: 0;",
                                "{sub.num}"
                            }
                            div {
                                style: "flex: 1; font-size: 12px; line-height: 1.4;",
                                div { "{sub.text}" }
                                if let Some(code) = sub.code_hint {
                                    div {
                                        style: "margin-top: 6px; background: #020617; border: 1px solid rgba(0, 229, 255, 0.2); border-radius: 6px; padding: 6px 8px; font-family: monospace; font-size: 11px; color: #38bdf8; word-break: break-all; user-select: none;",
                                        "{code}"
                                    }
                                }
                            }
                        }
                    }
                }

                if is_current && step == 4 {
                    button {
                        style: "background: #0284c7; color: #fff; border: none; padding: 10px 16px; border-radius: 8px; font-weight: 700; cursor: pointer;",
                        onclick: move |_| on_open_base.call(()),
                        "🏛️ Heimatbasis öffnen und Bot koppeln"
                    }
                }

                if is_current && (step == 6 || step == 8) {
                    div {
                        style: "display: flex; flex-direction: column; gap: 8px; background: rgba(15, 23, 42, 0.8); border: 1px solid rgba(0, 229, 255, 0.25); border-radius: 8px; padding: 12px;",
                        if step == 6 {
                            label {
                                style: "font-size: 12px; color: #cbd5e1;",
                                "X-Koordinate (ganze Zahl)"
                                input {
                                    r#type: "text", inputmode: "numeric", autocomplete: "off",
                                    disabled: waiting_for_submission,
                                    style: "display: block; box-sizing: border-box; width: 100%; margin-top: 4px; background: #020617; border: 1px solid #334155; color: #f1f5f9; padding: 8px; border-radius: 6px;",
                                    value: "{coordinate_x}",
                                    oninput: move |e| { coordinate_x.set(e.value()); input_error.set(None); submission.write().active = None; },
                                }
                            }
                            label {
                                style: "font-size: 12px; color: #cbd5e1;",
                                "Y-Koordinate (ganze Zahl)"
                                input {
                                    r#type: "text", inputmode: "numeric", autocomplete: "off",
                                    disabled: waiting_for_submission,
                                    style: "display: block; box-sizing: border-box; width: 100%; margin-top: 4px; background: #020617; border: 1px solid #334155; color: #f1f5f9; padding: 8px; border-radius: 6px;",
                                    value: "{coordinate_y}",
                                    oninput: move |e| { coordinate_y.set(e.value()); input_error.set(None); submission.write().active = None; },
                                }
                            }
                        } else {
                            label {
                                style: "font-size: 12px; color: #cbd5e1;",
                                "Asteroiden-ID"
                                input {
                                    r#type: "text", inputmode: "numeric", autocomplete: "off",
                                    disabled: waiting_for_submission,
                                    style: "display: block; box-sizing: border-box; width: 100%; margin-top: 4px; background: #020617; border: 1px solid #334155; color: #f1f5f9; padding: 8px; border-radius: 6px;",
                                    value: "{asteroid_id}",
                                    oninput: move |e| { asteroid_id.set(e.value()); input_error.set(None); submission.write().active = None; },
                                }
                            }
                        }
                        if let Some(error) = input_error() {
                            div { role: "alert", style: "color: #fca5a5; font-size: 12px;", "{error}" }
                        } else if let Some(error) = server_error {
                            div { role: "alert", style: "color: #fca5a5; font-size: 12px;", "{error}" }
                        }
                        button {
                            disabled: waiting_for_submission,
                            style: "background: #0284c7; color: #fff; border: none; padding: 10px; border-radius: 6px; font-weight: 700; cursor: pointer;",
                            onclick: move |_| {
                                if submission.peek().active.as_ref().is_some_and(|active| {
                                    matches!(active.status, SubmissionStatus::Waiting | SubmissionStatus::AwaitingProgress)
                                }) {
                                    return;
                                }
                                input_error.set(None);
                                let answer = if step == 6 {
                                    parse_coordinates(&coordinate_x(), &coordinate_y())
                                        .map(|(x, y)| TutorialAnswer::Coordinates(x, y))
                                } else {
                                    parse_asteroid_id(&asteroid_id()).map(TutorialAnswer::Asteroid)
                                };
                                let answer = match answer {
                                    Ok(answer) => answer,
                                    Err(error) => { input_error.set(Some(error.to_string())); submission.write().active = None; return; }
                                };
                                let ConnectionState::Connected(owner, _) = connection_state() else {
                                    submission.write().active = None;
                                    input_error.set(Some("Keine Verbindung zum Spielserver. Bitte versuche es erneut.".to_string()));
                                    return;
                                };
                                let Some(conn) = connection() else {
                                    submission.write().active = None;
                                    input_error.set(Some("Keine Verbindung zum Spielserver. Bitte versuche es erneut.".to_string()));
                                    return;
                                };
                                // Include all players in the watermark, since event IDs are global.
                                let after = all_events().iter().map(|event| event.event_id).max();
                                let id = submission.write().begin(step);
                                let callback = move |ctx: &ReducerEventContext, result| {
                                    finish_submission(submission, id, step, after, owner, ctx, result);
                                };
                                let result = match answer {
                                    TutorialAnswer::Coordinates(x, y) => conn.reducers.submit_drone_coordinates_then(x, y, callback),
                                    TutorialAnswer::Asteroid(asteroid_id) => conn.reducers.submit_asteroid_id_then(asteroid_id, callback),
                                };
                                if let Err(error) = result {
                                    submission.write().update_pending(id, SubmissionStatus::Error(format!("Eingabe konnte nicht gesendet werden: {error}")));
                                } else {
                                    spawn(async move {
                                        wait_for_submission_timeout().await;
                                        if let Ok(mut state) = submission.try_write() {
                                            state.update_pending(id, SubmissionStatus::Error("Keine Antwort erhalten. Bitte versuche es erneut.".to_string()));
                                        }
                                    });
                                }
                            },
                            if waiting_for_submission { "⏳ Server prüft …" } else { "Eingabe prüfen" }
                        }
                        if waiting_for_submission {
                            button {
                                style: "background: transparent; color: #94a3b8; border: 1px solid #475569; padding: 8px; border-radius: 6px; cursor: pointer;",
                                onclick: move |_| submission.write().fail_pending("Die Anfrage kann noch verarbeitet werden. Du kannst die Eingabe erneut senden."),
                                "Nicht länger warten"
                            }
                        }
                    }
                }

                // Lootbox-Shortcut zur Heimatbasis
                if is_current && is_loot_box_step(viewed) {
                    button {
                        style: "background: linear-gradient(90deg, #f59e0b, #fde047); color: #020617; border: none; padding: 12px 16px; border-radius: 10px; font-weight: 800; font-size: 13px; cursor: pointer; animation: pulse 1s infinite;",
                        onclick: move |_| on_open_base.call(()),
                        "🏛️ Heimatbasis öffnen & Lootbox holen"
                    }
                }

                // Befehle dieser Mission zum Nachschlagen
                if !coding_commands.is_empty() {
                    div {
                        style: "display: flex; flex-direction: column; gap: 8px;",
                        div { style: "font-size: 11px; font-weight: 700; color: #64748b; letter-spacing: 0.5px;", "🆕 BEFEHLE DIESER MISSION – KLICKE ZUM NACHSCHLAGEN:" }
                        div {
                            style: "display: flex; flex-wrap: wrap; gap: 6px;",
                            for cmd in coding_commands.iter() {
                                {
                                    let key = cmd.key.to_string();
                                    rsx! {
                                        button {
                                            key: "{cmd.key}",
                                            style: "background: rgba(250, 204, 21, 0.1); border: 1px solid rgba(250, 204, 21, 0.4); color: #fde047; padding: 4px 8px; border-radius: 6px; font-family: monospace; font-size: 11px; cursor: pointer;",
                                            onclick: move |_| on_open_hall.call(Some(key.clone())),
                                            "{cmd.icon} {cmd.title}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Feste Fehlerhilfe-Checkliste für die Coding-Missionen
                if viewed >= 6 {
                    div {
                        style: "background: rgba(30, 41, 59, 0.5); border: 1px solid rgba(255, 255, 255, 0.06); border-radius: 8px; padding: 12px;",
                        div { style: "font-size: 11px; font-weight: 700; color: #cbd5e1; margin-bottom: 6px;", "🛠️ Fehlerhilfe & Checkliste" }
                        ul {
                            style: "margin: 0; padding-left: 18px; font-size: 11px; color: #94a3b8; display: flex; flex-direction: column; gap: 4px; line-height: 1.4;",
                            li { "1. In Zed mit " code { style: "color: #38bdf8;", "Strg + S" } " gespeichert?" }
                            li { "2. Keine roten Kringellinien im Editor?" }
                            li { "3. Programm über den Run-Button (" code { style: "color: #38bdf8;", "▶" } " neben " code { style: "color: #38bdf8;", "fn main()" } ") neu gestartet?" }
                        }
                    }
                }
            }

            // Footer mit Weiter-Button nur für Stufe 3
            div {
                style: "padding: 14px 18px; background: rgba(15, 23, 42, 0.95); border-top: 1px solid rgba(0, 229, 255, 0.2); display: flex; justify-content: center; align-items: center;",
                if !is_current {
                    div {
                        style: "color: #94a3b8; font-size: 11px; font-weight: 600;",
                        "✓ Stufe abgeschlossen"
                    }
                } else if step == 3 {
                    button {
                        style: "width: 100%; background: linear-gradient(90deg, #0284c7, #00e5ff); color: #020617; border: none; padding: 10px 16px; border-radius: 8px; font-weight: 800; font-size: 13px; cursor: pointer; transition: all 0.2s; box-shadow: 0 4px 14px rgba(0, 229, 255, 0.3);",
                        onclick: move |_| on_advance.call(3),
                        "Ich habe Zed geöffnet und den Bot gestartet ➔ Weiter zu Stufe 4"
                    }
                } else if step >= TUTORIAL_FINAL_STEP {
                    div {
                        style: "color: #ffd700; font-size: 12px; font-weight: 700; display: flex; align-items: center; gap: 6px;",
                        span { "🏆" }
                        span { "Tutorial erfolgreich gemeistert!" }
                    }
                } else {
                    div {
                        style: "color: #64748b; font-size: 11px; font-weight: 600; display: flex; align-items: center; gap: 6px;",
                        span { style: "animation: spin 2s linear infinite;", "⏳" }
                        if step == 4 {
                            span { "Warte auf bestätigte Bot-Kopplung …" }
                        } else if step == 6 || step == 8 {
                            span { "Der Server prüft deine Eingabe und schaltet die nächste Mission frei." }
                        } else {
                            span { "Warte auf Erfüllung der Bedingung im Spiel..." }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod drawer_tests {
    use super::*;
    use crate::module_bindings::dioxus::{SpacetimeDbContext, TableSignals};
    use dioxus::core::{
        AttributeValue, DynamicNode, Event, ScopeId, TemplateAttribute, TemplateNode, VNode,
        VirtualDom,
    };
    use dioxus::html::{
        self, HasMouseData, InteractionElementOffset, InteractionLocation, ModifiersInteraction,
        PointerInteraction,
    };
    use std::{any::Any, cell::Cell, rc::Rc};

    #[component]
    fn TestDrawer(initial_step: u8) -> Element {
        let progress = use_signal(|| initial_step);
        let progress_handle = use_context::<Rc<Cell<Option<Signal<u8>>>>>();
        progress_handle.set(Some(progress));
        let is_collapsed = use_signal(|| false);
        let tables = TableSignals {
            admin_all_asteroids: use_signal_sync(Vec::new),
            admin_all_bases: use_signal_sync(Vec::new),
            admin_all_ships: use_signal_sync(Vec::new),
            ai_agent: use_signal_sync(Vec::new),
            config: use_signal_sync(Vec::new),
            game_event: use_signal_sync(Vec::new),
            my_active_riddle: use_signal_sync(Vec::new),
            my_ai_agents: use_signal_sync(Vec::new),
            my_base: use_signal_sync(Vec::new),
            my_research_attempts: use_signal_sync(Vec::new),
            my_ships: use_signal_sync(Vec::new),
            my_tutorial_state: use_signal_sync(Vec::new),
            player: use_signal_sync(Vec::new),
            player_achievement: use_signal_sync(Vec::new),
            total_players: use_signal_sync(Vec::new),
            unidentified_objects: use_signal_sync(Vec::new),
            unlocked_command: use_signal_sync(Vec::new),
            visible_asteroids: use_signal_sync(Vec::new),
            visible_bases: use_signal_sync(Vec::new),
            visible_drones: use_signal_sync(Vec::new),
        };
        let connection = use_signal_sync(|| None);
        let state = use_signal_sync(|| ConnectionState::Disconnected);
        let error = use_signal_sync(|| None);
        use_context_provider(|| SpacetimeDbContext {
            connection,
            state,
            error,
            tables,
        });
        rsx! {
            QuestDrawer {
                step: progress(), is_collapsed, events: Vec::new(), ships: Vec::new(),
                on_advance: |_| {}, on_open_hall: |_| {}, on_open_base: |_| {},
            }
        }
    }

    struct TestConverter;
    macro_rules! unused_event_converters {
        ($($method:ident => $data:ident),* $(,)?) => {
            $(fn $method(&self, _: &html::PlatformEventData) -> html::$data { unreachable!() })*
        };
    }
    impl html::HtmlEventConverter for TestConverter {
        fn convert_mouse_data(&self, _: &html::PlatformEventData) -> html::MouseData {
            html::MouseData::new(TestMouse)
        }
        fn convert_form_data(&self, event: &html::PlatformEventData) -> html::FormData {
            html::FormData::new(TestForm(event.downcast::<String>().unwrap().clone()))
        }
        unused_event_converters! {
            convert_animation_data => AnimationData, convert_cancel_data => CancelData,
            convert_clipboard_data => ClipboardData, convert_composition_data => CompositionData,
            convert_drag_data => DragData, convert_focus_data => FocusData,
            convert_image_data => ImageData,
            convert_keyboard_data => KeyboardData, convert_media_data => MediaData,
            convert_mounted_data => MountedData, convert_pointer_data => PointerData,
            convert_resize_data => ResizeData, convert_scroll_data => ScrollData,
            convert_selection_data => SelectionData, convert_toggle_data => ToggleData,
            convert_touch_data => TouchData, convert_transition_data => TransitionData,
            convert_visible_data => VisibleData, convert_wheel_data => WheelData,
        }
    }

    struct TestForm(String);
    impl html::HasFileData for TestForm {
        fn files(&self) -> Vec<html::FileData> {
            Vec::new()
        }
    }
    impl html::HasFormData for TestForm {
        fn value(&self) -> String {
            self.0.clone()
        }
        fn valid(&self) -> bool {
            true
        }
        fn values(&self) -> Vec<(String, html::FormValue)> {
            Vec::new()
        }
        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    struct TestMouse;
    impl InteractionLocation for TestMouse {
        fn client_coordinates(&self) -> html::geometry::ClientPoint {
            Default::default()
        }
        fn page_coordinates(&self) -> html::geometry::PagePoint {
            Default::default()
        }
        fn screen_coordinates(&self) -> html::geometry::ScreenPoint {
            Default::default()
        }
    }
    impl InteractionElementOffset for TestMouse {
        fn element_coordinates(&self) -> html::geometry::ElementPoint {
            Default::default()
        }
    }
    impl ModifiersInteraction for TestMouse {
        fn modifiers(&self) -> html::Modifiers {
            html::Modifiers::empty()
        }
    }
    impl PointerInteraction for TestMouse {
        fn held_buttons(&self) -> html::input_data::MouseButtonSet {
            Default::default()
        }
        fn trigger_button(&self) -> Option<html::input_data::MouseButton> {
            None
        }
    }
    impl HasMouseData for TestMouse {
        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    fn find_listener(
        vnode: &VNode,
        label: Option<&str>,
        event: &str,
    ) -> Option<dioxus::core::ListenerCallback> {
        fn in_template(
            node: &TemplateNode,
            vnode: &VNode,
            label: Option<&str>,
            event: &str,
        ) -> Option<dioxus::core::ListenerCallback> {
            if let TemplateNode::Element {
                tag,
                attrs,
                children,
                ..
            } = node
            {
                if match label {
                    Some(label) => attrs.iter().any(|attr| matches!(attr, TemplateAttribute::Static { name, value, .. } if *name == "aria-label" && *value == label)),
                    None => *tag == "input",
                } {
                    for attr in attrs.iter() {
                        if let TemplateAttribute::Dynamic { id } = attr {
                            for attr in vnode.dynamic_attrs[*id].iter() {
                                if attr.name == event {
                                    if let AttributeValue::Listener(callback) = &attr.value {
                                        return Some(callback.clone());
                                    }
                                }
                            }
                        }
                    }
                }
                for child in children.iter() {
                    if let Some(callback) = in_template(child, vnode, label, event) {
                        return Some(callback);
                    }
                }
            }
            None
        }
        for root in vnode.template.roots {
            if let Some(callback) = in_template(root, vnode, label, event) {
                return Some(callback);
            }
        }
        for dynamic in vnode.dynamic_nodes.iter() {
            if let DynamicNode::Fragment(children) = dynamic {
                for child in children {
                    if let Some(callback) = find_listener(child, label, event) {
                        return Some(callback);
                    }
                }
            }
        }
        None
    }

    fn drawer(dom: &VirtualDom) -> &VNode {
        dom.get_scope(ScopeId(4)).unwrap().root_node()
    }

    fn click(dom: &mut VirtualDom, label: &str) {
        let callback = find_listener(drawer(dom), Some(label), "onclick")
            .expect("navigation button not found");
        dom.in_runtime(|| {
            let data = Rc::new(html::PlatformEventData::new(Box::new(()))) as Rc<dyn Any>;
            callback.call(Event::new(data, true));
        });
        dom.render_immediate_to_vec();
    }

    fn setup(step: u8) -> (VirtualDom, Rc<Cell<Option<Signal<u8>>>>) {
        html::set_event_converter(Box::new(TestConverter));
        let progress = Rc::new(Cell::new(None));
        let mut dom =
            VirtualDom::new_with_props(TestDrawer, TestDrawerProps { initial_step: step })
                .with_root_context(progress.clone());
        dom.rebuild_to_vec();
        dom.render_immediate_to_vec();
        (dom, progress)
    }

    #[test]
    fn browsing_hides_current_controls_and_keeps_server_progress() {
        for (step, current_control) in [
            (3, "Bot gestartet"),
            (4, "Bot koppeln"),
            (5, "Lootbox holen"),
            (6, "X-Koordinate"),
            (8, "Asteroiden-ID"),
        ] {
            let (mut dom, progress) = setup(step);
            assert!(format!("{:?}", drawer(&dom)).contains(current_control));
            click(&mut dom, "Vorherige Stufe");
            let previous = format!("{:?}", drawer(&dom));
            assert!(
                !previous.contains(current_control),
                "control remained visible at step {step}"
            );
            assert!(previous.contains("Stufe abgeschlossen"));
            assert_eq!(dom.in_runtime(|| progress.get().unwrap()()), step);
        }
    }

    #[test]
    fn completed_missions_never_expose_their_operational_controls() {
        let (mut dom, progress) = setup(10);
        click(&mut dom, "Vorherige Stufe");
        assert!(!format!("{:?}", drawer(&dom)).contains("Lootbox holen"));
        click(&mut dom, "Vorherige Stufe");
        assert!(find_listener(drawer(&dom), None, "oninput").is_none());
        click(&mut dom, "Vorherige Stufe");
        click(&mut dom, "Vorherige Stufe");
        assert!(find_listener(drawer(&dom), None, "oninput").is_none());
        click(&mut dom, "Vorherige Stufe");
        click(&mut dom, "Vorherige Stufe");
        assert!(!format!("{:?}", drawer(&dom)).contains("Heimatbasis öffnen und Bot koppeln"));
        click(&mut dom, "Vorherige Stufe");
        assert!(
            !format!("{:?}", drawer(&dom)).contains("Ich habe Zed geöffnet und den Bot gestartet")
        );
        assert_eq!(dom.in_runtime(|| progress.get().unwrap()()), 10);
    }

    #[test]
    fn input_survives_browsing_but_is_cleared_on_server_progress() {
        let (mut dom, progress) = setup(6);
        let callback = find_listener(drawer(&dom), None, "oninput").unwrap();
        dom.in_runtime(|| {
            let data = Rc::new(html::PlatformEventData::new(Box::new("123456".to_string())))
                as Rc<dyn Any>;
            callback.call(Event::new(data, true));
        });
        dom.render_immediate_to_vec();
        assert!(format!("{:?}", drawer(&dom)).contains("123456"));
        click(&mut dom, "Vorherige Stufe");
        assert!(!format!("{:?}", drawer(&dom)).contains("123456"));
        click(&mut dom, "Nächste Stufe");
        assert!(format!("{:?}", drawer(&dom)).contains("123456"));

        dom.in_runtime(|| progress.get().unwrap().set(7));
        dom.render_immediate_to_vec();
        dom.render_immediate_to_vec();
        dom.in_runtime(|| progress.get().unwrap().set(6));
        dom.render_immediate_to_vec();
        dom.render_immediate_to_vec();
        let reset = format!("{:?}", drawer(&dom));
        assert!(reset.contains("X-Koordinate"));
        assert!(!reset.contains("123456"));
    }

    #[test]
    fn server_reset_clamps_the_view_and_removes_completion() {
        let (mut dom, progress) = setup(TUTORIAL_FINAL_STEP);
        assert!(format!("{:?}", drawer(&dom)).contains("🏆 MEISTER"));
        click(&mut dom, "Vorherige Stufe");
        click(&mut dom, "Vorherige Stufe");
        dom.in_runtime(|| progress.get().unwrap().set(16));
        dom.render_immediate_to_vec();
        dom.render_immediate_to_vec();
        let reset = format!("{:?}", drawer(&dom));
        assert!(reset.contains("STUFE 16 / 37"));
        assert!(reset.contains("Aktueller Fortschritt: Stufe 16 / 37"));
        assert!(!reset.contains("🏆 MEISTER"));
        assert!(reset.contains("Multitasking für Fortgeschrittene"));
    }

    #[test]
    fn browsing_uses_viewed_mission_cards_and_tracks_new_server_progress() {
        let (mut dom, progress) = setup(8);
        click(&mut dom, "Vorherige Stufe");
        click(&mut dom, "Vorherige Stufe");
        let previous = format!("{:?}", drawer(&dom));
        assert!(previous.contains("Wo ist deine Drohne?"));
        assert!(previous.contains("STUFE 6 / 37"));
        assert!(previous.contains("Aktueller Fortschritt: Stufe 8 / 37"));
        assert!(previous.contains("width: 21.62"));
        assert!(previous.contains("🛸 world.first_drone()"));
        assert!(!previous.contains("☄️ world.closest_asteroid_with_ore(pos)"));
        assert!(find_listener(drawer(&dom), None, "oninput").is_none());
        dom.in_runtime(|| progress.get().unwrap().set(9));
        dom.render_immediate_to_vec();
        dom.render_immediate_to_vec();
        let advanced = format!("{:?}", drawer(&dom));
        assert!(advanced.contains("Befehls-Lootbox #3"));
        assert!(advanced.contains("Lootbox holen"));
        assert_eq!(dom.in_runtime(|| progress.get().unwrap()()), 9);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spacetimedb_sdk::{Identity, Timestamp};

    fn event(id: u64, event_type: &str, message: &str) -> GameEvent {
        GameEvent {
            event_id: id,
            player: Identity::ZERO,
            event_type: event_type.to_string(),
            message: message.to_string(),
            timestamp: Timestamp::from_micros_since_unix_epoch(0),
        }
    }

    #[test]
    fn missions_follow_the_new_order() {
        let titles = [
            (3, "Die Brücke öffnen (uhlaunch & Zed)"),
            (4, "Bot mit der Heimatbasis koppeln"),
            (6, "Wo ist deine Drohne?"),
            (8, "Die Asteroiden-ID ermitteln"),
            (10, "Der erste Schürfzug"),
            (12, "Frachtraum voll! Kurs Heimatbasis"),
            (14, "Flottenausbau (Zweite Drohne bauen)"),
            (16, "Multitasking für Fortgeschrittene (Flottenschleife)"),
            (18, "Rechtzeitig an die Steckdose (Energiemanagement)"),
            (20, "Forschungsauftrag anfordern"),
            (22, "Messwerte für den Asteroiden-Scanner sortieren"),
            (37, "Tutorial abgeschlossen – Freies Spiel!"),
        ];
        for (step, title) in titles {
            assert_eq!(get_step_info(step).title, title);
        }
        for step in [5, 7, 9, 11, 13, 15, 17, 19, 21, 23, 25, 27, 29, 31, 33, 35] {
            assert!(get_step_info(step).title.contains("Lootbox"));
        }
    }

    #[test]
    fn validates_integer_inputs_and_boundaries() {
        assert_eq!(parse_coordinates(" -12 ", "+34"), Ok((-12, 34)));
        assert_eq!(
            parse_coordinates("-2147483648", "2147483647"),
            Ok((i32::MIN, i32::MAX))
        );
        for bad in ["", "1.0", "2147483648", "-2147483649"] {
            assert!(parse_coordinates(bad, "0").is_err());
            assert!(parse_coordinates("0", bad).is_err());
        }
        assert_eq!(parse_asteroid_id(" 4294967295 "), Ok(u32::MAX));
        for bad in ["", "-1", "1.0", "4294967296"] {
            assert!(parse_asteroid_id(bad).is_err());
        }
    }

    #[test]
    fn shows_only_errors_from_the_current_players_transaction() {
        let mut stale = event(10, "TUTORIAL_COORDINATES_ERROR", "alter Versuch");
        stale.timestamp = Timestamp::from_micros_since_unix_epoch(1);
        let mut other_player = event(12, "TUTORIAL_COORDINATES_ERROR", "andere Person");
        other_player.player = Identity::from_byte_array([1; 32]);
        let events = vec![
            event(8, "TUTORIAL_COORDINATES_ERROR", "älter"),
            stale,
            event(11, "TUTORIAL_ASTEROID_ERROR", "falsche Übung"),
            other_player,
            event(9, "TUTORIAL_COORDINATES_ERROR", "aktuell"),
        ];
        let lookup = |step, after| {
            submission_error(
                events.clone().into_iter(),
                step,
                after,
                Identity::ZERO,
                Timestamp::from_micros_since_unix_epoch(0),
            )
        };
        assert_eq!(lookup(6, Some(8)).as_deref(), Some("aktuell"));
        assert_eq!(lookup(6, Some(9)), None);
        assert_eq!(lookup(8, Some(10)).as_deref(), Some("falsche Übung"));
        assert_eq!(lookup(7, None), None);
    }

    #[test]
    fn timeout_disconnect_and_late_results_do_not_overwrite_retries() {
        let mut state = SubmissionState::default();
        let first = state.begin(6);
        state.update_pending(first, SubmissionStatus::Error("Zeitüberschreitung".into()));
        let second = state.begin(6);
        state.update_pending(first, SubmissionStatus::Error("verspäteter Fehler".into()));
        assert_eq!(
            state.active.as_ref().unwrap().status,
            SubmissionStatus::Waiting
        );
        state.update_pending(second, SubmissionStatus::AwaitingProgress);
        state.fail_pending("Verbindung unterbrochen");
        state.update_pending(second, SubmissionStatus::AwaitingProgress);
        assert_eq!(
            state.active.as_ref().unwrap().status,
            SubmissionStatus::Error("Verbindung unterbrochen".into())
        );
        state.active = None;
        let third = state.begin(8);
        state.update_pending(second, SubmissionStatus::Error("verspätet".into()));
        assert_eq!(state.active.as_ref().unwrap().id, third);
        assert_eq!(
            state.active.as_ref().unwrap().status,
            SubmissionStatus::Waiting
        );
    }
}
