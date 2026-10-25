use crate::module_bindings::{
    BaseStation, Config, PerceivedAsteroid, PerceivedBase, PerceivedDrone, Player, Ship,
    UnidentifiedContact, Vec2,
};
use dioxus::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;

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

#[cfg(target_arch = "wasm32")]
pub fn get_interpolated_asteroid_pos(
    ast: &PerceivedAsteroid,
    _time: f64,
    _world_w: f32,
    _world_h: f32,
) -> (f64, f64) {
    (ast.position.x as f64, ast.position.y as f64)
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

#[cfg(target_arch = "wasm32")]
fn draw_infinite_grid(
    ctx: &CanvasRenderingContext2d,
    min_x: f64,
    max_x: f64,
    min_y: f64,
    max_y: f64,
    zoom: f64,
) {
    let step = if zoom < 0.25 {
        2000.0
    } else if zoom < 0.7 {
        1000.0
    } else {
        500.0
    };

    let start_x = (min_x / step).floor() * step;
    let end_x = (max_x / step).ceil() * step;
    let start_y = (min_y / step).floor() * step;
    let end_y = (max_y / step).ceil() * step;

    let line_w = 1.0 / zoom;

    let mut x = start_x;
    while x <= end_x {
        ctx.begin_path();
        ctx.move_to(x, min_y);
        ctx.line_to(x, max_y);
        if x.abs() < 1.0 {
            ctx.set_stroke_style(&JsValue::from_str("rgba(0, 229, 255, 0.4)"));
            ctx.set_line_width(line_w * 2.0);
        } else {
            ctx.set_stroke_style(&JsValue::from_str("rgba(0, 229, 255, 0.05)"));
            ctx.set_line_width(line_w);
        }
        ctx.stroke();
        x += step;
    }

    let mut y = start_y;
    while y <= end_y {
        ctx.begin_path();
        ctx.move_to(min_x, y);
        ctx.line_to(max_x, y);
        if y.abs() < 1.0 {
            ctx.set_stroke_style(&JsValue::from_str("rgba(0, 229, 255, 0.4)"));
            ctx.set_line_width(line_w * 2.0);
        } else {
            ctx.set_stroke_style(&JsValue::from_str("rgba(0, 229, 255, 0.05)"));
            ctx.set_line_width(line_w);
        }
        ctx.stroke();
        y += step;
    }
}

#[cfg(target_arch = "wasm32")]
fn draw_offscreen_indicator(
    ctx: &CanvasRenderingContext2d,
    screen_w: f64,
    screen_h: f64,
    camera_x: f64,
    camera_y: f64,
    zoom: f64,
    target_x: f64,
    target_y: f64,
    label: &str,
    color: &str,
) {
    let half_w = screen_w / 2.0;
    let half_h = screen_h / 2.0;

    let sx = half_w + (target_x - camera_x) * zoom;
    let sy = half_h + (target_y - camera_y) * zoom;

    let margin = 50.0;
    // Check if within visible bounds
    if sx >= margin && sx <= screen_w - margin && sy >= margin && sy <= screen_h - margin {
        return;
    }

    let dx = sx - half_w;
    let dy = sy - half_h;
    let angle = dy.atan2(dx);

    let edge_x = half_w - margin;
    let edge_y = half_h - margin;

    let tan = dy / (if dx.abs() < 1e-4 { 1e-4 } else { dx });
    let (ix, iy) = if dx.abs() * edge_y > dy.abs() * edge_x {
        let ix = if dx > 0.0 {
            half_w + edge_x
        } else {
            half_w - edge_x
        };
        let iy = (half_h + (ix - half_w) * tan).clamp(margin, screen_h - margin);
        (ix, iy)
    } else {
        let iy = if dy > 0.0 {
            half_h + edge_y
        } else {
            half_h - edge_y
        };
        let ix = (half_w + (iy - half_h) / tan).clamp(margin, screen_w - margin);
        (ix, iy)
    };

    let world_dist = ((target_x - camera_x).powi(2) + (target_y - camera_y).powi(2)).sqrt();

    let _ = ctx.save();
    let _ = ctx.translate(ix, iy);

    // Arrow pointer
    let _ = ctx.save();
    let _ = ctx.rotate(angle);
    ctx.begin_path();
    ctx.move_to(12.0, 0.0);
    ctx.line_to(-8.0, -8.0);
    ctx.line_to(-4.0, 0.0);
    ctx.line_to(-8.0, 8.0);
    ctx.close_path();
    ctx.set_fill_style(&JsValue::from_str(color));
    ctx.fill();
    let _ = ctx.restore();

    // Distance label
    ctx.set_font("bold 11px monospace");
    ctx.set_fill_style(&JsValue::from_str("#ffffff"));
    ctx.set_text_align("center");
    let text = format!("{} ({:.0}m)", label, world_dist);
    let text_y = if iy < half_h { 20.0 } else { -14.0 };
    let _ = ctx.fill_text(&text, 0.0, text_y);

    let _ = ctx.restore();
}

pub fn render_player_radar(
    #[cfg(target_arch = "wasm32")] ctx: &CanvasRenderingContext2d,
    canvas_w: f64,
    canvas_h: f64,
    dpr: f64,
    camera_x: f64,
    camera_y: f64,
    zoom: f64,
    sensor_hub_pos: Vec2,
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
        // 1. Dark radar background
        ctx.set_fill_style(&JsValue::from_str("#060a17"));
        ctx.fill_rect(0.0, 0.0, canvas_w * dpr, canvas_h * dpr);

        let half_w = canvas_w / 2.0;
        let half_h = canvas_h / 2.0;

        let _ = ctx.save();
        let _ = ctx.scale(dpr, dpr);
        let _ = ctx.translate(half_w, half_h);
        let _ = ctx.scale(zoom, zoom);
        let _ = ctx.translate(-camera_x, -camera_y);

        let min_x = camera_x - half_w / zoom;
        let max_x = camera_x + half_w / zoom;
        let min_y = camera_y - half_h / zoom;
        let max_y = camera_y + half_h / zoom;

        // Draw Infinite Grid
        draw_infinite_grid(ctx, min_x, max_x, min_y, max_y, zoom);

        // Precompute player lookup (identity -> player)
        let player_map: HashMap<_, _> = players.iter().map(|p| (p.identity, p)).collect();

        // Dynamic radar sensor range rings centered at sensor_hub_pos
        let hub_x = sensor_hub_pos.x as f64;
        let hub_y = sensor_hub_pos.y as f64;

        let drone_scan_r = my_base.map_or(150.0, |b| {
            150.0 + (b.drone_scanner_level.saturating_sub(1) as f64 * 50.0)
        });
        let base_scan_r = my_base.map_or(300.0, |b| {
            300.0 + (b.base_scanner_level.saturating_sub(1) as f64 * 100.0)
        });
        let horizon_r = my_base.map_or(600.0, |b| {
            600.0 + (b.radar_horizon_level.saturating_sub(1) as f64 * 150.0)
        });

        // Ambient radar glow within scan horizon
        ctx.begin_path();
        let _ = ctx.arc(hub_x, hub_y, horizon_r, 0.0, std::f64::consts::TAU);
        ctx.set_fill_style(&JsValue::from_str("rgba(0, 229, 255, 0.02)"));
        ctx.fill();

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
            let _ = ctx.arc(hub_x, hub_y, *r, 0.0, std::f64::consts::TAU);
            ctx.set_stroke_style(&JsValue::from_str(stroke_color));
            ctx.set_line_width(1.0 / zoom);
            ctx.stroke();

            // Ring distance label
            ctx.set_font("10px monospace");
            ctx.set_fill_style(&JsValue::from_str("rgba(0, 229, 255, 0.45)"));
            ctx.set_text_align("left");
            let _ = ctx.fill_text(label, hub_x + 6.0, hub_y - *r + 12.0);
        }

        // 2. Draw Central Mega-Station (Permanent Beacon at 0.0, 0.0)
        let mega_world_x = config.map_or(0.0, |c| c.megastation_pos.x as f64);
        let mega_world_y = config.map_or(0.0, |c| c.megastation_pos.y as f64);

        // Safe zone boundary (250 units radius around Mega-Station)
        ctx.begin_path();
        let _ = ctx.arc(
            mega_world_x,
            mega_world_y,
            250.0,
            0.0,
            std::f64::consts::TAU,
        );
        ctx.set_stroke_style(&JsValue::from_str("rgba(80, 200, 255, 0.22)"));
        ctx.set_line_width(2.0 / zoom);
        ctx.stroke();

        if let (Some(pads_img), Some(ring_img)) =
            (get_megastation_pads_image(), get_megastation_ring_image())
        {
            if pads_img.complete()
                && pads_img.natural_width() > 0
                && ring_img.complete()
                && ring_img.natural_width() > 0
            {
                let pulse = (time * 1.5).sin() * 0.08 + 0.92;
                ctx.begin_path();
                let _ = ctx.arc(
                    mega_world_x,
                    mega_world_y,
                    55.0 * pulse,
                    0.0,
                    std::f64::consts::TAU,
                );
                ctx.set_fill_style(&JsValue::from_str("rgba(92, 225, 230, 0.08)"));
                ctx.fill();

                let sprite_w = 110.0;
                let sprite_h = sprite_w * (373.0 / 668.0);

                let _ = ctx.save();
                let _ = ctx.translate(mega_world_x, mega_world_y);

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
        let _ = ctx.fill_text("MEGA-STATION", mega_world_x, mega_world_y + 46.0);

        // 3. Draw Unidentified Objects (radar blips)
        for contact in unidentified_objects {
            let ux = contact.position.x as f64;
            let uy = contact.position.y as f64;

            let pulse =
                ((time * 3.5 + (contact.contact_id as f64 * 0.7)).sin() * 0.25 + 0.75) * 4.5;
            ctx.begin_path();
            let _ = ctx.arc(ux, uy, pulse, 0.0, std::f64::consts::TAU);
            ctx.set_fill_style(&JsValue::from_str("rgba(239, 68, 68, 0.75)"));
            ctx.fill();
            ctx.set_stroke_style(&JsValue::from_str("rgba(248, 113, 113, 0.9)"));
            ctx.set_line_width(1.0 / zoom);
            ctx.stroke();
        }

        // 4. Draw Identified Asteroids
        let mut asteroid_pos_map = HashMap::new();

        for ast in visible_asteroids {
            let ax = ast.position.x as f64;
            let ay = ast.position.y as f64;
            asteroid_pos_map.insert(ast.asteroid_id, (ax, ay));

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
            ctx.set_line_width(1.0 / zoom);
            ctx.stroke();

            if selected_entity == Some(SelectedEntity::Asteroid(ast.asteroid_id)) {
                draw_target_reticle(ctx, ax, ay, (ast.radius as f64) * 0.75, time, "#00e5ff");
            }
        }

        // 5. Draw Bases
        // 5A. Foreign identified bases
        for base in visible_bases {
            let is_my_base = my_base.map_or(false, |b| b.base_id == base.base_id);
            if !is_my_base {
                let bx = base.position.x as f64;
                let by = base.position.y as f64;
                let b_hue = player_map.get(&base.owner).map_or(200, |p| p.color_hue);
                let b_name = player_map
                    .get(&base.owner)
                    .map_or("Basis".to_string(), |p| format!("Basis ({})", p.name));

                ctx.begin_path();
                let _ = ctx.arc(bx, by, 18.0, 0.0, std::f64::consts::TAU);
                ctx.set_fill_style(&JsValue::from_str("#0f172a"));
                ctx.fill();
                ctx.set_stroke_style(&JsValue::from_str(&format!("hsl({}, 90%, 55%)", b_hue)));
                ctx.set_line_width(2.5 / zoom);
                ctx.stroke();

                ctx.set_stroke_style(&JsValue::from_str("#94a3b8"));
                ctx.set_line_width(1.5 / zoom);
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

        // 5B. Own Home Base
        if let Some(base) = my_base {
            let bx = base.position.x as f64;
            let by = base.position.y as f64;

            let sig_r = base.signal_radius as f64;
            ctx.begin_path();
            let _ = ctx.arc(bx, by, sig_r, 0.0, std::f64::consts::TAU);
            ctx.set_stroke_style(&JsValue::from_str(&format!(
                "hsla({}, 90%, 65%, 0.25)",
                color_hue
            )));
            ctx.set_line_width(2.0 / zoom);
            ctx.stroke();

            ctx.set_fill_style(&JsValue::from_str(&format!(
                "hsla({}, 90%, 60%, 0.03)",
                color_hue
            )));
            ctx.fill();

            ctx.begin_path();
            let _ = ctx.arc(bx, by, 22.0, 0.0, std::f64::consts::TAU);
            ctx.set_fill_style(&JsValue::from_str("#0f172a"));
            ctx.fill();
            ctx.set_stroke_style(&JsValue::from_str(&format!("hsl({}, 95%, 60%)", color_hue)));
            ctx.set_line_width(3.0 / zoom);
            ctx.stroke();

            ctx.set_stroke_style(&JsValue::from_str("#38bdf8"));
            ctx.set_line_width(2.0 / zoom);
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

        // 6. Draw Foreign Drones
        for d in visible_drones {
            let is_my_drone = my_ships.iter().any(|s| s.ship_id == d.ship_id);
            if !is_my_drone {
                let dx = d.position.x as f64;
                let dy = d.position.y as f64;
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
                ctx.set_line_width(1.5 / zoom);
                ctx.stroke();

                let _ = ctx.restore();

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

        // 7. Draw Player's Own Ships
        for s in my_ships {
            let sx = s.position.x as f64;
            let sy = s.position.y as f64;

            // Mining laser beam
            if s.is_mining {
                if let Some(target_id) = s.target_asteroid_id {
                    let ast_coords = asteroid_pos_map.get(&target_id).copied().or_else(|| {
                        visible_asteroids
                            .iter()
                            .find(|a| a.asteroid_id == target_id)
                            .map(|a| (a.position.x as f64, a.position.y as f64))
                    });

                    if let Some((ast_x, ast_y)) = ast_coords {
                        ctx.begin_path();
                        ctx.move_to(sx, sy);
                        ctx.line_to(ast_x, ast_y);
                        ctx.set_stroke_style(&JsValue::from_str("#00e5ff"));
                        ctx.set_line_width(2.0 / zoom);
                        ctx.stroke();
                    }
                }
            }

            // Waypoint path
            if let Some(ref target) = s.target_position {
                let tx = target.x as f64;
                let ty = target.y as f64;

                ctx.set_line_dash(&JsValue::from(js_sys::Array::of2(
                    &JsValue::from_f64(6.0),
                    &JsValue::from_f64(6.0),
                )))
                .ok();
                ctx.begin_path();
                ctx.move_to(sx, sy);
                ctx.line_to(tx, ty);
                ctx.set_stroke_style(&JsValue::from_str("rgba(0, 229, 255, 0.4)"));
                ctx.set_line_width(1.5 / zoom);
                ctx.stroke();
                ctx.set_line_dash(&JsValue::from(js_sys::Array::new())).ok();

                ctx.begin_path();
                let _ = ctx.arc(tx, ty, 5.0, 0.0, std::f64::consts::TAU);
                ctx.set_stroke_style(&JsValue::from_str("#00e5ff"));
                ctx.set_line_width(1.5 / zoom);
                ctx.stroke();
            }

            let _ = ctx.save();
            let _ = ctx.translate(sx, sy);
            let _ = ctx.rotate(s.heading_rad as f64);

            if s.ship_tier == 0 {
                // Tier-0 Builder Ship
                ctx.begin_path();
                ctx.move_to(22.0, 0.0);
                ctx.line_to(-16.0, -14.0);
                ctx.line_to(-8.0, 0.0);
                ctx.line_to(-16.0, 14.0);
                ctx.close_path();

                ctx.set_fill_style(&JsValue::from_str(&format!("hsl({}, 90%, 55%)", color_hue)));
                ctx.fill();
                ctx.set_stroke_style(&JsValue::from_str("#ffffff"));
                ctx.set_line_width(2.0 / zoom);
                ctx.stroke();

                ctx.set_stroke_style(&JsValue::from_str("#38bdf8"));
                ctx.set_line_width(2.0 / zoom);
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
                ctx.set_line_width(1.5 / zoom);
                ctx.stroke();
            }

            let _ = ctx.restore();

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

        // 8. Manual Target Marker if in Builder Mode
        if let Some(target) = target_pos {
            let tx = target.x as f64;
            let ty = target.y as f64;

            let pulse = (time * 8.0).sin() * 3.0 + 12.0;
            ctx.begin_path();
            let _ = ctx.arc(tx, ty, pulse, 0.0, std::f64::consts::TAU);
            ctx.set_stroke_style(&JsValue::from_str("#00e676"));
            ctx.set_line_width(2.0 / zoom);
            ctx.stroke();

            ctx.begin_path();
            ctx.move_to(tx - 6.0, ty);
            ctx.line_to(tx + 6.0, ty);
            ctx.move_to(tx, ty - 6.0);
            ctx.line_to(tx, ty + 6.0);
            ctx.stroke();
        }

        let _ = ctx.restore();

        // Screen-space overlay for offscreen indicators
        let _ = ctx.save();
        let _ = ctx.scale(dpr, dpr);

        // Indicator for Central Mega-Station
        draw_offscreen_indicator(
            ctx,
            canvas_w,
            canvas_h,
            camera_x,
            camera_y,
            zoom,
            mega_world_x,
            mega_world_y,
            "Mega-Station",
            "#00e5ff",
        );

        // Indicator for Home Base / Sensor Origin
        draw_offscreen_indicator(
            ctx, canvas_w, canvas_h, camera_x, camera_y, zoom, hub_x, hub_y, "Basis", "#ffd700",
        );

        let _ = ctx.restore();
    }
}
