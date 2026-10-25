use crate::module_bindings::{
    BaseStation, Config, PerceivedAsteroid, PerceivedBase, PerceivedDrone, Player, Ship,
    UnidentifiedContact, Vec2,
};
use dioxus::prelude::*;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

#[cfg(target_arch = "wasm32")]
static MEGA_STATION_PADS_ASSET: Asset = asset!("/assets/mega-station-pads.png");
#[cfg(target_arch = "wasm32")]
static MEGA_STATION_RING_ASSET: Asset = asset!("/assets/mega-station-ring.png");

#[cfg(target_arch = "wasm32")]
thread_local! {
    static MEGA_STATION_PADS_IMAGE: RefCell<Option<web_sys::HtmlImageElement>> = const { RefCell::new(None) };
    static MEGA_STATION_RING_IMAGE: RefCell<Option<web_sys::HtmlImageElement>> = const { RefCell::new(None) };
}

#[cfg(target_arch = "wasm32")]
fn get_megastation_pads_image() -> Option<web_sys::HtmlImageElement> {
    MEGA_STATION_PADS_IMAGE.with(|cell| {
        let mut opt = cell.borrow_mut();
        if let Some(ref img) = *opt {
            return Some(img.clone());
        }
        if let Ok(img) = web_sys::HtmlImageElement::new() {
            img.set_src(&MEGA_STATION_PADS_ASSET.to_string());
            *opt = Some(img.clone());
            Some(img)
        } else {
            None
        }
    })
}

#[cfg(target_arch = "wasm32")]
fn get_megastation_ring_image() -> Option<web_sys::HtmlImageElement> {
    MEGA_STATION_RING_IMAGE.with(|cell| {
        let mut opt = cell.borrow_mut();
        if let Some(ref img) = *opt {
            return Some(img.clone());
        }
        if let Ok(img) = web_sys::HtmlImageElement::new() {
            img.set_src(&MEGA_STATION_RING_ASSET.to_string());
            *opt = Some(img.clone());
            Some(img)
        } else {
            None
        }
    })
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;
#[cfg(target_arch = "wasm32")]
use web_sys::CanvasRenderingContext2d;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SelectedEntity {
    Heimatbasis,
    Asteroid(u32),
    MyShip(u32),
    ForeignDrone(u32),
    ForeignBase(u32),
}

#[derive(Default, Clone)]
struct PerceivedAsteroidInterpState {
    last_server_pos: (f32, f32),
    last_update_time: f64,
}

thread_local! {
    static PERCEIVED_ASTEROID_TRACKER: RefCell<HashMap<u32, PerceivedAsteroidInterpState>> = RefCell::new(HashMap::new());
}

#[cfg(target_arch = "wasm32")]
pub fn get_interpolated_asteroid_pos(
    ast: &PerceivedAsteroid,
    time: f64,
    world_w: f32,
    world_h: f32,
) -> (f64, f64) {
    PERCEIVED_ASTEROID_TRACKER.with(|tracker| {
        let mut map = tracker.borrow_mut();
        let state = map
            .entry(ast.asteroid_id)
            .or_insert(PerceivedAsteroidInterpState {
                last_server_pos: (ast.position.x, ast.position.y),
                last_update_time: time,
            });

        let dx = (ast.position.x - state.last_server_pos.0).abs();
        let dy = (ast.position.y - state.last_server_pos.1).abs();
        if dx > 0.05 || dy > 0.05 {
            state.last_server_pos = (ast.position.x, ast.position.y);
            state.last_update_time = time;
        }

        let dt = ((time - state.last_update_time) as f32).clamp(0.0, 1.0);

        let margin = 50.0;
        let min_x = -margin;
        let max_x = world_w + margin;
        let span_x = max_x - min_x;

        let min_y = -margin;
        let max_y = world_h + margin;
        let span_y = max_y - min_y;

        let mut x = state.last_server_pos.0 + ast.velocity.x * dt;
        let mut y = state.last_server_pos.1 + ast.velocity.y * dt;

        if span_x > 0.0 {
            while x < min_x {
                x += span_x;
            }
            while x > max_x {
                x -= span_x;
            }
        }

        if span_y > 0.0 {
            while y < min_y {
                y += span_y;
            }
            while y > max_y {
                y -= span_y;
            }
        }

        (x as f64, y as f64)
    })
}

#[cfg(target_arch = "wasm32")]
fn draw_target_reticle(
    ctx: &CanvasRenderingContext2d,
    x: f64,
    y: f64,
    radius: f64,
    time: f64,
    color: &str,
) {
    let pulse = (time * 6.0).sin() * 2.0;
    let r = radius + 8.0 + pulse;
    let arm = 7.0;

    ctx.set_stroke_style(&JsValue::from_str(color));
    ctx.set_line_width(2.0);

    // Top-left corner
    ctx.begin_path();
    ctx.move_to(x - r, y - r + arm);
    ctx.line_to(x - r, y - r);
    ctx.line_to(x - r + arm, y - r);
    ctx.stroke();

    // Top-right corner
    ctx.begin_path();
    ctx.move_to(x + r - arm, y - r);
    ctx.line_to(x + r, y - r);
    ctx.line_to(x + r, y - r + arm);
    ctx.stroke();

    // Bottom-right corner
    ctx.begin_path();
    ctx.move_to(x + r, y + r - arm);
    ctx.line_to(x + r, y + r);
    ctx.line_to(x + r - arm, y + r);
    ctx.stroke();

    // Bottom-left corner
    ctx.begin_path();
    ctx.move_to(x - r + arm, y + r);
    ctx.line_to(x - r, y + r);
    ctx.line_to(x - r, y + r - arm);
    ctx.stroke();
}

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
    selected_entity: Option<SelectedEntity>,
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

        if let (Some(pads_img), Some(ring_img)) =
            (get_megastation_pads_image(), get_megastation_ring_image())
        {
            if pads_img.complete()
                && pads_img.natural_width() > 0
                && ring_img.complete()
                && ring_img.natural_width() > 0
            {
                // Soft pulsing ambient aura on radar
                let pulse = (time * 1.5).sin() * 0.08 + 0.92;
                ctx.begin_path();
                let _ = ctx.arc(mega_x, mega_y, 55.0 * pulse, 0.0, std::f64::consts::TAU);
                ctx.set_fill_style(&JsValue::from_str("rgba(92, 225, 230, 0.08)"));
                ctx.fill();

                let sprite_w = 110.0;
                let sprite_h = sprite_w * (373.0 / 668.0);

                let _ = ctx.save();
                let _ = ctx.translate(mega_x, mega_y);

                // 1. Draw rotating ring layer (background)
                let ring_angle = (time * 0.15).rem_euclid(std::f64::consts::TAU);
                let _ = ctx.save();
                let _ = ctx.rotate(ring_angle);
                let _ = ctx.draw_image_with_html_image_element_and_dw_and_dh(
                    &ring_img,
                    -sprite_w / 2.0,
                    -sprite_h / 2.0,
                    sprite_w,
                    sprite_h,
                );
                let _ = ctx.restore();

                // 2. Draw stationary pads & panels layer (foreground)
                let _ = ctx.draw_image_with_html_image_element_and_dw_and_dh(
                    &pads_img,
                    -sprite_w / 2.0,
                    -sprite_h / 2.0,
                    sprite_w,
                    sprite_h,
                );

                let _ = ctx.restore();
            }
        }

        ctx.set_font("bold 11px sans-serif");
        ctx.set_text_align("center");
        ctx.set_fill_style(&JsValue::from_str("#e0f7fa"));
        let _ = ctx.fill_text("MEGA-STATION", mega_x, mega_y + 46.0);

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
        let world_w = config.map_or(3200.0, |c| c.world_size.x);
        let world_h = config.map_or(2000.0, |c| c.world_size.y);

        let mut asteroid_pos_map = HashMap::new();

        for ast in visible_asteroids {
            let (ix, iy) = get_interpolated_asteroid_pos(ast, time, world_w, world_h);
            asteroid_pos_map.insert(ast.asteroid_id, (ix, iy));

            let ax = ix - center_x;
            let ay = iy - center_y;

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

                if selected_entity == Some(SelectedEntity::Asteroid(ast.asteroid_id)) {
                    draw_target_reticle(ctx, ax, ay, (ast.radius as f64) * 0.75, time, "#00e5ff");
                }
            }
        }

        PERCEIVED_ASTEROID_TRACKER.with(|tracker| {
            let mut map = tracker.borrow_mut();
            if map.len() > visible_asteroids.len() + 15 {
                let active: HashSet<u32> =
                    visible_asteroids.iter().map(|a| a.asteroid_id).collect();
                map.retain(|id, _| active.contains(id));
            }
        });

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

                if selected_entity == Some(SelectedEntity::ForeignBase(base.base_id)) {
                    draw_target_reticle(ctx, bx, by, 22.0, time, "#f59e0b");
                }
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

            if selected_entity == Some(SelectedEntity::Heimatbasis) {
                draw_target_reticle(ctx, bx, by, 26.0, time, "#00e5ff");
            }
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

                if selected_entity == Some(SelectedEntity::ForeignDrone(d.ship_id)) {
                    draw_target_reticle(ctx, dx, dy, 18.0, time, "#f59e0b");
                }
            }
        }

        // 7. Draw Player's Own Ships & Drones (with full telemetry & movement paths)
        for s in my_ships {
            let sx = s.position.x as f64 - center_x;
            let sy = s.position.y as f64 - center_y;

            // Mining laser beam if currently mining
            if s.is_mining {
                if let Some(target_id) = s.target_asteroid_id {
                    let ast_coords = asteroid_pos_map.get(&target_id).copied().or_else(|| {
                        visible_asteroids
                            .iter()
                            .find(|a| a.asteroid_id == target_id)
                            .map(|a| (a.position.x as f64, a.position.y as f64))
                    });

                    if let Some((ix, iy)) = ast_coords {
                        let ast_x = ix - center_x;
                        let ast_y = iy - center_y;
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

            if selected_entity == Some(SelectedEntity::MyShip(s.ship_id)) {
                let r = if s.ship_tier == 0 { 24.0 } else { 20.0 };
                draw_target_reticle(ctx, sx, sy, r, time, "#00e5ff");
            }
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
