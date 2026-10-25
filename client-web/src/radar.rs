use crate::module_bindings::{
    BaseStation, Config, PerceivedAsteroid, PerceivedBase, PerceivedDrone, Player, Ship,
    UnidentifiedContact, Vec2,
};
use std::collections::HashMap;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;
#[cfg(target_arch = "wasm32")]
use web_sys::CanvasRenderingContext2d;

pub fn render_player_radar(
    #[cfg(target_arch = "wasm32")] ctx: &CanvasRenderingContext2d,
    canvas_w: f64,
    canvas_h: f64,
    dpr: f64,
    center_pos: Vec2,
    color_hue: u16,
    config: Option<&Config>,
    my_ships: &[Ship],
    my_base: Option<&BaseStation>,
    visible_drones: &[PerceivedDrone],
    visible_bases: &[PerceivedBase],
    visible_asteroids: &[PerceivedAsteroid],
    unidentified_objects: &[UnidentifiedContact],
    players: &[Player],
    target_pos: Option<Vec2>,
    time: f64,
) {
    #[cfg(target_arch = "wasm32")]
    {
        // 1. Dark radar background with subtle radial vignette
        ctx.set_fill_style(&JsValue::from_str("#060a17"));
        ctx.fill_rect(0.0, 0.0, canvas_w * dpr, canvas_h * dpr);

        let cx = canvas_w / 2.0;
        let cy = canvas_h / 2.0;

        let _ = ctx.save();
        let _ = ctx.scale(dpr, dpr);
        let _ = ctx.translate(cx, cy);

        // Precompute player lookup (identity -> player)
        let player_map: HashMap<_, _> = players.iter().map(|p| (p.identity, p)).collect();

        // Dynamic radar sensor range rings based on Base Station upgrades
        let drone_scan_r = my_base.map_or(150.0, |b| {
            150.0 + (b.drone_scanner_level.saturating_sub(1) as f64 * 50.0)
        });
        let base_scan_r = my_base.map_or(300.0, |b| {
            300.0 + (b.base_scanner_level.saturating_sub(1) as f64 * 100.0)
        });
        let horizon_r = my_base.map_or(600.0, |b| {
            600.0 + (b.radar_horizon_level.saturating_sub(1) as f64 * 150.0)
        });

        let drone_label = format!(
            "{:.0}m Drohnen-Scan (Stufe {})",
            drone_scan_r,
            my_base.map_or(1, |b| b.drone_scanner_level)
        );
        let base_label = format!(
            "{:.0}m Basen & Asteroiden (Stufe {})",
            base_scan_r,
            my_base.map_or(1, |b| b.base_scanner_level)
        );
        let horizon_label = format!(
            "{:.0}m Radar-Horizont (Stufe {})",
            horizon_r,
            my_base.map_or(1, |b| b.radar_horizon_level)
        );

        let rings = [
            (drone_scan_r, drone_label, "rgba(0, 229, 255, 0.15)"),
            (base_scan_r, base_label, "rgba(0, 229, 255, 0.18)"),
            (horizon_r, horizon_label, "rgba(0, 229, 255, 0.25)"),
        ];

        for (r, label, stroke_color) in &rings {
            ctx.begin_path();
            let _ = ctx.arc(0.0, 0.0, *r, 0.0, std::f64::consts::TAU);
            ctx.set_stroke_style(&JsValue::from_str(stroke_color));
            ctx.set_line_width(1.0);
            ctx.stroke();

            // Ring distance label
            ctx.set_font("10px monospace");
            ctx.set_fill_style(&JsValue::from_str("rgba(0, 229, 255, 0.45)"));
            ctx.set_text_align("left");
            let _ = ctx.fill_text(label, 6.0, -r + 12.0);
        }

        // Radar crosshairs
        ctx.set_stroke_style(&JsValue::from_str("rgba(0, 229, 255, 0.08)"));
        ctx.set_line_width(1.0);
        ctx.begin_path();
        ctx.move_to(-canvas_w / 2.0, 0.0);
        ctx.line_to(canvas_w / 2.0, 0.0);
        ctx.move_to(0.0, -canvas_h / 2.0);
        ctx.line_to(0.0, canvas_h / 2.0);
        ctx.stroke();

        let center_x = center_pos.x as f64;
        let center_y = center_pos.y as f64;

        // 2. Draw Central Mega-Station (Permanent Beacon)
        let mega_world_x = config.map_or(1600.0, |c| c.megastation_pos.x as f64);
        let mega_world_y = config.map_or(1000.0, |c| c.megastation_pos.y as f64);
        let mega_x = mega_world_x - center_x;
        let mega_y = mega_world_y - center_y;

        // Safe zone boundary (250 units radius around Mega-Station)
        ctx.begin_path();
        let _ = ctx.arc(mega_x, mega_y, 250.0, 0.0, std::f64::consts::TAU);
        ctx.set_stroke_style(&JsValue::from_str("rgba(80, 200, 255, 0.22)"));
        ctx.set_line_width(2.0);
        ctx.stroke();

        // Mega-Station icon
        ctx.begin_path();
        let _ = ctx.arc(mega_x, mega_y, 28.0, 0.0, std::f64::consts::TAU);
        ctx.set_fill_style(&JsValue::from_str("#1c3b57"));
        ctx.fill();
        ctx.set_stroke_style(&JsValue::from_str("#5ce1e6"));
        ctx.set_line_width(2.5);
        ctx.stroke();

        ctx.set_font("bold 11px sans-serif");
        ctx.set_text_align("center");
        ctx.set_fill_style(&JsValue::from_str("#e0f7fa"));
        let _ = ctx.fill_text("MEGA-STATION", mega_x, mega_y + 44.0);

        // 3. Draw Unidentified Objects (Faint red radar blip with no text label)
        for contact in unidentified_objects {
            let ux = contact.position.x as f64 - center_x;
            let uy = contact.position.y as f64 - center_y;

            if ux.abs() < cx + 40.0 && uy.abs() < cy + 40.0 {
                let pulse =
                    ((time * 3.5 + (contact.contact_id as f64 * 0.7)).sin() * 0.25 + 0.75) * 4.5;
                ctx.begin_path();
                let _ = ctx.arc(ux, uy, pulse, 0.0, std::f64::consts::TAU);
                ctx.set_fill_style(&JsValue::from_str("rgba(239, 68, 68, 0.75)"));
                ctx.fill();
                ctx.set_stroke_style(&JsValue::from_str("rgba(248, 113, 113, 0.9)"));
                ctx.set_line_width(1.0);
                ctx.stroke();
            }
        }

        // 4. Draw Identified Asteroids
        for ast in visible_asteroids {
            let ax = ast.position.x as f64 - center_x;
            let ay = ast.position.y as f64 - center_y;

            if ax.abs() < cx + 60.0 && ay.abs() < cy + 60.0 {
                let color = match ast.resource_type {
                    3 => "#a855f7", // Titanium (purple)
                    2 => "#06b6d4", // Ice (cyan)
                    1 => "#64748b", // Iron (slate)
                    _ => "#78716c", // Generic
                };
                let border = match ast.resource_type {
                    3 => "#d8b4fe",
                    2 => "#a5f3fc",
                    1 => "#94a3b8",
                    _ => "#a8a29e",
                };

                ctx.begin_path();
                let _ = ctx.arc(
                    ax,
                    ay,
                    (ast.radius as f64) * 0.75,
                    0.0,
                    std::f64::consts::TAU,
                );
                ctx.set_fill_style(&JsValue::from_str(color));
                ctx.fill();
                ctx.set_stroke_style(&JsValue::from_str(border));
                ctx.set_line_width(1.0);
                ctx.stroke();
            }
        }

        // 5. Draw Bases
        // 5A. Other identified bases in sensor range (from visible_bases)
        for base in visible_bases {
            // Check if this base is our own base
            let is_my_base = my_base.map_or(false, |b| b.base_id == base.base_id);
            if !is_my_base {
                let bx = base.position.x as f64 - center_x;
                let by = base.position.y as f64 - center_y;
                let b_hue = player_map.get(&base.owner).map_or(200, |p| p.color_hue);
                let b_name = player_map
                    .get(&base.owner)
                    .map_or("Basis".to_string(), |p| format!("Basis ({})", p.name));

                // Base structure icon
                ctx.begin_path();
                let _ = ctx.arc(bx, by, 18.0, 0.0, std::f64::consts::TAU);
                ctx.set_fill_style(&JsValue::from_str("#0f172a"));
                ctx.fill();
                ctx.set_stroke_style(&JsValue::from_str(&format!("hsl({}, 90%, 55%)", b_hue)));
                ctx.set_line_width(2.5);
                ctx.stroke();

                // Cross arms
                ctx.set_stroke_style(&JsValue::from_str("#94a3b8"));
                ctx.set_line_width(1.5);
                ctx.begin_path();
                ctx.move_to(bx - 24.0, by);
                ctx.line_to(bx + 24.0, by);
                ctx.move_to(bx, by - 24.0);
                ctx.line_to(bx, by + 24.0);
                ctx.stroke();

                ctx.set_font("bold 11px sans-serif");
                ctx.set_text_align("center");
                ctx.set_fill_style(&JsValue::from_str("#e2e8f0"));
                let _ = ctx.fill_text(&b_name, bx, by + 36.0);
            }
        }

        // 5B. Draw Player's Own Home Base (if deployed)
        if let Some(base) = my_base {
            let bx = base.position.x as f64 - center_x;
            let by = base.position.y as f64 - center_y;

            // Signal radius circle
            let sig_r = base.signal_radius as f64;
            ctx.begin_path();
            let _ = ctx.arc(bx, by, sig_r, 0.0, std::f64::consts::TAU);
            ctx.set_stroke_style(&JsValue::from_str(&format!(
                "hsla({}, 90%, 65%, 0.25)",
                color_hue
            )));
            ctx.set_line_width(2.0);
            ctx.stroke();

            // Inner fill tint
            ctx.set_fill_style(&JsValue::from_str(&format!(
                "hsla({}, 90%, 60%, 0.03)",
                color_hue
            )));
            ctx.fill();

            // Base structure icon
            ctx.begin_path();
            let _ = ctx.arc(bx, by, 22.0, 0.0, std::f64::consts::TAU);
            ctx.set_fill_style(&JsValue::from_str("#0f172a"));
            ctx.fill();
            ctx.set_stroke_style(&JsValue::from_str(&format!("hsl({}, 95%, 60%)", color_hue)));
            ctx.set_line_width(3.0);
            ctx.stroke();

            // Solar panels / cross arms
            ctx.set_stroke_style(&JsValue::from_str("#38bdf8"));
            ctx.set_line_width(2.0);
            ctx.begin_path();
            ctx.move_to(bx - 32.0, by);
            ctx.line_to(bx + 32.0, by);
            ctx.move_to(bx, by - 32.0);
            ctx.line_to(bx, by + 32.0);
            ctx.stroke();

            ctx.set_font("bold 12px sans-serif");
            ctx.set_text_align("center");
            ctx.set_fill_style(&JsValue::from_str("#ffffff"));
            let _ = ctx.fill_text("HEIMATBASIS", bx, by + 46.0);
        }

        // 6. Draw Other Identified Drones (from visible_drones)
        for d in visible_drones {
            let is_my_drone = my_ships.iter().any(|s| s.ship_id == d.ship_id);
            if !is_my_drone {
                let dx = d.position.x as f64 - center_x;
                let dy = d.position.y as f64 - center_y;
                let d_hue = player_map.get(&d.owner).map_or(180, |p| p.color_hue);
                let p_name = player_map
                    .get(&d.owner)
                    .map_or("".to_string(), |p| p.name.clone());

                let _ = ctx.save();
                let _ = ctx.translate(dx, dy);
                let _ = ctx.rotate(d.heading_rad as f64);

                ctx.begin_path();
                ctx.move_to(16.0, 0.0);
                ctx.line_to(-12.0, -10.0);
                ctx.line_to(-6.0, 0.0);
                ctx.line_to(-12.0, 10.0);
                ctx.close_path();

                ctx.set_fill_style(&JsValue::from_str(&format!("hsl({}, 85%, 50%)", d_hue)));
                ctx.fill();
                ctx.set_stroke_style(&JsValue::from_str("#ffffff"));
                ctx.set_line_width(1.5);
                ctx.stroke();

                let _ = ctx.restore();

                // Label
                ctx.set_font("10px monospace");
                ctx.set_text_align("center");
                ctx.set_fill_style(&JsValue::from_str("#cbd5e1"));
                let label = if p_name.is_empty() {
                    format!("Drohne #{}", d.ship_id)
                } else {
                    format!("{}: #{}", p_name, d.ship_id)
                };
                let _ = ctx.fill_text(&label, dx, dy - 18.0);
            }
        }

        // 7. Draw Player's Own Ships & Drones (with full telemetry & movement paths)
        for s in my_ships {
            let sx = s.position.x as f64 - center_x;
            let sy = s.position.y as f64 - center_y;

            // Mining laser beam if currently mining
            if s.is_mining {
                if let Some(target_id) = s.target_asteroid_id {
                    if let Some(ast) = visible_asteroids
                        .iter()
                        .find(|a| a.asteroid_id == target_id)
                    {
                        let ast_x = ast.position.x as f64 - center_x;
                        let ast_y = ast.position.y as f64 - center_y;
                        ctx.begin_path();
                        ctx.move_to(sx, sy);
                        ctx.line_to(ast_x, ast_y);
                        ctx.set_stroke_style(&JsValue::from_str("#00e5ff"));
                        ctx.set_line_width(2.0);
                        ctx.stroke();
                    }
                }
            }

            // Target flight line if moving
            if let Some(ref target) = s.target_position {
                let tx = target.x as f64 - center_x;
                let ty = target.y as f64 - center_y;

                ctx.begin_path();
                ctx.move_to(sx, sy);
                ctx.line_to(tx, ty);
                ctx.set_stroke_style(&JsValue::from_str("rgba(0, 230, 118, 0.4)"));
                ctx.set_line_width(1.5);
                ctx.stroke();

                // Target indicator
                let pulse = (time * 8.0).sin() * 3.0 + 10.0;
                ctx.begin_path();
                let _ = ctx.arc(tx, ty, pulse, 0.0, std::f64::consts::TAU);
                ctx.set_stroke_style(&JsValue::from_str("#00e676"));
                ctx.set_line_width(2.0);
                ctx.stroke();
            }

            // Draw Ship Body
            let _ = ctx.save();
            let _ = ctx.translate(sx, sy);
            let _ = ctx.rotate(s.heading_rad as f64);

            if s.is_stranded {
                // Distress ring around stranded ship
                ctx.begin_path();
                let _ = ctx.arc(0.0, 0.0, 26.0, 0.0, std::f64::consts::TAU);
                ctx.set_stroke_style(&JsValue::from_str("rgba(239, 68, 68, 0.8)"));
                ctx.set_line_width(2.0);
                ctx.stroke();
            }

            if s.ship_tier == 0 {
                // Tier-0 Construction / Builder Ship (Mothership)
                ctx.begin_path();
                ctx.move_to(22.0, 0.0);
                ctx.line_to(-16.0, -14.0);
                ctx.line_to(-10.0, 0.0);
                ctx.line_to(-16.0, 14.0);
                ctx.close_path();

                ctx.set_fill_style(&JsValue::from_str(&format!("hsl({}, 90%, 52%)", color_hue)));
                ctx.fill();
                ctx.set_stroke_style(&JsValue::from_str("#ffffff"));
                ctx.set_line_width(2.0);
                ctx.stroke();

                // Solar wings for builder ship
                ctx.set_stroke_style(&JsValue::from_str("#38bdf8"));
                ctx.set_line_width(2.0);
                ctx.begin_path();
                ctx.move_to(-2.0, -20.0);
                ctx.line_to(-2.0, 20.0);
                ctx.stroke();
            } else {
                // Tier-1+ Drone / Miner
                ctx.begin_path();
                ctx.move_to(16.0, 0.0);
                ctx.line_to(-12.0, -10.0);
                ctx.line_to(-6.0, 0.0);
                ctx.line_to(-12.0, 10.0);
                ctx.close_path();

                ctx.set_fill_style(&JsValue::from_str(&format!("hsl({}, 85%, 50%)", color_hue)));
                ctx.fill();
                ctx.set_stroke_style(&JsValue::from_str("#ffffff"));
                ctx.set_line_width(1.5);
                ctx.stroke();
            }

            let _ = ctx.restore();

            // Ship label above/below ship
            ctx.set_font("10px monospace");
            ctx.set_text_align("center");
            ctx.set_fill_style(&JsValue::from_str("#cbd5e1"));
            let label = if s.ship_tier == 0 {
                "Bauschiff (Mutterschiff)".to_string()
            } else {
                format!("Drohne #{}", s.ship_id)
            };
            let _ = ctx.fill_text(&label, sx, sy - 20.0);
        }

        // 8. Draw Manual Target Marker if in Builder Mode
        if let Some(target) = target_pos {
            let tx = target.x as f64 - center_x;
            let ty = target.y as f64 - center_y;

            let pulse = (time * 8.0).sin() * 3.0 + 12.0;
            ctx.begin_path();
            let _ = ctx.arc(tx, ty, pulse, 0.0, std::f64::consts::TAU);
            ctx.set_stroke_style(&JsValue::from_str("#00e676"));
            ctx.set_line_width(2.0);
            ctx.stroke();

            // Cross in marker
            ctx.begin_path();
            ctx.move_to(tx - 6.0, ty);
            ctx.line_to(tx + 6.0, ty);
            ctx.move_to(tx, ty - 6.0);
            ctx.line_to(tx, ty + 6.0);
            ctx.stroke();
        }

        let _ = ctx.restore();
    }
}
