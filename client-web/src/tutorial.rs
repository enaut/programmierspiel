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
            story: "Deine Heimatbasis hat einen Bordcomputer. Du programmierst ihn in Rust und startest deinen Bot im Zed-Editor.",
            subtasks: vec![
                SubTask {
                    num: "1",
                    text: "Öffne uhlaunch auf dem Desktop und starte 'programmierspiel'. Der Zed-Editor öffnet sich.",
                    code_hint: None,
                },
                SubTask {
                    num: "2",
                    text: "Öffne links im Ordner src die Datei main.rs. Klicke neben fn main() auf den Run-Button ▶, um den Bot zu starten.",
                    code_hint: Some("▶ fn main()"),
                },
                SubTask {
                    num: "3",
                    text: "Suche im Terminal unten nach 'Verbindung erfolgreich!' und der darunter ausgegebenen Identität. Bestätige anschließend hier, dass der Bot gestartet ist.",
                    code_hint: None,
                },
            ],
        },
        4 => StepInfo {
            title: "Bot mit der Heimatbasis koppeln",
            story: "Die Identität ist die Adresse deines Bots; mit ihr erlaubst du ihm die Steuerung deiner Basis. Ein Token ist dagegen ein geheimer Zugangsschlüssel: Gib ihn niemals als Identität ein oder an andere weiter. Beim ersten Start speichert der Bot seinen Token selbst in config.toml.",
            subtasks: vec![
                SubTask {
                    num: "1",
                    text: "Kopiere im Zed-Terminal die 64-stellige Zeichenfolge hinter 'Identität:' – nicht den Token.",
                    code_hint: None,
                },
                SubTask {
                    num: "2",
                    text: "Öffne die Heimatbasis und füge die Identität unter 'Heimatbasis-AI Steuerung' ein. Klicke auf 'Bot binden'.",
                    code_hint: None,
                },
                SubTask {
                    num: "3",
                    text: "Lass den Bot laufen. Die Mission wartet automatisch auf die vom Server bestätigte Kopplung.",
                    code_hint: None,
                },
            ],
        },
        5 => StepInfo {
            title: "🎁 Befehls-Lootbox #1: Deine erste Drohne",
            story: "Dein Bot ist verbunden! In der Heimatbasis warten if let Some und world.first_drone() auf dich.",
            subtasks: vec![
                SubTask {
                    num: "1",
                    text: "Öffne deine Heimatbasis und klicke auf die leuchtende Befehls-Lootbox.",
                    code_hint: None,
                },
                SubTask {
                    num: "2",
                    text: "Lies die Karten und sammle beide Befehle ein.",
                    code_hint: None,
                },
            ],
        },
        7 => StepInfo {
            title: "🎁 Befehls-Lootbox #2: Asteroiden finden",
            story: "Mit dem nächsten Befehl findest du bekannte Asteroiden, die noch Erz enthalten.",
            subtasks: vec![
                SubTask {
                    num: "1",
                    text: "Öffne die Heimatbasis und die leuchtende Befehls-Lootbox.",
                    code_hint: None,
                },
                SubTask {
                    num: "2",
                    text: "Lies die Karte und sammle den Befehl ein.",
                    code_hint: None,
                },
            ],
        },
        9 => StepInfo {
            title: "🎁 Befehls-Lootbox #3: Erz schürfen",
            story: "Jetzt lernst du den Befehl, mit dem deine Drohne einen Asteroiden abbaut.",
            subtasks: vec![
                SubTask {
                    num: "1",
                    text: "Öffne die Heimatbasis und die leuchtende Befehls-Lootbox.",
                    code_hint: None,
                },
                SubTask {
                    num: "2",
                    text: "Lies die Karte und sammle den Befehl ein.",
                    code_hint: None,
                },
            ],
        },
        11 => StepInfo {
            title: "🎁 Befehls-Lootbox #4: Laderaum & Andocken",
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
        13 => StepInfo {
            title: "🎁 Befehls-Lootbox #5: Flotten-Konstruktion",
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
        15 => StepInfo {
            title: "🎁 Befehls-Lootbox #6: Schwarm-Schleife",
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
        17 => StepInfo {
            title: "🎁 Befehls-Lootbox #7: Telemetrie & Algorithmen",
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
        19 => StepInfo {
            title: "🎁 Befehls-Lootbox #8: Schwere Flotte",
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
            title: "Wo ist deine Drohne?",
            story: "In main.rs wird tick etwa zehnmal pro Sekunde aufgerufen. world enthält den aktuellen Weltzustand, actions sammelt deine Befehle. Schreibe deinen Code in tick: Nur wenn eine Drohne vorhanden ist, kannst du ihre Position auslesen.",
            subtasks: vec![
                SubTask {
                    num: "1",
                    text: "Wähle die erste Drohne mit world.first_drone() und if let Some aus. Bei None wird der Block übersprungen.",
                    code_hint: Some("if let Some(drone) = world.first_drone() { ... }"),
                },
                SubTask {
                    num: "2",
                    text: "Setze im Block selbst ein println! zusammen: Gib drone.position.x und drone.position.y mit je {:.0} ohne Nachkommastellen aus. {:.0} rundet die angezeigte Zahl.",
                    code_hint: Some("println!(\"... {:.0} ... {:.0} ...\", ..., ...);"),
                },
                SubTask {
                    num: "3",
                    text: "Speichere, starte den Bot neu und übertrage die beiden ausgegebenen ganzen Zahlen in die Felder unten.",
                    code_hint: None,
                },
            ],
        },
        8 => StepInfo {
            title: "Die Asteroiden-ID ermitteln",
            story: "Deine Sensoren identifizieren Asteroiden in Reichweite der Basis oder deiner Drohnen. Jede so identifizierte Asteroiden-ID ist gültig; die Suche wählt davon einen passenden Asteroiden mit Erz.",
            subtasks: vec![
                SubTask {
                    num: "1",
                    text: "Suche innerhalb des Drohnen-Blocks mit world.closest_asteroid_with_ore(drone.position). Packe das mögliche Ergebnis mit if let Some aus; bei None wurde kein passender Asteroid gefunden.",
                    code_hint: Some(
                        "if let Some(asteroid) = world.closest_asteroid_with_ore(...) { ... }",
                    ),
                },
                SubTask {
                    num: "2",
                    text: "Gib im Asteroiden-Block asteroid.asteroid_id mit println! im Terminal aus. Setze die Ausgabe selbst zusammen.",
                    code_hint: Some("println!(\"ID: {}\", ...);"),
                },
                SubTask {
                    num: "3",
                    text: "Speichere, starte den Bot neu und gib die ausgegebene ID unten ein.",
                    code_hint: None,
                },
            ],
        },
        10 => StepInfo {
            title: "Der erste Schürfzug",
            story: "Deine Drohne und ein Asteroid mit Erz sind gefunden. Gib den Abbau-Befehl in tick: Die Drohne fliegt erst zum Ziel und schürft dort. Die Mission endet, sobald der Server erfolgreichen Abbau registriert.",
            subtasks: vec![
                SubTask {
                    num: "1",
                    text: "Verwende die gefundenen Werte für Drohne und Asteroid in den verschachtelten if-let-Blöcken.",
                    code_hint: None,
                },
                SubTask {
                    num: "2",
                    text: "Erteile dort den Befehl mit Drohnen-ID und Asteroiden-ID.",
                    code_hint: Some("actions.mine(drone.ship_id, asteroid.asteroid_id);"),
                },
                SubTask {
                    num: "3",
                    text: "Speichere, starte den Bot neu und beobachte den Abbau im Spiel.",
                    code_hint: None,
                },
            ],
        },
        12 => StepInfo {
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
        14 => StepInfo {
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
        16 => StepInfo {
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
        18 => StepInfo {
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
    let mut show_celebration = use_signal(|| false);
    let mut coordinate_x = use_signal(String::new);
    let mut coordinate_y = use_signal(String::new);
    let mut asteroid_id = use_signal(String::new);
    let mut input_error = use_signal(|| Option::<String>::None);
    let mut submission = use_signal_sync(SubmissionState::default);

    // Audio & Feier-Trigger bei Stufenaufstieg
    use_effect(use_reactive((&step,), move |(step,)| {
        let previous = last_seen_step();
        if step != previous {
            last_seen_step.set(step);
            coordinate_x.set(String::new());
            coordinate_y.set(String::new());
            asteroid_id.set(String::new());
            input_error.set(None);
            submission.write().active = None;
            if step > previous {
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

    let step_info = get_step_info(step);
    let coding_commands = commands_for_coding_step(step);
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
                span { "🛸 Mission: Stufe {step}/20" }
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
                            "STUFE {step} / 20"
                        }
                        if step >= 20 {
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
                        style: "height: 100%; width: {(step.min(20) as f32 / 20.0) * 100.0}%; background: linear-gradient(90deg, #00b4d8, #00e5ff); border-radius: 3px; transition: width 0.5s ease-out;"
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

                if step == 4 {
                    button {
                        style: "background: #0284c7; color: #fff; border: none; padding: 10px 16px; border-radius: 8px; font-weight: 700; cursor: pointer;",
                        onclick: move |_| on_open_base.call(()),
                        "🏛️ Heimatbasis öffnen und Bot koppeln"
                    }
                }

                if step == 6 || step == 8 {
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

                // Feste Fehlerhilfe-Checkliste für die Coding-Missionen
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
                        "Ich habe Zed geöffnet und den Bot gestartet ➔ Weiter zu Stufe 4"
                    }
                } else if step >= 20 {
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
            (18, "Sensor-Upgrade durch Algorithmen"),
            (20, "Tutorial abgeschlossen – Freies Spiel!"),
        ];
        for (step, title) in titles {
            assert_eq!(get_step_info(step).title, title);
        }
        for step in [5, 7, 9, 11, 13, 15, 17, 19] {
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
