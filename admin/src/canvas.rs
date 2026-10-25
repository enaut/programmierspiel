use crate::module_bindings::{Asteroid, BaseStation, Config, Player, Ship};
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

pub fn render_space_scene(
    #[cfg(target_arch = "wasm32")] ctx: &CanvasRenderingContext2d,
    width: f64,
    height: f64,
    camera_x: f64,
    camera_y: f64,
    zoom: f64,
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

        let half_w = width / 2.0;
        let half_h = height / 2.0;

        let min_x = camera_x - half_w / zoom;
        let max_x = camera_x + half_w / zoom;
        let min_y = camera_y - half_h / zoom;
        let max_y = camera_y + half_h / zoom;

        let _ = ctx.save();
        let _ = ctx.translate(half_w, half_h);
        let _ = ctx.scale(zoom, zoom);
        let _ = ctx.translate(-camera_x, -camera_y);

        // 2. Infinite Grid Lines
        draw_grid(ctx, min_x, max_x, min_y, max_y, zoom);

        // 3. Draw Central Mega-Station (at origin 0.0, 0.0)
        let mega_x = config.map_or(0.0, |c| c.megastation_pos.x as f64);
        let mega_y = config.map_or(0.0, |c| c.megastation_pos.y as f64);
        draw_megastation(ctx, mega_x, mega_y, anim_time);

        // 4. Draw Base Stations & Signal Radii
        for base in base_stations {
            let hue = player_map.get(&base.owner).map_or(200, |p| p.color_hue);
            draw_base_station(ctx, base, hue, anim_time);
        }

        let margin = 100.0;
        let cull_min_x = min_x - margin;
        let cull_max_x = max_x + margin;
        let cull_min_y = min_y - margin;
        let cull_max_y = max_y + margin;

        let mut asteroid_pos_map = HashMap::new();

        // 5. Draw Asteroids (Culled to viewport)
        for asteroid in asteroids {
            let ax = asteroid.position.x as f64;
            let ay = asteroid.position.y as f64;
            asteroid_pos_map.insert(asteroid.asteroid_id, (ax, ay));

            if ax >= cull_min_x && ax <= cull_max_x && ay >= cull_min_y && ay <= cull_max_y {
                draw_asteroid(ctx, asteroid, (ax, ay));
            }
        }

        // 6. Draw Mining Beams
        for ship in ships {
            if ship.is_mining {
                if let Some(target_id) = ship.target_asteroid_id {
                    if let Some(&ast_pos) = asteroid_pos_map.get(&target_id) {
                        draw_mining_laser(ctx, ship, ast_pos, anim_time);
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
fn draw_grid(
    ctx: &CanvasRenderingContext2d,
    min_x: f64,
    max_x: f64,
    min_y: f64,
    max_y: f64,
    zoom: f64,
) {
    let step = if zoom < 0.2 {
        2000.0
    } else if zoom < 0.6 {
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
            ctx.set_stroke_style(&JsValue::from_str("rgba(255, 255, 255, 0.05)"));
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
            ctx.set_stroke_style(&JsValue::from_str("rgba(255, 255, 255, 0.05)"));
            ctx.set_line_width(line_w);
        }
        ctx.stroke();
        y += step;
    }
}

#[cfg(target_arch = "wasm32")]
fn draw_megastation(ctx: &CanvasRenderingContext2d, x: f64, y: f64, time: f64) {
    let _ = ctx.save();
    let _ = ctx.translate(x, y);

    // Safe zone perimeter ring (250 units radius around Mega-Station)
    ctx.begin_path();
    let _ = ctx.arc(0.0, 0.0, 250.0, 0.0, std::f64::consts::TAU);
    ctx.set_stroke_style(&JsValue::from_str("rgba(80, 200, 255, 0.18)"));
    ctx.set_line_width(2.0);
    ctx.stroke();

    // Check if both Mega Station sprite assets are loaded and ready
    if let (Some(pads_img), Some(ring_img)) =
        (get_megastation_pads_image(), get_megastation_ring_image())
    {
        if pads_img.complete()
            && pads_img.natural_width() > 0
            && ring_img.complete()
            && ring_img.natural_width() > 0
        {
            // Soft pulsating ambient aura behind the station
            let pulse = (time * 1.5).sin() * 0.08 + 0.92;
            ctx.begin_path();
            let _ = ctx.arc(0.0, 0.0, 110.0 * pulse, 0.0, std::f64::consts::TAU);
            ctx.set_fill_style(&JsValue::from_str("rgba(92, 225, 230, 0.07)"));
            ctx.fill();

            let sprite_w = 220.0;
            let sprite_h = sprite_w * (373.0 / 668.0);

            // 1. Draw rotating ring layer (background, slowly rotating around center)
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
        }
    }

    // Label
    ctx.set_font("bold 14px sans-serif");
    ctx.set_text_align("center");
    ctx.set_fill_style(&JsValue::from_str("#e0f7fa"));
    let _ = ctx.fill_text("MEGA-STATION", 0.0, 95.0);

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
fn draw_asteroid(ctx: &CanvasRenderingContext2d, ast: &Asteroid, (x, y): (f64, f64)) {
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
fn draw_mining_laser(ctx: &CanvasRenderingContext2d, ship: &Ship, (ax, ay): (f64, f64), time: f64) {
    let sx = ship.position.x as f64;
    let sy = ship.position.y as f64;

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
