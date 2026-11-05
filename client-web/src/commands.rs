//! Befehls-Ruhmeshalle: Katalog aller lernbaren Befehle, Lootbox-Enthüllung und Nachschlage-Modal.

use crate::audio::SoundEffects;
use crate::module_bindings::claim_next_command_reducer::claim_next_command;
use crate::module_bindings::dioxus::use_spacetimedb_context;
use dioxus::prelude::*;

/// Tutorial-Stufen, in denen eine Befehls-Lootbox in der Heimatbasis wartet.
pub const LOOT_BOX_STEPS: [u8; 9] = [5, 7, 9, 11, 13, 15, 17, 19, 21];

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

pub static COMMANDS: [CommandInfo; 54] = [
    CommandInfo {
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
    CommandInfo {
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
    CommandInfo {
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
    CommandInfo {
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
    CommandInfo {
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
    CommandInfo {
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
    CommandInfo {
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
    CommandInfo {
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
    CommandInfo {
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
    CommandInfo {
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
    CommandInfo {
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
    CommandInfo {
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
    CommandInfo {
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
    CommandInfo {
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
    CommandInfo {
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
    CommandInfo {
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
    CommandInfo {
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
    CommandInfo {
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
    CommandInfo {
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
    CommandInfo {
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
    CommandInfo {
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
    CommandInfo {
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
    CommandInfo {
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
    CommandInfo {
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
    CommandInfo {
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
    CommandInfo {
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
    CommandInfo {
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
    CommandInfo {
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
    CommandInfo {
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
    CommandInfo {
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
    // ==========================================
    // Freies Spiel (Stufe 22): Vollständige SDK-Referenz
    // ==========================================
    CommandInfo {
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
    CommandInfo {
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
    CommandInfo {
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
    CommandInfo {
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
    CommandInfo {
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
    CommandInfo {
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
    CommandInfo {
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
    CommandInfo {
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
    CommandInfo {
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
    CommandInfo {
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
    CommandInfo {
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
    CommandInfo {
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
    CommandInfo {
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
    CommandInfo {
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
    CommandInfo {
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
    CommandInfo {
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
    CommandInfo {
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
    CommandInfo {
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
    CommandInfo {
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
    CommandInfo {
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
    CommandInfo {
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
    CommandInfo {
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
        ];
        let expected_free_play: &[&str] = &[
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
        assert_eq!(
            LOOT_BOX_STEPS.as_slice(),
            expected.iter().map(|(step, _)| *step).collect::<Vec<_>>()
        );
        for (step, keys) in expected {
            let box_keys: Vec<_> = commands_for_box(*step).iter().map(|c| c.key).collect();
            let coding_keys: Vec<_> = commands_for_coding_step(*step + 1)
                .iter()
                .map(|c| c.key)
                .collect();
            assert_eq!(box_keys, *keys);
            assert_eq!(coding_keys, *keys);
            assert!(is_loot_box_step(*step));
            assert!(!is_loot_box_step(*step + 1));
        }
        let free_play_keys: Vec<_> = commands_for_box(22).iter().map(|c| c.key).collect();
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
                "IF_LET_SOME".into()
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
