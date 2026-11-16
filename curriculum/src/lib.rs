//! `curriculum`: Centralized declarative catalog of tutorial steps, quests, and command reference cards
//! for the `programmierspiel` workspace.
//!
//! Zero external dependencies (`no_std` compatible). Used by `server`, `client-web`, `admin`, and `space_bot_sdk`.

#![no_std]

/// Maximum tutorial step index (Free Play).
pub const FINAL_STEP: u8 = 37;

/// Tutorial steps at which a command loot box is waiting in the Heimatbasis.
pub const LOOT_BOX_STEPS: [u8; 16] = [5, 7, 9, 11, 13, 15, 17, 19, 21, 23, 25, 27, 29, 31, 33, 35];

/// Returns `true` if a loot box is waiting at this tutorial step.
pub const fn is_loot_box_step(step: u8) -> bool {
    matches!(
        step,
        5 | 7 | 9 | 11 | 13 | 15 | 17 | 19 | 21 | 23 | 25 | 27 | 29 | 31 | 33 | 35
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepKind {
    /// Initial onboarding/setup (steps 0..=4).
    Onboarding,
    /// Command lootbox in home base (odd steps 5..=35).
    LootBox { number: u8 },
    /// Coding quest exercising unlocked commands (even steps 6..=36).
    Coding,
    /// Tutorial complete; Free-Play sandbox and remaining SDK catalog unlocked (step 37).
    FreePlay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationMode {
    /// Step advanced manually by learner (e.g. Step 0, Step 3).
    ManualAdvance,
    /// Step advanced on claiming all loot box cards (odd steps 5..=35).
    LootBoxClaim,
    /// Verified via interactive coordinate input (Step 6).
    CoordinateQuiz,
    /// Verified via interactive asteroid ID input (Step 8).
    AsteroidIdQuiz,
    /// Verified by server-side game events (mining, docking, building, navigation, research).
    ServerEvent,
    /// Sandbox free play; no progression verification.
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SubTask {
    pub num: &'static str,
    pub text: &'static str,
    pub code_hint: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Command {
    pub key: &'static str,
    pub box_step: u8,
    pub icon: &'static str,
    pub category: &'static str,
    pub title: &'static str,
    pub summary: &'static str,
    pub what: &'static str,
    pub how: &'static str,
    pub example: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Step {
    pub index: u8,
    pub kind: StepKind,
    pub verification: VerificationMode,
    pub title: &'static str,
    pub short_title: &'static str,
    pub admin_label: &'static str,
    pub story: &'static str,
    pub subtasks: &'static [SubTask],
    pub unlocked_keys: &'static [&'static str],
    pub reference_keys: &'static [&'static str],
}

impl Step {
    pub const fn is_loot_box(&self) -> bool {
        matches!(self.kind, StepKind::LootBox { .. })
    }

    pub const fn is_free_play(&self) -> bool {
        matches!(self.kind, StepKind::FreePlay)
    }

    /// Full command cards unlocked at this step.
    pub fn unlocked_commands(&self) -> impl Iterator<Item = &'static Command> {
        self.unlocked_keys.iter().filter_map(|k| command(k))
    }

    /// Command cards referenced/introduced for this mission.
    pub fn reference_commands(&self) -> impl Iterator<Item = &'static Command> {
        self.reference_keys.iter().filter_map(|k| command(k))
    }

    /// Command keys that should be unlocked when catching up a player to this step.
    /// Unlocks all loot boxes strictly before `self.index`. If `self` is FreePlay, also unlocks FreePlay cards.
    pub fn catch_up_command_keys(&self) -> impl Iterator<Item = &'static str> {
        let is_fp = self.is_free_play();
        let target = self.index;
        (0..target)
            .filter_map(step)
            .filter(|s| s.is_loot_box())
            .flat_map(|s| s.unlocked_keys.iter().copied())
            .chain(if is_fp { self.unlocked_keys } else { &[] }.iter().copied())
    }
}

// ==========================================
// Public Entry Points
// ==========================================

/// Query step by index (0..=FINAL_STEP). Returns `None` if out of bounds.
pub fn step(index: u8) -> Option<&'static Step> {
    STEPS.get(index as usize)
}

/// Query command card by server key (e.g. "IF_LET_SOME").
pub fn command(key: &str) -> Option<&'static Command> {
    COMMANDS.iter().find(|c| c.key == key)
}

/// Complete catalog of all 103 command reference cards.
pub const fn all_commands() -> &'static [Command] {
    &COMMANDS
}

/// Complete catalog of all 38 tutorial steps (0..=37).
pub const fn all_steps() -> &'static [Step] {
    &STEPS
}

/// Ordered command keys contained in the loot box of the given tutorial step.
pub fn loot_box_commands(step_index: u8) -> &'static [&'static str] {
    match step(step_index) {
        Some(s) if s.is_loot_box() => s.unlocked_keys,
        _ => &[],
    }
}

/// Additional SDK commands unlocked automatically once reaching Free Play (step 37).
pub fn free_play_commands() -> &'static [&'static str] {
    STEPS[FINAL_STEP as usize].unlocked_keys
}

// ==========================================
// Static Subtasks
// ==========================================

const LOOT_BOX_SUBTASKS: &[SubTask] = &[
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
];

// ==========================================
// Step Unlocked Key Slices
// ==========================================

const KEYS_STEP_5: &[&str] = &[
    "IF_LET_SOME",
    "WORLD_FIRST_DRONE",
    "PRINTLN",
    "DRONE_POSITION",
    "FORMAT_NO_DECIMALS",
    "PRINTLN_MACRO",
];
const KEYS_STEP_7: &[&str] = &["WORLD_CLOSEST_ASTEROID", "ASTEROID_ID"];
const KEYS_STEP_9: &[&str] = &["DRONE_SHIP_ID", "ACTIONS_MINE"];
const KEYS_STEP_11: &[&str] = &["IF_ELSE", "DRONE_CARGO_IS_FULL", "ACTIONS_DOCK"];
const KEYS_STEP_13: &[&str] = &[
    "WORLD_FLEET_LEN",
    "WORLD_BASE_STORED_ORE",
    "COMPARISONS",
    "LOGICAL_AND",
    "ACTIONS_BUILD_DRONE_1",
    "COMPARE_AND",
];
const KEYS_STEP_15: &[&str] = &["FOR_IN_FLEET"];
const KEYS_STEP_17: &[&str] = &[
    "DRONE_IS_DOCKED",
    "SHIP_ENERGY",
    "SHIP_MAX_ENERGY",
    "SHIP_ENERGY_PERCENT",
    "ELSE_IF",
];
const KEYS_STEP_19: &[&str] = &[
    "WORLD_ACTIVE_RIDDLE",
    "OPTION_IS_NONE",
    "ACTIONS_REQUEST_RESEARCH",
];
const KEYS_STEP_21: &[&str] = &[
    "LET_BINDING",
    "CLONE_AND_SORT",
    "RIDDLE_SORT",
    "ACTIONS_SOLVE_RIDDLE",
];
const KEYS_STEP_23: &[&str] = &[
    "WORLD_BASE",
    "ACTIONS_BUILD_DRONE_2",
    "BASE_STORED_ENERGY",
    "SDK_DRONE_COSTS",
];
const KEYS_STEP_25: &[&str] = &[
    "ACTIONS_FLY_TO",
    "VEC2_NEW",
    "BASE_POSITION",
    "BASE_SIGNAL_RADIUS",
    "BASE_DISTANCE_TO",
    "BASE_IS_IN_RANGE",
];
const KEYS_STEP_27: &[&str] = &[
    "ACTIONS_STOP",
    "SHIP_VELOCITY",
    "SHIP_SPEED",
    "SHIP_MAX_SPEED",
    "SHIP_HEADING",
    "SHIP_TARGET_POSITION",
    "SHIP_DISTANCE_TO",
];
const KEYS_STEP_29: &[&str] = &[
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
    "VEC2_DISTANCE",
];
const KEYS_STEP_31: &[&str] = &[
    "WORLD_SHIP",
    "DRONE_TIER",
    "SHIP_CARGO_ORE",
    "SHIP_MAX_CARGO",
    "SHIP_CARGO_FREE_SPACE",
    "SHIP_TARGET_ASTEROID_ID",
    "SHIP_IS_MINING",
];
const KEYS_STEP_33: &[&str] = &["SHIP_IS_STRANDED", "SHIP_SIGNAL_COUNTDOWN"];
const KEYS_STEP_35: &[&str] = &[
    "WORLD_OTHER_DRONES",
    "OTHER_DRONE_SHIP_ID",
    "OTHER_DRONE_TIER",
    "OTHER_DRONE_POSITION",
    "OTHER_DRONE_HEADING",
    "OTHER_DRONE_SPEED",
    "OTHER_DRONE_IS_MINING",
    "WORLD_OTHER_BASES",
    "OTHER_BASE_ID",
    "OTHER_BASE_POSITION",
    "OTHER_BASE_SIGNAL_RADIUS",
    "OTHER_BASE_LEVEL",
    "OTHER_BASE_SCANNER_LEVELS",
];

const KEYS_FREE_PLAY: &[&str] = &[
    "VEC2_ZERO",
    "VEC2_XY",
    "WORLD_TICK",
    "BASE_LEVEL",
    "BASE_DRONE_SCANNER_LEVEL",
    "BASE_BASE_SCANNER_LEVEL",
    "BASE_RADAR_HORIZON_LEVEL",
    "BASE_DRONE_SCANNER_RANGE",
    "BASE_BASE_SCANNER_RANGE",
    "BASE_RADAR_HORIZON_RANGE",
    "BASE_RESEARCH_LEVELS",
    "RIDDLE_DISTANCE",
    "RIDDLE_DENOISE",
    "RIDDLE_ENERGY",
    "UPGRADE_TRACK_BASE_ASTEROID",
    "UPGRADE_TRACK_RADAR_HORIZON",
    "UPGRADE_TRACK_NAME",
    "RIDDLE_TASK_DISTANCE",
    "RIDDLE_TASK_DENOISE",
    "RIDDLE_SOLUTION_DISTANCE",
    "RIDDLE_SOLUTION_DENOISE",
];

const EMPTY_KEYS: &[&str] = &[];

// ==========================================
// Static Steps Catalog (0..=37)
// ==========================================

pub static STEPS: [Step; 38] = [
    // Step 0
    Step {
        index: 0,
        kind: StepKind::Onboarding,
        verification: VerificationMode::ManualAdvance,
        title: "Willkommen an Bord, Commander!",
        short_title: "Registrierung",
        admin_label: "0/37 Registrierung",
        story: "Im Asteroidengürtel wimmelt es von automatischen Transportern und fremden Stationen. Um deine Schiffe auf den Radarsystemen der Mega-Station eindeutig zu identifizieren, benötigt deine Flotte deinen Pilotenausweis und eigene Signal-Farben.",
        subtasks: &[
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
        unlocked_keys: EMPTY_KEYS,
        reference_keys: EMPTY_KEYS,
    },
    // Step 1
    Step {
        index: 1,
        kind: StepKind::Onboarding,
        verification: VerificationMode::ServerEvent,
        title: "Der Jungfernflug (Manuelle Steuerung)",
        short_title: "Flug",
        admin_label: "1/37 Flug",
        story: "Dein Bauschiff (Tier 0) startet nah herangezoomt an der Mega-Station bei (0, 0). Bedienung der Karte: Mit dem Mausrad (Scrollen) zoomst du rein und raus, und durch Klicken und Halten mit der linken Maustaste verschiebst du die Karte. Wichtig: Du kannst deine Basis nur außerhalb des roten bzw. grünen Sicherheitskreises um die Mega-Station errichten!",
        subtasks: &[
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
        unlocked_keys: EMPTY_KEYS,
        reference_keys: EMPTY_KEYS,
    },
    // Step 2
    Step {
        index: 2,
        kind: StepKind::Onboarding,
        verification: VerificationMode::ServerEvent,
        title: "Das Fundament (Heimatbasis errichten)",
        short_title: "Basisbau",
        admin_label: "2/37 Basisbau",
        story: "Hervorragend gewählt, Commander! Du befindest dich außerhalb des grünen Sicherheitskreises der Mega-Station und hast genügend Erz im Scanradius. Durch die Aktivierung der Montage-Sequenz entfaltet sich dein Bauschiff zu deiner Heimatbasis.",
        subtasks: &[
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
        unlocked_keys: EMPTY_KEYS,
        reference_keys: EMPTY_KEYS,
    },
    // Step 3
    Step {
        index: 3,
        kind: StepKind::Onboarding,
        verification: VerificationMode::ManualAdvance,
        title: "Die Brücke öffnen (uhlaunch & Zed)",
        short_title: "Bot Start",
        admin_label: "3/37 Bot Start",
        story: "Deine Heimatbasis hat einen Bordcomputer. Du programmierst ihn in Rust und startest deinen Bot im Zed-Editor.",
        subtasks: &[
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
        unlocked_keys: EMPTY_KEYS,
        reference_keys: EMPTY_KEYS,
    },
    // Step 4
    Step {
        index: 4,
        kind: StepKind::Onboarding,
        verification: VerificationMode::ServerEvent,
        title: "Bot mit der Heimatbasis koppeln",
        short_title: "Bot-Kopplung",
        admin_label: "4/37 Bot-Kopplung",
        story: "Die Identität ist die Adresse deines Bots; mit ihr erlaubst du ihm die Steuerung deiner Basis. Ein Token ist dagegen ein geheimer Zugangsschlüssel: Gib ihn niemals als Identität ein oder an andere weiter. Beim ersten Start speichert der Bot seinen Token selbst in config.toml.",
        subtasks: &[
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
        unlocked_keys: EMPTY_KEYS,
        reference_keys: EMPTY_KEYS,
    },
    // Step 5: Lootbox 1
    Step {
        index: 5,
        kind: StepKind::LootBox { number: 1 },
        verification: VerificationMode::LootBoxClaim,
        title: "🎁 Befehls-Lootbox #1: Deine erste Drohne",
        short_title: "🎁 Lootbox 1: Drohne",
        admin_label: "5/37 🎁 Lootbox 1: Drohne",
        story: "Dein Bot ist verbunden! In der Heimatbasis warten die ersten Befehle auf dich: if let Some, world.first_drone(), println! und die Position deiner Drohne.",
        subtasks: LOOT_BOX_SUBTASKS,
        unlocked_keys: KEYS_STEP_5,
        reference_keys: EMPTY_KEYS,
    },
    // Step 6: Coding Mission 1
    Step {
        index: 6,
        kind: StepKind::Coding,
        verification: VerificationMode::CoordinateQuiz,
        title: "Wo ist deine Drohne?",
        short_title: "Drohne",
        admin_label: "6/37 Drohne",
        story: "In main.rs wird tick etwa zehnmal pro Sekunde aufgerufen. world enthält den aktuellen Weltzustand, actions sammelt deine Befehle. Schreibe deinen Code in tick: Nur wenn eine Drohne vorhanden ist, kannst du ihre Position auslesen.",
        subtasks: &[
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
        unlocked_keys: EMPTY_KEYS,
        reference_keys: KEYS_STEP_5,
    },
    // Step 7: Lootbox 2
    Step {
        index: 7,
        kind: StepKind::LootBox { number: 2 },
        verification: VerificationMode::LootBoxClaim,
        title: "🎁 Befehls-Lootbox #2: Asteroiden finden",
        short_title: "🎁 Lootbox 2: Asteroid",
        admin_label: "7/37 🎁 Lootbox 2: Asteroid",
        story: "Mit dem nächsten Befehl findest du bekannte Asteroiden, die noch Erz enthalten, und liest ihre Kennnummer(ID) aus.",
        subtasks: LOOT_BOX_SUBTASKS,
        unlocked_keys: KEYS_STEP_7,
        reference_keys: EMPTY_KEYS,
    },
    // Step 8: Coding Mission 2
    Step {
        index: 8,
        kind: StepKind::Coding,
        verification: VerificationMode::AsteroidIdQuiz,
        title: "Die Asteroiden-ID ermitteln",
        short_title: "Asteroid",
        admin_label: "8/37 Asteroid",
        story: "Deine Sensoren identifizieren Asteroiden in Reichweite der Basis oder deiner Drohnen. Jede so identifizierte Asteroiden-ID ist gültig; die Suche wählt davon einen passenden Asteroiden mit Erz.",
        subtasks: &[
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
        unlocked_keys: EMPTY_KEYS,
        reference_keys: KEYS_STEP_7,
    },
    // Step 9: Lootbox 3
    Step {
        index: 9,
        kind: StepKind::LootBox { number: 3 },
        verification: VerificationMode::LootBoxClaim,
        title: "🎁 Befehls-Lootbox #3: Erz schürfen",
        short_title: "🎁 Lootbox 3: Mining",
        admin_label: "9/37 🎁 Lootbox 3: Mining",
        story: "Jetzt lernst du die Kennnummer deiner Drohne und den Befehl, mit dem sie einen Asteroiden abbaut.",
        subtasks: LOOT_BOX_SUBTASKS,
        unlocked_keys: KEYS_STEP_9,
        reference_keys: EMPTY_KEYS,
    },
    // Step 10: Coding Mission 3
    Step {
        index: 10,
        kind: StepKind::Coding,
        verification: VerificationMode::ServerEvent,
        title: "Der erste Schürfzug",
        short_title: "Erstes Erz",
        admin_label: "10/37 Erstes Erz",
        story: "Deine Drohne und ein Asteroid mit Erz sind gefunden. Gib den Abbau-Befehl in tick: Die Drohne fliegt erst zum Ziel und schürft dort. Die Mission endet, sobald der Server erfolgreichen Abbau registriert.",
        subtasks: &[
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
        unlocked_keys: EMPTY_KEYS,
        reference_keys: KEYS_STEP_9,
    },
    // Step 11: Lootbox 4
    Step {
        index: 11,
        kind: StepKind::LootBox { number: 4 },
        verification: VerificationMode::LootBoxClaim,
        title: "🎁 Befehls-Lootbox #4: Laderaum & Andocken",
        short_title: "🎁 Lootbox 4",
        admin_label: "11/37 🎁 Lootbox 4",
        story: "Erstes Erz im Frachtraum! In deiner Heimatbasis wartet eine Lootbox mit Befehlen zur Fallunterscheidung, Andocken und Ausladen.",
        subtasks: LOOT_BOX_SUBTASKS,
        unlocked_keys: KEYS_STEP_11,
        reference_keys: EMPTY_KEYS,
    },
    // Step 12: Coding Mission 4
    Step {
        index: 12,
        kind: StepKind::Coding,
        verification: VerificationMode::ServerEvent,
        title: "Frachtraum voll! Kurs Heimatbasis",
        short_title: "Lieferung",
        admin_label: "12/37 Lieferung",
        story: "Deine Drohne kann nur 50 Einheiten Erz tragen. Ist der Frachtraum voll, stoppt der Bohrer automatisch. Damit das Erz verarbeitet werden kann, muss die Drohne an der Heimatbasis andocken. Ziel: Sammle mindestens 50 Erz an deiner Basis!",
        subtasks: &[
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
        unlocked_keys: EMPTY_KEYS,
        reference_keys: KEYS_STEP_11,
    },
    // Step 13: Lootbox 5
    Step {
        index: 13,
        kind: StepKind::LootBox { number: 5 },
        verification: VerificationMode::LootBoxClaim,
        title: "🎁 Befehls-Lootbox #5: Flotten-Konstruktion",
        short_title: "🎁 Lootbox 5",
        admin_label: "13/37 🎁 Lootbox 5",
        story: "Deine Lieferung war erfolgreich! Öffne die Lootbox in der Heimatbasis, um zu lernen, wie du Werte vergleichst, Bedingungen verknüpfst und neue Drohnen baust.",
        subtasks: LOOT_BOX_SUBTASKS,
        unlocked_keys: KEYS_STEP_13,
        reference_keys: EMPTY_KEYS,
    },
    // Step 14: Coding Mission 5
    Step {
        index: 14,
        kind: StepKind::Coding,
        verification: VerificationMode::ServerEvent,
        title: "Flottenausbau (Zweite Drohne bauen)",
        short_title: "Drohne 2",
        admin_label: "14/37 Drohne 2",
        story: "Mit 50 Einheiten Erz im Lager verfügt deine Basis über genügend Rohstoffe, um ein weiteres Schiff der Stufe 1 in Auftrag zu geben. Mehr Drohnen bedeuten schnellere Rohstoffförderung!",
        subtasks: &[
            SubTask {
                num: "1",
                text: "Prüfe, ob du weniger als 2 Drohnen und mindestens 50 Erz hast:",
                code_hint: Some("if world.fleet.len() < 2 && world.base.stored_ore >= 50 { ... }"),
            },
            SubTask {
                num: "2",
                text: "Gib den Bau einer Stufe-1-Drohne in Auftrag:",
                code_hint: Some("actions.build_drone(1);"),
            },
        ],
        unlocked_keys: EMPTY_KEYS,
        reference_keys: KEYS_STEP_13,
    },
    // Step 15: Lootbox 6
    Step {
        index: 15,
        kind: StepKind::LootBox { number: 6 },
        verification: VerificationMode::LootBoxClaim,
        title: "🎁 Befehls-Lootbox #6: Schwarm-Schleife",
        short_title: "🎁 Lootbox 6",
        admin_label: "15/37 🎁 Lootbox 6",
        story: "Zwei Drohnen im Einsatz! In der Heimatbasis wartet die Schleife, mit der du die ganze Flotte steuerst.",
        subtasks: LOOT_BOX_SUBTASKS,
        unlocked_keys: KEYS_STEP_15,
        reference_keys: EMPTY_KEYS,
    },
    // Step 16: Coding Mission 6
    Step {
        index: 16,
        kind: StepKind::Coding,
        verification: VerificationMode::ServerEvent,
        title: "Multitasking für Fortgeschrittene (Flottenschleife)",
        short_title: "Flottenloop",
        admin_label: "16/37 Flottenloop",
        story: "Bisher hat dein Code nur die erste Drohne mit world.first_drone() gesteuert. In Rust nutzen wir eine for-Schleife, um jedem Schiff in world.fleet automatisch denselben Ablauf zuzuweisen.",
        subtasks: &[
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
        unlocked_keys: EMPTY_KEYS,
        reference_keys: KEYS_STEP_15,
    },
    // Step 17: Lootbox 7
    Step {
        index: 17,
        kind: StepKind::LootBox { number: 7 },
        verification: VerificationMode::LootBoxClaim,
        title: "🎁 Befehls-Lootbox #7: Energiemanagement & Laden",
        short_title: "🎁 Lootbox 7: Energie",
        admin_label: "17/37 🎁 Lootbox 7: Energie",
        story: "Deine Flotte schürft fleißig! Doch Fliegen und Bohren verbrauchen Batterie-Energie. In der Heimatbasis wartet eine Lootbox, mit der deine Drohnen rechtzeitig vor leerer Batterie umkehren und an der Basis vollladen.",
        subtasks: LOOT_BOX_SUBTASKS,
        unlocked_keys: KEYS_STEP_17,
        reference_keys: EMPTY_KEYS,
    },
    // Step 18: Coding Mission 7
    Step {
        index: 18,
        kind: StepKind::Coding,
        verification: VerificationMode::ServerEvent,
        title: "Rechtzeitig an die Steckdose (Energiemanagement)",
        short_title: "Energiemanagement",
        admin_label: "18/37 Energiemanagement",
        story: "Wenn eine Drohne das Erz an der Basis abliefert, ist ihr Frachtraum sofort wieder leer – ohne Zusatzprüfung fliegt sie ungeladen wieder los, bis ihr mitten im All der Strom ausgeht und nur noch der langsame Not-Generator tröpfchenweise hilft. Halte angedockte Drohnen an der Basis, bis ihre Batterie voll ist (1.0), und schicke Drohnen mit niedrigem Batteriestand (< 0.25) rechtzeitig zum Andocken!",
        subtasks: &[
            SubTask {
                num: "1",
                text: "Prüfe in deiner for-Schleife zuerst, ob die Drohne angedockt ist und ihre Batterie noch lädt (der Block bleibt leer, damit sie an der Basis weiterlädt):",
                code_hint: Some("if drone.is_docked() && drone.energy_percent() < 1.0 { }"),
            },
            SubTask {
                num: "2",
                text: "Prüfe mit else if, ob der Frachtraum voll oder die Batterie unter 25 % gefallen ist, und befiehl das Andocken:",
                code_hint: Some(
                    "else if drone.cargo_is_full() || drone.energy_percent() < 0.25 { actions.dock(drone.ship_id); }",
                ),
            },
            SubTask {
                num: "3",
                text: "Lass die Drohne im else-Zweig wie bisher den nächsten Asteroiden mit Erz abbauen und warte, bis eine Drohne an der Basis vollständig aufgeladen wurde.",
                code_hint: None,
            },
        ],
        unlocked_keys: EMPTY_KEYS,
        reference_keys: KEYS_STEP_17,
    },
    // Step 19: Lootbox 8
    Step {
        index: 19,
        kind: StepKind::LootBox { number: 8 },
        verification: VerificationMode::LootBoxClaim,
        title: "🎁 Befehls-Lootbox #8: Forschungsauftrag",
        short_title: "🎁 Lootbox 8: Forschung",
        admin_label: "19/37 🎁 Lootbox 8: Forschung",
        story: "Deine Drohnen laden zuverlässig an der Basis auf! Jetzt lernst du, wann ein Forschungsrätsel aktiv ist und wie du einen neuen Auftrag anforderst.",
        subtasks: LOOT_BOX_SUBTASKS,
        unlocked_keys: KEYS_STEP_19,
        reference_keys: EMPTY_KEYS,
    },
    // Step 20: Request a research task
    Step {
        index: 20,
        kind: StepKind::Coding,
        verification: VerificationMode::ServerEvent,
        title: "Forschungsauftrag anfordern",
        short_title: "Forschungsauftrag",
        admin_label: "20/37 Forschungsauftrag",
        story: "Die Heimatbasis kann einen Auftrag für den Basen- und Asteroidenscanner anfordern. Fordere genau dann einen Auftrag an, wenn noch keiner aktiv ist. Der Server bestätigt den Auftrag und schaltet danach die nächste Mission frei.",
        subtasks: &[
            SubTask {
                num: "1",
                text: "Prüfe, ob gerade kein Forschungsrätsel aktiv ist:",
                code_hint: Some("if world.active_riddle.is_none() { ... }"),
            },
            SubTask {
                num: "2",
                text: "Fordere im if-Block einen Auftrag für den Basen- und Asteroidenscanner an:",
                code_hint: Some("actions.request_research(UpgradeTrack::BaseAsteroidScanner);"),
            },
        ],
        unlocked_keys: EMPTY_KEYS,
        reference_keys: KEYS_STEP_19,
    },
    // Step 21: Lootbox 9
    Step {
        index: 21,
        kind: StepKind::LootBox { number: 9 },
        verification: VerificationMode::LootBoxClaim,
        title: "🎁 Befehls-Lootbox #9: Sortier-Rätsel",
        short_title: "🎁 Lootbox 9: Sortieren",
        admin_label: "21/37 🎁 Lootbox 9: Sortieren",
        story: "Der Forschungsauftrag ist da! Diese Lootbox zeigt dir, wie du die Messwerte für den Basen- und Asteroidenscanner ordnest und das Ergebnis einreichst.",
        subtasks: LOOT_BOX_SUBTASKS,
        unlocked_keys: KEYS_STEP_21,
        reference_keys: EMPTY_KEYS,
    },
    // Step 22: Solve a sorting riddle
    Step {
        index: 22,
        kind: StepKind::Coding,
        verification: VerificationMode::ServerEvent,
        title: "Messwerte für den Asteroiden-Scanner sortieren",
        short_title: "Sortier-Rätsel",
        admin_label: "22/37 Sortier-Rätsel",
        story: "Der Asteroiden-Scanner liefert eine Liste unsortierter Messwerte. Erstelle eine veränderbare Kopie, sortiere sie aufsteigend und reiche sie ein. Der Server prüft deine Lösung.",
        subtasks: &[
            SubTask {
                num: "1",
                text: "Packe das Sortier-Rätsel aus und kopiere die Messwerte in eine veränderbare Liste:",
                code_hint: Some(
                    "if let Some(RiddleTask::Sort { numbers }) = world.active_riddle {\n    let mut sorted = numbers.clone();\n    ...\n}",
                ),
            },
            SubTask {
                num: "2",
                text: "Sortiere die Kopie vom kleinsten zum größten Wert:",
                code_hint: Some("sorted.sort();"),
            },
            SubTask {
                num: "3",
                text: "Reiche die sortierte Liste im if-Block ein:",
                code_hint: Some("actions.solve_riddle(RiddleSolution::Sort(sorted));"),
            },
        ],
        unlocked_keys: EMPTY_KEYS,
        reference_keys: KEYS_STEP_21,
    },
    // Step 23: Lootbox 10
    Step {
        index: 23,
        kind: StepKind::LootBox { number: 10 },
        verification: VerificationMode::LootBoxClaim,
        title: "🎁 Befehls-Lootbox #10: Schwere Drohne",
        short_title: "🎁 Lootbox 10: Tier 2",
        admin_label: "23/37 🎁 Lootbox 10: Tier 2",
        story: "Deine Forschung ist angelaufen! Jetzt lernst du, wie du die Vorräte der Basis prüfst und eine schwere Stufe-2-Drohne in Auftrag gibst.",
        subtasks: LOOT_BOX_SUBTASKS,
        unlocked_keys: KEYS_STEP_23,
        reference_keys: EMPTY_KEYS,
    },
    // Step 24: Build a tier-2 drone
    Step {
        index: 24,
        kind: StepKind::Coding,
        verification: VerificationMode::ServerEvent,
        title: "Eine schwere Drohne bauen",
        short_title: "Drohne Tier 2",
        admin_label: "24/37 Drohne Tier 2",
        story: "Eine schwere Drohne trägt dreimal so viel Erz, kostet aber mehr Erz und Energie. Prüfe beides, bevor du den Bauauftrag erteilst.",
        subtasks: &[
            SubTask {
                num: "1",
                text: "Prüfe, ob genug Erz und mindestens 200 Energie in der Basis liegen:",
                code_hint: Some(
                    "if world.base.stored_ore >= space_bot_sdk::DRONE_TIER_2_COST && world.base.stored_energy >= 200.0 { ... }",
                ),
            },
            SubTask {
                num: "2",
                text: "Gib innerhalb des if-Blocks den Auftrag für eine Stufe-2-Drohne:",
                code_hint: Some("actions.build_drone(2);"),
            },
        ],
        unlocked_keys: EMPTY_KEYS,
        reference_keys: KEYS_STEP_23,
    },
    // Step 25: Lootbox 11
    Step {
        index: 25,
        kind: StepKind::LootBox { number: 11 },
        verification: VerificationMode::LootBoxClaim,
        title: "🎁 Befehls-Lootbox #11: Kurs setzen",
        short_title: "🎁 Lootbox 11: Navigation",
        admin_label: "25/37 🎁 Lootbox 11: Navigation",
        story: "Deine Flotte wächst! In dieser Lootbox findest du die Befehle, mit denen du eine Drohne zu einem berechneten Ziel im Raum schickst.",
        subtasks: LOOT_BOX_SUBTASKS,
        unlocked_keys: KEYS_STEP_25,
        reference_keys: EMPTY_KEYS,
    },
    // Step 26: Fly to a safe destination
    Step {
        index: 26,
        kind: StepKind::Coding,
        verification: VerificationMode::ServerEvent,
        title: "Eine Drohne auf Kurs schicken",
        short_title: "Kurs setzen",
        admin_label: "26/37 Kurs setzen",
        story: "Schicke eine Drohne zu einem Punkt 300 Meter rechts von deiner Basis. Der Server bestätigt den Flugauftrag, wenn das Ziel dort und innerhalb der Funkreichweite liegt.",
        subtasks: &[
            SubTask {
                num: "1",
                text: "Berechne einen Zielpunkt 300 Meter rechts von deiner Basis:",
                code_hint: Some(
                    "let target = Vec2::new(world.base.position.x + 300.0, world.base.position.y);",
                ),
            },
            SubTask {
                num: "2",
                text: "Schicke deine erste Drohne zu diesem Ziel:",
                code_hint: Some("actions.fly_to(drone.ship_id, target);"),
            },
        ],
        unlocked_keys: EMPTY_KEYS,
        reference_keys: KEYS_STEP_25,
    },
    // Step 27: Lootbox 12
    Step {
        index: 27,
        kind: StepKind::LootBox { number: 12 },
        verification: VerificationMode::LootBoxClaim,
        title: "🎁 Befehls-Lootbox #12: Flugstatus & Anhalten",
        short_title: "🎁 Lootbox 12: Flugstatus",
        admin_label: "27/37 🎁 Lootbox 12: Flugstatus",
        story: "Deine Drohne ist unterwegs. Diese Karten zeigen dir, woran du ihren Flug erkennst und wie du sie gezielt anhältst.",
        subtasks: LOOT_BOX_SUBTASKS,
        unlocked_keys: KEYS_STEP_27,
        reference_keys: EMPTY_KEYS,
    },
    // Step 28: Stop a drone at its destination
    Step {
        index: 28,
        kind: StepKind::Coding,
        verification: VerificationMode::ServerEvent,
        title: "Am Ziel anhalten",
        short_title: "Drohne anhalten",
        admin_label: "28/37 Drohne anhalten",
        story: "Lass deine Drohne den Kurs fortsetzen und halte sie an, sobald sie den Zielpunkt erreicht hat. Der Server prüft, ob sie am Ziel gestoppt wurde.",
        subtasks: &[
            SubTask {
                num: "1",
                text: "Berechne denselben Zielpunkt wie in der vorherigen Mission:",
                code_hint: Some(
                    "let target = Vec2::new(world.base.position.x + 300.0, world.base.position.y);",
                ),
            },
            SubTask {
                num: "2",
                text: "Wenn die Drohne steht und weniger als 30 Meter vom Ziel entfernt ist, halte sie an:",
                code_hint: Some(
                    "if drone.speed == 0.0 && drone.distance_to(&target) < 30.0 {\n    actions.stop(drone.ship_id);\n}",
                ),
            },
        ],
        unlocked_keys: EMPTY_KEYS,
        reference_keys: KEYS_STEP_27,
    },
    // Step 29: Lootbox 13
    Step {
        index: 29,
        kind: StepKind::LootBox { number: 13 },
        verification: VerificationMode::LootBoxClaim,
        title: "🎁 Befehls-Lootbox #13: Asteroiden erkunden",
        short_title: "🎁 Lootbox 13: Asteroiden",
        admin_label: "29/37 🎁 Lootbox 13: Asteroiden",
        story: "Der Kurs sitzt! Jetzt lernst du, welche Asteroiden deine Scanner bereits erfasst haben und welche davon noch Erz enthalten.",
        subtasks: LOOT_BOX_SUBTASKS,
        unlocked_keys: KEYS_STEP_29,
        reference_keys: EMPTY_KEYS,
    },
    // Step 30: Inspect and mine a known asteroid
    Step {
        index: 30,
        kind: StepKind::Coding,
        verification: VerificationMode::ServerEvent,
        title: "Einen bekannten Asteroiden auswählen",
        short_title: "Asteroiden-Scan",
        admin_label: "30/37 Asteroiden-Scan",
        story: "Zeige die bekannten Asteroiden mit ihrer Rohstoffart, Erzmenge und Entfernung an. Schicke danach eine Drohne zum nächsten bekannten Asteroiden mit Erz. Der Server bestätigt den Schritt, sobald tatsächlich Erz abgebaut wird.",
        subtasks: &[
            SubTask {
                num: "1",
                text: "Gehe die von deinen Scannern erfassten Asteroiden durch und zeige ihre Daten an:",
                code_hint: Some(
                    "for asteroid in world.asteroids {\n    println!(\"{}: {} Erz, {:.0} m entfernt\", asteroid.resource_name(), asteroid.remaining_ore, asteroid.distance_to(&drone.position));\n}",
                ),
            },
            SubTask {
                num: "2",
                text: "Finde den nächsten bekannten Asteroiden, der noch Erz enthält:",
                code_hint: Some(
                    "if let Some(asteroid) = world.closest_asteroid_with_ore(drone.position) { ... }",
                ),
            },
            SubTask {
                num: "3",
                text: "Befiehl der Drohne im if-Block, diesen Asteroiden abzubauen:",
                code_hint: Some("actions.mine(drone.ship_id, asteroid.asteroid_id);"),
            },
        ],
        unlocked_keys: EMPTY_KEYS,
        reference_keys: KEYS_STEP_29,
    },
    // Step 31: Lootbox 14
    Step {
        index: 31,
        kind: StepKind::LootBox { number: 14 },
        verification: VerificationMode::LootBoxClaim,
        title: "🎁 Befehls-Lootbox #14: Flotten-Status",
        short_title: "🎁 Lootbox 14: Flotte",
        admin_label: "31/37 🎁 Lootbox 14: Flotte",
        story: "Deine Drohne baut wieder Erz ab. In dieser Lootbox lernst du, ein bestimmtes Schiff zu finden und seinen Auftrag und Frachtraum zu prüfen.",
        subtasks: LOOT_BOX_SUBTASKS,
        unlocked_keys: KEYS_STEP_31,
        reference_keys: EMPTY_KEYS,
    },
    // Step 32: Inspect a ship and return a full drone
    Step {
        index: 32,
        kind: StepKind::Coding,
        verification: VerificationMode::ServerEvent,
        title: "Ein volles Schiff zurückrufen",
        short_title: "Flotten-Status",
        admin_label: "32/37 Flotten-Status",
        story: "Suche die Drohne anhand ihrer Kennnummer. Prüfe ihre Stufe, ihr aktuelles Ziel und ihren Frachtraum. Wenn der Frachtraum voll ist, schicke sie zurück zur Basis.",
        subtasks: &[
            SubTask {
                num: "1",
                text: "Finde das aktuelle Schiff in deiner Flotte anhand seiner Kennnummer:",
                code_hint: Some("if let Some(ship) = world.ship(drone.ship_id) { ... }"),
            },
            SubTask {
                num: "2",
                text: "Zeige Stufe und aktuelles Ziel an:",
                code_hint: Some(
                    "println!(\"Stufe {}: Ziel gesetzt = {}\", ship.ship_tier, ship.target_asteroid_id.is_some());",
                ),
            },
            SubTask {
                num: "3",
                text: "Wenn der Frachtraum voll ist, docke das Schiff an der Basis an:",
                code_hint: Some("if ship.cargo_is_full() { actions.dock(ship.ship_id); }"),
            },
        ],
        unlocked_keys: EMPTY_KEYS,
        reference_keys: KEYS_STEP_31,
    },
    // Step 33: Lootbox 15
    Step {
        index: 33,
        kind: StepKind::LootBox { number: 15 },
        verification: VerificationMode::LootBoxClaim,
        title: "🎁 Befehls-Lootbox #15: Funkpuffer",
        short_title: "🎁 Lootbox 15: Sicherheit",
        admin_label: "33/37 🎁 Lootbox 15: Sicherheit",
        story: "Deine Drohne kann Erz sicher zur Basis bringen. Jetzt lernst du, den verbleibenden Funkpuffer zu beobachten, damit kein Schiff im All strandet.",
        subtasks: LOOT_BOX_SUBTASKS,
        unlocked_keys: KEYS_STEP_33,
        reference_keys: EMPTY_KEYS,
    },
    // Step 34: Return before losing signal or energy
    Step {
        index: 34,
        kind: StepKind::Coding,
        verification: VerificationMode::ServerEvent,
        title: "Vor dem Stranden umkehren",
        short_title: "Funkpuffer",
        admin_label: "34/37 Funkpuffer",
        story: "Schicke eine Drohne zurück, sobald ihr Funkpuffer knapp wird oder ihre Batterie unter 25 Prozent fällt. Der Server bestätigt den Schritt, wenn der Rückkehrbefehl bei niedrigem Puffer oder Batteriestand eingeht.",
        subtasks: &[
            SubTask {
                num: "1",
                text: "Prüfe zuerst, ob der Funkpuffer knapp ist:",
                code_hint: Some("if drone.signal_countdown < 10 { ... }"),
            },
            SubTask {
                num: "2",
                text: "Falls nicht, prüfe mit else if, ob weniger als 25 Prozent Batterie übrig sind:",
                code_hint: Some("else if drone.energy_percent() < 0.25 { ... }"),
            },
            SubTask {
                num: "3",
                text: "Schicke die Drohne in beiden Fällen zur Heimatbasis:",
                code_hint: Some("actions.dock(drone.ship_id);"),
            },
        ],
        unlocked_keys: EMPTY_KEYS,
        reference_keys: KEYS_STEP_33,
    },
    // Step 35: Lootbox 16
    Step {
        index: 35,
        kind: StepKind::LootBox { number: 16 },
        verification: VerificationMode::LootBoxClaim,
        title: "🎁 Befehls-Lootbox #16: Die Nachbarschaft beobachten",
        short_title: "🎁 Lootbox 16: Nachbarschaft",
        admin_label: "35/37 🎁 Lootbox 16: Nachbarschaft",
        story: "Deine Flotte ist sicher unterwegs. Diese Karten zeigen dir, welche fremden Drohnen und Heimatbasen deine Sensoren in der Nähe erkennen.",
        subtasks: LOOT_BOX_SUBTASKS,
        unlocked_keys: KEYS_STEP_35,
        reference_keys: EMPTY_KEYS,
    },
    // Step 36: Observe neighboring players
    Step {
        index: 36,
        kind: StepKind::Coding,
        verification: VerificationMode::ServerEvent,
        title: "Fremde Schiffe im Sensorbereich",
        short_title: "Nachbarschaft",
        admin_label: "36/37 Nachbarschaft",
        story: "Zeige fremde Drohnen und Heimatbasen im Sensorbereich an. Danach führe deine eigene Erzlieferung fort. Der Server schaltet Freies Spiel frei, sobald deine Flotte Erz zur Basis geliefert hat. Die Listen können leer sein, wenn gerade keine fremden Schiffe in Reichweite sind.",
        subtasks: &[
            SubTask {
                num: "1",
                text: "Zeige die Kennnummern und Positionen sichtbarer fremder Drohnen an:",
                code_hint: Some(
                    "for other in world.other_drones {\n    println!(\"Fremde Drohne #{} bei {:.0}, {:.0}\", other.ship_id, other.position.x, other.position.y);\n}",
                ),
            },
            SubTask {
                num: "2",
                text: "Zeige danach die Kennnummern und Positionen fremder Basen an:",
                code_hint: Some(
                    "for base in world.other_bases {\n    println!(\"Fremde Basis #{} bei {:.0}, {:.0}\", base.base_id, base.position.x, base.position.y);\n}",
                ),
            },
            SubTask {
                num: "3",
                text: "Lass deine Flotte weiter Erz abbauen und an der Heimatbasis abliefern.",
                code_hint: Some(
                    "if drone.cargo_is_full() { actions.dock(drone.ship_id); } else if let Some(asteroid) = world.closest_asteroid_with_ore(drone.position) { actions.mine(drone.ship_id, asteroid.asteroid_id); }",
                ),
            },
        ],
        unlocked_keys: EMPTY_KEYS,
        reference_keys: KEYS_STEP_35,
    },
    // Step 37: Free Play
    Step {
        index: 37,
        kind: StepKind::FreePlay,
        verification: VerificationMode::None,
        title: "Tutorial abgeschlossen – Freies Spiel!",
        short_title: "Sandbox 🚀",
        admin_label: "37/37 Sandbox 🚀",
        story: "Fantastische Arbeit, Commander! Du kannst Forschungsaufträge lösen, Drohnen gezielt steuern, Asteroiden auswählen und deine Flotte sicher führen. Die übrigen Karten der Befehls-Ruhmeshalle helfen dir bei eigenen Ideen.",
        subtasks: &[
            SubTask {
                num: "1",
                text: "Probiere die noch gesperrten Nachschlagekarten in der Befehls-Ruhmeshalle aus.",
                code_hint: None,
            },
            SubTask {
                num: "2",
                text: "Baue deine Flotte aus und optimiere deine Erzlieferungen.",
                code_hint: None,
            },
            SubTask {
                num: "3",
                text: "Erforsche die verschiedenen Forschungszweige und führe die Rangliste an!",
                code_hint: None,
            },
        ],
        unlocked_keys: KEYS_FREE_PLAY,
        reference_keys: KEYS_FREE_PLAY,
    },
];

// ==========================================
// Static Commands Catalog (103 Cards)
// ==========================================

pub static COMMANDS: [Command; 103] = [
    Command {
        key: "IF_LET_SOME",
        box_step: 5,
        icon: "📦",
        category: "Rust-Syntax",
        title: "if let Some(x) = ...",
        summary: "Packt einen Wert sicher aus, falls er existiert.",
        what: "Option bedeutet: Ein Ergebnis kann vorhanden sein (Some) oder fehlen (None). Zum Beispiel ist keine Drohne verfügbar, wenn die Flotte leer ist. if let Some führt seinen Block nur aus, wenn ein Wert vorhanden ist; bei None wird er übersprungen.",
        how: "Links von = steht Some mit einem selbst gewählten Namen, rechts die Abfrage. Nur innerhalb der geschweiften Klammern steht der ausgepackte Wert unter diesem Namen zur Verfügung.",
        example: "if let Some(drone) = world.first_drone() {
    // hier existiert eine Drohne
}",
    },
    Command {
        key: "WORLD_FIRST_DRONE",
        box_step: 5,
        icon: "🛸",
        category: "Welt & Sensoren",
        title: "world.first_drone()",
        summary: "Liefert die erste Drohne deiner Flotte.",
        what: "Gibt die erste Drohne aus world.fleet zurück – ideal für den Einstieg, solange du nur eine Drohne steuerst. Das Ergebnis ist ein Option: Gibt es keine Drohne, ist es None.",
        how: "Rufe die Methode ohne Parameter auf und packe das Ergebnis mit if let Some(...) aus. Die Drohne bietet z. B. drone.ship_id und drone.position.",
        example: "if let Some(drone) = world.first_drone() {
    let id = drone.ship_id;
}",
    },
    Command {
        key: "PRINTLN",
        box_step: 5,
        icon: "🖨️",
        category: "Rust-Syntax",
        title: "println!(\"...\", wert)",
        summary: "Schreibt eine Zeile in das Terminal.",
        what: "println! gibt Text im Terminal von Zed aus – so kann dir dein Bot Dinge zeigen, die er gerade weiß. Jedes {} im Text ist ein Platzhalter: An dieser Stelle wird einer deiner Werte eingesetzt.",
        how: "Schreibe den Text in Anführungszeichen. Setze für jeden Platzhalter {} hinter dem Text, durch Kommas getrennt, einen Wert in derselben Reihenfolge. Am Ende steht ein Semikolon.",
        example: "println!(\"Meine Drohne hat die ID {}\", drone.ship_id);",
    },
    Command {
        key: "DRONE_POSITION",
        box_step: 5,
        icon: "📍",
        category: "Schiffs-Status",
        title: "drone.position.x / drone.position.y",
        summary: "Die Koordinaten deiner Drohne.",
        what: "Jede Drohne hat eine Position im Raum. position.x ist der waagerechte Wert, position.y der senkrechte. Beide sind Kommazahlen, zum Beispiel 120.7.",
        how: "Lies die Werte mit einem Punkt hinter dem Namen aus, ohne Klammern: erst drone, dann .position, dann .x oder .y.",
        example: "let x = drone.position.x;
let y = drone.position.y;",
    },
    Command {
        key: "FORMAT_NO_DECIMALS",
        box_step: 5,
        icon: "🎯",
        category: "Rust-Syntax",
        title: "{:.0}",
        summary: "Zeigt eine Kommazahl ohne Nachkommastellen.",
        what: "Ein normaler Platzhalter {} zeigt eine Kommazahl mit allen Stellen an. {:.0} rundet die angezeigte Zahl auf eine ganze Zahl. Die Zahl selbst bleibt unverändert, nur die Anzeige wird gerundet.",
        how: "Schreibe {:.0} statt {} in den Text von println!. Die Zahl hinter dem Punkt gibt an, wie viele Nachkommastellen erscheinen.",
        example: "println!(\"x = {:.0}\", drone.position.x);",
    },
    Command {
        key: "PRINTLN_MACRO",
        box_step: 5,
        icon: "🖨️",
        category: "Rust-Syntax",
        title: "println!(\"...\", wert)",
        summary: "Schreibt eine Zeile ins Terminal.",
        what: "println! ist ein Makro (erkennbar am Ausrufezeichen) und gibt Text samt Zeilenumbruch aus. Jeder Platzhalter {} im Text wird der Reihe nach durch einen Wert hinter dem Text ersetzt. {:.0} zeigt eine Kommazahl ohne Nachkommastellen.",
        how: "Setze den Text in Anführungszeichen, füge je Wert einen Platzhalter ein und liste die Werte nach einem Komma auf. Die Anzahl der Platzhalter muss zur Anzahl der Werte passen.",
        example: "println!(\"Position: {:.0} / {:.0}\", drone.position.x, drone.position.y);",
    },
    Command {
        key: "WORLD_CLOSEST_ASTEROID",
        box_step: 7,
        icon: "☄️",
        category: "Welt & Sensoren",
        title: "world.closest_asteroid_with_ore(pos)",
        summary: "Findet den nächsten bekannten Asteroiden mit Erz.",
        what: "Durchsucht die von Basis- oder Drohnenscannern identifizierten Asteroiden und liefert den nächsten mit Erz. Ohne passenden bekannten Asteroiden ist das Ergebnis None. closest_asteroid sucht dagegen auch leere Asteroiden.",
        how: "Übergib die Position, von der aus gesucht wird, zum Beispiel drone.position. Packe das Option mit if let Some aus. Der Asteroid besitzt das Feld asteroid_id.",
        example: "if let Some(asteroid) = world.closest_asteroid_with_ore(drone.position) {
    // asteroid.asteroid_id verwenden
}",
    },
    Command {
        key: "ASTEROID_ID",
        box_step: 7,
        icon: "🪪",
        category: "Welt & Sensoren",
        title: "asteroid.asteroid_id",
        summary: "Die Kennnummer eines Asteroiden.",
        what: "Jeder Asteroid hat eine eigene ganze Zahl als Ausweis, seine ID. Mit ihr erkennst du ihn eindeutig und sagst deiner Drohne später, welchen Asteroiden sie ansteuern soll.",
        how: "Lies das Feld ohne Klammern aus: der Name des gefundenen Asteroiden, ein Punkt, dann asteroid_id. Zum Anzeigen setzt du es in println! ein.",
        example: "println!(\"ID: {}\", asteroid.asteroid_id);",
    },
    Command {
        key: "DRONE_SHIP_ID",
        box_step: 9,
        icon: "🏷️",
        category: "Schiffs-Status",
        title: "drone.ship_id",
        summary: "Die Kennnummer deiner Drohne.",
        what: "Jedes Schiff deiner Flotte besitzt eine eindeutige ganze Zahl als Kennnummer. Alle Schiffs-Befehle in actions verlangen diese ID als ersten Parameter, damit der Bordcomputer weiß, welche Drohne gemeint ist.",
        how: "Lies das Feld mit einem Punkt ohne Klammern aus: erst der Name der Drohne, dann .ship_id.",
        example: "if let Some(drone) = world.first_drone() {
    println!(\"Drohne #{}\", drone.ship_id);
}",
    },
    Command {
        key: "ACTIONS_MINE",
        box_step: 9,
        icon: "⛏️",
        category: "Schiffs-Befehle",
        title: "actions.mine(ship_id, asteroid_id)",
        summary: "Schickt eine Drohne zum Erzabbau an einen Asteroiden.",
        what: "Beauftragt eine Drohne, zum angegebenen Asteroiden zu fliegen und Erz abzubauen. Der Abbau geschieht nicht sofort; der Bohrer stoppt, wenn der Frachtraum voll ist.",
        how: "Übergib zuerst die ship_id der Drohne und dann die asteroid_id des Ziels. Setze den Befehl in tick ab, nachdem du Drohne und Asteroid mit if let Some gefunden hast.",
        example: "actions.mine(drone.ship_id, asteroid.asteroid_id);",
    },
    Command {
        key: "IF_ELSE",
        box_step: 11,
        icon: "🔀",
        category: "Rust-Syntax",
        title: "if ... { ... } else { ... }",
        summary: "Entscheidet zwischen zwei Möglichkeiten.",
        what: "Ist die Bedingung wahr, läuft der erste Block, sonst der else-Block. So kann deine Drohne je nach Situation unterschiedlich handeln.",
        how: "Schreibe hinter if eine Bedingung (Wahr/Falsch). In den ersten Block kommt, was bei wahr passiert, in den else-Block der Rest.",
        example: "if drone.cargo_is_full() {
    // Fracht abliefern
} else {
    // weiter schürfen
}",
    },
    Command {
        key: "DRONE_CARGO_IS_FULL",
        box_step: 11,
        icon: "📊",
        category: "Schiffs-Status",
        title: "drone.cargo_is_full()",
        summary: "Prüft, ob der Frachtraum voll ist.",
        what: "Liefert true, wenn die Drohne ihre maximale Erzmenge geladen hat, sonst false. Eine Tier-1-Drohne trägt höchstens 50 Erz.",
        how: "Rufe die Methode auf einer Drohne auf und nutze das Ergebnis direkt als Bedingung in einem if.",
        example: "if drone.cargo_is_full() {
    actions.dock(drone.ship_id);
}",
    },
    Command {
        key: "ACTIONS_DOCK",
        box_step: 11,
        icon: "🏛️",
        category: "Schiffs-Befehle",
        title: "actions.dock(ship_id)",
        summary: "Schickt eine Drohne zur Heimatbasis zum Andocken.",
        what: "Die Drohne fliegt zur Heimatbasis, liefert dort ihr Erz ab und lädt ihre Batterie aus dem Energiespeicher der Basis wieder auf.",
        how: "Übergib die ship_id der Drohne, die zurückkehren soll.",
        example: "actions.dock(drone.ship_id);",
    },
    Command {
        key: "WORLD_FLEET_LEN",
        box_step: 13,
        icon: "🔢",
        category: "Welt & Sensoren",
        title: "world.fleet.len()",
        summary: "Zählt die Schiffe deiner Flotte.",
        what: "world.fleet ist die Liste all deiner Schiffe. Mit .len() erfährst du, wie viele es sind.",
        how: "Vergleiche die Anzahl mit einer Zahl, z. B. um nur dann eine neue Drohne zu bauen, wenn du weniger als zwei hast.",
        example: "if world.fleet.len() < 2 {
    // Platz für eine weitere Drohne
}",
    },
    Command {
        key: "WORLD_BASE_STORED_ORE",
        box_step: 13,
        icon: "🪨",
        category: "Welt & Sensoren",
        title: "world.base.stored_ore",
        summary: "Das in der Heimatbasis gelagerte Erz.",
        what: "Ein Feld deiner Heimatbasis, das zeigt, wie viel Erz dort gelagert ist. Mit diesem Erz bezahlst du neue Drohnen und Forschung.",
        how: "Lies das Feld ohne Klammern aus und vergleiche es mit den Kosten, z. B. mit >= 50.",
        example: "if world.base.stored_ore >= 50 {
    // genug Erz vorhanden
}",
    },
    Command {
        key: "COMPARISONS",
        box_step: 13,
        icon: "⚖️",
        category: "Rust-Syntax",
        title: "< / <= / > / >= / == / !=",
        summary: "Vergleicht zwei Zahlen und liefert wahr oder falsch.",
        what: "Ein Vergleich prüft, wie zwei Werte zueinander stehen: < (kleiner als), <= (kleiner oder gleich), > (größer als), >= (größer oder gleich), == (genau gleich) und != (ungleich). Das Ergebnis ist ein Wahrheitswert (true oder false) für if.",
        how: "Schreibe links den ersten Wert, in die Mitte das Vergleichszeichen und rechts den zweiten Wert. Für Gleichheit nutzt du zwei Gleichheitszeichen ==, da ein einzelnes = Werte zuweist.",
        example: "if world.fleet.len() < 2 {
    // weniger als 2 Schiffe
}",
    },
    Command {
        key: "LOGICAL_AND",
        box_step: 13,
        icon: "🔗",
        category: "Rust-Syntax",
        title: "&& / ||",
        summary: "Verknüpft zwei Bedingungen miteinander.",
        what: "Mit && (logisches Und) müssen beide Bedingungen gleichzeitig wahr sein, damit der if-Block ausgeführt wird. Mit || (logisches Oder) genügt es, wenn mindestens eine von beiden wahr ist.",
        how: "Schreibe die erste Bedingung, dann && (oder ||) und dahinter die zweite Bedingung.",
        example: "if world.fleet.len() < 2 && world.base.stored_ore >= 50 {
    actions.build_drone(1);
}",
    },
    Command {
        key: "ACTIONS_BUILD_DRONE_1",
        box_step: 13,
        icon: "🔧",
        category: "Basis-Befehle",
        title: "actions.build_drone(1)",
        summary: "Baut eine Stufe-1-Drohne für 50 Erz und 100 Energie.",
        what: "Gibt an der Heimatbasis den Bau einer Stufe-1-Drohne in Auftrag. Sie kostet 50 Erz sowie 100 Energie aus der Basis und trägt bis zu 50 Erz Fracht. Ist deine Flotte komplett leer (0 Schiffe), ist eine Notfall-Drohne kostenlos.",
        how: "Übergib die Stufe 1. Prüfe vorher, ob genug Erz in der Basis gelagert ist.",
        example: "if world.fleet.len() < 2 && world.base.stored_ore >= 50 {
    actions.build_drone(1);
}",
    },
    Command {
        key: "COMPARE_AND",
        box_step: 13,
        icon: "⚖️",
        category: "Rust-Syntax",
        title: "<, >=, &&",
        summary: "Vergleicht Zahlen und verknüpft Bedingungen.",
        what: "Vergleiche wie <, >, <= und >= liefern wahr oder falsch. Mit && (und) müssen beide Bedingungen gleichzeitig wahr sein, damit der Block läuft.",
        how: "Setze links und rechts eine Zahl oder ein Feld ein, z. B. world.fleet.len() < 2. Verbinde mehrere Prüfungen mit && in derselben if-Bedingung.",
        example: "if world.fleet.len() < 2 && world.base.stored_ore >= 50 {
    // beides trifft zu
}",
    },
    Command {
        key: "FOR_IN_FLEET",
        box_step: 15,
        icon: "🔁",
        category: "Rust-Syntax",
        title: "for drone in world.fleet { ... }",
        summary: "Wiederholt Befehle für jedes Schiff der Flotte.",
        what: "Eine for-Schleife geht die Liste world.fleet Schiff für Schiff durch. In jedem Durchlauf steht dir ein Schiff unter dem Namen drone zur Verfügung – so steuerst du beliebig viele Drohnen mit demselben Code.",
        how: "Schreibe for, einen Namen für das aktuelle Schiff, in und die Liste. Alles in den geschweiften Klammern wird für jedes Schiff ausgeführt.",
        example: "for drone in world.fleet {
    if drone.cargo_is_full() {
        actions.dock(drone.ship_id);
    }
}",
    },
    Command {
        key: "DRONE_IS_DOCKED",
        box_step: 17,
        icon: "🔌",
        category: "Schiffs-Status",
        title: "drone.is_docked()",
        summary: "Prüft, ob die Drohne aktuell an der Heimatbasis angedockt ist.",
        what: "Liefert true, wenn die Drohne ruhig im Ladebereich der Heimatbasis steht, sonst false. Solange sie angedockt ist, lädt die Basis ihre Batterie automatisch wieder auf.",
        how: "Rufe drone.is_docked() mit leeren Klammern auf und nutze das Ergebnis als Bedingung in einem if.",
        example: "if drone.is_docked() {
    // Drohne steht an der Heimatbasis
}",
    },
    Command {
        key: "ELSE_IF",
        box_step: 17,
        icon: "🪜",
        category: "Rust-Syntax",
        title: "if ... { ... } else if ... { ... } else { ... }",
        summary: "Prüft mehrere Bedingungen der Reihe nach.",
        what: "Mit else if hängst du eine weitere Prüfung an ein if an. Sobald die erste Bedingung wahr ist, wird nur ihr Block ausgeführt und alle folgenden Zweige werden übersprungen; trifft keine zu, läuft am Ende der else-Block.",
        how: "Schreibe nach der schließenden Klammer des ersten if-Blocks else if, dahinter die nächste Bedingung und ihren Block, und zum Schluss else für alle übrigen Fälle.",
        example: "if drone.is_docked() && drone.energy_percent() < 1.0 {
    // an der Basis weiterladen
} else if drone.cargo_is_full() || drone.energy_percent() < 0.25 {
    actions.dock(drone.ship_id);
} else {
    // weiter schürfen
}",
    },
    Command {
        key: "SHIP_ENERGY",
        box_step: 17,
        icon: "🔋",
        category: "Schiffs-Status",
        title: "drone.energy",
        summary: "Der aktuelle Batteriestand.",
        what: "Eine Kommazahl (f32) mit der Energie, die der Drohne noch bleibt. Ohne Energie strandet sie.",
        how: "Vergleiche den Wert mit einer Schwelle und docke rechtzeitig an, um die Batterie aufzuladen.",
        example: "if drone.energy < 20.0 {
    actions.dock(drone.ship_id);
}",
    },
    Command {
        key: "SHIP_MAX_ENERGY",
        box_step: 17,
        icon: "⚡",
        category: "Schiffs-Status",
        title: "drone.max_energy",
        summary: "Die Kapazität der Batterie.",
        what: "Eine Kommazahl (f32) mit der größtmöglichen Energie der Drohne. Sie hängt von der Schiffsstufe ab.",
        how: "Nutze sie zusammen mit drone.energy, oder greife gleich zu drone.energy_percent().",
        example: "let frei = drone.max_energy - drone.energy;",
    },
    Command {
        key: "SHIP_ENERGY_PERCENT",
        box_step: 17,
        icon: "📶",
        category: "Schiffs-Status",
        title: "drone.energy_percent()",
        summary: "Der Batteriestand als Anteil von 0.0 bis 1.0.",
        what: "Teilt energy durch max_energy und begrenzt das Ergebnis auf 0.0 bis 1.0. So vergleichst du Drohnen verschiedener Stufen fair.",
        how: "Rufe die Methode ohne Parameter auf. 0.2 bedeutet 20 Prozent.",
        example: "if drone.energy_percent() < 0.2 {
    actions.dock(drone.ship_id);
}",
    },
    Command {
        key: "WORLD_ACTIVE_RIDDLE",
        box_step: 19,
        icon: "🧩",
        category: "Forschung",
        title: "world.active_riddle",
        summary: "Das aktuell aktive Forschungsrätsel.",
        what: "Enthält das Rätsel, das du angefordert hast, als Option – oder None, wenn gerade keins aktiv ist. Je nach gewähltem Forschungsbaum steckt darin eine Variante von RiddleTask (Sort, Distance, Denoise oder Energy).",
        how: "Prüfe mit world.active_riddle.is_none(), ob du ein neues Rätsel anfordern kannst, und packe ein vorhandenes Rätsel mit if let Some(...) = world.active_riddle aus.",
        example: "if let Some(RiddleTask::Sort { numbers }) = world.active_riddle {
    // numbers ist die Zahlenreihe
}",
    },
    Command {
        key: "OPTION_IS_NONE",
        box_step: 19,
        icon: "❔",
        category: "Rust-Syntax",
        title: "opt.is_none() / opt.is_some()",
        summary: "Prüft, ob ein Option-Wert fehlt oder vorhanden ist.",
        what: "Wenn du ein Option nicht sofort auspacken willst, sondern nur wissen möchtest, ob es leer ist, liefert .is_none() den Wert true bei None. Umgekehrt liefert .is_some() den Wert true, wenn Some(...) darin steckt.",
        how: "Hänge .is_none() oder .is_some() mit Punkt und leeren Klammern an ein Option-Feld wie world.active_riddle an.",
        example: "if world.active_riddle.is_none() {
    actions.request_research(UpgradeTrack::DroneScanner);
}",
    },
    Command {
        key: "ACTIONS_REQUEST_RESEARCH",
        box_step: 19,
        icon: "🧪",
        category: "Forschung",
        title: "actions.request_research(track)",
        summary: "Fordert ein Forschungsrätsel für einen der vier Technologiebäume an.",
        what: "Startet ein Forschungsrätsel an der Heimatbasis (kostet Erz je nach Stufe). Du kannst frei zwischen vier Zweigen wählen: UpgradeTrack::DroneScanner (Drohnen-Scan), UpgradeTrack::BaseAsteroidScanner (Basis-Scan), UpgradeTrack::RadarHorizon (Radar-Horizont) und UpgradeTrack::EnergyProduction (Energieproduktion).",
        how: "Übergib die gewünschte UpgradeTrack-Variante, wenn world.active_riddle.is_none() wahr ist.",
        example: "if world.active_riddle.is_none() {
    actions.request_research(UpgradeTrack::DroneScanner);
}",
    },
    Command {
        key: "LET_BINDING",
        box_step: 21,
        icon: "📝",
        category: "Rust-Syntax",
        title: "let x = ...; / let mut x = ...;",
        summary: "Speichert ein Zwischenergebnis unter einem Namen.",
        what: "Mit let legst du eine Variable an, um einen berechneten Wert zu merken. Standardmäßig sind Variablen in Rust unveränderbar; schreibst du let mut, darfst du den Inhalt danach verändern (z. B. hochzählen oder sortieren).",
        how: "Schreibe let (oder let mut), dann den Variablennamen, ein = und den Startwert, abgeschlossen mit einem Semikolon.",
        example: "let mut count = 0;
count += 1;",
    },
    Command {
        key: "CLONE_AND_SORT",
        box_step: 21,
        icon: "📑",
        category: "Rust-Syntax",
        title: "let mut sorted = numbers.clone(); sorted.sort();",
        summary: "Legt eine veränderbare Kopie einer Liste an und sortiert sie.",
        what: ".clone() erzeugt eine eigene Kopie einer Liste, damit die Daten in world unberührt bleiben. let mut macht die Kopie veränderbar, und .sort() ordnet ganze Zahlen aufsteigend vom kleinsten zum größten Wert.",
        how: "Kopiere die Liste mit .clone() in eine let mut-Variable und rufe danach .sort() auf dieser Variable auf.",
        example: "let mut sorted = numbers.clone();
sorted.sort();",
    },
    Command {
        key: "RIDDLE_SORT",
        box_step: 21,
        icon: "🔢",
        category: "Forschung",
        title: "RiddleTask::Sort { numbers } / RiddleSolution::Sort(sorted)",
        summary: "Rätsel für Zweig 2 (Basen- und Asteroidenscanner): Zahlenreihe aufsteigend sortieren.",
        what: "Beim Forschungszweig UpgradeTrack::BaseAsteroidScanner erhältst du eine unsortierte Liste ganzer Zahlen (numbers: &Vec<i64>). Als Lösung reichst du dieselben Zahlen aufsteigend sortiert als RiddleSolution::Sort(...) ein.",
        how: "Packe RiddleTask::Sort { numbers } mit if let Some aus, klone und sortiere numbers und übergib RiddleSolution::Sort(sorted) an actions.solve_riddle.",
        example: "if let Some(RiddleTask::Sort { numbers }) = world.active_riddle {
    let mut sorted = numbers.clone();
    sorted.sort();
    actions.solve_riddle(RiddleSolution::Sort(sorted));
}",
    },
    Command {
        key: "ACTIONS_SOLVE_RIDDLE",
        box_step: 21,
        icon: "✅",
        category: "Forschung",
        title: "actions.solve_riddle(solution)",
        summary: "Reicht die Lösung des aktiven Forschungsrätsels ein.",
        what: "Schickt deine berechnete Lösung an die Basis. Ist sie richtig, steigt dein Forschungsfortschritt im gewählten Zweig. Bei falschen Antworten droht ein Durchbrenn-Risiko des Moduls.",
        how: "Verpacke dein Ergebnis passend zum aktiven Zweig in RiddleSolution::Sort, Distance, Denoise oder Energy und übergib es an actions.solve_riddle(...).",
        example: "actions.solve_riddle(RiddleSolution::Sort(sorted));",
    },
    Command {
        key: "ACTIONS_BUILD_DRONE_2",
        box_step: 23,
        icon: "🚀",
        category: "Basis-Befehle",
        title: "actions.build_drone(2)",
        summary: "Baut eine schwere Stufe-2-Drohne für 150 Erz und 200 Energie.",
        what: "Gibt eine schwere Stufe-2-Drohne in Auftrag. Sie kostet 150 Erz sowie 200 Energie aus der Basis, hat dreifache Frachtkapazität (150 Erz), höhere Geschwindigkeit (150 px/s) und eine doppelt so große Batterie (200 Energie).",
        how: "Übergib die Stufe 2 und achte darauf, dass genug Erz und Energie in der Basis gelagert sind.",
        example: "if world.base.stored_ore >= 150 && world.base.stored_energy >= 200.0 {
    actions.build_drone(2);
}",
    },
    Command {
        key: "SDK_DRONE_COSTS",
        box_step: 23,
        icon: "🪙",
        category: "Basis-Befehle",
        title: "DRONE_TIER_1_COST / DRONE_TIER_2_COST",
        summary: "Konstanten für die Erzkosten einer Stufe-1- (50) und Stufe-2-Drohne (150).",
        what: "Das space_bot_sdk stellt die Baukosten als feste Konstanten bereit: DRONE_TIER_1_COST (50 Erz) und DRONE_TIER_2_COST (150 Erz). So musst du keine losen Zahlen in deinen Code schreiben.",
        how: "Importiere die Konstanten mit use space_bot_sdk::{DRONE_TIER_1_COST, DRONE_TIER_2_COST}; und nutze sie im Vergleich.",
        example: "if world.base.stored_ore >= space_bot_sdk::DRONE_TIER_1_COST {
    actions.build_drone(1);
}",
    },
    Command {
        key: "BASE_STORED_ENERGY",
        box_step: 23,
        icon: "🔋",
        category: "Basis-Status",
        title: "world.base.stored_energy",
        summary: "Die in der Basis gelagerte Energie.",
        what: "Eine ganze Zahl (u32): Energie-Vorrat der Basis, aus dem angedockte Drohnen ihre Batterie laden.",
        how: "Lies das Feld aus und vergleiche es mit einem Mindestwert.",
        example: "if world.base.stored_energy > 0 {
    // Aufladen möglich
}",
    },
    Command {
        key: "WORLD_BASE",
        box_step: 23,
        icon: "🏛️",
        category: "Welt & Sensoren",
        title: "world.base",
        summary: "Deine Heimatbasis.",
        what: "Die Basisstation mit allen Feldern und Methoden, die diese Karten beschreiben, z. B. stored_ore oder position.",
        how: "Lies Felder direkt aus, z. B. world.base.level, und rufe Methoden mit Klammern auf.",
        example: "let lager = world.base.stored_ore;",
    },
    Command {
        key: "VEC2_NEW",
        box_step: 25,
        icon: "🆕",
        category: "Rust-Syntax",
        title: "Vec2::new(x, y)",
        summary: "Erzeugt einen Punkt aus zwei Kommazahlen.",
        what: "Vec2 ist ein Punkt oder eine Richtung im Weltraum mit den Feldern x und y. Vec2::new baut einen neuen aus zwei f32-Zahlen.",
        how: "Übergib zuerst x, dann y. Zahlen mit Komma schreibst du mit Punkt, z. B. 100.0.",
        example: "let ziel = Vec2::new(100.0, -50.0);",
    },
    Command {
        key: "BASE_POSITION",
        box_step: 25,
        icon: "📍",
        category: "Basis-Status",
        title: "world.base.position",
        summary: "Wo deine Basis liegt.",
        what: "Ein Vec2 mit den Koordinaten der Heimatbasis.",
        how: "Übergib die Position an distance_to-Methoden, z. B. um den Rückweg einer Drohne zu messen.",
        example: "let heimweg = drone.distance_to(&world.base.position);",
    },
    Command {
        key: "BASE_SIGNAL_RADIUS",
        box_step: 25,
        icon: "📡",
        category: "Basis-Status",
        title: "world.base.signal_radius",
        summary: "Der Funk- und Schutzradius der Basis.",
        what: "Eine Kommazahl (f32): innerhalb dieses Kreises ist Funkverkehr möglich und die Basis geschützt.",
        how: "Vergleiche ihn mit einer Entfernung, oder nutze gleich world.base.is_in_range(&pos).",
        example: "let reicht = d <= world.base.signal_radius;",
    },
    Command {
        key: "BASE_DISTANCE_TO",
        box_step: 25,
        icon: "📏",
        category: "Basis-Status",
        title: "world.base.distance_to(&pos)",
        summary: "Der Abstand der Basis zu einer Position.",
        what: "Liefert die Entfernung als Kommazahl (f32) zwischen der Basis und einem Punkt.",
        how: "Übergib die Referenz auf eine Position mit &.",
        example: "let d = world.base.distance_to(&asteroid.position);",
    },
    Command {
        key: "BASE_IS_IN_RANGE",
        box_step: 25,
        icon: "🧿",
        category: "Basis-Status",
        title: "world.base.is_in_range(&pos)",
        summary: "Prüft, ob ein Punkt im Funkradius der Basis liegt.",
        what: "Liefert true, wenn der Punkt höchstens signal_radius von der Basis entfernt ist.",
        how: "Übergib die Referenz auf eine Position und nutze das Ergebnis als Bedingung.",
        example: "if world.base.is_in_range(&drone.position) {
    // Drohne ist in Funkreichweite
}",
    },
    Command {
        key: "ACTIONS_FLY_TO",
        box_step: 25,
        icon: "🧭",
        category: "Schiffs-Befehle",
        title: "actions.fly_to(ship_id, pos)",
        summary: "Schickt eine Drohne zu einem Punkt.",
        what: "Beauftragt eine Drohne, zur angegebenen Position zu fliegen. Der Flug dauert, er geschieht nicht sofort.",
        how: "Übergib die ship_id und dann eine Position als Vec2, z. B. Vec2::new(100.0, 50.0) oder asteroid.position.",
        example: "actions.fly_to(drone.ship_id, Vec2::new(100.0, 50.0));",
    },
    Command {
        key: "SHIP_VELOCITY",
        box_step: 27,
        icon: "🧭",
        category: "Schiffs-Status",
        title: "drone.velocity",
        summary: "Geschwindigkeit und Richtung als Vec2.",
        what: "Ein Vec2, das angibt, um wie viel sich die Position der Drohne pro Zeitschritt in x- und y-Richtung ändert.",
        how: "Lies drone.velocity.x und drone.velocity.y aus. Für nur den Betrag der Geschwindigkeit nutze drone.speed.",
        example: "println!(\"{:.1} / {:.1}\", drone.velocity.x, drone.velocity.y);",
    },
    Command {
        key: "SHIP_TARGET_POSITION",
        box_step: 27,
        icon: "🎯",
        category: "Schiffs-Status",
        title: "drone.target_position",
        summary: "Das aktuelle Flugziel der Drohne.",
        what: "Ein Option<Vec2>: Some(ziel), solange die Drohne zu einem Ziel unterwegs ist, sonst None.",
        how: "Packe das Feld mit if let Some aus, um das Ziel zu lesen. Bei None hat die Drohne gerade kein Ziel.",
        example: "if let Some(ziel) = drone.target_position {
    println!(\"Ziel: {:.0} / {:.0}\", ziel.x, ziel.y);
}",
    },
    Command {
        key: "SHIP_HEADING",
        box_step: 27,
        icon: "🧲",
        category: "Schiffs-Status",
        title: "drone.heading_rad",
        summary: "Blickrichtung der Drohne in Radiant.",
        what: "Der Winkel, in den die Drohne zeigt, als Kommazahl (f32) im Bogenmaß: 0 zeigt nach rechts, ein halber Kreis ist etwa 3,14.",
        how: "Lies das Feld aus. Mit .to_degrees() rechnest du den Winkel in Grad um.",
        example: "let grad = drone.heading_rad.to_degrees();",
    },
    Command {
        key: "SHIP_SPEED",
        box_step: 27,
        icon: "💨",
        category: "Schiffs-Status",
        title: "drone.speed",
        summary: "Die aktuelle Geschwindigkeit.",
        what: "Eine Kommazahl (f32): wie schnell die Drohne gerade fliegt. Steht sie still, ist der Wert 0.",
        how: "Vergleiche den Wert z. B. mit 0.0, um zu prüfen, ob die Drohne unterwegs ist.",
        example: "if drone.speed > 0.0 {
    // Drohne fliegt
}",
    },
    Command {
        key: "SHIP_MAX_SPEED",
        box_step: 27,
        icon: "🚀",
        category: "Schiffs-Status",
        title: "drone.max_speed",
        summary: "Die höchste mögliche Geschwindigkeit.",
        what: "Eine Kommazahl (f32) mit dem Tempolimit der Drohne. Es hängt von der Schiffsstufe ab.",
        how: "Teile speed durch max_speed, um die Auslastung zu berechnen.",
        example: "let anteil = drone.speed / drone.max_speed;",
    },
    Command {
        key: "SHIP_DISTANCE_TO",
        box_step: 27,
        icon: "📏",
        category: "Schiffs-Status",
        title: "drone.distance_to(&pos)",
        summary: "Der Abstand der Drohne zu einer Position.",
        what: "Liefert die Entfernung als Kommazahl (f32) zwischen der Drohne und einem Punkt im Weltraum.",
        how: "Übergib die Referenz auf eine Position mit &, z. B. &asteroid.position.",
        example: "let weg = drone.distance_to(&asteroid.position);",
    },
    Command {
        key: "ACTIONS_STOP",
        box_step: 27,
        icon: "🛑",
        category: "Schiffs-Befehle",
        title: "actions.stop(ship_id)",
        summary: "Hält eine Drohne an.",
        what: "Bricht den aktuellen Flug- oder Schürfauftrag der Drohne ab. Sie bleibt dann stehen.",
        how: "Übergib die ship_id der Drohne, die anhalten soll.",
        example: "actions.stop(drone.ship_id);",
    },
    Command {
        key: "VEC2_DISTANCE",
        box_step: 29,
        icon: "📐",
        category: "Rust-Syntax",
        title: "a.distance(&b)",
        summary: "Der Abstand zwischen zwei Punkten.",
        what: "Liefert die Luftlinie zwischen zwei Vec2 als Kommazahl (f32). Die anderen distance_to-Methoden nutzen sie intern.",
        how: "Rufe sie auf einem Punkt auf und übergib den anderen mit &.",
        example: "let d = drone.position.distance(&asteroid.position);",
    },
    Command {
        key: "WORLD_CLOSEST_ASTEROID_ANY",
        box_step: 29,
        icon: "🛰️",
        category: "Welt & Sensoren",
        title: "world.closest_asteroid(pos)",
        summary: "Findet den nächsten bekannten Asteroiden, auch ohne Erz.",
        what: "Wie closest_asteroid_with_ore, aber ohne Rücksicht darauf, ob noch Erz vorhanden ist. Ohne bekannten Asteroiden ist das Ergebnis None.",
        how: "Übergib die Suchposition und packe das Option mit if let Some aus.",
        example: "if let Some(asteroid) = world.closest_asteroid(drone.position) {
    // auch leere Asteroiden
}",
    },
    Command {
        key: "WORLD_ASTEROIDS",
        box_step: 29,
        icon: "🌌",
        category: "Welt & Sensoren",
        title: "world.asteroids",
        summary: "Alle identifizierten Asteroiden.",
        what: "Eine Liste aller Asteroiden, die deine Scanner erkannt haben. Sie kann leer sein.",
        how: "Gehe sie mit einer for-Schleife durch oder frage mit .len() nach der Anzahl.",
        example: "for asteroid in world.asteroids {
    println!(\"{}\", asteroid.asteroid_id);
}",
    },
    Command {
        key: "ASTEROID_POSITION",
        box_step: 29,
        icon: "📍",
        category: "Welt & Sensoren",
        title: "asteroid.position",
        summary: "Wo der Asteroid gerade ist.",
        what: "Ein Vec2 mit den Koordinaten x und y des Asteroiden. Asteroiden treiben langsam, die Position ändert sich also.",
        how: "Übergib die Position an Methoden wie drone.distance_to oder lies .x und .y aus.",
        example: "let weg = drone.distance_to(&asteroid.position);",
    },
    Command {
        key: "ASTEROID_VELOCITY",
        box_step: 29,
        icon: "🧭",
        category: "Welt & Sensoren",
        title: "asteroid.velocity",
        summary: "Wohin der Asteroid treibt.",
        what: "Ein Vec2 mit der Bewegung des Asteroiden pro Zeitschritt. So kannst du abschätzen, wo er später ist.",
        how: "Lies asteroid.velocity.x und asteroid.velocity.y aus.",
        example: "let vx = asteroid.velocity.x;",
    },
    Command {
        key: "ASTEROID_RESOURCE_TYPE",
        box_step: 29,
        icon: "💎",
        category: "Welt & Sensoren",
        title: "asteroid.resource_type",
        summary: "Die Rohstoffart als Zahl.",
        what: "Eine kleine Zahl (u8): 1 steht für Eisen, 2 für Eis, 3 für Titan. Den lesbaren Namen liefert asteroid.resource_name().",
        how: "Vergleiche die Zahl direkt, wenn du nach einem bestimmten Rohstoff suchst.",
        example: "if asteroid.resource_type == 3 {
    // Titan
}",
    },
    Command {
        key: "ASTEROID_REMAINING_ORE",
        box_step: 29,
        icon: "🪨",
        category: "Welt & Sensoren",
        title: "asteroid.remaining_ore",
        summary: "Wie viel Erz noch im Asteroiden steckt.",
        what: "Eine ganze Zahl (u32): die Menge, die noch abgebaut werden kann. Bei 0 ist der Asteroid leer.",
        how: "Vergleiche sie mit dem freien Frachtplatz deiner Drohne, um lohnende Ziele zu finden.",
        example: "if asteroid.remaining_ore >= drone.cargo_free_space() {
    // reicht für eine volle Ladung
}",
    },
    Command {
        key: "ASTEROID_RADIUS",
        box_step: 29,
        icon: "⭕",
        category: "Welt & Sensoren",
        title: "asteroid.radius",
        summary: "Die Größe des Asteroiden.",
        what: "Eine Kommazahl (f32): der Radius des Asteroiden im Weltraum.",
        how: "Lies das Feld aus, z. B. um große Asteroiden zu bevorzugen.",
        example: "if asteroid.radius > 20.0 {
    // großer Brocken
}",
    },
    Command {
        key: "ASTEROID_DISTANCE_TO",
        box_step: 29,
        icon: "📏",
        category: "Welt & Sensoren",
        title: "asteroid.distance_to(&pos)",
        summary: "Der Abstand des Asteroiden zu einer Position.",
        what: "Liefert die Entfernung als Kommazahl (f32) zwischen dem Asteroiden und einem Punkt.",
        how: "Übergib die Referenz auf eine Position mit &, z. B. &world.base.position.",
        example: "let weg = asteroid.distance_to(&world.base.position);",
    },
    Command {
        key: "ASTEROID_IS_DEPLETED",
        box_step: 29,
        icon: "🕳️",
        category: "Welt & Sensoren",
        title: "asteroid.is_depleted()",
        summary: "Prüft, ob der Asteroid leer ist.",
        what: "Liefert true, wenn remaining_ore gleich 0 ist, sonst false.",
        how: "Rufe die Methode ohne Parameter auf und nutze sie als Bedingung.",
        example: "if !asteroid.is_depleted() {
    actions.mine(drone.ship_id, asteroid.asteroid_id);
}",
    },
    Command {
        key: "ASTEROID_RESOURCE_NAME",
        box_step: 29,
        icon: "🏷️",
        category: "Welt & Sensoren",
        title: "asteroid.resource_name()",
        summary: "Der lesbare Name des Rohstoffs.",
        what: "Liefert einen Text wie \"Iron\", \"Ice\" oder \"Titanium\" statt der Zahl aus resource_type.",
        how: "Rufe die Methode ohne Parameter auf und gib das Ergebnis z. B. mit println! aus.",
        example: "println!(\"Rohstoff: {}\", asteroid.resource_name());",
    },
    Command {
        key: "WORLD_SHIP",
        box_step: 31,
        icon: "🔎",
        category: "Welt & Sensoren",
        title: "world.ship(ship_id)",
        summary: "Sucht ein Schiff deiner Flotte anhand der ID.",
        what: "Liefert ein Option<&Ship>: Some(schiff), wenn es ein Schiff mit dieser ID in deiner Flotte gibt, sonst None.",
        how: "Übergib die ship_id und packe das Ergebnis mit if let Some aus.",
        example: "if let Some(schiff) = world.ship(drone.ship_id) {
    // schiff.position verwenden
}",
    },
    Command {
        key: "DRONE_TIER",
        box_step: 31,
        icon: "⭐",
        category: "Schiffs-Status",
        title: "drone.ship_tier",
        summary: "Die Baustufe des Schiffs (1 oder 2).",
        what: "Gibt an, um welchen Schiffstyp es sich handelt: 1 für die leichte Standard-Drohne (50 Fracht, 100 Energie), 2 für die schwere Stufe-2-Drohne (150 Fracht, 200 Energie).",
        how: "Lies drone.ship_tier aus, um großen und kleinen Drohnen unterschiedliche Aufgaben zuzuweisen.",
        example: "if drone.ship_tier == 2 {
    // schwere Drohne für weit entfernte Asteroiden nutzen
}",
    },
    Command {
        key: "SHIP_CARGO_ORE",
        box_step: 31,
        icon: "🪨",
        category: "Schiffs-Status",
        title: "drone.cargo_ore",
        summary: "Das geladene Erz.",
        what: "Eine ganze Zahl (u32): so viel Erz hat die Drohne gerade im Frachtraum.",
        how: "Lies das Feld aus und vergleiche es z. B. mit 0, um zu prüfen, ob schon etwas geladen ist.",
        example: "if drone.cargo_ore > 0 {
    // Erz an Bord
}",
    },
    Command {
        key: "SHIP_MAX_CARGO",
        box_step: 31,
        icon: "📦",
        category: "Schiffs-Status",
        title: "drone.max_cargo",
        summary: "Die Größe des Frachtraums.",
        what: "Eine ganze Zahl (u32): so viel Erz passt maximal in die Drohne. Bei Stufe 1 sind es 50.",
        how: "Vergleiche sie mit cargo_ore, oder nutze drone.cargo_is_full().",
        example: "let platz = drone.max_cargo - drone.cargo_ore;",
    },
    Command {
        key: "SHIP_TARGET_ASTEROID_ID",
        box_step: 31,
        icon: "☄️",
        category: "Schiffs-Status",
        title: "drone.target_asteroid_id",
        summary: "Der Asteroid, den die Drohne gerade ansteuert.",
        what: "Ein Option<u32>: Some(id), wenn die Drohne einem Schürfauftrag folgt, sonst None.",
        how: "Packe das Feld mit if let Some aus. So vermeidest du, einer beschäftigten Drohne einen neuen Auftrag zu geben.",
        example: "if let Some(id) = drone.target_asteroid_id {
    println!(\"Ziel-Asteroid: {}\", id);
}",
    },
    Command {
        key: "SHIP_IS_MINING",
        box_step: 31,
        icon: "⛏️",
        category: "Schiffs-Status",
        title: "drone.is_mining",
        summary: "Ob die Drohne gerade schürft.",
        what: "Ein bool: true, solange die Drohne an einem Asteroiden Erz abbaut, sonst false.",
        how: "Nutze das Feld direkt als Bedingung, z. B. um beschäftigte Drohnen zu überspringen.",
        example: "if !drone.is_mining {
    // Drohne ist frei
}",
    },
    Command {
        key: "SHIP_CARGO_FREE_SPACE",
        box_step: 31,
        icon: "🕳️",
        category: "Schiffs-Status",
        title: "drone.cargo_free_space()",
        summary: "Wie viel Erz noch in den Frachtraum passt.",
        what: "Liefert max_cargo minus cargo_ore als ganze Zahl (u32), niemals weniger als 0.",
        how: "Rufe die Methode ohne Parameter auf und vergleiche das Ergebnis mit einer Zahl.",
        example: "if drone.cargo_free_space() < 10 {
    actions.dock(drone.ship_id);
}",
    },
    Command {
        key: "SHIP_IS_STRANDED",
        box_step: 33,
        icon: "🆘",
        category: "Schiffs-Status",
        title: "drone.is_stranded",
        summary: "Ob die Drohne gestrandet ist.",
        what: "Ein bool: true, wenn der Drohne die Energie ausgegangen ist und sie nicht mehr fliegen kann.",
        how: "Prüfe das Feld und warne dich selbst, bevor es dazu kommt, z. B. mit drone.energy.",
        example: "if drone.is_stranded {
    println!(\"Drohne {} ist gestrandet\", drone.ship_id);
}",
    },
    Command {
        key: "SHIP_SIGNAL_COUNTDOWN",
        box_step: 33,
        icon: "📡",
        category: "Schiffs-Status",
        title: "drone.signal_countdown",
        summary: "Der Countdown des Notsignals.",
        what: "Eine ganze Zahl (u16): läuft für gestrandete Drohnen herunter, bevor das Notsignal verloren geht.",
        how: "Lies das Feld aus, wenn drone.is_stranded wahr ist.",
        example: "if drone.is_stranded {
    println!(\"Noch {}\", drone.signal_countdown);
}",
    },
    Command {
        key: "WORLD_OTHER_DRONES",
        box_step: 35,
        icon: "🛸",
        category: "Welt & Sensoren",
        title: "world.other_drones",
        summary: "Fremde Drohnen in Sichtweite.",
        what: "Eine Liste der Drohnen anderer Spieler, die deine Scanner erkennen. Sie kann leer sein.",
        how: "Gehe sie mit einer for-Schleife durch. Jedes Element hat die Felder, die die folgenden Karten beschreiben.",
        example: "for fremde in world.other_drones {
    println!(\"{}\", fremde.ship_id);
}",
    },
    Command {
        key: "WORLD_OTHER_BASES",
        box_step: 35,
        icon: "🏰",
        category: "Welt & Sensoren",
        title: "world.other_bases",
        summary: "Fremde Basen in Sichtweite.",
        what: "Eine Liste der Basen anderer Spieler, die deine Scanner erkennen. Sie kann leer sein.",
        how: "Gehe sie mit einer for-Schleife durch oder frage mit .len() nach der Anzahl.",
        example: "if world.other_bases.len() > 0 {
    // fremde Basis in der Nähe
}",
    },
    Command {
        key: "OTHER_DRONE_SHIP_ID",
        box_step: 35,
        icon: "🪪",
        category: "Fremde Drohnen",
        title: "other_drone.ship_id",
        summary: "Die Nummer einer fremden Drohne.",
        what: "Eine ganze Zahl (u32): die ID der Drohne im Spiel.",
        how: "Lies das Feld aus einer Drohne aus world.other_drones.",
        example: "println!(\"{}\", fremde.ship_id);",
    },
    Command {
        key: "OTHER_DRONE_TIER",
        box_step: 35,
        icon: "🏷️",
        category: "Fremde Drohnen",
        title: "other_drone.ship_tier",
        summary: "Die Stufe einer fremden Drohne.",
        what: "Eine kleine Zahl (u8): zeigt, wie leistungsfähig die fremde Drohne ist.",
        how: "Lies das Feld aus einer Drohne aus world.other_drones.",
        example: "if fremde.ship_tier >= 2 {
    // schwere Drohne
}",
    },
    Command {
        key: "OTHER_DRONE_POSITION",
        box_step: 35,
        icon: "📍",
        category: "Fremde Drohnen",
        title: "other_drone.position",
        summary: "Wo eine fremde Drohne ist.",
        what: "Ein Vec2 mit den Koordinaten der fremden Drohne.",
        how: "Übergib die Position an distance_to-Methoden, um Abstand zu halten.",
        example: "let d = drone.distance_to(&fremde.position);",
    },
    Command {
        key: "OTHER_DRONE_HEADING",
        box_step: 35,
        icon: "🧲",
        category: "Fremde Drohnen",
        title: "other_drone.heading_rad",
        summary: "Blickrichtung einer fremden Drohne in Radiant.",
        what: "Eine Kommazahl (f32) im Bogenmaß, genau wie drone.heading_rad.",
        how: "Lies das Feld aus. Mit .to_degrees() rechnest du in Grad um.",
        example: "let grad = fremde.heading_rad.to_degrees();",
    },
    Command {
        key: "OTHER_DRONE_SPEED",
        box_step: 35,
        icon: "💨",
        category: "Fremde Drohnen",
        title: "other_drone.speed",
        summary: "Die Geschwindigkeit einer fremden Drohne.",
        what: "Eine Kommazahl (f32): wie schnell die fremde Drohne fliegt.",
        how: "Lies das Feld aus einer Drohne aus world.other_drones.",
        example: "if fremde.speed > 0.0 {
    // sie bewegt sich
}",
    },
    Command {
        key: "OTHER_DRONE_IS_MINING",
        box_step: 35,
        icon: "⛏️",
        category: "Fremde Drohnen",
        title: "other_drone.is_mining",
        summary: "Ob eine fremde Drohne schürft.",
        what: "Ein bool: true, wenn die fremde Drohne gerade Erz abbaut.",
        how: "Nutze das Feld direkt als Bedingung.",
        example: "if fremde.is_mining {
    // sie konkurriert um Erz
}",
    },
    Command {
        key: "OTHER_BASE_ID",
        box_step: 35,
        icon: "🪪",
        category: "Fremde Basen",
        title: "other_base.base_id",
        summary: "Die Nummer einer fremden Basis.",
        what: "Eine ganze Zahl (u32): die ID der fremden Basis.",
        how: "Lies das Feld aus einer Basis aus world.other_bases.",
        example: "println!(\"{}\", fremde_basis.base_id);",
    },
    Command {
        key: "OTHER_BASE_POSITION",
        box_step: 35,
        icon: "📍",
        category: "Fremde Basen",
        title: "other_base.position",
        summary: "Wo eine fremde Basis liegt.",
        what: "Ein Vec2 mit den Koordinaten der fremden Basis.",
        how: "Übergib die Position an distance_to-Methoden.",
        example: "let d = drone.distance_to(&fremde_basis.position);",
    },
    Command {
        key: "OTHER_BASE_SIGNAL_RADIUS",
        box_step: 35,
        icon: "📡",
        category: "Fremde Basen",
        title: "other_base.signal_radius",
        summary: "Der Funkradius einer fremden Basis.",
        what: "Eine Kommazahl (f32): innerhalb dieses Kreises ist die fremde Basis geschützt.",
        how: "Vergleiche ihn mit dem Abstand deiner Drohne.",
        example: "if d > fremde_basis.signal_radius {
    // außerhalb ihres Kreises
}",
    },
    Command {
        key: "OTHER_BASE_LEVEL",
        box_step: 35,
        icon: "🏅",
        category: "Fremde Basen",
        title: "other_base.level",
        summary: "Die Ausbaustufe einer fremden Basis.",
        what: "Eine kleine Zahl (u8) mit dem Level der fremden Basis.",
        how: "Lies das Feld aus einer Basis aus world.other_bases.",
        example: "println!(\"Level {}\", fremde_basis.level);",
    },
    Command {
        key: "OTHER_BASE_SCANNER_LEVELS",
        box_step: 35,
        icon: "🔭",
        category: "Fremde Basen",
        title: "other_base.drone_scanner_level",
        summary: "Die Scanner-Stufen einer fremden Basis.",
        what: "Kleine Zahlen (u8): drone_scanner_level, base_scanner_level und radar_horizon_level zeigen, wie weit die fremde Basis sieht.",
        how: "Lies die Felder einzeln aus einer Basis aus world.other_bases.",
        example: "let s = fremde_basis.radar_horizon_level;",
    },
    Command {
        key: "RIDDLE_DISTANCE",
        box_step: 37,
        icon: "📐",
        category: "Forschung",
        title: "RiddleTask::Distance { base, targets } / RiddleSolution::Distance(d)",
        summary: "Rätsel für Zweig 1 (Drohnen-Scanner): Abstände berechnen.",
        what: "Beim Forschungszweig UpgradeTrack::DroneScanner erhältst du die Position base (Vec2) und eine Liste von Zielpunkten targets (&Vec<Vec2>). Für jeden Zielpunkt berechnest du den Abstand zu base.",
        how: "Gehe targets durch, berechne für jeden Punkt base.distance(t) und sammle die Ergebnisse in einem Vec<f32>, den du mit RiddleSolution::Distance(...) einreichst.",
        example: "if let Some(RiddleTask::Distance { base, targets }) = world.active_riddle {
    let dists = targets.iter().map(|t| base.distance(t)).collect();
    actions.solve_riddle(RiddleSolution::Distance(dists));
}",
    },
    Command {
        key: "RIDDLE_DENOISE",
        box_step: 37,
        icon: "📡",
        category: "Forschung",
        title: "RiddleTask::Denoise { readings, threshold } / RiddleSolution::Denoise(c)",
        summary: "Rätsel für Zweig 3 (Radar-Horizont): Störsignale herausfiltern.",
        what: "Beim Zweig UpgradeTrack::RadarHorizon erhältst du Messwerte readings (&Vec<f32>) und einen Schwellenwert threshold (&f32). Ein Messwert ist ein isoliertes Störsignal (Spike), wenn er um mehr als threshold vom Durchschnitt seiner beiden direkten Nachbarn abweicht (Randwerte am Anfang und Ende bleiben immer erhalten).",
        how: "Prüfe für jeden inneren Index i (von 1 bis len - 2), ob (readings[i] - (readings[i - 1] + readings[i + 1]) / 2.0).abs() <= *threshold gilt, und sammle alle gültigen Werte in einem Vec<f32> für RiddleSolution::Denoise.",
        example: "if let Some(RiddleTask::Denoise { readings, threshold }) = world.active_riddle {
    let mut cleaned = Vec::new();
    for i in 0..readings.len() {
        if i == 0 || i + 1 == readings.len() || (readings[i] - (readings[i - 1] + readings[i + 1]) / 2.0).abs() <= *threshold {
            cleaned.push(readings[i]);
        }
    }
    actions.solve_riddle(RiddleSolution::Denoise(cleaned));
}",
    },
    Command {
        key: "RIDDLE_ENERGY",
        box_step: 37,
        icon: "⚡",
        category: "Forschung",
        title: "RiddleTask::Energy { consumers, budget } / RiddleSolution::Energy(n)",
        summary: "Rätsel für Zweig 4 (Energieproduktion): Maximale Verbraucherzahl im Budget.",
        what: "Beim Zweig UpgradeTrack::EnergyProduction erhältst du eine Liste von Energieverbrauchern consumers (&Vec<i64>) und ein Gesamtbudget budget (&i64). Gesucht ist die maximale Anzahl an Verbrauchern, die zusammen höchstens budget verbrauchen (kleinste zuerst).",
        how: "Klone und sortiere consumers aufsteigend, ziehe der Reihe nach jeden Wert vom verbleibenden Budget ab, solange er hineinpasst, und zähle mit. Übergib die Anzahl als RiddleSolution::Energy(count).",
        example: "if let Some(RiddleTask::Energy { consumers, budget }) = world.active_riddle {
    let mut list = consumers.clone();
    list.sort();
    let mut rem = *budget;
    let mut count = 0;
    for c in list {
        if c <= rem { rem -= c; count += 1; }
    }
    actions.solve_riddle(RiddleSolution::Energy(count));
}",
    },
    Command {
        key: "BASE_RESEARCH_LEVELS",
        box_step: 37,
        icon: "🔬",
        category: "Forschung",
        title: "world.base.*_level / *_progress / *_range()",
        summary: "Forschungsstufen, Fortschritt (0–99) und aktuelle Scanreichweiten der Basis.",
        what: "Lies für alle vier Zweige Stufe und Fortschritt aus: drone_scanner_level / _progress, base_scanner_level / _progress, radar_horizon_level / _progress und energy_production_level / _progress. Dazu liefern drone_scanner_range(), base_scanner_range() und radar_horizon_range() die aktuellen Reichweiten in Metern.",
        how: "Frage die Stufen ab, um automatisch den Forschungszweig mit der niedrigsten Stufe weiterzuentwickeln.",
        example: "if world.active_riddle.is_none() && world.base.energy_production_level < 5 {
    actions.request_research(UpgradeTrack::EnergyProduction);
}",
    },
    Command {
        key: "VEC2_ZERO",
        box_step: 37,
        icon: "0️⃣",
        category: "Rust-Syntax",
        title: "Vec2::ZERO",
        summary: "Der Punkt (0, 0).",
        what: "Eine Konstante für den Ursprung des Koordinatensystems, wo die Mega-Station liegt.",
        how: "Schreibe sie ohne Klammern, z. B. als Ziel oder Vergleichswert.",
        example: "let abstand = drone.distance_to(&Vec2::ZERO);",
    },
    Command {
        key: "VEC2_XY",
        box_step: 37,
        icon: "↔️",
        category: "Rust-Syntax",
        title: "pos.x, pos.y",
        summary: "Die beiden Koordinaten eines Punkts.",
        what: "Jeder Vec2 hat die Felder x (waagerecht) und y (senkrecht) als Kommazahlen (f32).",
        how: "Lies sie ohne Klammern aus, z. B. drone.position.x. Mit {:.0} im println! rundest du sie.",
        example: "println!(\"{:.0} / {:.0}\", drone.position.x, drone.position.y);",
    },
    Command {
        key: "WORLD_TICK",
        box_step: 37,
        icon: "⏱️",
        category: "Welt & Sensoren",
        title: "world.tick",
        summary: "Der aktuelle Zeitschritt.",
        what: "Eine ganze Zahl (u64), die mit jedem Zeitschritt des Servers hochzählt. Damit kannst du Abläufe über Zeit steuern.",
        how: "Nutze den Rest einer Division (%), um etwas nur alle paar Schritte zu tun.",
        example: "if world.tick % 100 == 0 {
    println!(\"Schritt {}\", world.tick);
}",
    },
    Command {
        key: "BASE_LEVEL",
        box_step: 37,
        icon: "🏅",
        category: "Basis-Status",
        title: "world.base.level",
        summary: "Die Ausbaustufe der Basis.",
        what: "Eine kleine Zahl (u8) mit dem Level der Heimatbasis.",
        how: "Lies das Feld aus und gib es z. B. mit println! aus.",
        example: "println!(\"Basis-Level {}\", world.base.level);",
    },
    Command {
        key: "BASE_DRONE_SCANNER_LEVEL",
        box_step: 37,
        icon: "🔭",
        category: "Basis-Status",
        title: "world.base.drone_scanner_level",
        summary: "Die Stufe des Drohnen-Scanners.",
        what: "Eine kleine Zahl (u8): Je höher, desto weiter sehen deine Drohnen. Du erhöhst sie mit dem Forschungszweig DroneScanner.",
        how: "Lies das Feld aus, um zu prüfen, ob sich weitere Forschung lohnt.",
        example: "if world.base.drone_scanner_level < 3 {
    // weiter forschen
}",
    },
    Command {
        key: "BASE_BASE_SCANNER_LEVEL",
        box_step: 37,
        icon: "🛰️",
        category: "Basis-Status",
        title: "world.base.base_scanner_level",
        summary: "Die Stufe des Basen- und Asteroidenscanners.",
        what: "Eine kleine Zahl (u8): Je höher, desto weiter erkennt die Basis Asteroiden und fremde Basen. Forschungszweig: BaseAsteroidScanner.",
        how: "Lies das Feld aus, um zu prüfen, ob sich weitere Forschung lohnt.",
        example: "let stufe = world.base.base_scanner_level;",
    },
    Command {
        key: "BASE_RADAR_HORIZON_LEVEL",
        box_step: 37,
        icon: "🌐",
        category: "Basis-Status",
        title: "world.base.radar_horizon_level",
        summary: "Die Stufe des Radar-Horizonts.",
        what: "Eine kleine Zahl (u8): Je höher, desto weiter siehst du unbekannte Kontakte. Forschungszweig: RadarHorizon.",
        how: "Lies das Feld aus, um zu prüfen, ob sich weitere Forschung lohnt.",
        example: "let stufe = world.base.radar_horizon_level;",
    },
    Command {
        key: "BASE_DRONE_SCANNER_RANGE",
        box_step: 37,
        icon: "🔭",
        category: "Basis-Status",
        title: "world.base.drone_scanner_range()",
        summary: "Die Reichweite der Drohnen-Scanner.",
        what: "Liefert die Sichtweite deiner Drohnen als Kommazahl (f32): Stufe 1 sind 150, jede weitere Stufe bringt 50 dazu.",
        how: "Rufe die Methode ohne Parameter auf und vergleiche sie mit Entfernungen.",
        example: "let sicht = world.base.drone_scanner_range();",
    },
    Command {
        key: "BASE_BASE_SCANNER_RANGE",
        box_step: 37,
        icon: "🛰️",
        category: "Basis-Status",
        title: "world.base.base_scanner_range()",
        summary: "Die Reichweite des Basen- und Asteroidenscanners.",
        what: "Liefert die Sichtweite der Basis für Asteroiden und fremde Basen als Kommazahl (f32): Stufe 1 sind 300, jede weitere Stufe bringt 100 dazu.",
        how: "Rufe die Methode ohne Parameter auf und vergleiche sie mit Entfernungen.",
        example: "let sicht = world.base.base_scanner_range();",
    },
    Command {
        key: "BASE_RADAR_HORIZON_RANGE",
        box_step: 37,
        icon: "🌐",
        category: "Basis-Status",
        title: "world.base.radar_horizon_range()",
        summary: "Die Reichweite des Radar-Horizonts.",
        what: "Liefert die Sichtweite für unbekannte Kontakte als Kommazahl (f32): Stufe 1 sind 600, jede weitere Stufe bringt 150 dazu.",
        how: "Rufe die Methode ohne Parameter auf und vergleiche sie mit Entfernungen.",
        example: "let sicht = world.base.radar_horizon_range();",
    },
    Command {
        key: "UPGRADE_TRACK_BASE_ASTEROID",
        box_step: 37,
        icon: "🛰️",
        category: "Forschung",
        title: "UpgradeTrack::BaseAsteroidScanner",
        summary: "Forschungszweig für den Basen- und Asteroidenscan.",
        what: "Erhöht die Scan-Reichweite für Asteroiden und fremde Basen von 300 auf bis zu 700. Das Rätsel dazu ist eine Sortieraufgabe.",
        how: "Übergib den Zweig an actions.request_research.",
        example: "actions.request_research(UpgradeTrack::BaseAsteroidScanner);",
    },
    Command {
        key: "UPGRADE_TRACK_RADAR_HORIZON",
        box_step: 37,
        icon: "🌐",
        category: "Forschung",
        title: "UpgradeTrack::RadarHorizon",
        summary: "Forschungszweig für den Radar-Horizont.",
        what: "Erhöht die Reichweite für unbekannte Kontakte von 600 auf bis zu 1200. Das Rätsel dazu ist das Entfernen von Störsignalen.",
        how: "Übergib den Zweig an actions.request_research.",
        example: "actions.request_research(UpgradeTrack::RadarHorizon);",
    },
    Command {
        key: "UPGRADE_TRACK_NAME",
        box_step: 37,
        icon: "🏷️",
        category: "Forschung",
        title: "track.name()",
        summary: "Der lesbare Name eines Forschungszweigs.",
        what: "Liefert einen deutschen Text wie \"Drohnen-Scanner\" für einen UpgradeTrack.",
        how: "Rufe die Methode auf einem UpgradeTrack auf und gib das Ergebnis z. B. mit println! aus.",
        example: "println!(\"{}\", UpgradeTrack::RadarHorizon.name());",
    },
    Command {
        key: "RIDDLE_TASK_DISTANCE",
        box_step: 37,
        icon: "📐",
        category: "Forschung",
        title: "RiddleTask::Distance { base, targets }",
        summary: "Das Distanzrätsel des Drohnen-Scanners.",
        what: "Enthält den Punkt base und eine Liste targets mit Zielpunkten. Gesucht sind die Abstände von base zu jedem Ziel.",
        how: "Packe es mit if let Some(RiddleTask::Distance { base, targets }) aus und berechne mit base.distance(&ziel) jeden Abstand.",
        example: "if let Some(RiddleTask::Distance { base, targets }) = world.active_riddle {
    // Abstände berechnen
}",
    },
    Command {
        key: "RIDDLE_TASK_DENOISE",
        box_step: 37,
        icon: "🔇",
        category: "Forschung",
        title: "RiddleTask::Denoise { readings, threshold }",
        summary: "Das Rätsel des Radar-Horizonts.",
        what: "Enthält eine Messreihe readings und einen Schwellenwert threshold. Werte über der Schwelle sind Störsignale und müssen entfernt werden.",
        how: "Packe es mit if let Some(RiddleTask::Denoise { readings, threshold }) aus und behalte nur Werte unter der Schwelle.",
        example: "if let Some(RiddleTask::Denoise { readings, threshold }) = world.active_riddle {
    // Störsignale entfernen
}",
    },
    Command {
        key: "RIDDLE_SOLUTION_DISTANCE",
        box_step: 37,
        icon: "✅",
        category: "Forschung",
        title: "RiddleSolution::Distance(distances)",
        summary: "Die Lösung des Distanz-Rätsels.",
        what: "Verpackt die Liste der berechneten Abstände (f32) als Antwort auf RiddleTask::Distance.",
        how: "Übergib sie an actions.solve_riddle.",
        example: "actions.solve_riddle(RiddleSolution::Distance(distances));",
    },
    Command {
        key: "RIDDLE_SOLUTION_DENOISE",
        box_step: 37,
        icon: "✅",
        category: "Forschung",
        title: "RiddleSolution::Denoise(cleaned)",
        summary: "Die Lösung des Rausch-Rätsels.",
        what: "Verpackt die bereinigte Messreihe (f32) als Antwort auf RiddleTask::Denoise.",
        how: "Übergib sie an actions.solve_riddle.",
        example: "actions.solve_riddle(RiddleSolution::Denoise(cleaned));",
    },
];

// ==========================================
// Unit Tests (Zero External Dependencies)
// ==========================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steps_are_contiguous_from_0_to_final_step() {
        assert_eq!(STEPS.len(), (FINAL_STEP + 1) as usize);
        for (i, s) in STEPS.iter().enumerate() {
            assert_eq!(s.index as usize, i);
        }
    }

    #[test]
    fn all_commands_count_is_103() {
        assert_eq!(COMMANDS.len(), 103);
    }

    #[test]
    fn all_command_keys_are_unique_and_discoverable() {
        for c in &COMMANDS {
            assert_eq!(command(c.key).map(|x| x.key), Some(c.key));
        }
    }

    #[test]
    fn all_unlocked_and_reference_keys_exist_in_command_catalog() {
        for s in &STEPS {
            for key in s.unlocked_keys {
                assert!(
                    command(key).is_some(),
                    "Step {} unlocked non-existent key: {}",
                    s.index,
                    key
                );
            }
            for key in s.reference_keys {
                assert!(
                    command(key).is_some(),
                    "Step {} referenced non-existent key: {}",
                    s.index,
                    key
                );
            }
        }
    }

    #[test]
    fn loot_box_steps_match_constant_and_alternate() {
        let box_indices: [u8; 16] = [
            STEPS[5].index,
            STEPS[7].index,
            STEPS[9].index,
            STEPS[11].index,
            STEPS[13].index,
            STEPS[15].index,
            STEPS[17].index,
            STEPS[19].index,
            STEPS[21].index,
            STEPS[23].index,
            STEPS[25].index,
            STEPS[27].index,
            STEPS[29].index,
            STEPS[31].index,
            STEPS[33].index,
            STEPS[35].index,
        ];
        assert_eq!(box_indices, LOOT_BOX_STEPS);
        for s in &LOOT_BOX_STEPS {
            assert!(is_loot_box_step(*s));
            assert!(STEPS[*s as usize].is_loot_box());
        }
    }

    #[test]
    fn coding_steps_reference_preceding_loot_box_keys() {
        for s in [6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 32, 34, 36] {
            let coding_step = &STEPS[s];
            let preceding_box = &STEPS[s - 1];
            assert_eq!(
                coding_step.reference_keys,
                preceding_box.unlocked_keys,
                "Coding step {} did not reference preceding box {} keys",
                s,
                s - 1
            );
        }
    }

    #[test]
    fn catch_up_command_keys_accumulates_correctly() {
        // Step 6 catch-up: should include step 5 cards
        let keys_6: [_; 5] = [
            "IF_LET_SOME",
            "WORLD_FIRST_DRONE",
            "PRINTLN",
            "DRONE_POSITION",
            "FORMAT_NO_DECIMALS",
        ];
        let caught_up_6: [_; 5] = {
            let mut arr = [""; 5];
            for (slot, k) in arr.iter_mut().zip(STEPS[6].catch_up_command_keys()) {
                *slot = k;
            }
            arr
        };
        assert_eq!(caught_up_6, keys_6);
    }
}
