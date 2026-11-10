//! `curriculum`: Centralized declarative catalog of tutorial steps, quests, and command reference cards
//! for the `programmierspiel` workspace.
//!
//! Zero external dependencies (`no_std` compatible). Used by `server`, `client-web`, `admin`, and `space_bot_sdk`.

#![no_std]

/// Maximum tutorial step index (Free Play).
pub const FINAL_STEP: u8 = 22;

/// Tutorial steps at which a command loot box is waiting in the Heimatbasis.
pub const LOOT_BOX_STEPS: [u8; 9] = [5, 7, 9, 11, 13, 15, 17, 19, 21];

/// Returns `true` if a loot box is waiting at this tutorial step.
pub const fn is_loot_box_step(step: u8) -> bool {
    matches!(step, 5 | 7 | 9 | 11 | 13 | 15 | 17 | 19 | 21)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepKind {
    /// Initial onboarding/setup (steps 0..=4).
    Onboarding,
    /// Command lootbox in home base (odd steps 5..=21).
    LootBox { number: u8 },
    /// Coding quest exercising unlocked commands (even steps 6..=20).
    Coding,
    /// Tutorial complete; Free-Play sandbox and remaining SDK catalog unlocked (step 22).
    FreePlay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationMode {
    /// Step advanced manually by learner (e.g. Step 0, Step 3).
    ManualAdvance,
    /// Step advanced on claiming all loot box cards (odd steps 5..=21).
    LootBoxClaim,
    /// Verified via interactive coordinate input (Step 6).
    CoordinateQuiz,
    /// Verified via interactive asteroid ID input (Step 8).
    AsteroidIdQuiz,
    /// Verified by server-side game events (mining, docking, building, research).
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

/// Complete catalog of all 54 command reference cards.
pub const fn all_commands() -> &'static [Command] {
    &COMMANDS
}

/// Complete catalog of all 23 tutorial steps (0..=22).
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

/// Additional SDK commands unlocked automatically once reaching Free Play (step 22).
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
];
const KEYS_STEP_15: &[&str] = &["FOR_IN_FLEET"];
const KEYS_STEP_17: &[&str] = &["DRONE_IS_DOCKED", "DRONE_ENERGY", "ELSE_IF"];
const KEYS_STEP_19: &[&str] = &[
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
];
const KEYS_STEP_21: &[&str] = &["ACTIONS_BUILD_DRONE_2"];

const KEYS_FREE_PLAY: &[&str] = &[
    "ACTIONS_FLY_TO",
    "ACTIONS_STOP",
    "VEC2_NEW",
    "VEC2_DISTANCE",
    "WORLD_SHIP",
    "WORLD_CLOSEST_ASTEROID_ANY",
    "WORLD_ASTEROIDS",
    "WORLD_OTHER_DRONES",
    "WORLD_OTHER_BASES",
    "WORLD_TICK",
    "DRONE_TIER",
    "DRONE_CARGO",
    "DRONE_MOTION",
    "DRONE_TARGETS_AND_STATUS",
    "DRONE_SIGNAL_AND_STRANDED",
    "DRONE_DISTANCE_TO",
    "BASE_STORED_ENERGY",
    "BASE_POSITION_AND_SIGNAL",
    "BASE_RESEARCH_LEVELS",
    "ASTEROID_POSITION_AND_DISTANCE",
    "ASTEROID_ORE_AND_TYPE",
    "SDK_DRONE_COSTS",
];

const EMPTY_KEYS: &[&str] = &[];

// ==========================================
// Static Steps Catalog (0..=22)
// ==========================================

pub static STEPS: [Step; 23] = [
    // Step 0
    Step {
        index: 0,
        kind: StepKind::Onboarding,
        verification: VerificationMode::ManualAdvance,
        title: "Willkommen an Bord, Commander!",
        short_title: "Registrierung",
        admin_label: "0/22 Registrierung",
        story: "Im Asteroidengürtel wimmelt es von automatischen Transportern und fremden Stationen. Um deine Schiffe auf den Radarsystemen der Mega-Station eindeutig zu identifizieren, benötigt deine Flotte deinen Pilotenausweis und eigene Signal-Farben.",
        subtasks: &[
            SubTask { num: "1", text: "Wähle mit den Farbreglern deine Primär- und Sekundärfarbe für dein Raumschiff aus.", code_hint: None },
            SubTask { num: "2", text: "Prüfe die Vorschau deines Drohnen-Raumschiffs im Hangar.", code_hint: None },
            SubTask { num: "3", text: "Klicke auf 'Pilotenausweis erstellen & starten'.", code_hint: None },
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
        admin_label: "1/22 Flug",
        story: "Dein Bauschiff (Tier 0) startet nah herangezoomt an der Mega-Station bei (0, 0). Bedienung der Karte: Mit dem Mausrad (Scrollen) zoomst du rein und raus, und durch Klicken und Halten mit der linken Maustaste verschiebst du die Karte. Wichtig: Du kannst deine Basis nur außerhalb des roten bzw. grünen Sicherheitskreises um die Mega-Station errichten!",
        subtasks: &[
            SubTask { num: "1", text: "Scrolle mit dem Mausrad heraus, um die Umgebung zu überblicken (Linksklick gedrückt halten bewegt die Karte).", code_hint: None },
            SubTask { num: "2", text: "Klicke kurz mit der linken Maustaste auf einen Zielpunkt außerhalb des roten Kreises, um dein Bauschiff dorthin zu steuern, bis der Kreis grün wird (Abstand ≥ 250).", code_hint: None },
            SubTask { num: "3", text: "Achte rechts im Panel bei Position & Geschwindigkeit auf den Asteroiden-Scanradius: Suche einen Platz, an dem mindestens 400 Erz in Reichweite sind!", code_hint: None },
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
        admin_label: "2/22 Basisbau",
        story: "Hervorragend gewählt, Commander! Du befindest dich außerhalb des grünen Sicherheitskreises der Mega-Station und hast genügend Erz im Scanradius. Durch die Aktivierung der Montage-Sequenz entfaltet sich dein Bauschiff zu deiner Heimatbasis.",
        subtasks: &[
            SubTask { num: "1", text: "Vergewissere dich, dass du außerhalb des roten/grünen Kreises stehst (Abstand ≥ 250) und rechts im Panel mindestens 400 Erz im Scanradius angezeigt werden.", code_hint: None },
            SubTask { num: "2", text: "Klicke unten rechts im Seitenpanel auf den leuchtenden Button '🏛️ Heimatbasis hier errichten'.", code_hint: None },
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
        admin_label: "3/22 Bot Start",
        story: "Deine Heimatbasis hat einen Bordcomputer. Du programmierst ihn in Rust und startest deinen Bot im Zed-Editor.",
        subtasks: &[
            SubTask { num: "1", text: "Öffne uhlaunch auf dem Desktop und starte 'programmierspiel'. Der Zed-Editor öffnet sich.", code_hint: None },
            SubTask { num: "2", text: "Öffne links im Ordner src die Datei main.rs. Klicke neben fn main() auf den Run-Button ▶, um den Bot zu starten.", code_hint: Some("▶ fn main()") },
            SubTask { num: "3", text: "Suche im Terminal unten nach 'Verbindung erfolgreich!' und der darunter ausgegebenen Identität. Bestätige anschließend hier, dass der Bot gestartet ist.", code_hint: None },
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
        admin_label: "4/22 Bot-Kopplung",
        story: "Die Identität ist die Adresse deines Bots; mit ihr erlaubst du ihm die Steuerung deiner Basis. Ein Token ist dagegen ein geheimer Zugangsschlüssel: Gib ihn niemals als Identität ein oder an andere weiter. Beim ersten Start speichert der Bot seinen Token selbst in config.toml.",
        subtasks: &[
            SubTask { num: "1", text: "Kopiere im Zed-Terminal die 64-stellige Zeichenfolge hinter 'Identität:' – nicht den Token.", code_hint: None },
            SubTask { num: "2", text: "Öffne die Heimatbasis und füge die Identität unter 'Heimatbasis-AI Steuerung' ein. Klicke auf 'Bot binden'.", code_hint: None },
            SubTask { num: "3", text: "Lass den Bot laufen. Die Mission wartet automatisch auf die vom Server bestätigte Kopplung.", code_hint: None },
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
        admin_label: "5/22 🎁 Lootbox 1: Drohne",
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
        admin_label: "6/22 Drohne",
        story: "In main.rs wird tick etwa zehnmal pro Sekunde aufgerufen. world enthält den aktuellen Weltzustand, actions sammelt deine Befehle. Schreibe deinen Code in tick: Nur wenn eine Drohne vorhanden ist, kannst du ihre Position auslesen.",
        subtasks: &[
            SubTask { num: "1", text: "Wähle die erste Drohne mit world.first_drone() und if let Some aus. Bei None wird der Block übersprungen.", code_hint: Some("if let Some(drone) = world.first_drone() { ... }") },
            SubTask { num: "2", text: "Setze im Block selbst ein println! zusammen: Gib drone.position.x und drone.position.y mit je {:.0} ohne Nachkommastellen aus. {:.0} rundet die angezeigte Zahl.", code_hint: Some("println!(\"... {:.0} ... {:.0} ...\", ..., ...);") },
            SubTask { num: "3", text: "Speichere, starte den Bot neu und übertrage die beiden ausgegebenen ganzen Zahlen in die Felder unten.", code_hint: None },
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
        admin_label: "7/22 🎁 Lootbox 2: Asteroid",
        story: "Mit dem nächsten Befehl findest du bekannte Asteroiden, die noch Erz enthalten, und liest ihre Kennnummer aus.",
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
        admin_label: "8/22 Asteroid",
        story: "Deine Sensoren identifizieren Asteroiden in Reichweite der Basis oder deiner Drohnen. Jede so identifizierte Asteroiden-ID ist gültig; die Suche wählt davon einen passenden Asteroiden mit Erz.",
        subtasks: &[
            SubTask { num: "1", text: "Suche innerhalb des Drohnen-Blocks mit world.closest_asteroid_with_ore(drone.position). Packe das mögliche Ergebnis mit if let Some aus; bei None wurde kein passender Asteroid gefunden.", code_hint: Some("if let Some(asteroid) = world.closest_asteroid_with_ore(...) { ... }") },
            SubTask { num: "2", text: "Gib im Asteroiden-Block asteroid.asteroid_id mit println! im Terminal aus. Setze die Ausgabe selbst zusammen.", code_hint: Some("println!(\"ID: {}\", ...);") },
            SubTask { num: "3", text: "Speichere, starte den Bot neu und gib die ausgegebene ID unten ein.", code_hint: None },
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
        admin_label: "9/22 🎁 Lootbox 3: Mining",
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
        admin_label: "10/22 Erstes Erz",
        story: "Deine Drohne und ein Asteroid mit Erz sind gefunden. Gib den Abbau-Befehl in tick: Die Drohne fliegt erst zum Ziel und schürft dort. Die Mission endet, sobald der Server erfolgreichen Abbau registriert.",
        subtasks: &[
            SubTask { num: "1", text: "Verwende die gefundenen Werte für Drohne und Asteroid in den verschachtelten if-let-Blöcken.", code_hint: None },
            SubTask { num: "2", text: "Erteile dort den Befehl mit Drohnen-ID und Asteroiden-ID.", code_hint: Some("actions.mine(drone.ship_id, asteroid.asteroid_id);") },
            SubTask { num: "3", text: "Speichere, starte den Bot neu und beobachte den Abbau im Spiel.", code_hint: None },
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
        admin_label: "11/22 🎁 Lootbox 4",
        story: "Erstes Erz im Frachtraum! In deiner Heimatbasis wartet eine Lootbox mit Befehlen zum Verzweigen und Andocken.",
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
        admin_label: "12/22 Lieferung",
        story: "Deine Drohne kann nur 50 Einheiten Erz tragen. Ist der Frachtraum voll, stoppt der Bohrer automatisch. Damit das Erz verarbeitet werden kann, muss die Drohne an der Heimatbasis andocken. Ziel: Sammle mindestens 50 Erz an deiner Basis!",
        subtasks: &[
            SubTask { num: "1", text: "Prüfe, ob der Laderaum der Drohne voll ist:", code_hint: Some("if drone.cargo_is_full() { ... } else { ... }") },
            SubTask { num: "2", text: "Befiehl bei voller Fracht das Andocken:", code_hint: Some("actions.dock(drone.ship_id);") },
            SubTask { num: "3", text: "Beobachte das Radar, bis die Basis mindestens 50 Erz gelagert hat.", code_hint: None },
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
        admin_label: "13/22 🎁 Lootbox 5",
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
        admin_label: "14/22 Drohne 2",
        story: "Mit 50 Einheiten Erz im Lager verfügt deine Basis über genügend Rohstoffe, um ein weiteres Schiff der Stufe 1 in Auftrag zu geben. Mehr Drohnen bedeuten schnellere Rohstoffförderung!",
        subtasks: &[
            SubTask { num: "1", text: "Prüfe, ob du weniger als 2 Drohnen und mindestens 50 Erz hast:", code_hint: Some("if world.fleet.len() < 2 && world.base.stored_ore >= 50 { ... }") },
            SubTask { num: "2", text: "Gib den Bau einer Stufe-1-Drohne in Auftrag:", code_hint: Some("actions.build_drone(1);") },
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
        admin_label: "15/22 🎁 Lootbox 6",
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
        admin_label: "16/22 Flottenloop",
        story: "Bisher hat dein Code nur die erste Drohne mit world.first_drone() gesteuert. In Rust nutzen wir eine for-Schleife, um jedem Schiff in world.fleet automatisch denselben Ablauf zuzuweisen.",
        subtasks: &[
            SubTask { num: "1", text: "Ersetze world.first_drone() durch eine for-Schleife über alle Schiffe:", code_hint: Some("for drone in world.fleet { ... }") },
            SubTask { num: "2", text: "Befiehl jedem Schiff: Wenn voll, docken; sonst erzhaltigen Asteroiden abbauen.", code_hint: None },
            SubTask { num: "3", text: "Beobachte, wie sich beide Drohnen unabhängig voneinander bewegen.", code_hint: None },
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
        admin_label: "17/22 🎁 Lootbox 7: Energie",
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
        admin_label: "18/22 Energiemanagement",
        story: "Wenn eine Drohne das Erz an der Basis abliefert, ist ihr Frachtraum sofort wieder leer – ohne Zusatzprüfung fliegt sie ungeladen wieder los, bis ihr mitten im All der Strom ausgeht und nur noch der langsame Not-Generator tröpfchenweise hilft. Halte angedockte Drohnen an der Basis, bis ihre Batterie voll ist (1.0), und schicke Drohnen mit niedrigem Batteriestand (< 0.25) rechtzeitig zum Andocken!",
        subtasks: &[
            SubTask { num: "1", text: "Prüfe in deiner for-Schleife zuerst, ob die Drohne angedockt ist und ihre Batterie noch lädt (der Block bleibt leer, damit sie an der Basis weiterlädt):", code_hint: Some("if drone.is_docked() && drone.energy_percent() < 1.0 { }") },
            SubTask { num: "2", text: "Prüfe mit else if, ob der Frachtraum voll oder die Batterie unter 25 % gefallen ist, und befiehl das Andocken:", code_hint: Some("else if drone.cargo_is_full() || drone.energy_percent() < 0.25 { actions.dock(drone.ship_id); }") },
            SubTask { num: "3", text: "Lass die Drohne im else-Zweig wie bisher den nächsten Asteroiden mit Erz abbauen und warte, bis eine Drohne an der Basis vollständig aufgeladen wurde.", code_hint: None },
        ],
        unlocked_keys: EMPTY_KEYS,
        reference_keys: KEYS_STEP_17,
    },
    // Step 19: Lootbox 8
    Step {
        index: 19,
        kind: StepKind::LootBox { number: 8 },
        verification: VerificationMode::LootBoxClaim,
        title: "🎁 Befehls-Lootbox #8: Telemetrie & Algorithmen",
        short_title: "🎁 Lootbox 8: Forschung",
        admin_label: "19/22 🎁 Lootbox 8: Forschung",
        story: "Deine Drohnen laden zuverlässig an der Basis auf! Öffne die Lootbox in der Heimatbasis für die Befehle aller vier Forschungsbäume.",
        subtasks: LOOT_BOX_SUBTASKS,
        unlocked_keys: KEYS_STEP_19,
        reference_keys: EMPTY_KEYS,
    },
    // Step 20: Coding Mission 8
    Step {
        index: 20,
        kind: StepKind::Coding,
        verification: VerificationMode::ServerEvent,
        title: "Forschung: Vier Technologiebäume",
        short_title: "Forschung",
        admin_label: "20/22 Forschung",
        story: "Deine Heimatbasis kann in vier Forschungsbäumen ausgebaut werden, und du darfst frei wählen, welchen du zuerst und in welcher Reihenfolge du sie erforschst: 1) Drohnen-Scanner (größere Scanreichweite der Drohnen, Rätsel: Zahlen sortieren), 2) Basen- & Asteroidenscan (größerer Scanbereich der Basis, Rätsel: Distanzen berechnen), 3) Radar-Horizont (Kontakte weiter erkennen, Rätsel: Störsignale filtern), 4) Energieproduktion (die Basis erzeugt mehr Energie und kann mehr Drohnen versorgen: anfangs 3 Stufe-1-Drohnen, voll erforscht 18; Stufe-2-Drohnen brauchen doppelt so viel Energie; Rätsel: Energiebudget). Drohnen laden an der Basis auf und verbrauchen dabei deren Energie. Löse für diese Aufgabe ein Rätsel eines beliebigen Baums.",
        subtasks: &[
            SubTask { num: "1", text: "Wähle einen beliebigen Forschungsbaum und fordere ein Rätsel an, falls noch keines aktiv ist:", code_hint: Some("if world.active_riddle.is_none() { actions.request_research(UpgradeTrack::DroneScanner); } // oder BaseAsteroidScanner, RadarHorizon, EnergyProduction") },
            SubTask { num: "2", text: "Prüfe auf ein aktives Rätsel und berechne die Lösung passend zum gewählten Baum (z. B. Sort: sortieren, Energy: kleinste Verbraucher zuerst addieren, bis das Budget erreicht ist):", code_hint: Some("if let Some(RiddleTask::Sort { numbers }) = world.active_riddle { let mut sorted = numbers.clone(); sorted.sort(); ... }") },
            SubTask { num: "3", text: "Reiche die Lösung innerhalb des if-Blocks mit actions.solve_riddle(...) ein:", code_hint: Some("actions.solve_riddle(RiddleSolution::Sort(sorted));") },
        ],
        unlocked_keys: EMPTY_KEYS,
        reference_keys: KEYS_STEP_19,
    },
    // Step 21: Lootbox 9
    Step {
        index: 21,
        kind: StepKind::LootBox { number: 9 },
        verification: VerificationMode::LootBoxClaim,
        title: "🎁 Befehls-Lootbox #9: Schwere Flotte",
        short_title: "🎁 Lootbox 9: Tier 2",
        admin_label: "21/22 🎁 Lootbox 9: Tier 2",
        story: "Forschung abgeschlossen! Die letzte Lootbox verrät dir, wie du schwere Stufe-2-Drohnen baust.",
        subtasks: LOOT_BOX_SUBTASKS,
        unlocked_keys: KEYS_STEP_21,
        reference_keys: EMPTY_KEYS,
    },
    // Step 22: Free Play
    Step {
        index: 22,
        kind: StepKind::FreePlay,
        verification: VerificationMode::None,
        title: "Tutorial abgeschlossen – Freies Spiel!",
        short_title: "Sandbox 🚀",
        admin_label: "22/22 Sandbox 🚀",
        story: "Fantastische Arbeit, Commander! Du beherrschst alle Grundlagen der interstellaren Programmierung: Navigation, Abbau, Multitasking-Schleifen und algorithmische Forschung.",
        subtasks: &[
            SubTask { num: "1", text: "Baue schwere Stufe-2-Drohnen mit 3-facher Frachtkapazität.", code_hint: Some("actions.build_drone(2);") },
            SubTask { num: "2", text: "Verteile deine Drohnen gezielt auf unterschiedliche Asteroiden.", code_hint: None },
            SubTask { num: "3", text: "Erforsche den Radar-Horizont und führe die Rangliste auf dem Klassenbeamer an!", code_hint: None },
        ],
        unlocked_keys: KEYS_FREE_PLAY,
        reference_keys: KEYS_FREE_PLAY,
    },
];

// ==========================================
// Static Commands Catalog (54 Cards)
// ==========================================

pub static COMMANDS: [Command; 54] = [
    Command {
        key: "IF_LET_SOME",
        box_step: 5,
        icon: "📦",
        category: "Rust-Syntax",
        title: "if let Some(x) = ...",
        summary: "Packt einen Wert sicher aus, falls er existiert.",
        what: "Option bedeutet: Ein Ergebnis kann vorhanden sein (Some) oder fehlen (None). Zum Beispiel ist keine Drohne verfügbar, wenn die Flotte leer ist. if let Some führt seinen Block nur aus, wenn ein Wert vorhanden ist; bei None wird er übersprungen.",
        how: "Links von = steht Some mit einem selbst gewählten Namen, rechts die Abfrage. Nur innerhalb der geschweiften Klammern steht der ausgepackte Wert unter diesem Namen zur Verfügung.",
        example: "if let Some(drone) = world.first_drone() {\n    // hier existiert eine Drohne\n}",
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
        example: "if let Some(drone) = world.first_drone() {\n    let id = drone.ship_id;\n}",
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
        example: "let x = drone.position.x;\nlet y = drone.position.y;",
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
        key: "WORLD_CLOSEST_ASTEROID",
        box_step: 7,
        icon: "☄️",
        category: "Welt & Sensoren",
        title: "world.closest_asteroid_with_ore(pos)",
        summary: "Findet den nächsten bekannten Asteroiden mit Erz.",
        what: "Durchsucht die von Basis- oder Drohnenscannern identifizierten Asteroiden und liefert den nächsten mit Erz. Ohne passenden bekannten Asteroiden ist das Ergebnis None. closest_asteroid sucht dagegen auch leere Asteroiden.",
        how: "Übergib die Position, von der aus gesucht wird, zum Beispiel drone.position. Packe das Option mit if let Some aus. Der Asteroid besitzt das Feld asteroid_id.",
        example: "if let Some(asteroid) = world.closest_asteroid_with_ore(drone.position) {\n    // asteroid.asteroid_id verwenden\n}",
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
        example: "if let Some(drone) = world.first_drone() {\n    println!(\"Drohne #{}\", drone.ship_id);\n}",
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
        example: "if drone.cargo_is_full() {\n    // Fracht abliefern\n} else {\n    // weiter schürfen\n}",
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
        example: "if drone.cargo_is_full() {\n    actions.dock(drone.ship_id);\n}",
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
        example: "if world.fleet.len() < 2 {\n    // Platz für eine weitere Drohne\n}",
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
        example: "if world.base.stored_ore >= 50 {\n    // genug Erz vorhanden\n}",
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
        example: "if world.fleet.len() < 2 {\n    // weniger als 2 Schiffe\n}",
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
        example: "if world.fleet.len() < 2 && world.base.stored_ore >= 50 {\n    actions.build_drone(1);\n}",
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
        example: "if world.fleet.len() < 2 && world.base.stored_ore >= 50 {\n    actions.build_drone(1);\n}",
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
        example: "for drone in world.fleet {\n    if drone.cargo_is_full() {\n        actions.dock(drone.ship_id);\n    }\n}",
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
        example: "if drone.is_docked() {\n    // Drohne steht an der Heimatbasis\n}",
    },
    Command {
        key: "DRONE_ENERGY",
        box_step: 17,
        icon: "🔋",
        category: "Schiffs-Status",
        title: "drone.energy / drone.max_energy / drone.energy_percent()",
        summary: "Batteriestand, maximale Kapazität und Anteil von 0.0 bis 1.0.",
        what: "Fliegen und Schürfen verbrauchen Batterie-Energie. Sinkt drone.energy auf 0, bleibt die Drohne liegen und muss auf den langsamen Not-Generator warten. drone.energy_percent() gibt den Füllstand als Zahl zwischen 0.0 (leer) und 1.0 (voll) zurück.",
        how: "Prüfe z. B. mit drone.energy_percent() < 0.25, ob weniger als 25 % übrig sind, und mit drone.energy_percent() < 1.0, ob die Batterie beim Laden noch nicht ganz voll ist.",
        example: "if drone.energy_percent() < 0.25 {\n    actions.dock(drone.ship_id);\n}",
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
        example: "if drone.is_docked() && drone.energy_percent() < 1.0 {\n    // an der Basis weiterladen\n} else if drone.cargo_is_full() || drone.energy_percent() < 0.25 {\n    actions.dock(drone.ship_id);\n} else {\n    // weiter schürfen\n}",
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
        example: "if let Some(RiddleTask::Sort { numbers }) = world.active_riddle {\n    // numbers ist die Zahlenreihe\n}",
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
        example: "if world.active_riddle.is_none() {\n    actions.request_research(UpgradeTrack::DroneScanner);\n}",
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
        example: "if world.active_riddle.is_none() {\n    actions.request_research(UpgradeTrack::DroneScanner);\n}",
    },
    Command {
        key: "LET_BINDING",
        box_step: 19,
        icon: "📝",
        category: "Rust-Syntax",
        title: "let x = ...; / let mut x = ...;",
        summary: "Speichert ein Zwischenergebnis unter einem Namen.",
        what: "Mit let legst du eine Variable an, um einen berechneten Wert zu merken. Standardmäßig sind Variablen in Rust unveränderbar; schreibst du let mut, darfst du den Inhalt danach verändern (z. B. hochzählen oder sortieren).",
        how: "Schreibe let (oder let mut), dann den Variablennamen, ein = und den Startwert, abgeschlossen mit einem Semikolon.",
        example: "let mut count = 0;\ncount += 1;",
    },
    Command {
        key: "CLONE_AND_SORT",
        box_step: 19,
        icon: "📑",
        category: "Rust-Syntax",
        title: "let mut sorted = numbers.clone(); sorted.sort();",
        summary: "Legt eine veränderbare Kopie einer Liste an und sortiert sie.",
        what: ".clone() erzeugt eine eigene Kopie einer Liste, damit die Daten in world unberührt bleiben. let mut macht die Kopie veränderbar, und .sort() ordnet ganze Zahlen aufsteigend vom kleinsten zum größten Wert.",
        how: "Kopiere die Liste mit .clone() in eine let mut-Variable und rufe danach .sort() auf dieser Variable auf.",
        example: "let mut sorted = numbers.clone();\nsorted.sort();",
    },
    Command {
        key: "RIDDLE_SORT",
        box_step: 19,
        icon: "🔢",
        category: "Forschung",
        title: "RiddleTask::Sort { numbers } / RiddleSolution::Sort(sorted)",
        summary: "Rätsel für Zweig 1 (Drohnen-Scanner): Zahlenreihe aufsteigend sortieren.",
        what: "Beim Forschungszweig UpgradeTrack::DroneScanner erhältst du eine unsortierte Liste ganzer Zahlen (numbers: &Vec<i64>). Als Lösung reichst du dieselben Zahlen aufsteigend sortiert als RiddleSolution::Sort(...) ein.",
        how: "Packe RiddleTask::Sort { numbers } mit if let Some aus, klone und sortiere numbers und übergib RiddleSolution::Sort(sorted) an actions.solve_riddle.",
        example: "if let Some(RiddleTask::Sort { numbers }) = world.active_riddle {\n    let mut sorted = numbers.clone();\n    sorted.sort();\n    actions.solve_riddle(RiddleSolution::Sort(sorted));\n}",
    },
    Command {
        key: "RIDDLE_DISTANCE",
        box_step: 19,
        icon: "📐",
        category: "Forschung",
        title: "RiddleTask::Distance { base, targets } / RiddleSolution::Distance(d)",
        summary: "Rätsel für Zweig 2 (Basen- & Asteroidenscan): Abstände berechnen.",
        what: "Beim Zweig UpgradeTrack::BaseAsteroidScanner bekommst du die Position base (Vec2) und eine Liste von Zielpunkten targets (&Vec<Vec2>). Für jeden Zielpunkt berechnest du den Abstand zu base.",
        how: "Gehe targets durch, berechne für jeden Punkt base.distance(t) und sammle die Ergebnisse in einem Vec<f32>, den du mit RiddleSolution::Distance(...) einreichst.",
        example: "if let Some(RiddleTask::Distance { base, targets }) = world.active_riddle {\n    let dists = targets.iter().map(|t| base.distance(t)).collect();\n    actions.solve_riddle(RiddleSolution::Distance(dists));\n}",
    },
    Command {
        key: "RIDDLE_DENOISE",
        box_step: 19,
        icon: "📡",
        category: "Forschung",
        title: "RiddleTask::Denoise { readings, threshold } / RiddleSolution::Denoise(c)",
        summary: "Rätsel für Zweig 3 (Radar-Horizont): Störsignale herausfiltern.",
        what: "Beim Zweig UpgradeTrack::RadarHorizon erhältst du Messwerte readings (&Vec<f32>) und einen Schwellenwert threshold (&f32). Ein Messwert ist ein isoliertes Störsignal (Spike), wenn er um mehr als threshold vom Durchschnitt seiner beiden direkten Nachbarn abweicht (Randwerte am Anfang und Ende bleiben immer erhalten).",
        how: "Prüfe für jeden inneren Index i (von 1 bis len - 2), ob (readings[i] - (readings[i - 1] + readings[i + 1]) / 2.0).abs() <= *threshold gilt, und sammle alle gültigen Werte in einem Vec<f32> für RiddleSolution::Denoise.",
        example: "if let Some(RiddleTask::Denoise { readings, threshold }) = world.active_riddle {\n    let mut cleaned = Vec::new();\n    for i in 0..readings.len() {\n        if i == 0 || i + 1 == readings.len() || (readings[i] - (readings[i - 1] + readings[i + 1]) / 2.0).abs() <= *threshold {\n            cleaned.push(readings[i]);\n        }\n    }\n    actions.solve_riddle(RiddleSolution::Denoise(cleaned));\n}",
    },
    Command {
        key: "RIDDLE_ENERGY",
        box_step: 19,
        icon: "⚡",
        category: "Forschung",
        title: "RiddleTask::Energy { consumers, budget } / RiddleSolution::Energy(n)",
        summary: "Rätsel für Zweig 4 (Energieproduktion): Maximale Verbraucherzahl im Budget.",
        what: "Beim Zweig UpgradeTrack::EnergyProduction erhältst du eine Liste von Energieverbrauchern consumers (&Vec<i64>) und ein Gesamtbudget budget (&i64). Gesucht ist die maximale Anzahl an Verbrauchern, die zusammen höchstens budget verbrauchen (kleinste zuerst).",
        how: "Klone und sortiere consumers aufsteigend, ziehe der Reihe nach jeden Wert vom verbleibenden Budget ab, solange er hineinpasst, und zähle mit. Übergib die Anzahl als RiddleSolution::Energy(count).",
        example: "if let Some(RiddleTask::Energy { consumers, budget }) = world.active_riddle {\n    let mut list = consumers.clone();\n    list.sort();\n    let mut rem = *budget;\n    let mut count = 0;\n    for c in list {\n        if c <= rem { rem -= c; count += 1; }\n    }\n    actions.solve_riddle(RiddleSolution::Energy(count));\n}",
    },
    Command {
        key: "ACTIONS_SOLVE_RIDDLE",
        box_step: 19,
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
        box_step: 21,
        icon: "🚀",
        category: "Basis-Befehle",
        title: "actions.build_drone(2)",
        summary: "Baut eine schwere Stufe-2-Drohne für 150 Erz und 200 Energie.",
        what: "Gibt eine schwere Stufe-2-Drohne in Auftrag. Sie kostet 150 Erz sowie 200 Energie aus der Basis, hat dreifache Frachtkapazität (150 Erz), höhere Geschwindigkeit (150 px/s) und eine doppelt so große Batterie (200 Energie).",
        how: "Übergib die Stufe 2 und achte darauf, dass genug Erz und Energie in der Basis gelagert sind.",
        example: "if world.base.stored_ore >= 150 && world.base.stored_energy >= 200.0 {\n    actions.build_drone(2);\n}",
    },
    Command {
        key: "ACTIONS_FLY_TO",
        box_step: 22,
        icon: "🧭",
        category: "Schiffs-Befehle",
        title: "actions.fly_to(ship_id, target)",
        summary: "Steuert eine Drohne direkt zu 2D-Koordinaten im Weltraum.",
        what: "Befiehlt einem Schiff, zur angegebenen Vec2-Zielposition zu fliegen – nützlich zum Erkunden neuer Sektoren oder zum gezielten Positionieren von Drohnen.",
        how: "Übergib die ship_id der Drohne und das Ziel als Vec2 (z. B. Vec2::new(x, y)).",
        example: "actions.fly_to(drone.ship_id, Vec2::new(500.0, -300.0));",
    },
    Command {
        key: "ACTIONS_STOP",
        box_step: 22,
        icon: "🛑",
        category: "Schiffs-Befehle",
        title: "actions.stop(ship_id)",
        summary: "Stoppt eine Drohne sofort an ihrer aktuellen Position.",
        what: "Bricht den aktuellen Flug- oder Schürfbefehl des Schiffs ab und setzt seine Geschwindigkeit sofort auf 0.",
        how: "Übergib die ship_id des Schiffs, das anhalten soll.",
        example: "actions.stop(drone.ship_id);",
    },
    Command {
        key: "VEC2_NEW",
        box_step: 22,
        icon: "📐",
        category: "Welt & Sensoren",
        title: "Vec2::new(x, y) / Vec2::ZERO",
        summary: "Erzeugt einen 2D-Koordinatenpunkt im Weltraum.",
        what: "Vec2 speichert eine Position oder Richtung mit zwei f32-Kommazahlen x und y. Vec2::ZERO ist der Ursprungspunkt (0.0, 0.0) bei der Mega-Station.",
        how: "Rufe Vec2::new(x, y) mit zwei Kommazahlen auf oder verwende Vec2::ZERO.",
        example: "let target = Vec2::new(250.0, 400.0);\nlet center = Vec2::ZERO;",
    },
    Command {
        key: "VEC2_DISTANCE",
        box_step: 22,
        icon: "📏",
        category: "Welt & Sensoren",
        title: "pos.distance(&other)",
        summary: "Berechnet den Abstand zwischen zwei Punkten.",
        what: "Liefert die direkte Entfernung (euklidische Distanz) zwischen zwei Vec2-Positionen als f32-Kommazahl.",
        how: "Rufe .distance(&other) auf einem Vec2 auf und übergib die zweite Position als Referenz mit &.",
        example: "let dist = drone.position.distance(&world.base.position);",
    },
    Command {
        key: "WORLD_SHIP",
        box_step: 22,
        icon: "🔍",
        category: "Welt & Sensoren",
        title: "world.ship(ship_id)",
        summary: "Sucht ein eigenes Schiff gezielt anhand seiner ID.",
        what: "Durchsucht world.fleet nach dem Schiff mit der angegebenen ship_id und gibt Some(&Ship) zurück, oder None, falls kein Schiff mit dieser ID existiert.",
        how: "Übergib die gesuchte ship_id und packe das Ergebnis mit if let Some(ship) aus.",
        example: "if let Some(ship) = world.ship(1) {\n    println!(\"Schiff #1 bei {:.0}, {:.0}\", ship.position.x, ship.position.y);\n}",
    },
    Command {
        key: "WORLD_CLOSEST_ASTEROID_ANY",
        box_step: 22,
        icon: "🪨",
        category: "Welt & Sensoren",
        title: "world.closest_asteroid(pos)",
        summary: "Findet den geometrisch nächsten Asteroiden (auch ohne Erz).",
        what: "Liefert den am nächsten gelegenen sichtbaren Asteroiden unabhängig davon, ob er noch Erz enthält. Für den Abbau ist meistens world.closest_asteroid_with_ore(pos) besser geeignet.",
        how: "Übergib eine Vec2-Position wie drone.position und packe das Option mit if let Some aus.",
        example: "if let Some(ast) = world.closest_asteroid(drone.position) {\n    println!(\"Rest-Erz: {}\", ast.remaining_ore);\n}",
    },
    Command {
        key: "WORLD_ASTEROIDS",
        box_step: 22,
        icon: "🌌",
        category: "Welt & Sensoren",
        title: "world.asteroids",
        summary: "Liste aller aktuell von deinen Scannern erfassten Asteroiden.",
        what: "Ein Slice (&[PerceivedAsteroid]) mit allen Asteroiden im Scanbereich deiner Basis und deiner Drohnen. Damit kannst du eigene Filter bauen, z. B. Drohnen auf verschiedene Asteroiden verteilen.",
        how: "Iteriere mit for ast in world.asteroids oder nutze .iter().filter(...), um gezielt Asteroiden auszuwählen.",
        example: "for ast in world.asteroids {\n    if ast.remaining_ore >= 100 {\n        // ergiebiger Asteroid\n    }\n}",
    },
    Command {
        key: "WORLD_OTHER_DRONES",
        box_step: 22,
        icon: "👀",
        category: "Welt & Sensoren",
        title: "world.other_drones",
        summary: "Liste aller sichtbaren fremden Drohnen im Sensorbereich.",
        what: "Enthält alle fremden Schiffe (&[PerceivedDrone]), die deine Scanner aktuell erfassen. Jeder Eintrag bietet ship_id, owner, ship_tier, position, heading_rad, speed, is_mining und is_docked().",
        how: "Gehe die Liste mit einer for-Schleife durch oder prüfe mit .len(), ob fremde Drohnen in der Nähe sind.",
        example: "for other in world.other_drones {\n    println!(\"Fremde Drohne #{} bei {:.0},{:.0}\", other.ship_id, other.position.x, other.position.y);\n}",
    },
    Command {
        key: "WORLD_OTHER_BASES",
        box_step: 22,
        icon: "🛰️",
        category: "Welt & Sensoren",
        title: "world.other_bases",
        summary: "Liste aller sichtbaren fremden Heimatbasen.",
        what: "Enthält alle fremden Raumstationen (&[PerceivedBase]) im Sensorbereich mit base_id, owner, position, signal_radius, level sowie deren Scanner-Stufen.",
        how: "Iteriere über world.other_bases, um Nachbarstationen zu erkennen.",
        example: "for base in world.other_bases {\n    println!(\"Fremde Basis #{} auf Stufe {}\", base.base_id, base.level);\n}",
    },
    Command {
        key: "WORLD_TICK",
        box_step: 22,
        icon: "⏱️",
        category: "Welt & Sensoren",
        title: "world.tick",
        summary: "Fortlaufende Nummer des aktuellen Aufrufs von tick.",
        what: "Ein Zähler (u64), der bei jedem Aufruf deiner tick-Methode um 1 steigt (ca. 10 Ticks pro Sekunde). Praktisch, um Ausgaben oder Aktionen nur alle paar Sekunden auszuführen.",
        how: "Prüfe z. B. mit dem Modulo-Operator world.tick % 50 == 0, ob gerade jeder 50. Tick erreicht ist.",
        example: "if world.tick % 50 == 0 {\n    println!(\"Tick #{}: {} Schiffe aktiv\", world.tick, world.fleet.len());\n}",
    },
    Command {
        key: "DRONE_TIER",
        box_step: 22,
        icon: "⭐",
        category: "Schiffs-Status",
        title: "drone.ship_tier",
        summary: "Die Baustufe des Schiffs (1 oder 2).",
        what: "Gibt an, um welchen Schiffstyp es sich handelt: 1 für die leichte Standard-Drohne (50 Fracht, 100 Energie), 2 für die schwere Stufe-2-Drohne (150 Fracht, 200 Energie).",
        how: "Lies drone.ship_tier aus, um großen und kleinen Drohnen unterschiedliche Aufgaben zuzuweisen.",
        example: "if drone.ship_tier == 2 {\n    // schwere Drohne für weit entfernte Asteroiden nutzen\n}",
    },
    Command {
        key: "DRONE_CARGO",
        box_step: 22,
        icon: "📦",
        category: "Schiffs-Status",
        title: "drone.cargo_ore / drone.max_cargo / drone.cargo_free_space()",
        summary: "Geladenes Erz, maximale Frachtkapazität und freier Laderaum.",
        what: "drone.cargo_ore zeigt die aktuell geladene Erzmenge, drone.max_cargo das Limit (50 bzw. 150) und drone.cargo_free_space() berechnet, wie viele Einheiten noch hineinpassen.",
        how: "Nutze die Felder oder Methoden, um z. B. schon bei fast vollem Laderaum heimzukehren.",
        example: "if drone.cargo_free_space() < 10 {\n    actions.dock(drone.ship_id);\n}",
    },
    Command {
        key: "DRONE_MOTION",
        box_step: 22,
        icon: "💨",
        category: "Schiffs-Status",
        title: "drone.velocity / drone.speed / drone.max_speed / drone.heading_rad",
        summary: "Bewegungsvektor, aktuelles Tempo, Höchstgeschwindigkeit und Blickrichtung.",
        what: "Liefert die physikalischen Bewegungsdaten der Drohne: velocity als Vec2, speed und max_speed in Einheiten pro Sekunde sowie den Winkel heading_rad im Bogenmaß.",
        how: "Lies die Felder direkt auf der Drohne aus, z. B. um zu prüfen, ob das Schiff steht (drone.speed == 0.0).",
        example: "if drone.speed == 0.0 && !drone.is_mining {\n    // Drohne ist gerade untätig\n}",
    },
    Command {
        key: "DRONE_TARGETS_AND_STATUS",
        box_step: 22,
        icon: "🎯",
        category: "Schiffs-Status",
        title: "drone.target_position / drone.target_asteroid_id / drone.is_mining",
        summary: "Aktuelles Flugziel, anvisierter Asteroid und Bohrstatus.",
        what: "target_position (Option<Vec2>) zeigt das Flugziel, target_asteroid_id (Option<u32>) den Asteroiden, den die Drohne abbaut oder anfliegt, und is_mining (bool) ist true, sobald der Bohrer aktiv arbeitet.",
        how: "Prüfe z. B. drone.target_asteroid_id, um zu sehen, welche Asteroiden bereits von deiner Flotte bearbeitet werden.",
        example: "if drone.is_mining {\n    println!(\"Drohne #{} schürft aktiv\", drone.ship_id);\n}",
    },
    Command {
        key: "DRONE_SIGNAL_AND_STRANDED",
        box_step: 22,
        icon: "📶",
        category: "Schiffs-Status",
        title: "drone.is_stranded / drone.signal_countdown",
        summary: "Strandungs-Status und verbleibender Funk-Puffer.",
        what: "Verlässt eine Drohne den Signalradius der Basis, zählt signal_countdown (in Sekunden) herunter. Bei 0 oder leerer Batterie wird is_stranded = true und die Drohne nimmt keine Befehle mehr an.",
        how: "Halte deine Drohnen innerhalb von world.base.signal_radius oder überwache drone.signal_countdown.",
        example: "if drone.signal_countdown < 10 {\n    actions.dock(drone.ship_id);\n}",
    },
    Command {
        key: "DRONE_DISTANCE_TO",
        box_step: 22,
        icon: "📏",
        category: "Schiffs-Status",
        title: "drone.distance_to(&target)",
        summary: "Berechnet den Abstand von der Drohne zu einer Zielposition.",
        what: "Kurzform für drone.position.distance(&target). Liefert die Entfernung der Drohne zum Punkt target als f32.",
        how: "Übergib die Zielposition als Referenz &Vec2.",
        example: "let dist_home = drone.distance_to(&world.base.position);",
    },
    Command {
        key: "BASE_STORED_ENERGY",
        box_step: 22,
        icon: "⚡",
        category: "Basis-Befehle",
        title: "world.base.stored_energy",
        summary: "Die aktuell in der Heimatbasis gespeicherte Energie.",
        what: "Deine Basis produziert laufend Energie (abhängig von energy_production_level) und speichert sie. Angedockte Drohnen laden daraus ihre Batterien auf, und auch der Drohnenbau verbraucht Basis-Energie (100 bzw. 200).",
        how: "Prüfe world.base.stored_energy vor dem Bau neuer Drohnen, damit genug Reserve zum Laden deiner Flotte bleibt.",
        example: "if world.base.stored_ore >= 50 && world.base.stored_energy >= 150.0 {\n    actions.build_drone(1);\n}",
    },
    Command {
        key: "BASE_POSITION_AND_SIGNAL",
        box_step: 22,
        icon: "📡",
        category: "Welt & Sensoren",
        title: "world.base.position / signal_radius / is_in_range(&pos) / distance_to(&pos)",
        summary: "Position, Funkradius und Reichweiten-Prüfung der Heimatbasis.",
        what: "world.base.position ist der Standort deiner Station, signal_radius (1200m) ihre maximale Funkreichweite. Mit world.base.is_in_range(&pos) prüfst du direkt, ob ein Punkt innerhalb des Funkbereichs liegt.",
        how: "Nutze world.base.is_in_range(&asteroid.position), bevor du eine Drohne zu weit entfernten Asteroiden schickst.",
        example: "if world.base.is_in_range(&asteroid.position) {\n    actions.mine(drone.ship_id, asteroid.asteroid_id);\n}",
    },
    Command {
        key: "BASE_RESEARCH_LEVELS",
        box_step: 22,
        icon: "🔬",
        category: "Forschung",
        title: "world.base.*_level / *_progress / *_range()",
        summary: "Forschungsstufen, Fortschritt (0–99) und aktuelle Scanreichweiten der Basis.",
        what: "Lies für alle vier Zweige Stufe und Fortschritt aus: drone_scanner_level / _progress, base_scanner_level / _progress, radar_horizon_level / _progress und energy_production_level / _progress. Dazu liefern drone_scanner_range(), base_scanner_range() und radar_horizon_range() die aktuellen Reichweiten in Metern.",
        how: "Frage die Stufen ab, um automatisch den Forschungszweig mit der niedrigsten Stufe weiterzuentwickeln.",
        example: "if world.active_riddle.is_none() && world.base.energy_production_level < 5 {\n    actions.request_research(UpgradeTrack::EnergyProduction);\n}",
    },
    Command {
        key: "ASTEROID_POSITION_AND_DISTANCE",
        box_step: 22,
        icon: "☄️",
        category: "Welt & Sensoren",
        title: "asteroid.position / velocity / radius / distance_to(&pos)",
        summary: "Koordinaten, Bewegung, Größe und Abstand eines Asteroiden.",
        what: "Jeder PerceivedAsteroid liefert seine position (Vec2), seine Geschwindigkeit velocity (Vec2), seinen Größenradius radius (f32) und die Hilfsmethode asteroid.distance_to(&pos).",
        how: "Nutze asteroid.distance_to(&drone.position), um Abstände zu vergleichen.",
        example: "let dist = asteroid.distance_to(&drone.position);",
    },
    Command {
        key: "ASTEROID_ORE_AND_TYPE",
        box_step: 22,
        icon: "💎",
        category: "Welt & Sensoren",
        title: "asteroid.remaining_ore / is_depleted() / resource_type / resource_name()",
        summary: "Verbleibende Erzmenge und Rohstoffart eines Asteroiden.",
        what: "remaining_ore zeigt, wie viel Erz noch im Asteroiden steckt; is_depleted() ist true, wenn er leer ist (0 Erz). resource_type (1..3) bzw. resource_name() liefert die Rohstoffart (\"Iron\", \"Ice\", \"Titanium\").",
        how: "Prüfe remaining_ore oder !asteroid.is_depleted(), um ergiebige Asteroiden zu bevorzugen.",
        example: "if !asteroid.is_depleted() {\n    println!(\"{}: {} Erz\", asteroid.resource_name(), asteroid.remaining_ore);\n}",
    },
    Command {
        key: "SDK_DRONE_COSTS",
        box_step: 22,
        icon: "🪙",
        category: "Basis-Befehle",
        title: "DRONE_TIER_1_COST / DRONE_TIER_2_COST",
        summary: "Konstanten für die Erzkosten einer Stufe-1- (50) und Stufe-2-Drohne (150).",
        what: "Das space_bot_sdk stellt die Baukosten als feste Konstanten bereit: DRONE_TIER_1_COST (50 Erz) und DRONE_TIER_2_COST (150 Erz). So musst du keine losen Zahlen in deinen Code schreiben.",
        how: "Importiere die Konstanten mit use space_bot_sdk::{DRONE_TIER_1_COST, DRONE_TIER_2_COST}; und nutze sie im Vergleich.",
        example: "if world.base.stored_ore >= space_bot_sdk::DRONE_TIER_1_COST {\n    actions.build_drone(1);\n}",
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
    fn all_commands_count_is_54() {
        assert_eq!(COMMANDS.len(), 54);
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
        let box_indices: [u8; 9] = [
            STEPS[5].index,
            STEPS[7].index,
            STEPS[9].index,
            STEPS[11].index,
            STEPS[13].index,
            STEPS[15].index,
            STEPS[17].index,
            STEPS[19].index,
            STEPS[21].index,
        ];
        assert_eq!(box_indices, LOOT_BOX_STEPS);
        for s in &LOOT_BOX_STEPS {
            assert!(is_loot_box_step(*s));
            assert!(STEPS[*s as usize].is_loot_box());
        }
    }

    #[test]
    fn coding_steps_reference_preceding_loot_box_keys() {
        for s in [6, 8, 10, 12, 14, 16, 18, 20] {
            let coding_step = &STEPS[s];
            let preceding_box = &STEPS[s - 1];
            assert_eq!(
                coding_step.reference_keys, preceding_box.unlocked_keys,
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
