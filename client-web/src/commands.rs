//! Befehls-Ruhmeshalle: Katalog aller lernbaren Befehle, Lootbox-Enthüllung und Nachschlage-Modal.

use crate::audio::SoundEffects;
use crate::module_bindings::claim_next_command_reducer::claim_next_command;
use crate::module_bindings::dioxus::use_spacetimedb_context;
use dioxus::prelude::*;

/// Tutorial-Stufen, in denen eine Befehls-Lootbox in der Heimatbasis wartet.
pub const LOOT_BOX_STEPS: [u8; 6] = [5, 7, 9, 11, 13, 15];

/// Gibt `true` zurück, wenn bei dieser Tutorial-Stufe eine Lootbox wartet.
pub fn is_loot_box_step(step: u8) -> bool {
    LOOT_BOX_STEPS.contains(&step)
}

pub struct CommandInfo {
    /// Eindeutiger Schlüssel (identisch zum Server-Schlüssel in `unlocked_command`).
    pub key: &'static str,
    /// Tutorial-Stufe der Lootbox, in der der Befehl freigeschaltet wird.
    pub box_step: u8,
    pub icon: &'static str,
    pub category: &'static str,
    /// Schreibweise des Befehls.
    pub title: &'static str,
    pub summary: &'static str,
    pub what: &'static str,
    pub how: &'static str,
    pub example: &'static str,
}

pub static COMMANDS: [CommandInfo; 16] = [
    CommandInfo {
        key: "IF_LET_SOME",
        box_step: 5,
        icon: "📦",
        category: "Rust-Syntax",
        title: "if let Some(x) = ...",
        summary: "Packt einen Wert sicher aus, falls er existiert.",
        what: "Viele Abfragen liefern ein Option: Entweder gibt es einen Wert (Some) oder nichts (None). Mit if let Some(x) = ... wird der Block nur ausgeführt, wenn ein Wert vorhanden ist – so stürzt dein Programm nie ab, wenn z. B. keine Drohne oder kein Asteroid in Reichweite ist.",
        how: "Schreibe links von = das Muster Some(name) und rechts den Ausdruck, der ein Option liefert. Innerhalb der geschweiften Klammern kannst du den ausgepackten Wert unter name benutzen.",
        example: "if let Some(drone) = world.first_drone() {\n    // hier existiert eine Drohne\n}",
    },
    CommandInfo {
        key: "WORLD_FIRST_DRONE",
        box_step: 5,
        icon: "🛸",
        category: "Welt & Sensoren",
        title: "world.first_drone()",
        summary: "Liefert die erste Drohne deiner Flotte.",
        what: "Gibt die erste Drohne aus world.fleet zurück – ideal für den Einstieg, solange du nur eine Drohne steuerst. Das Ergebnis ist ein Option: Gibt es keine Drohne, ist es None.",
        how: "Rufe die Methode ohne Parameter auf und packe das Ergebnis mit if let Some(...) aus. Die Drohne bietet z. B. drone.ship_id und drone.position.",
        example: "if let Some(drone) = world.first_drone() {\n    let id = drone.ship_id;\n}",
    },
    CommandInfo {
        key: "WORLD_CLOSEST_ASTEROID",
        box_step: 5,
        icon: "☄️",
        category: "Welt & Sensoren",
        title: "world.closest_asteroid_with_ore(pos)",
        summary: "Findet den nächsten Asteroiden – bevorzugt mit Erz.",
        what: "world.closest_asteroid_with_ore(pos) liefert den nächstgelegenen Asteroiden, der noch Erz enthält. Daneben gibt es world.closest_asteroid(pos): Das liefert den nächsten Asteroiden egal ob leer oder nicht – dort würde deine Drohne umsonst hinfliegen. Deshalb nimmst du fürs Schürfen die Variante mit _with_ore.",
        how: "Übergib die Position, von der aus gesucht werden soll (meist drone.position). Das Ergebnis ist ein Option, das du mit if let Some(...) auspackst.",
        example: "if let Some(asteroid) = world.closest_asteroid_with_ore(drone.position) {\n    // asteroid.asteroid_id verwenden\n}",
    },
    CommandInfo {
        key: "ACTIONS_MINE",
        box_step: 5,
        icon: "⛏️",
        category: "Schiffs-Befehle",
        title: "actions.mine(ship_id, asteroid_id)",
        summary: "Schickt eine Drohne zum Erzabbau an einen Asteroiden.",
        what: "Befiehlt einer Drohne, zum angegebenen Asteroiden zu fliegen und dort Erz abzubauen. Der Bohrer stoppt automatisch, wenn der Frachtraum voll ist.",
        how: "Übergib zuerst die ship_id der Drohne, dann die asteroid_id des Ziels.",
        example: "actions.mine(drone.ship_id, asteroid.asteroid_id);",
    },
    CommandInfo {
        key: "IF_ELSE",
        box_step: 7,
        icon: "🔀",
        category: "Rust-Syntax",
        title: "if ... { ... } else { ... }",
        summary: "Entscheidet zwischen zwei Möglichkeiten.",
        what: "Ist die Bedingung wahr, läuft der erste Block, sonst der else-Block. So kann deine Drohne je nach Situation unterschiedlich handeln.",
        how: "Schreibe hinter if eine Bedingung (Wahr/Falsch). In den ersten Block kommt, was bei wahr passiert, in den else-Block der Rest.",
        example: "if drone.cargo_is_full() {\n    // Fracht abliefern\n} else {\n    // weiter schürfen\n}",
    },
    CommandInfo {
        key: "DRONE_CARGO_IS_FULL",
        box_step: 7,
        icon: "📊",
        category: "Schiffs-Status",
        title: "drone.cargo_is_full()",
        summary: "Prüft, ob der Frachtraum voll ist.",
        what: "Liefert true, wenn die Drohne ihre maximale Erzmenge geladen hat, sonst false. Eine Tier-1-Drohne trägt höchstens 50 Erz.",
        how: "Rufe die Methode auf einer Drohne auf und nutze das Ergebnis direkt als Bedingung in einem if.",
        example: "if drone.cargo_is_full() {\n    actions.dock(drone.ship_id);\n}",
    },
    CommandInfo {
        key: "ACTIONS_DOCK",
        box_step: 7,
        icon: "🏛️",
        category: "Schiffs-Befehle",
        title: "actions.dock(ship_id)",
        summary: "Schickt eine Drohne zur Heimatbasis zum Andocken.",
        what: "Die Drohne fliegt zur Heimatbasis, liefert dort ihr Erz ab und lädt ihre Batterie wieder auf.",
        how: "Übergib die ship_id der Drohne, die zurückkehren soll.",
        example: "actions.dock(drone.ship_id);",
    },
    CommandInfo {
        key: "WORLD_FLEET_LEN",
        box_step: 9,
        icon: "🔢",
        category: "Welt & Sensoren",
        title: "world.fleet.len()",
        summary: "Zählt die Schiffe deiner Flotte.",
        what: "world.fleet ist die Liste all deiner Schiffe. Mit .len() erfährst du, wie viele es sind.",
        how: "Vergleiche die Anzahl mit einer Zahl, z. B. um nur dann eine neue Drohne zu bauen, wenn du weniger als zwei hast.",
        example: "if world.fleet.len() < 2 {\n    // Platz für eine weitere Drohne\n}",
    },
    CommandInfo {
        key: "WORLD_BASE_STORED_ORE",
        box_step: 9,
        icon: "🪨",
        category: "Welt & Sensoren",
        title: "world.base.stored_ore",
        summary: "Das in der Heimatbasis gelagerte Erz.",
        what: "Ein Feld deiner Heimatbasis, das zeigt, wie viel Erz dort gelagert ist. Mit diesem Erz bezahlst du neue Drohnen und Forschung.",
        how: "Lies das Feld ohne Klammern aus und vergleiche es mit den Kosten, z. B. mit >= 50.",
        example: "if world.base.stored_ore >= 50 {\n    // genug Erz vorhanden\n}",
    },
    CommandInfo {
        key: "ACTIONS_BUILD_DRONE_1",
        box_step: 9,
        icon: "🔧",
        category: "Basis-Befehle",
        title: "actions.build_drone(1)",
        summary: "Baut eine Stufe-1-Drohne für 50 Erz.",
        what: "Gibt an der Heimatbasis den Bau einer Stufe-1-Drohne in Auftrag. Sie kostet 50 Erz und trägt 50 Erz Fracht. Mehr Drohnen bedeuten schnellere Rohstoffförderung.",
        how: "Übergib die Stufe 1. Prüfe vorher, ob genug Erz gelagert ist.",
        example: "if world.fleet.len() < 2 && world.base.stored_ore >= 50 {\n    actions.build_drone(1);\n}",
    },
    CommandInfo {
        key: "FOR_IN_FLEET",
        box_step: 11,
        icon: "🔁",
        category: "Rust-Syntax",
        title: "for drone in world.fleet { ... }",
        summary: "Wiederholt Befehle für jedes Schiff der Flotte.",
        what: "Eine for-Schleife geht die Liste world.fleet Schiff für Schiff durch. In jedem Durchlauf steht dir ein Schiff unter dem Namen drone zur Verfügung – so steuerst du beliebig viele Drohnen mit demselben Code.",
        how: "Schreibe for, einen Namen für das aktuelle Schiff, in und die Liste. Alles in den geschweiften Klammern wird für jedes Schiff ausgeführt.",
        example: "for drone in world.fleet {\n    if drone.cargo_is_full() {\n        actions.dock(drone.ship_id);\n    }\n}",
    },
    CommandInfo {
        key: "WORLD_ACTIVE_RIDDLE",
        box_step: 13,
        icon: "🧩",
        category: "Forschung",
        title: "world.active_riddle",
        summary: "Das aktuell aktive Forschungsrätsel.",
        what: "Enthält das Rätsel, das du angefordert hast – oder None, wenn gerade keins aktiv ist. Mit is_none() prüfst du, ob du ein neues anfordern darfst. Bei einem Sortier-Rätsel erhältst du die Zahlenreihe als RiddleTask::Sort { numbers }.",
        how: "Prüfe mit is_none(), ob kein Rätsel aktiv ist. Zum Auslesen packst du es mit if let Some(RiddleTask::Sort { numbers }) aus.",
        example: "if let Some(RiddleTask::Sort { numbers }) = world.active_riddle {\n    // numbers ist die Zahlenreihe\n}",
    },
    CommandInfo {
        key: "ACTIONS_REQUEST_RESEARCH",
        box_step: 13,
        icon: "🧪",
        category: "Forschung",
        title: "actions.request_research(track)",
        summary: "Fordert ein Forschungsrätsel an.",
        what: "Startet Forschung für einen Sensor-Zweig. Das kostet Erz aus deiner Basis; mit UpgradeTrack::DroneScanner verbesserst du die Reichweite deiner Drohnen-Scanner.",
        how: "Übergib den gewünschten UpgradeTrack. Fordere nur an, wenn noch kein Rätsel aktiv ist.",
        example: "if world.active_riddle.is_none() {\n    actions.request_research(UpgradeTrack::DroneScanner);\n}",
    },
    CommandInfo {
        key: "CLONE_AND_SORT",
        box_step: 13,
        icon: "📑",
        category: "Rust-Syntax",
        title: "let mut sorted = numbers.clone(); sorted.sort();",
        summary: "Legt eine veränderbare Kopie an und sortiert sie.",
        what: ".clone() erzeugt eine Kopie der Liste, damit das Original unberührt bleibt. let mut macht die Variable veränderbar, und .sort() ordnet die Zahlen aufsteigend.",
        how: "Kopiere die Liste in eine mut-Variable und rufe darauf sort() auf. Danach ist die Kopie sortiert.",
        example: "let mut sorted = numbers.clone();\nsorted.sort();",
    },
    CommandInfo {
        key: "ACTIONS_SOLVE_RIDDLE",
        box_step: 13,
        icon: "✅",
        category: "Forschung",
        title: "actions.solve_riddle(solution)",
        summary: "Reicht die Lösung des Rätsels ein.",
        what: "Schickt deine berechnete Lösung an die Basis. Ist sie richtig, steigt dein Forschungsfortschritt. Bei falschen Antworten droht ein Durchbrenn-Risiko des Moduls.",
        how: "Verpacke deine sortierte Liste in RiddleSolution::Sort(...) und übergib sie.",
        example: "actions.solve_riddle(RiddleSolution::Sort(sorted));",
    },
    CommandInfo {
        key: "ACTIONS_BUILD_DRONE_2",
        box_step: 15,
        icon: "🚀",
        category: "Basis-Befehle",
        title: "actions.build_drone(2)",
        summary: "Baut eine schwere Stufe-2-Drohne für 150 Erz.",
        what: "Gibt eine schwere Stufe-2-Drohne in Auftrag. Sie kostet 150 Erz, hat dreifache Frachtkapazität (150 Erz) und eine größere Batterie.",
        how: "Übergib die Stufe 2 und achte darauf, dass genug Erz in der Basis gelagert ist.",
        example: "if world.base.stored_ore >= 150 {\n    actions.build_drone(2);\n}",
    },
];

pub fn find_command(key: &str) -> Option<&'static CommandInfo> {
    COMMANDS.iter().find(|c| c.key == key)
}

/// Befehle einer Lootbox in der festgelegten Reihenfolge.
pub fn commands_for_box(step: u8) -> Vec<&'static CommandInfo> {
    COMMANDS.iter().filter(|c| c.box_step == step).collect()
}

/// Befehle, die in einer Tutorial-Stufe (Coding-Schritt) neu zur Verfügung stehen: die Lootbox direkt davor.
pub fn commands_for_coding_step(step: u8) -> Vec<&'static CommandInfo> {
    if step > 0 {
        commands_for_box(step - 1)
    } else {
        Vec::new()
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

    let current = selected().and_then(|k| find_command(&k));
    let current_idx = current.and_then(|c| unlocked_list.iter().position(|x| x.key == c.key));
    let prev_list = unlocked_list.clone();
    let next_list = unlocked_list.clone();

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
                        style: "padding: 24px; display: grid; grid-template-columns: repeat(auto-fill, minmax(250px, 1fr)); gap: 12px;",
                        for cmd in COMMANDS.iter() {
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
            if !opened() || next.is_none() {
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
