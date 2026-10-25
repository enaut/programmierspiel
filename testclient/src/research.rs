pub use space_bot_sdk::UpgradeTrack;

/// 🧪 HIER IMPLEMENTIERST DU DEINE FORSCHUNGS-ALGORITHMEN!
///
/// Deine Heimatbasis kann durch das automatische Lösen von Programmierrätseln
/// aufgerüstet werden.
///
/// Jedes gelöste Rätsel bringt 1% Fortschritt (100 Rätsel pro Stufe) und verbraucht
/// 1 Einheit abgebautes Erz aus deinem Basislager.
///
/// ⚠️ ACHTUNG: Bei einer falschen Antwort besteht ein 50%-Risiko, dass das
/// Forschungsmodul durchbrennt und der Fortschritt auf 0% zurückgesetzt wird!
/// Teste deine Funktionen daher sorgfältig!

// =============================================================================
// TRACK 1: Drohnen-Scanner Reichweite (Stufe 1..5)
// =============================================================================
/// Aufgabe: Sortiere die empfangenen Telemetrie-Zahlen in aufsteigender Reihenfolge.
///
/// Eingabe: Ein Slice mit vorzeichenbehafteten Ganzzahlen (z.B. `[42, -15, 8, 0, -3]`)
/// Ausgabe: Ein neuer `Vec<i64>` mit den Zahlen aufsteigend sortiert (z.B. `[-15, -3, 0, 8, 42]`)
pub fn solve_sorting(numbers: &[i64]) -> Vec<i64> {
    let mut sorted = numbers.to_vec();
    sorted.sort();
    sorted
}

// =============================================================================
// TRACK 2: Basen- & Asteroiden-Scan Reichweite (Stufe 1..5)
// =============================================================================
/// Aufgabe: Berechne die euklidischen Distanzen von deiner Heimatbasis zu allen Zielpunkten.
///
/// Eingabe:
/// - `base`: Koordinaten der Heimatbasis `[bx, by]`
/// - `targets`: Liste von Zielkoordinaten `[[x1, y1], [x2, y2], ...]`
///
/// Formel für Distanz: `sqrt((x - bx)^2 + (y - by)^2)`
///
/// Ausgabe:
/// - `Vec<f32>` mit den berechneten Distanzen, gerundet auf 2 Nachkommastellen.
pub fn solve_distances(base: [f32; 2], targets: &[[f32; 2]]) -> Vec<f32> {
    let bx = base[0];
    let by = base[1];

    targets
        .iter()
        .map(|t| {
            let dx = t[0] - bx;
            let dy = t[1] - by;
            let dist = (dx * dx + dy * dy).sqrt();
            (dist * 100.0).round() / 100.0
        })
        .collect()
}

// =============================================================================
// TRACK 3: Radar-Horizont Reichweite (Stufe 1..5)
// =============================================================================
/// Aufgabe: Bereinige eine Sensor-Signalreihe von extremen Störspitzen (Spikes).
///
/// Ein Messwert an Index `i` (mit `0 < i < len - 1`) gilt als Spike, wenn er sowohl vom
/// linken als auch vom rechten Nachbarn in dieselbe Richtung um mehr als `threshold` abweicht:
///
/// - Positiver Spike: `readings[i] - readings[i - 1] > threshold` UND `readings[i] - readings[i + 1] > threshold`
/// - Negativer Spike: `readings[i - 1] - readings[i] > threshold` UND `readings[i + 1] - readings[i] > threshold`
///
/// Randelemente (Index 0 und len-1) weichen nur von ihrem einzigen Nachbarn um mehr als `threshold` ab.
///
/// Ausgabe:
/// - `Vec<f32>` mit allen Werten, aus denen die Spikes entfernt wurden (Reihenfolge bleibt erhalten).
pub fn solve_denoise(readings: &[f32], threshold: f32) -> Vec<f32> {
    if readings.len() <= 2 {
        return readings.to_vec();
    }

    let mut cleaned = Vec::new();
    let len = readings.len();

    for i in 0..len {
        let is_spike = if i == 0 {
            (readings[0] - readings[1]).abs() > threshold
        } else if i == len - 1 {
            (readings[len - 1] - readings[len - 2]).abs() > threshold
        } else {
            let left_diff = readings[i] - readings[i - 1];
            let right_diff = readings[i] - readings[i + 1];
            (left_diff > threshold && right_diff > threshold)
                || (left_diff < -threshold && right_diff < -threshold)
        };

        if !is_spike {
            cleaned.push(readings[i]);
        }
    }

    cleaned
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sorting() {
        let input = vec![10, -5, 3, 100, 0];
        let result = solve_sorting(&input);
        assert_eq!(result, vec![-5, 0, 3, 10, 100]);
    }

    #[test]
    fn test_distances() {
        let base = [0.0, 0.0];
        let targets = vec![[3.0, 4.0], [1.0, 1.0]];
        let result = solve_distances(base, &targets);
        assert_eq!(result, vec![5.0, 1.41]);
    }

    #[test]
    fn test_denoise() {
        let readings = vec![10.0, 10.2, 85.0, 10.1, 9.9];
        let cleaned = solve_denoise(&readings, 15.0);
        assert_eq!(cleaned, vec![10.0, 10.2, 10.1, 9.9]);
    }
}
