//! Befehls-Ruhmeshalle: Katalog aller lernbaren Befehle, Lootbox-Enthüllung und Nachschlage-Modal.

use crate::audio::SoundEffects;
use crate::module_bindings::claim_next_command_reducer::claim_next_command;
use crate::module_bindings::dioxus::use_spacetimedb_context;
use dioxus::prelude::*;

pub use curriculum::{
    COMMANDS, Command as CommandInfo, LOOT_BOX_STEPS, all_commands, command as find_command,
    is_loot_box_step,
};

/// Befehle einer Lootbox in der festgelegten Reihenfolge.
pub fn commands_for_box(step: u8) -> Vec<&'static CommandInfo> {
    curriculum::step(step)
        .map(|s| s.unlocked_commands().collect())
        .unwrap_or_default()
}

/// Befehle, die in einer Tutorial-Stufe (Coding-Schritt) neu zur Verfügung stehen: die Lootbox direkt davor.
pub fn commands_for_coding_step(step: u8) -> Vec<&'static CommandInfo> {
    curriculum::step(step)
        .map(|s| s.reference_commands().collect())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loot_boxes_and_following_missions_match_the_tutorial() {
        let expected: &[(u8, &[&str])] = &[
            (
                5,
                &[
                    "IF_LET_SOME",
                    "WORLD_FIRST_DRONE",
                    "PRINTLN",
                    "DRONE_POSITION",
                    "FORMAT_NO_DECIMALS",
                    "PRINTLN_MACRO",
                ],
            ),
            (7, &["WORLD_CLOSEST_ASTEROID", "ASTEROID_ID"]),
            (9, &["DRONE_SHIP_ID", "ACTIONS_MINE"]),
            (11, &["IF_ELSE", "DRONE_CARGO_IS_FULL", "ACTIONS_DOCK"]),
            (
                13,
                &[
                    "WORLD_FLEET_LEN",
                    "WORLD_BASE_STORED_ORE",
                    "COMPARISONS",
                    "LOGICAL_AND",
                    "ACTIONS_BUILD_DRONE_1",
                    "COMPARE_AND",
                ],
            ),
            (15, &["FOR_IN_FLEET"]),
            (17, &["DRONE_IS_DOCKED", "DRONE_ENERGY", "ELSE_IF"]),
            (
                19,
                &[
                    "WORLD_ACTIVE_RIDDLE",
                    "OPTION_IS_NONE",
                    "ACTIONS_REQUEST_RESEARCH",
                    "LET_BINDING",
                    "CLONE_AND_SORT",
                    "RIDDLE_SORT",
                    "RIDDLE_DISTANCE",
                    "RIDDLE_DENOISE",
                    "RIDDLE_ENERGY",
                    "ACTIONS_SOLVE_RIDDLE",
                ],
            ),
            (21, &["ACTIONS_BUILD_DRONE_2"]),
            (
                22,
                &[
                    "SHIP_TIER",
                    "SHIP_VELOCITY",
                    "SHIP_TARGET_POSITION",
                    "SHIP_HEADING",
                    "SHIP_SPEED",
                    "SHIP_MAX_SPEED",
                    "SHIP_ENERGY",
                    "SHIP_MAX_ENERGY",
                    "SHIP_CARGO_ORE",
                    "SHIP_MAX_CARGO",
                    "SHIP_TARGET_ASTEROID_ID",
                    "SHIP_IS_MINING",
                    "SHIP_IS_STRANDED",
                    "SHIP_SIGNAL_COUNTDOWN",
                    "SHIP_DISTANCE_TO",
                    "SHIP_CARGO_FREE_SPACE",
                    "SHIP_ENERGY_PERCENT",
                    "WORLD_SHIP",
                ],
            ),
            (
                23,
                &[
                    "WORLD_ASTEROIDS",
                    "WORLD_CLOSEST_ASTEROID_ANY",
                    "ASTEROID_POSITION",
                    "ASTEROID_VELOCITY",
                    "ASTEROID_RESOURCE_TYPE",
                    "ASTEROID_REMAINING_ORE",
                    "ASTEROID_RADIUS",
                    "ASTEROID_DISTANCE_TO",
                    "ASTEROID_IS_DEPLETED",
                    "ASTEROID_RESOURCE_NAME",
                    "VEC2_NEW",
                    "VEC2_ZERO",
                    "VEC2_XY",
                    "VEC2_DISTANCE",
                ],
            ),
            (
                24,
                &[
                    "WORLD_BASE",
                    "WORLD_TICK",
                    "BASE_POSITION",
                    "BASE_SIGNAL_RADIUS",
                    "BASE_STORED_ENERGY",
                    "BASE_LEVEL",
                    "BASE_DRONE_SCANNER_LEVEL",
                    "BASE_BASE_SCANNER_LEVEL",
                    "BASE_RADAR_HORIZON_LEVEL",
                    "BASE_DISTANCE_TO",
                    "BASE_IS_IN_RANGE",
                    "BASE_DRONE_SCANNER_RANGE",
                    "BASE_BASE_SCANNER_RANGE",
                    "BASE_RADAR_HORIZON_RANGE",
                ],
            ),
            (
                25,
                &[
                    "WORLD_OTHER_DRONES",
                    "WORLD_OTHER_BASES",
                    "OTHER_DRONE_SHIP_ID",
                    "OTHER_DRONE_TIER",
                    "OTHER_DRONE_POSITION",
                    "OTHER_DRONE_HEADING",
                    "OTHER_DRONE_SPEED",
                    "OTHER_DRONE_IS_MINING",
                    "OTHER_BASE_ID",
                    "OTHER_BASE_POSITION",
                    "OTHER_BASE_SIGNAL_RADIUS",
                    "OTHER_BASE_LEVEL",
                    "OTHER_BASE_SCANNER_LEVELS",
                    "CONTACT_ID",
                    "CONTACT_POSITION",
                ],
            ),
            (
                26,
                &[
                    "ACTIONS_FLY_TO",
                    "ACTIONS_STOP",
                    "UPGRADE_TRACK_BASE_ASTEROID",
                    "UPGRADE_TRACK_RADAR_HORIZON",
                    "UPGRADE_TRACK_NAME",
                    "RIDDLE_TASK_DISTANCE",
                    "RIDDLE_TASK_DENOISE",
                    "RIDDLE_SOLUTION_DISTANCE",
                    "RIDDLE_SOLUTION_DENOISE",
                ],
            ),
        ];
        let expected_free_play: &[&str] = &[
            "DRONE_TIER",
            "DRONE_CARGO",
            "DRONE_MOTION",
            "DRONE_TARGETS_AND_STATUS",
            "DRONE_SIGNAL_AND_STRANDED",
            "DRONE_DISTANCE_TO",
            "BASE_POSITION_AND_SIGNAL",
            "BASE_RESEARCH_LEVELS",
            "ASTEROID_POSITION_AND_DISTANCE",
            "ASTEROID_ORE_AND_TYPE",
            "SDK_DRONE_COSTS",
        ];
        assert_eq!(
            LOOT_BOX_STEPS.as_slice(),
            expected.iter().map(|(step, _)| *step).collect::<Vec<_>>()
        );
        for (step, keys) in expected {
            let box_keys: Vec<_> = commands_for_box(*step).iter().map(|c| c.key).collect();
            assert_eq!(box_keys, *keys);
            if *step <= 19 {
                let coding_keys: Vec<_> = commands_for_coding_step(*step + 1)
                    .iter()
                    .map(|c| c.key)
                    .collect();
                assert_eq!(coding_keys, *keys);
            }
            assert!(is_loot_box_step(*step));
        }
        let free_play_keys: Vec<_> = commands_for_box(curriculum::FINAL_STEP)
            .iter()
            .map(|c| c.key)
            .collect();
        assert_eq!(free_play_keys, expected_free_play);
        assert!(commands_for_coding_step(0).is_empty());
    }

    #[test]
    fn previously_claimed_box_can_be_completed_after_reset() {
        let commands = commands_for_box(5);
        assert!(!box_fully_claimed(&commands, &[]));
        assert!(!box_fully_claimed(&commands, &["IF_LET_SOME".into()]));
        assert!(box_fully_claimed(
            &commands,
            &[
                "FORMAT_NO_DECIMALS".into(),
                "DRONE_POSITION".into(),
                "PRINTLN".into(),
                "WORLD_FIRST_DRONE".into(),
                "IF_LET_SOME".into(),
                "PRINTLN_MACRO".into()
            ]
        ));
        assert!(!box_fully_claimed(&commands_for_box(6), &[]));
    }
}

const KEYFRAMES: &str = "@keyframes lootShake { 0%,100% { transform: rotate(0deg) scale(1);} 20% { transform: rotate(-8deg) scale(1.05);} 40% { transform: rotate(8deg) scale(1.08);} 60% { transform: rotate(-6deg) scale(1.05);} 80% { transform: rotate(6deg) scale(1.08);} } \
@keyframes lootGlow { 0%,100% { box-shadow: 0 0 24px rgba(250,204,21,0.45);} 50% { box-shadow: 0 0 56px rgba(250,204,21,0.9);} } \
@keyframes cardPop { 0% { transform: scale(0.4) rotateY(90deg); opacity: 0;} 70% { transform: scale(1.06) rotateY(0deg); opacity: 1;} 100% { transform: scale(1) rotateY(0deg); opacity: 1;} }";

fn card_style(unlocked: bool) -> &'static str {
    if unlocked {
        "background: linear-gradient(135deg, rgba(250, 204, 21, 0.1) 0%, rgba(15, 23, 42, 0.95) 100%); border: 1px solid rgba(250, 204, 21, 0.45); border-radius: 10px; padding: 14px; cursor: pointer; user-select: none;"
    } else {
        "background: rgba(15, 23, 42, 0.5); border: 1px dashed rgba(255, 255, 255, 0.12); border-radius: 10px; padding: 14px; opacity: 0.55; user-select: none;"
    }
}

/// Befehls-Ruhmeshalle: Übersicht aller Befehle mit Detailansicht (nicht kopierbar).
#[component]
pub fn CommandHallModal(
    unlocked: Vec<String>,
    initial: Option<String>,
    on_close: EventHandler<()>,
) -> Element {
    let mut selected = use_signal(move || initial.clone());

    let unlocked_list: Vec<&'static CommandInfo> = COMMANDS
        .iter()
        .filter(|c| unlocked.iter().any(|u| u == c.key))
        .collect();
    let unlocked_count = unlocked_list.len();
    let mut selected_category = use_signal(String::new);
    let category = selected_category();
    let mut categories: Vec<&'static str> = Vec::new();
    for command in COMMANDS.iter() {
        if !categories.contains(&command.category) {
            categories.push(command.category);
        }
    }
    categories.sort_unstable();
    let visible_commands: Vec<&'static CommandInfo> = COMMANDS
        .iter()
        .filter(|command| category.is_empty() || command.category == category)
        .collect();

    let visible_unlocked_list: Vec<&'static CommandInfo> = visible_commands
        .iter()
        .copied()
        .filter(|command| unlocked.iter().any(|u| u == command.key))
        .collect();
    let current = selected().and_then(|k| find_command(&k));
    let current_idx = current.and_then(|c| {
        visible_unlocked_list
            .iter()
            .position(|command| command.key == c.key)
    });
    let prev_list = visible_unlocked_list.clone();
    let next_list = visible_unlocked_list.clone();

    rsx! {
            div {
                style: "position: fixed; inset: 0; background: rgba(3, 7, 18, 0.85); backdrop-filter: blur(6px); display: flex; align-items: center; justify-content: center; z-index: 1000; padding: 20px; user-select: none;",
                onclick: move |_| on_close.call(()),
                div {
                    style: "background: #0b1329; border: 1px solid rgba(250, 204, 21, 0.4); border-radius: 16px; width: 100%; max-width: 900px; max-height: 90vh; overflow-y: auto; box-shadow: 0 24px 64px rgba(0,0,0,0.8); display: flex; flex-direction: column; user-select: none;",
                    onclick: move |e| e.stop_propagation(),

                    div {
                        style: "padding: 20px 24px; border-bottom: 1px solid rgba(80, 120, 200, 0.2); display: flex; justify-content: space-between; align-items: center; background: rgba(15, 23, 42, 0.6);",
                        div {
                            style: "display: flex; align-items: center; gap: 10px;",
                            span { style: "font-size: 24px;", "📖" }
                            div {
                                h2 { style: "margin: 0; font-size: 18px; font-weight: 700; color: #ffd700;", "Befehls-Ruhmeshalle" }
                                div { style: "font-size: 11px; color: #94a3b8; margin-top: 2px;", "{unlocked_count} / {COMMANDS.len()} Befehle gesammelt" }
                            }
                        }
                        button {
                            style: "background: rgba(255, 255, 255, 0.08); border: none; color: #cbd5e1; font-size: 16px; width: 32px; height: 32px; border-radius: 50%; cursor: pointer;",
                            onclick: move |_| on_close.call(()),
                            "✕"
                        }
                    }
                    div {
                        style: "padding: 12px 24px; border-bottom: 1px solid rgba(80, 120, 200, 0.2); display: flex; align-items: center; gap: 8px;",
                        label { style: "font-size: 12px; font-weight: 700; color: #cbd5e1;", "Kategorie" }
                        select {
                            style: "background: #0f172a; border: 1px solid rgba(255,255,255,0.2); color: #cbd5e1; padding: 7px 10px; border-radius: 6px; font-size: 12px; cursor: pointer;",
                            value: "{category}",
                            onchange: move |event| {
                                selected_category.set(event.value());
                                selected.set(None);
                            },
                            option { value: "", "Alle Kategorien" }
                            for category_option in categories.iter() {
                                option { value: "{category_option}", "{category_option}" }
                            }
                        }
                    }

                    if let Some(cmd) = current {
                        div {
                            style: "padding: 24px; display: flex; flex-direction: column; gap: 16px;",
                            div {
                                style: "display: flex; justify-content: space-between; align-items: center;",
                                button {
                                    style: "background: rgba(255,255,255,0.06); border: 1px solid rgba(255,255,255,0.15); color: #cbd5e1; padding: 6px 12px; border-radius: 6px; font-size: 12px; cursor: pointer;",
                                    onclick: move |_| selected.set(None),
                                    "◀ Zur Übersicht"
                                }
                                div {
                                    style: "display: flex; gap: 8px;",
                                    button {
                                        style: "background: rgba(255,255,255,0.06); border: 1px solid rgba(255,255,255,0.15); color: #cbd5e1; padding: 6px 12px; border-radius: 6px; font-size: 12px; cursor: pointer;",
                                        onclick: move |_| {
                                            if let Some(i) = current_idx {
                                                if i > 0 {
                                                    selected.set(Some(prev_list[i - 1].key.to_string()));
                                                }
                                            }
                                        },
                                        "◀ Vorheriger"
                                    }
                                    button {
                                        style: "background: rgba(255,255,255,0.06); border: 1px solid rgba(255,255,255,0.15); color: #cbd5e1; padding: 6px 12px; border-radius: 6px; font-size: 12px; cursor: pointer;",
                                        onclick: move |_| {
                                            if let Some(i) = current_idx {
                                                if i + 1 < next_list.len() {
                                                    selected.set(Some(next_list[i + 1].key.to_string()));
                                                }
                                            }
                                        },
                                        "Nächster ▶"
                                    }
                                }
                            }
                            div {
                                style: "display: flex; align-items: center; gap: 12px;",
                                span { style: "font-size: 36px;", "{cmd.icon}" }
                                div {
                                    span { style: "font-size: 10px; font-weight: 700; padding: 2px 8px; border-radius: 4px; background: rgba(0, 229, 255, 0.15); color: #00e5ff;", "{cmd.category}" }
                                    div { style: "font-family: monospace; font-size: 16px; font-weight: 700; color: #ffd700; margin-top: 6px;", "{cmd.title}" }
                                }
                            }
                            div {
                                div { style: "font-size: 11px; font-weight: 700; color: #64748b; letter-spacing: 0.5px; margin-bottom: 4px;", "WAS MACHT ES?" }
                                p { style: "margin: 0; font-size: 13px; line-height: 1.6; color: #cbd5e1;", "{cmd.what}" }
                            }
                            div {
                                div { style: "font-size: 11px; font-weight: 700; color: #64748b; letter-spacing: 0.5px; margin-bottom: 4px;", "WIE BENUTZT MAN ES?" }
                                p { style: "margin: 0; font-size: 13px; line-height: 1.6; color: #cbd5e1;", "{cmd.how}" }
                            }
                            pre {
                                style: "margin: 0; background: #020617; border: 1px solid rgba(0, 229, 255, 0.25); border-radius: 8px; padding: 14px; font-family: monospace; font-size: 13px; color: #38bdf8; white-space: pre-wrap; user-select: none;",
                                "{cmd.example}"
                            }
                            div { style: "font-size: 11px; color: #64748b;", "✍️ Tippe den Code selbst in Zed – nur so lernst du ihn wirklich." }
                        }
                    } else {
                        div {
                            style: "padding: 24px;",
                            div {
                                style: "display: grid; grid-template-columns: repeat(auto-fill, minmax(250px, 1fr)); gap: 12px;",
                                for cmd in visible_commands.iter() {
                                {
                                    let is_unlocked = unlocked.iter().any(|u| u == cmd.key);
                                    let key = cmd.key.to_string();
                                    rsx! {
                                        div {
                                            key: "{cmd.key}",
                                            style: card_style(is_unlocked),
                                            onclick: move |_| {
                                                if is_unlocked {
                                                    selected.set(Some(key.clone()));
                                                }
                                            },
                                            div {
                                                style: "display: flex; align-items: center; justify-content: space-between; margin-bottom: 6px;",
                                                span { style: "font-size: 24px;", if is_unlocked { "{cmd.icon}" } else { "❓" } }
                                                if is_unlocked {
                                                    span { style: "font-size: 10px; font-weight: 700; padding: 2px 6px; border-radius: 4px; background: rgba(0, 230, 118, 0.2); color: #00e676;", "✔ GESAMMELT" }
                                                } else {
                                                    span { style: "font-size: 10px; font-weight: 700; padding: 2px 6px; border-radius: 4px; background: rgba(255,255,255,0.05); color: #94a3b8;", "🔒 Stufe {cmd.box_step}" }
                                                }
                                            }
                                            if is_unlocked {
                                                div { style: "font-size: 10px; color: #00e5ff; margin-bottom: 2px;", "{cmd.category}" }
                                                div { style: "font-family: monospace; font-size: 12px; font-weight: 700; color: #ffd700; margin-bottom: 4px; word-break: break-all;", "{cmd.title}" }
                                                div { style: "font-size: 11px; color: #94a3b8; line-height: 1.3;", "{cmd.summary}" }
                                            } else {
                                                div { style: "font-size: 12px; font-weight: 700; color: #cbd5e1; margin-bottom: 4px;", "???" }
                                                div { style: "font-size: 11px; color: #94a3b8;", "🔒 Freigeschaltet ab Stufe {cmd.box_step}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn box_fully_claimed(box_commands: &[&CommandInfo], unlocked: &[String]) -> bool {
    !box_commands.is_empty()
        && box_commands
            .iter()
            .all(|command| unlocked.iter().any(|key| key == command.key))
}

/// Lootbox in der Heimatbasis: Öffnen, danach Karte für Karte einsammeln.
/// Jeder Klick auf "Einsammeln" speichert den Befehl serverseitig; nach der letzten Karte schaltet der Server die nächste Mission frei.
#[component]
pub fn LootBoxRevealModal(step: u8, unlocked: Vec<String>, on_close: EventHandler<()>) -> Element {
    let ctx = use_spacetimedb_context();
    let mut opened = use_signal(|| false);

    let box_cmds = commands_for_box(step);
    let total = box_cmds.len();
    let claimed = box_cmds
        .iter()
        .filter(|c| unlocked.iter().any(|u| u == c.key))
        .count();
    let next = box_cmds
        .iter()
        .find(|c| !unlocked.iter().any(|u| u == c.key))
        .copied();

    rsx! {
        div {
            style: "position: fixed; inset: 0; background: rgba(3, 7, 18, 0.92); backdrop-filter: blur(8px); display: flex; align-items: center; justify-content: center; z-index: 1100; padding: 20px; user-select: none;",
            style { "{KEYFRAMES}" }
            if box_fully_claimed(&box_cmds, &unlocked) {
                div {
                    style: "display: flex; flex-direction: column; align-items: center; gap: 20px; text-align: center;",
                    div { style: "font-size: 64px;", "🎁" }
                    div { style: "font-size: 20px; font-weight: 800; color: #ffd700;", "Alle Befehle bereits eingesammelt" }
                    div { style: "font-size: 13px; color: #94a3b8;", "Schließe diese Lootbox ab, um die nächste Mission freizuschalten." }
                    button {
                        style: "background: linear-gradient(90deg, #f59e0b, #fde047); color: #020617; border: none; padding: 12px 16px; border-radius: 10px; font-weight: 800; font-size: 14px; cursor: pointer;",
                        onclick: move |_| {
                            if let Some(conn) = (ctx.connection)() {
                                let _ = conn.reducers.claim_next_command();
                            }
                        },
                        "Lootbox abschließen ✔"
                    }
                    button {
                        style: "background: rgba(255,255,255,0.06); border: 1px solid rgba(255,255,255,0.15); color: #94a3b8; padding: 6px 14px; border-radius: 6px; font-size: 12px; cursor: pointer;",
                        onclick: move |_| on_close.call(()),
                        "Später abschließen"
                    }
                }
            } else if !opened() {
                div {
                    style: "display: flex; flex-direction: column; align-items: center; gap: 20px; text-align: center;",
                    div {
                        style: "font-size: 96px; width: 180px; height: 180px; display: flex; align-items: center; justify-content: center; border-radius: 28px; background: radial-gradient(circle, rgba(250,204,21,0.2), rgba(15,23,42,0.9)); border: 2px solid rgba(250, 204, 21, 0.7); cursor: pointer; animation: lootShake 1.2s infinite, lootGlow 2s infinite;",
                        onclick: move |_| {
                            SoundEffects::play_success_ping();
                            opened.set(true);
                        },
                        "🎁"
                    }
                    div { style: "font-size: 20px; font-weight: 800; color: #ffd700;", "Befehls-Lootbox" }
                    div { style: "font-size: 13px; color: #94a3b8;", "{total} neue Befehle warten auf dich. Klicke die Box, um sie zu öffnen!" }
                    button {
                        style: "background: rgba(255,255,255,0.06); border: 1px solid rgba(255,255,255,0.15); color: #94a3b8; padding: 6px 14px; border-radius: 6px; font-size: 12px; cursor: pointer;",
                        onclick: move |_| on_close.call(()),
                        "Später öffnen"
                    }
                }
            } else if let Some(cmd) = next {
                div {
                    key: "{cmd.key}",
                    style: "width: 100%; max-width: 520px; background: linear-gradient(160deg, #1e293b 0%, #0b1329 100%); border: 2px solid rgba(250, 204, 21, 0.7); border-radius: 18px; padding: 28px; display: flex; flex-direction: column; gap: 14px; animation: cardPop 0.6s ease-out, lootGlow 2s infinite; user-select: none;",
                    div {
                        style: "display: flex; justify-content: space-between; align-items: center;",
                        span { style: "font-size: 11px; font-weight: 700; padding: 3px 10px; border-radius: 6px; background: rgba(0, 229, 255, 0.15); color: #00e5ff;", "{cmd.category}" }
                        span { style: "font-size: 11px; color: #94a3b8; font-weight: 700;", "Karte {claimed + 1} / {total}" }
                    }
                    div { style: "font-size: 54px; text-align: center;", "{cmd.icon}" }
                    div { style: "font-family: monospace; font-size: 17px; font-weight: 800; color: #ffd700; text-align: center; word-break: break-all;", "{cmd.title}" }
                    p { style: "margin: 0; font-size: 13px; line-height: 1.6; color: #cbd5e1;", "{cmd.what}" }
                    div {
                        div { style: "font-size: 11px; font-weight: 700; color: #64748b; letter-spacing: 0.5px; margin-bottom: 4px;", "WIE BENUTZT MAN ES?" }
                        p { style: "margin: 0 0 8px 0; font-size: 13px; line-height: 1.6; color: #cbd5e1;", "{cmd.how}" }
                        pre {
                            style: "margin: 0; background: #020617; border: 1px solid rgba(0, 229, 255, 0.25); border-radius: 8px; padding: 12px; font-family: monospace; font-size: 12px; color: #38bdf8; white-space: pre-wrap; user-select: none;",
                            "{cmd.example}"
                        }
                    }
                    button {
                        style: "background: linear-gradient(90deg, #f59e0b, #fde047); color: #020617; border: none; padding: 12px 16px; border-radius: 10px; font-weight: 800; font-size: 14px; cursor: pointer;",
                        onclick: move |_| {
                            SoundEffects::play_success_ping();
                            if let Some(conn) = (ctx.connection)() {
                                let _ = conn.reducers.claim_next_command();
                            }
                        },
                        if claimed + 1 >= total { "Befehl einsammeln & abschließen ✔" } else { "Befehl einsammeln & weiter ➔" }
                    }
                }
            }
        }
    }
}
