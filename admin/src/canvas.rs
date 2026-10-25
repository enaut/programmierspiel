use crate::module_bindings::{Asteroid, BaseStation, Config, Player, Ship};
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

#[derive(Default, Clone)]
struct AsteroidInterpState {
    last_server_pos: (f32, f32),
    last_update_time: f64,
}

thread_local! {
    static ASTEROID_TRACKER: RefCell<HashMap<u32, AsteroidInterpState>> = RefCell::new(HashMap::new());
}

#[cfg(target_arch = "wasm32")]
fn get_interpolated_asteroid_pos(
    ast: &Asteroid,
    anim_time: f64,
    world_w: f32,
    world_h: f32,
) -> (f64, f64) {
    ASTEROID_TRACKER.with(|tracker| {
        let mut map = tracker.borrow_mut();
        let state = map.entry(ast.asteroid_id).or_insert(AsteroidInterpState {
            last_server_pos: (ast.position.x, ast.position.y),
            last_update_time: anim_time,
        });

        // Check if server sent a new position
        let dx = (ast.position.x - state.last_server_pos.0).abs();
        let dy = (ast.position.y - state.last_server_pos.1).abs();
        if dx > 0.05 || dy > 0.05 {
            state.last_server_pos = (ast.position.x, ast.position.y);
            state.last_update_time = anim_time;
        }

        let dt = ((anim_time - state.last_update_time) as f32).clamp(0.0, 1.0);

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

        let world_w = config.map_or(3200.0, |c| c.world_size.x);
        let world_h = config.map_or(2000.0, |c| c.world_size.y);

        let mut interp_asteroid_positions = HashMap::new();

        // 5. Draw Asteroids
        for asteroid in asteroids {
            let interp_pos = get_interpolated_asteroid_pos(asteroid, anim_time, world_w, world_h);
            interp_asteroid_positions.insert(asteroid.asteroid_id, interp_pos);
            draw_asteroid(ctx, asteroid, interp_pos);
        }

        // 6. Draw Mining Beams
        for ship in ships {
            if ship.is_mining {
                if let Some(target_id) = ship.target_asteroid_id {
                    if let Some(&ast_pos) = interp_asteroid_positions.get(&target_id) {
                        draw_mining_laser(ctx, ship, ast_pos, anim_time);
                    }
                }
            }
        }

        ASTEROID_TRACKER.with(|tracker| {
            let mut map = tracker.borrow_mut();
            if map.len() > asteroids.len() + 15 {
                let active: HashSet<u32> = asteroids.iter().map(|a| a.asteroid_id).collect();
                map.retain(|id, _| active.contains(id));
            }
        });

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
