use crate::module_bindings::{Asteroid, BaseStation, Config, Player, Ship};
use std::collections::HashMap;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;
#[cfg(target_arch = "wasm32")]
use web_sys::CanvasRenderingContext2d;

pub fn render_space_scene(
    #[cfg(target_arch = "wasm32")] ctx: &CanvasRenderingContext2d,
    width: f64,
    height: f64,
    config: Option<&Config>,
    players: &[Player],
    base_stations: &[BaseStation],
    asteroids: &[Asteroid],
    ships: &[Ship],
    anim_time: f64,
) {
    #[cfg(target_arch = "wasm32")]
    {
        // 1. Clear & Dark Space Background
        ctx.set_fill_style(&JsValue::from_str("#070b19"));
        ctx.fill_rect(0.0, 0.0, width, height);

        // Precompute player lookup (identity -> player)
        let player_map: HashMap<_, _> = players.iter().map(|p| (p.identity, p)).collect();

        // World scale transformation (fit world_size into canvas view)
        let world_w = config.map_or(3200.0, |c| c.world_size.x as f64);
        let world_h = config.map_or(2000.0, |c| c.world_size.y as f64);
        let scale_x = width / world_w;
        let scale_y = height / world_h;
        let scale = scale_x.min(scale_y);

        let offset_x = (width - world_w * scale) / 2.0;
        let offset_y = (height - world_h * scale) / 2.0;

        let _ = ctx.save();
        let _ = ctx.translate(offset_x, offset_y);
        let _ = ctx.scale(scale, scale);

        // 2. Faint Grid Lines
        draw_grid(ctx, world_w, world_h, scale);

        // 3. Draw Central Mega-Station
        let mega_x = config.map_or(1600.0, |c| c.megastation_pos.x as f64);
        let mega_y = config.map_or(1000.0, |c| c.megastation_pos.y as f64);
        draw_megastation(ctx, mega_x, mega_y, anim_time);

        // 4. Draw Base Stations & Signal Radii
        for base in base_stations {
            let hue = player_map.get(&base.owner).map_or(200, |p| p.color_hue);
            draw_base_station(ctx, base, hue, anim_time);
        }

        // 5. Draw Asteroids
        for asteroid in asteroids {
            draw_asteroid(ctx, asteroid);
        }

        // 6. Draw Mining Beams
        let asteroid_map: HashMap<_, _> = asteroids.iter().map(|a| (a.asteroid_id, a)).collect();
        for ship in ships {
            if ship.is_mining {
                if let Some(target_id) = ship.target_asteroid_id {
                    if let Some(ast) = asteroid_map.get(&target_id) {
                        draw_mining_laser(ctx, ship, ast, anim_time);
                    }
                }
            }
        }

        // 7. Draw Ships
        for ship in ships {
            let hue = player_map.get(&ship.owner).map_or(180, |p| p.color_hue);
            draw_ship(ctx, ship, hue, anim_time);
        }

        let _ = ctx.restore();
    }
}

#[cfg(target_arch = "wasm32")]
fn draw_grid(ctx: &CanvasRenderingContext2d, w: f64, h: f64, scale: f64) {
    ctx.set_stroke_style(&JsValue::from_str("rgba(255, 255, 255, 0.04)"));
    ctx.set_line_width(1.0 / scale);

    let step = 200.0;
    let mut x = 0.0;
    while x <= w {
        ctx.begin_path();
        ctx.move_to(x, 0.0);
        ctx.line_to(x, h);
        ctx.stroke();
        x += step;
    }

    let mut y = 0.0;
    while y <= h {
        ctx.begin_path();
        ctx.move_to(0.0, y);
        ctx.line_to(w, y);
        ctx.stroke();
        y += step;
    }
}

#[cfg(target_arch = "wasm32")]
fn draw_megastation(ctx: &CanvasRenderingContext2d, x: f64, y: f64, time: f64) {
    let _ = ctx.save();
    let _ = ctx.translate(x, y);

    // Safe zone perimeter ring
    ctx.begin_path();
    let _ = ctx.arc(0.0, 0.0, 250.0, 0.0, std::f64::consts::TAU);
    ctx.set_stroke_style(&JsValue::from_str("rgba(80, 200, 255, 0.15)"));
    ctx.set_line_width(2.0);
    ctx.stroke();

    // Rotating outer docking spokes
    let _ = ctx.rotate(time * 0.15);
    ctx.set_stroke_style(&JsValue::from_str("rgba(160, 220, 255, 0.6)"));
    ctx.set_line_width(4.0);
    for i in 0..6 {
        let angle = i as f64 * (std::f64::consts::TAU / 6.0);
        ctx.begin_path();
        ctx.move_to(angle.cos() * 30.0, angle.sin() * 30.0);
        ctx.line_to(angle.cos() * 70.0, angle.sin() * 70.0);
        ctx.stroke();
    }

    // Outer ring
    ctx.begin_path();
    let _ = ctx.arc(0.0, 0.0, 70.0, 0.0, std::f64::consts::TAU);
    ctx.stroke();

    // Core Station Hub
    ctx.begin_path();
    let _ = ctx.arc(0.0, 0.0, 35.0, 0.0, std::f64::consts::TAU);
    ctx.set_fill_style(&JsValue::from_str("#1c3b57"));
    ctx.fill();
    ctx.set_stroke_style(&JsValue::from_str("#5ce1e6"));
    ctx.set_line_width(3.0);
    ctx.stroke();

    // Glowing core
    let pulse = (time * 2.0).sin() * 0.2 + 0.8;
    ctx.begin_path();
    let _ = ctx.arc(0.0, 0.0, 15.0 * pulse, 0.0, std::f64::consts::TAU);
    ctx.set_fill_style(&JsValue::from_str("rgba(92, 225, 230, 0.8)"));
    ctx.fill();

    // Label
    ctx.set_font("14px sans-serif");
    ctx.set_text_align("center");
    ctx.set_fill_style(&JsValue::from_str("#e0f7fa"));
    let _ = ctx.fill_text("MEGA-STATION", 0.0, 100.0);

    let _ = ctx.restore();
}

#[cfg(target_arch = "wasm32")]
fn draw_base_station(ctx: &CanvasRenderingContext2d, base: &BaseStation, hue: u16, _time: f64) {
    let x = base.position.x as f64;
    let y = base.position.y as f64;
    let radius = base.signal_radius as f64;

    // Signal range circular boundary
    ctx.begin_path();
    let _ = ctx.arc(x, y, radius, 0.0, std::f64::consts::TAU);
    ctx.set_stroke_style(&JsValue::from_str(&format!(
        "hsla({}, 85%, 65%, 0.12)",
        hue
    )));
    ctx.set_line_width(2.0);
    ctx.stroke();

    // Base Station Hub
    let _ = ctx.save();
    let _ = ctx.translate(x, y);

    // Outer ring
    ctx.begin_path();
    let _ = ctx.arc(0.0, 0.0, 24.0, 0.0, std::f64::consts::TAU);
    ctx.set_fill_style(&JsValue::from_str("#0f1b29"));
    ctx.fill();
    ctx.set_stroke_style(&JsValue::from_str(&format!("hsl({}, 90%, 60%)", hue)));
    ctx.set_line_width(3.0);
    ctx.stroke();

    // Inner icon / ore count
    ctx.set_font("bold 11px sans-serif");
    ctx.set_text_align("center");
    ctx.set_fill_style(&JsValue::from_str("#ffffff"));
    let _ = ctx.fill_text(&format!("{} Erz", base.stored_ore), 0.0, 4.0);

    let _ = ctx.restore();
}

#[cfg(target_arch = "wasm32")]
fn draw_asteroid(ctx: &CanvasRenderingContext2d, ast: &Asteroid) {
    let x = ast.position.x as f64;
    let y = ast.position.y as f64;
    let r = ast.radius as f64;

    let (color, border) = match ast.resource_type {
        3 => ("#7b1fa2", "#e1bee7"), // Titanium (Purple/Gold)
        2 => ("#00838f", "#80deea"), // Ice (Cyan)
        1 => ("#455a64", "#cfd8dc"), // Iron (Slate)
        _ => ("#5d4037", "#bcaaa4"), // Generic (Brown/Orange)
    };

    ctx.begin_path();
    let _ = ctx.arc(x, y, r, 0.0, std::f64::consts::TAU);
    ctx.set_fill_style(&JsValue::from_str(color));
    ctx.fill();
    ctx.set_stroke_style(&JsValue::from_str(border));
    ctx.set_line_width(1.5);
    ctx.stroke();

    // Ore text
    ctx.set_font("10px sans-serif");
    ctx.set_text_align("center");
    ctx.set_fill_style(&JsValue::from_str("#ffffff"));
    let _ = ctx.fill_text(&format!("{}", ast.remaining_ore), x, y + 3.0);
}

#[cfg(target_arch = "wasm32")]
fn draw_mining_laser(ctx: &CanvasRenderingContext2d, ship: &Ship, ast: &Asteroid, time: f64) {
    let sx = ship.position.x as f64;
    let sy = ship.position.y as f64;
    let ax = ast.position.x as f64;
    let ay = ast.position.y as f64;

    let pulse = (time * 15.0).sin().abs() * 0.4 + 0.6;
    ctx.begin_path();
    ctx.move_to(sx, sy);
    ctx.line_to(ax, ay);
    ctx.set_stroke_style(&JsValue::from_str(&format!(
        "rgba(255, 230, 80, {})",
        pulse
    )));
    ctx.set_line_width(2.5);
    ctx.stroke();
}

#[cfg(target_arch = "wasm32")]
fn draw_ship(ctx: &CanvasRenderingContext2d, ship: &Ship, hue: u16, time: f64) {
    let x = ship.position.x as f64;
    let y = ship.position.y as f64;
    let heading = ship.heading_rad as f64;

    let _ = ctx.save();
    let _ = ctx.translate(x, y);

    // If stranded, draw pulsing red distress beacon
    if ship.is_stranded {
        let pulse = (time * 4.0).sin() * 8.0 + 20.0;
        ctx.begin_path();
        let _ = ctx.arc(0.0, 0.0, pulse, 0.0, std::f64::consts::TAU);
        ctx.set_stroke_style(&JsValue::from_str("rgba(255, 60, 60, 0.7)"));
        ctx.set_line_width(2.0);
        ctx.stroke();
    }

    let _ = ctx.rotate(heading);

    // Engine thruster flame if moving
    if ship.speed > 5.0 {
        let flame = (time * 25.0).sin().abs() * 8.0 + 8.0;
        ctx.begin_path();
        ctx.move_to(-12.0, -4.0);
        ctx.line_to(-12.0 - flame, 0.0);
        ctx.line_to(-12.0, 4.0);
        ctx.set_fill_style(&JsValue::from_str(&format!("hsl({}, 100%, 70%)", hue)));
        ctx.fill();
    }

    // Ship Hull (triangle)
    ctx.begin_path();
    ctx.move_to(14.0, 0.0);
    ctx.line_to(-10.0, -9.0);
    ctx.line_to(-6.0, 0.0);
    ctx.line_to(-10.0, 9.0);
    ctx.close_path();

    ctx.set_fill_style(&JsValue::from_str(&format!("hsl({}, 75%, 45%)", hue)));
    ctx.fill();
    ctx.set_stroke_style(&JsValue::from_str(&format!("hsl({}, 95%, 70%)", hue)));
    ctx.set_line_width(1.5);
    ctx.stroke();

    let _ = ctx.restore();

    // Mini Energy / Cargo Bars above ship
    let bar_w = 24.0;
    let bar_h = 3.0;
    let bar_x = x - bar_w / 2.0;
    let bar_y = y - 18.0;

    // Energy bar (cyan)
    ctx.set_fill_style(&JsValue::from_str("rgba(0, 0, 0, 0.5)"));
    ctx.fill_rect(bar_x, bar_y, bar_w, bar_h);
    let energy_ratio = (ship.energy / ship.max_energy.max(1.0)) as f64;
    ctx.set_fill_style(&JsValue::from_str("#00e5ff"));
    ctx.fill_rect(bar_x, bar_y, bar_w * energy_ratio.clamp(0.0, 1.0), bar_h);

    // Cargo bar (gold)
    ctx.set_fill_style(&JsValue::from_str("rgba(0, 0, 0, 0.5)"));
    ctx.fill_rect(bar_x, bar_y + 4.0, bar_w, bar_h);
    let cargo_ratio = ship.cargo_ore as f64 / ship.max_cargo.max(1) as f64;
    ctx.set_fill_style(&JsValue::from_str("#ffd700"));
    ctx.fill_rect(
        bar_x,
        bar_y + 4.0,
        bar_w * cargo_ratio.clamp(0.0, 1.0),
        bar_h,
    );
}
