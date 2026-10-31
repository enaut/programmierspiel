use crate::audio::SoundEffects;
use crate::commands::{commands_for_coding_step, is_loot_box_step};
use crate::module_bindings::{GameEvent, Ship};
use dioxus::prelude::*;

pub struct SubTask {
    pub num: &'static str,
    pub text: &'static str,
    pub code_hint: Option<&'static str>,
}

pub struct StepInfo {
    pub title: &'static str,
    pub story: &'static str,
    pub subtasks: Vec<SubTask>,
}

pub fn get_step_info(step: u8) -> StepInfo {
    match step {
        0 => StepInfo {
            title: "Willkommen an Bord, Commander!",
            story: "Im Asteroidengürtel wimmelt es von automatischen Transportern und fremden Stationen. Um deine Schiffe auf den Radarsystemen der Mega-Station eindeutig zu identifizieren, benötigt deine Flotte deinen Pilotenausweis und eigene Signal-Farben.",
            subtasks: vec![
                SubTask {
                    num: "1",
                    text: "Wähle mit den Farbreglern deine Primär- und Sekundärfarbe für dein Raumschiff aus.",
                    code_hint: None,
                },
                SubTask {
                    num: "2",
                    text: "Prüfe die Vorschau deines Drohnen-Raumschiffs im Hangar.",
                    code_hint: None,
                },
                SubTask {
                    num: "3",
                    text: "Klicke auf 'Pilotenausweis erstellen & starten'.",
                    code_hint: None,
                },
            ],
        },
        1 => StepInfo {
            title: "Der Jungfernflug (Manuelle Steuerung)",
            story: "Dein Bauschiff (Tier 0) startet nah herangezoomt an der Mega-Station bei (0, 0). Bedienung der Karte: Mit dem Mausrad (Scrollen) zoomst du rein und raus, und durch Klicken und Halten mit der linken Maustaste verschiebst du die Karte. Wichtig: Du kannst deine Basis nur außerhalb des roten bzw. grünen Sicherheitskreises um die Mega-Station errichten!",
            subtasks: vec![
                SubTask {
                    num: "1",
                    text: "Scrolle mit dem Mausrad heraus, um die Umgebung zu überblicken (Linksklick gedrückt halten bewegt die Karte).",
                    code_hint: None,
                },
                SubTask {
                    num: "2",
                    text: "Klicke kurz mit der linken Maustaste auf einen Zielpunkt außerhalb des roten Kreises, um dein Bauschiff dorthin zu steuern, bis der Kreis grün wird (Abstand ≥ 250).",
                    code_hint: None,
                },
                SubTask {
                    num: "3",
                    text: "Achte rechts im Panel bei Position & Geschwindigkeit auf den Asteroiden-Scanradius: Suche einen Platz, an dem mindestens 400 Erz in Reichweite sind!",
                    code_hint: None,
                },
            ],
        },
        2 => StepInfo {
            title: "Das Fundament (Heimatbasis errichten)",
            story: "Hervorragend gewählt, Commander! Du befindest dich außerhalb des grünen Sicherheitskreises der Mega-Station und hast genügend Erz im Scanradius. Durch die Aktivierung der Montage-Sequenz entfaltet sich dein Bauschiff zu deiner Heimatbasis.",
            subtasks: vec![
                SubTask {
                    num: "1",
                    text: "Vergewissere dich, dass du außerhalb des roten/grünen Kreises stehst (Abstand ≥ 250) und rechts im Panel mindestens 400 Erz im Scanradius angezeigt werden.",
                    code_hint: None,
                },
                SubTask {
                    num: "2",
                    text: "Klicke unten rechts im Seitenpanel auf den leuchtenden Button '🏛️ Heimatbasis hier errichten'.",
                    code_hint: None,
                },
            ],
        },
        3 => StepInfo {
            title: "Die Brücke öffnen (uhlaunch & Zed)",
            story: "Deine Heimatbasis besitzt einen Bordcomputer, den du im Zed-Editor mit Rust-Code programmierst. Damit sich dein Programm mit deiner Station verbinden kann, musst du dein persönliches Token aus dem Dashboard in die Datei config.toml im Zed-Editor kopieren.",
            subtasks: vec![
                SubTask {
                    num: "1",
                    text: "Öffne auf deinem Desktop die Anwendung uhlaunch und starte 'programmierspiel'. Der Zed-Editor öffnet sich daraufhin automatisch.",
                    code_hint: None,
                },
                SubTask {
                    num: "2",
                    text: "Suche hier im Web-Dashboard auf der rechten Seite das Feld 'DEIN PERSÖNLICHER API-TOKEN' und klicke auf den Kopieren-Button (📋).",
                    code_hint: None,
                },
                SubTask {
                    num: "3",
                    text: "Wechsle in den Zed-Editor, suche links in der Dateiliste die Datei 'config.toml' und öffne sie. Füge dein kopiertes Token bei 'auth_token' genau zwischen den Anführungszeichen (\"...\") ein und speichere mit Strg + S.",
                    code_hint: Some("auth_token = \"<dein_kopiertes_token>\""),
                },
            ],
        },
        4 => StepInfo {
            title: "Der erste Funkkontakt (Run-Button in Zed)",
            story: "In src/main.rs ist bereits das Grundgerüst deiner Station vorbereitet. Wenn du das Programm über den Run-Button in Zed startest, öffnet sich unten das Terminal und dein Bot verbindet sich direkt mit deiner Raumstation.",
            subtasks: vec![
                SubTask {
                    num: "1",
                    text: "Öffne im Zed-Editor links in der Dateiliste im Ordner 'src' die Datei 'main.rs'.",
                    code_hint: None,
                },
                SubTask {
                    num: "2",
                    text: "Suche im Code bei 'fn main()' links neben der Zeilennummer den kleinen Run-Button (▶) und klicke darauf.",
                    code_hint: Some("▶ fn main()"),
                },
                SubTask {
                    num: "3",
                    text: "Unten im Zed-Editor öffnet sich das Terminal: Suche dort in der Textausgabe nach 'Verbindung erfolgreich!'.",
                    code_hint: Some("✅ [SPACE BOT] Verbindung erfolgreich!"),
                },
            ],
        },
        5 => StepInfo {
            title: "🎁 Befehls-Lootbox #1: Bergungs-Protokolle",
            story: "Dein Bot läuft! Zur Belohnung wartet in deiner Heimatbasis eine Lootbox mit den ersten Befehlen, um Erz zu schürfen.",
            subtasks: vec![
                SubTask {
                    num: "1",
                    text: "Öffne deine Heimatbasis (Button '🏛️ Heimatbasis öffnen' oder oben in der Leiste).",
                    code_hint: None,
                },
                SubTask {
                    num: "2",
                    text: "Klicke auf die leuchtende Befehls-Lootbox und sammle jede Karte mit einem Klick ein.",
                    code_hint: None,
                },
            ],
        },
        7 => StepInfo {
            title: "🎁 Befehls-Lootbox #2: Laderaum & Andocken",
            story: "Erstes Erz im Frachtraum! In deiner Heimatbasis wartet eine Lootbox mit Befehlen zum Verzweigen und Andocken.",
            subtasks: vec![
                SubTask {
                    num: "1",
                    text: "Öffne deine Heimatbasis (Button '🏛️ Heimatbasis öffnen' oder oben in der Leiste).",
                    code_hint: None,
                },
                SubTask {
                    num: "2",
                    text: "Klicke auf die leuchtende Befehls-Lootbox und sammle jede Karte mit einem Klick ein.",
                    code_hint: None,
                },
            ],
        },
        9 => StepInfo {
            title: "🎁 Befehls-Lootbox #3: Flotten-Konstruktion",
            story: "Deine Lieferung war erfolgreich! Öffne die Lootbox in der Heimatbasis, um zu lernen, wie du neue Drohnen baust.",
            subtasks: vec![
                SubTask {
                    num: "1",
                    text: "Öffne deine Heimatbasis (Button '🏛️ Heimatbasis öffnen' oder oben in der Leiste).",
                    code_hint: None,
                },
                SubTask {
                    num: "2",
                    text: "Klicke auf die leuchtende Befehls-Lootbox und sammle jede Karte mit einem Klick ein.",
                    code_hint: None,
                },
            ],
        },
        11 => StepInfo {
            title: "🎁 Befehls-Lootbox #4: Schwarm-Schleife",
            story: "Zwei Drohnen im Einsatz! In der Heimatbasis wartet die Schleife, mit der du die ganze Flotte steuerst.",
            subtasks: vec![
                SubTask {
                    num: "1",
                    text: "Öffne deine Heimatbasis (Button '🏛️ Heimatbasis öffnen' oder oben in der Leiste).",
                    code_hint: None,
                },
                SubTask {
                    num: "2",
                    text: "Klicke auf die leuchtende Befehls-Lootbox und sammle jede Karte mit einem Klick ein.",
                    code_hint: None,
                },
            ],
        },
        13 => StepInfo {
            title: "🎁 Befehls-Lootbox #5: Telemetrie & Algorithmen",
            story: "Deine Flotte arbeitet! Öffne die Lootbox in der Heimatbasis für die Befehle der Sensorforschung.",
            subtasks: vec![
                SubTask {
                    num: "1",
                    text: "Öffne deine Heimatbasis (Button '🏛️ Heimatbasis öffnen' oder oben in der Leiste).",
                    code_hint: None,
                },
                SubTask {
                    num: "2",
                    text: "Klicke auf die leuchtende Befehls-Lootbox und sammle jede Karte mit einem Klick ein.",
                    code_hint: None,
                },
            ],
        },
        15 => StepInfo {
            title: "🎁 Befehls-Lootbox #6: Schwere Flotte",
            story: "Forschung abgeschlossen! Die letzte Lootbox verrät dir, wie du schwere Stufe-2-Drohnen baust.",
            subtasks: vec![
                SubTask {
                    num: "1",
                    text: "Öffne deine Heimatbasis (Button '🏛️ Heimatbasis öffnen' oder oben in der Leiste).",
                    code_hint: None,
                },
                SubTask {
                    num: "2",
                    text: "Klicke auf die leuchtende Befehls-Lootbox und sammle jede Karte mit einem Klick ein.",
                    code_hint: None,
                },
            ],
        },
        6 => StepInfo {
            title: "Der erste Schürfzug (Drohne steuern & abbauen)",
            story: "Deine Drohne wartet auf Instruktionen. Im Weltraum erfassen Sensoren alle Objekte in Reichweite. Nutze world.closest_asteroid_with_ore(...), um einen erzhaltigen Asteroiden zu finden, und actions.mine(...), um den Abbau-Befehl zu erteilen.",
            subtasks: vec![
                SubTask {
                    num: "1",
                    text: "Wähle deine Drohne mit world.first_drone() aus:",
                    code_hint: Some("if let Some(drone) = world.first_drone() { ... }"),
                },
                SubTask {
                    num: "2",
                    text: "Finde den nächsten erzhaltigen Asteroiden:",
                    code_hint: Some(
                        "if let Some(asteroid) = world.closest_asteroid_with_ore(drone.position) { ... }",
                    ),
                },
                SubTask {
                    num: "3",
                    text: "Erteile den Abbau-Befehl:",
                    code_hint: Some("actions.mine(drone.ship_id, asteroid.asteroid_id);"),
                },
            ],
        },
        8 => StepInfo {
            title: "Frachtraum voll! Kurs Heimatbasis",
            story: "Deine Drohne kann nur 50 Einheiten Erz tragen. Ist der Frachtraum voll, stoppt der Bohrer automatisch. Damit das Erz verarbeitet werden kann, muss die Drohne an der Heimatbasis andocken. Ziel: Sammle mindestens 50 Erz an deiner Basis!",
            subtasks: vec![
                SubTask {
                    num: "1",
                    text: "Prüfe, ob der Laderaum der Drohne voll ist:",
                    code_hint: Some("if drone.cargo_is_full() { ... } else { ... }"),
                },
                SubTask {
                    num: "2",
                    text: "Befiehl bei voller Fracht das Andocken:",
                    code_hint: Some("actions.dock(drone.ship_id);"),
                },
                SubTask {
                    num: "3",
                    text: "Beobachte das Radar, bis die Basis mindestens 50 Erz gelagert hat.",
                    code_hint: None,
                },
            ],
        },
        10 => StepInfo {
            title: "Flottenausbau (Zweite Drohne bauen)",
            story: "Mit 50 Einheiten Erz im Lager verfügt deine Basis über genügend Rohstoffe, um ein weiteres Schiff der Stufe 1 in Auftrag zu geben. Mehr Drohnen bedeuten schnellere Rohstoffförderung!",
            subtasks: vec![
                SubTask {
                    num: "1",
                    text: "Prüfe, ob du weniger als 2 Drohnen und mindestens 50 Erz hast:",
                    code_hint: Some(
                        "if world.fleet.len() < 2 && world.base.stored_ore >= 50 { ... }",
                    ),
                },
                SubTask {
                    num: "2",
                    text: "Gib den Bau einer Stufe-1-Drohne in Auftrag:",
                    code_hint: Some("actions.build_drone(1);"),
                },
            ],
        },
        12 => StepInfo {
            title: "Multitasking für Fortgeschrittene (Flottenschleife)",
            story: "Bisher hat dein Code nur die erste Drohne mit world.first_drone() gesteuert. In Rust nutzen wir eine for-Schleife, um jedem Schiff in world.fleet automatisch denselben Ablauf zuzuweisen.",
            subtasks: vec![
                SubTask {
                    num: "1",
                    text: "Ersetze world.first_drone() durch eine for-Schleife über alle Schiffe:",
                    code_hint: Some("for drone in world.fleet { ... }"),
                },
                SubTask {
                    num: "2",
                    text: "Befiehl jedem Schiff: Wenn voll, docken; sonst erzhaltigen Asteroiden abbauen.",
                    code_hint: None,
                },
                SubTask {
                    num: "3",
                    text: "Beobachte, wie sich beide Drohnen unabhängig voneinander bewegen.",
                    code_hint: None,
                },
            ],
        },
        14 => StepInfo {
            title: "Sensor-Upgrade durch Algorithmen",
            story: "Die Sensoren deiner Heimatbasis erfassen Asteroiden aktuell nur auf kurze Distanz. Um weiter entfernte Erzvorkommen zu entdecken, musst du das Sensorsystem kalibrieren, indem du Messreihen sortierst.",
            subtasks: vec![
                SubTask {
                    num: "1",
                    text: "Fordere ein Forschungsrätsel für den Drohnen-Scanner an, falls noch keines aktiv ist:",
                    code_hint: Some(
                        "if world.active_riddle.is_none() { actions.request_research(UpgradeTrack::DroneScanner); }",
                    ),
                },
                SubTask {
                    num: "2",
                    text: "Prüfe auf ein aktives Sortier-Rätsel, klone die Zahlenreihe und sortiere sie aufsteigend:",
                    code_hint: Some(
                        "if let Some(RiddleTask::Sort { numbers }) = world.active_riddle { let mut sorted = numbers.clone(); sorted.sort(); ... }",
                    ),
                },
                SubTask {
                    num: "3",
                    text: "Reiche die sortierte Liste innerhalb des if-Blocks mit actions.solve_riddle(...) ein:",
                    code_hint: Some("actions.solve_riddle(RiddleSolution::Sort(sorted));"),
                },
            ],
        },
        _ => StepInfo {
            title: "Tutorial abgeschlossen – Freies Spiel!",
            story: "Fantastische Arbeit, Commander! Du beherrschst alle Grundlagen der interstellaren Programmierung: Navigation, Abbau, Multitasking-Schleifen und algorithmische Forschung.",
            subtasks: vec![
                SubTask {
                    num: "1",
                    text: "Baue schwere Stufe-2-Drohnen mit 3-facher Frachtkapazität.",
                    code_hint: Some("actions.build_drone(2);"),
                },
                SubTask {
                    num: "2",
                    text: "Verteile deine Drohnen gezielt auf unterschiedliche Asteroiden.",
                    code_hint: None,
                },
                SubTask {
                    num: "3",
                    text: "Erforsche den Radar-Horizont und führe die Rangliste auf dem Klassenbeamer an!",
                    code_hint: None,
                },
            ],
        },
    }
}

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
    let mut last_seen_step = use_signal(|| step);
    let mut show_celebration = use_signal(|| false);

    // Audio & Feier-Trigger bei Stufenaufstieg
    use_effect(move || {
        if step > last_seen_step() {
            last_seen_step.set(step);
            show_celebration.set(true);
            SoundEffects::play_success_ping();
        }
    });

    let step_info = get_step_info(step);
    let coding_commands = commands_for_coding_step(step);

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
                span { "🛸 Mission: Stufe {step}/16" }
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
                    style: "display: flex; justify-content: space-between; align-items: center;",
                    div {
                        style: "display: flex; align-items: center; gap: 8px;",
                        span {
                            style: "background: rgba(0, 229, 255, 0.15); color: #00e5ff; border: 1px solid rgba(0, 229, 255, 0.3); padding: 3px 8px; border-radius: 6px; font-size: 11px; font-weight: 800; letter-spacing: 0.5px;",
                            "STUFE {step} / 16"
                        }
                        if step == 16 {
                            span { style: "font-size: 11px; color: #ffd700; font-weight: 700;", "🏆 MEISTER" }
                        }
                    }
                    button {
                        style: "background: rgba(255, 255, 255, 0.06); border: 1px solid rgba(255, 255, 255, 0.15); color: #94a3b8; border-radius: 6px; width: 28px; height: 28px; cursor: pointer; display: flex; align-items: center; justify-content: center; font-size: 12px; transition: all 0.2s;",
                        onclick: move |_| is_collapsed.set(true),
                        "◀"
                    }
                }
                // Fortschrittsbalken
                div {
                    style: "width: 100%; height: 6px; background: #0f172a; border-radius: 3px; overflow: hidden;",
                    div {
                        style: "height: 100%; width: {(step as f32 / 16.0) * 100.0}%; background: linear-gradient(90deg, #00b4d8, #00e5ff); border-radius: 3px; transition: width 0.5s ease-out;"
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

                // Lootbox-Shortcut zur Heimatbasis
                if is_loot_box_step(step) {
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

                // Feste Fehlerhilfe-Checkliste (erst ab Stufe 5, wenn in Zed eigener Code geschrieben wird)
                if step >= 6 {
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
                if step == 3 {
                    button {
                        style: "width: 100%; background: linear-gradient(90deg, #0284c7, #00e5ff); color: #020617; border: none; padding: 10px 16px; border-radius: 8px; font-weight: 800; font-size: 13px; cursor: pointer; transition: all 0.2s; box-shadow: 0 4px 14px rgba(0, 229, 255, 0.3);",
                        onclick: move |_| on_advance.call(3),
                        "Ich habe Zed geöffnet ➔ Weiter zu Schritt 4"
                    }
                } else if step == 16 {
                    div {
                        style: "color: #ffd700; font-size: 12px; font-weight: 700; display: flex; align-items: center; gap: 6px;",
                        span { "🏆" }
                        span { "Tutorial erfolgreich gemeistert!" }
                    }
                } else {
                    div {
                        style: "color: #64748b; font-size: 11px; font-weight: 600; display: flex; align-items: center; gap: 6px;",
                        span { style: "animation: spin 2s linear infinite;", "⏳" }
                        span { "Warte auf Erfüllung der Bedingung im Spiel..." }
                    }
                }
            }
        }
    }
}
