use crate::math::{SimpleRng, Vec2};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SortPayload {
    pub numbers: Vec<i64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SortSolution {
    pub sorted: Vec<i64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DistancePayload {
    pub base: [f32; 2],
    pub targets: Vec<[f32; 2]>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DistanceSolution {
    pub distances: Vec<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DenoisePayload {
    pub readings: Vec<f32>,
    pub threshold: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DenoiseSolution {
    pub cleaned: Vec<f32>,
}

/// Generates riddle payload and expected solution based on track and target level.
pub fn generate_riddle(track: u8, target_level: u8, base_pos: Vec2, seed: u64) -> (String, String) {
    let mut rng = SimpleRng::new(seed);

    match track {
        1 => {
            // Track 1: Drone Scanner — Sorting Lists
            // Difficulty scales with target level
            let count = match target_level {
                2 => 15,
                3 => 30,
                4 => 60,
                _ => 100,
            };

            let mut numbers = Vec::with_capacity(count);
            for _ in 0..count {
                numbers.push(rng.next_i64_range(-1000, 1000));
            }

            let mut sorted = numbers.clone();
            sorted.sort();

            let payload = serde_json::to_string(&SortPayload { numbers }).unwrap_or_default();
            let solution = serde_json::to_string(&SortSolution { sorted }).unwrap_or_default();
            (payload, solution)
        }
        2 => {
            // Track 2: Base & Asteroid Range — Calculating Distances
            let count = match target_level {
                2 => 5,
                3 => 10,
                4 => 20,
                _ => 35,
            };

            let mut targets = Vec::with_capacity(count);
            let mut distances = Vec::with_capacity(count);

            for _ in 0..count {
                let tx = base_pos.x + rng.next_i64_range(-800, 800) as f32;
                let ty = base_pos.y + rng.next_i64_range(-600, 600) as f32;
                targets.push([tx, ty]);

                let dx = tx - base_pos.x;
                let dy = ty - base_pos.y;
                let dist = (dx * dx + dy * dy).sqrt();
                let rounded = (dist * 100.0).round() / 100.0;
                distances.push(rounded);
            }

            let payload = serde_json::to_string(&DistancePayload {
                base: [base_pos.x, base_pos.y],
                targets,
            })
            .unwrap_or_default();
            let solution =
                serde_json::to_string(&DistanceSolution { distances }).unwrap_or_default();
            (payload, solution)
        }
        3 => {
            // Track 3: Radar Horizon — Signal Denoising (Spike Removal)
            let count = match target_level {
                2 => 20,
                3 => 40,
                4 => 80,
                _ => 120,
            };

            let threshold = 15.0;
            let mut readings = Vec::with_capacity(count);
            let mut cleaned = Vec::with_capacity(count);
            let mut spike_cooldown = 2u8;

            for i in 0..count {
                // Base smooth wave
                let base_val = 50.0 + ((i as f32 * 0.2).sin() * 10.0);
                let rounded_base = (base_val * 10.0).round() / 10.0;

                // Inject isolated spike with ~15% probability (at least 2 clean points between spikes)
                let is_spike =
                    i > 1 && i < count - 2 && spike_cooldown == 0 && (rng.next_u64() % 5 == 0);

                if is_spike {
                    spike_cooldown = 2;
                    let spike_offset = if rng.next_u64() % 2 == 0 { 60.0 } else { -60.0 };
                    readings.push(rounded_base + spike_offset);
                } else {
                    spike_cooldown = spike_cooldown.saturating_sub(1);
                    readings.push(rounded_base);
                    cleaned.push(rounded_base);
                }
            }

            let payload = serde_json::to_string(&DenoisePayload {
                readings,
                threshold,
            })
            .unwrap_or_default();
            let solution = serde_json::to_string(&DenoiseSolution { cleaned }).unwrap_or_default();
            (payload, solution)
        }
        _ => ("{}".to_string(), "{}".to_string()),
    }
}

/// Verifies whether the student's solution matches the expected answer.
pub fn verify_solution(
    track: u8,
    _payload_json: &str,
    expected_json: &str,
    student_json: &str,
) -> bool {
    let student_trimmed = student_json.trim();
    if student_trimmed.is_empty() {
        return false;
    }

    match track {
        1 => {
            // Track 1: Sort
            // Accept either {"sorted": [...]} or direct [...]
            let student_list: Option<Vec<i64>> =
                serde_json::from_str::<SortSolution>(student_trimmed)
                    .map(|s| s.sorted)
                    .or_else(|_| serde_json::from_str::<Vec<i64>>(student_trimmed))
                    .ok();

            let expected_list: Option<Vec<i64>> =
                serde_json::from_str::<SortSolution>(expected_json)
                    .map(|s| s.sorted)
                    .or_else(|_| serde_json::from_str::<Vec<i64>>(expected_json))
                    .ok();

            match (student_list, expected_list) {
                (Some(s), Some(e)) => s == e,
                _ => false,
            }
        }
        2 => {
            // Track 2: Distances
            // Accept either {"distances": [...]} or direct [...]
            let student_list: Option<Vec<f32>> =
                serde_json::from_str::<DistanceSolution>(student_trimmed)
                    .map(|s| s.distances)
                    .or_else(|_| serde_json::from_str::<Vec<f32>>(student_trimmed))
                    .ok();

            let expected_list: Option<Vec<f32>> =
                serde_json::from_str::<DistanceSolution>(expected_json)
                    .map(|s| s.distances)
                    .or_else(|_| serde_json::from_str::<Vec<f32>>(expected_json))
                    .ok();

            match (student_list, expected_list) {
                (Some(s), Some(e)) => {
                    if s.len() != e.len() {
                        return false;
                    }
                    for (a, b) in s.iter().zip(e.iter()) {
                        if (a - b).abs() > 0.1 {
                            return false;
                        }
                    }
                    true
                }
                _ => false,
            }
        }
        3 => {
            // Track 3: Denoise
            // Accept either {"cleaned": [...]} or direct [...]
            let student_list: Option<Vec<f32>> =
                serde_json::from_str::<DenoiseSolution>(student_trimmed)
                    .map(|s| s.cleaned)
                    .or_else(|_| serde_json::from_str::<Vec<f32>>(student_trimmed))
                    .ok();

            let expected_list: Option<Vec<f32>> =
                serde_json::from_str::<DenoiseSolution>(expected_json)
                    .map(|s| s.cleaned)
                    .or_else(|_| serde_json::from_str::<Vec<f32>>(expected_json))
                    .ok();

            match (student_list, expected_list) {
                (Some(s), Some(e)) => {
                    if s.len() != e.len() {
                        return false;
                    }
                    for (a, b) in s.iter().zip(e.iter()) {
                        if (a - b).abs() > 0.05 {
                            return false;
                        }
                    }
                    true
                }
                _ => false,
            }
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_track_1_sort() {
        let (payload, expected) = generate_riddle(1, 2, Vec2::new(100.0, 100.0), 12345);
        let parsed_payload: SortPayload = serde_json::from_str(&payload).unwrap();
        assert_eq!(parsed_payload.numbers.len(), 15);

        // Verification with valid solution
        assert!(verify_solution(1, &payload, &expected, &expected));

        // Verification with direct array
        let expected_sol: SortSolution = serde_json::from_str(&expected).unwrap();
        let direct_json = serde_json::to_string(&expected_sol.sorted).unwrap();
        assert!(verify_solution(1, &payload, &expected, &direct_json));

        // Incorrect solution fails
        assert!(!verify_solution(
            1,
            &payload,
            &expected,
            "{\"sorted\": [1, 2, 3]}"
        ));
    }

    #[test]
    fn test_track_2_distances() {
        let (payload, expected) = generate_riddle(2, 2, Vec2::new(100.0, 100.0), 54321);
        let parsed_payload: DistancePayload = serde_json::from_str(&payload).unwrap();
        assert_eq!(parsed_payload.targets.len(), 5);

        // Verification with valid solution
        assert!(verify_solution(2, &payload, &expected, &expected));

        // Incorrect solution fails
        assert!(!verify_solution(
            2,
            &payload,
            &expected,
            "{\"distances\": [0.0]}"
        ));
    }

    #[test]
    fn test_track_3_denoise() {
        for seed in [11111, 22222, 54321, 99999, 123456] {
            let (payload, expected) = generate_riddle(3, 2, Vec2::new(100.0, 100.0), seed);
            let parsed_payload: DenoisePayload = serde_json::from_str(&payload).unwrap();
            assert_eq!(parsed_payload.readings.len(), 20);

            // Verification with valid solution
            assert!(verify_solution(3, &payload, &expected, &expected));

            // Test our solver implementation against the generated riddle
            let mut cleaned = Vec::new();
            let readings = &parsed_payload.readings;
            let threshold = parsed_payload.threshold;
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
            let solver_json = serde_json::to_string(&DenoiseSolution {
                cleaned: cleaned.clone(),
            })
            .unwrap();
            assert!(
                verify_solution(3, &payload, &expected, &solver_json),
                "Solver failed for seed {seed}"
            );

            // Incorrect solution fails
            assert!(!verify_solution(
                3,
                &payload,
                &expected,
                "{\"cleaned\": []}"
            ));
        }
    }
}
