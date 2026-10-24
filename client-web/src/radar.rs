use crate::module_bindings::{Asteroid, BaseStation, Config, Ship, Vec2};

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
    ships: &[Ship],
    my_base: Option<&BaseStation>,
    asteroids: &[Asteroid],
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

        // Radar range rings (relative to radar center)
        ctx.set_stroke_style(&JsValue::from_str("rgba(0, 229, 255, 0.12)"));
        ctx.set_line_width(1.0);
        for r in [100.0, 200.0, 300.0, 450.0, 600.0] {
            ctx.begin_path();
            let _ = ctx.arc(0.0, 0.0, r, 0.0, std::f64::consts::TAU);
            ctx.stroke();

            // Ring distance labels
            ctx.set_font("10px monospace");
            ctx.set_fill_style(&JsValue::from_str("rgba(0, 229, 255, 0.3)"));
            ctx.set_text_align("left");
            let _ = ctx.fill_text(&format!("{:.0}m", r), 4.0, -r + 12.0);
        }

        // Radar crosshairs
        ctx.begin_path();
        ctx.move_to(-canvas_w / 2.0, 0.0);
        ctx.line_to(canvas_w / 2.0, 0.0);
        ctx.move_to(0.0, -canvas_h / 2.0);
        ctx.line_to(0.0, canvas_h / 2.0);
        ctx.stroke();

        let center_x = center_pos.x as f64;
        let center_y = center_pos.y as f64;

        // 2. Draw Mega-Station
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

        // 3. Draw Asteroids relative to center
        for ast in asteroids {
            let ax = ast.position.x as f64 - center_x;
            let ay = ast.position.y as f64 - center_y;

            // Only draw if within radar viewport
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

        // 4. Draw Base Station if deployed
        if let Some(base) = my_base {
            let bx = base.position.x as f64 - center_x;
            let by = base.position.y as f64 - center_y;

            // Base signal range circle (pulsing border)
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

        // 5. Draw Ships & Drones
        for s in ships {
            let sx = s.position.x as f64 - center_x;
            let sy = s.position.y as f64 - center_y;

            // Mining laser beam if currently mining
            if s.is_mining {
                if let Some(target_id) = s.target_asteroid_id {
                    if let Some(ast) = asteroids.iter().find(|a| a.asteroid_id == target_id) {
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
                // Tier-0 Construction / Builder Ship (Larger, specialized)
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
                "Bauschiff".to_string()
            } else {
                format!("Drohne #{}", s.ship_id)
            };
            let _ = ctx.fill_text(&label, sx, sy - 20.0);
        }

        // 6. Draw Manual Target Marker if in Builder Mode
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
