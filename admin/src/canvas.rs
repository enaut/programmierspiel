use crate::module_bindings::{Asteroid, BaseStation, Config, Player, Ship};
use dioxus::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;

#[cfg(target_arch = "wasm32")]
static MEGA_STATION_PADS_ASSET: Asset = asset!("/assets/mega-station-pads.png");
#[cfg(target_arch = "wasm32")]
static MEGA_STATION_RING_ASSET: Asset = asset!("/assets/mega-station-ring.png");
#[cfg(target_arch = "wasm32")]
static ASTEROID1_BASE_ASSET: Asset = asset!("/assets/asteroid1.png");
#[cfg(target_arch = "wasm32")]
static ASTEROID1_MASK_ASSET: Asset = asset!("/assets/asteroid1-mask.png");
#[cfg(target_arch = "wasm32")]
static ASTEROID2_BASE_ASSET: Asset = asset!("/assets/asteroid2.png");
#[cfg(target_arch = "wasm32")]
static ASTEROID2_MASK_ASSET: Asset = asset!("/assets/asteroid2-mask.png");
#[cfg(target_arch = "wasm32")]
static PLAYER_STATION_BASE_ASSET: Asset = asset!("/assets/player-station.png");
#[cfg(target_arch = "wasm32")]
static PLAYER_STATION_MASK1_ASSET: Asset = asset!("/assets/player-station-mask1.png");
#[cfg(target_arch = "wasm32")]
static PLAYER_STATION_MASK2_ASSET: Asset = asset!("/assets/player-station-mask2.png");
#[cfg(target_arch = "wasm32")]
static DRONE1_BASE_ASSET: Asset = asset!("/assets/drone1.png");
#[cfg(target_arch = "wasm32")]
static DRONE1_MASK1_ASSET: Asset = asset!("/assets/drone1-mask1.png");
#[cfg(target_arch = "wasm32")]
static DRONE1_MASK2_ASSET: Asset = asset!("/assets/drone1-mask2.png");
#[cfg(target_arch = "wasm32")]
static DRONE_TIER2_BASE_ASSET: Asset = asset!("/assets/drone-tier2.png");
#[cfg(target_arch = "wasm32")]
static DRONE_TIER2_MASK1_ASSET: Asset = asset!("/assets/drone-tier2-mask1.png");
#[cfg(target_arch = "wasm32")]
static DRONE_TIER2_MASK2_ASSET: Asset = asset!("/assets/drone-tier2-mask2.png");

#[cfg(target_arch = "wasm32")]
struct SpriteAssetPair {
    base_asset: &'static Asset,
    mask_asset: &'static Asset,
}

#[cfg(target_arch = "wasm32")]
type DualMaskImages = (
    web_sys::HtmlImageElement,
    web_sys::HtmlImageElement,
    web_sys::HtmlImageElement,
);

#[cfg(target_arch = "wasm32")]
thread_local! {
    static MEGA_STATION_PADS_IMAGE: RefCell<Option<web_sys::HtmlImageElement>> = const { RefCell::new(None) };
    static MEGA_STATION_RING_IMAGE: RefCell<Option<web_sys::HtmlImageElement>> = const { RefCell::new(None) };

    static ASTEROID_PAIRS: [SpriteAssetPair; 2] = [
        SpriteAssetPair { base_asset: &ASTEROID1_BASE_ASSET, mask_asset: &ASTEROID1_MASK_ASSET },
        SpriteAssetPair { base_asset: &ASTEROID2_BASE_ASSET, mask_asset: &ASTEROID2_MASK_ASSET },
    ];
    static ASTEROID_IMAGES: RefCell<[Option<(web_sys::HtmlImageElement, web_sys::HtmlImageElement)>; 2]> = const { RefCell::new([None, None]) };
    static ASTEROID_CANVAS_CACHE: RefCell<HashMap<(u8, u8), web_sys::HtmlCanvasElement>> = RefCell::new(HashMap::new());

    static PLAYER_STATION_IMAGES: RefCell<Option<DualMaskImages>> = const { RefCell::new(None) };
    static PLAYER_STATION_CANVAS_CACHE: RefCell<HashMap<(u16, u16), web_sys::HtmlCanvasElement>> = RefCell::new(HashMap::new());

    static DRONE1_IMAGES: RefCell<Option<DualMaskImages>> = const { RefCell::new(None) };
    static DRONE1_CANVAS_CACHE: RefCell<HashMap<(u16, u16), web_sys::HtmlCanvasElement>> = RefCell::new(HashMap::new());

    static DRONE_TIER2_IMAGES: RefCell<Option<DualMaskImages>> = const { RefCell::new(None) };
    static DRONE_TIER2_CANVAS_CACHE: RefCell<HashMap<(u16, u16), web_sys::HtmlCanvasElement>> = RefCell::new(HashMap::new());
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
fn get_asteroid_images(
    model: usize,
) -> Option<(web_sys::HtmlImageElement, web_sys::HtmlImageElement)> {
    let idx = model % 2;
    ASTEROID_IMAGES.with(|cell| {
        let mut opt = cell.borrow_mut();
        if let Some(ref pair) = opt[idx] {
            return Some(pair.clone());
        }
        let (base_asset, mask_asset) = ASTEROID_PAIRS.with(|pairs| {
            (
                pairs[idx].base_asset.to_string(),
                pairs[idx].mask_asset.to_string(),
            )
        });
        if let (Ok(base_img), Ok(mask_img)) = (
            web_sys::HtmlImageElement::new(),
            web_sys::HtmlImageElement::new(),
        ) {
            base_img.set_src(&base_asset);
            mask_img.set_src(&mask_asset);
            let pair = (base_img, mask_img);
            opt[idx] = Some(pair.clone());
            Some(pair)
        } else {
            None
        }
    })
}

#[cfg(target_arch = "wasm32")]
fn get_player_station_images() -> Option<DualMaskImages> {
    PLAYER_STATION_IMAGES.with(|cell| {
        let mut opt = cell.borrow_mut();
        if let Some(ref trio) = *opt {
            return Some(trio.clone());
        }
        if let (Ok(base_img), Ok(mask1_img), Ok(mask2_img)) = (
            web_sys::HtmlImageElement::new(),
            web_sys::HtmlImageElement::new(),
            web_sys::HtmlImageElement::new(),
        ) {
            base_img.set_src(&PLAYER_STATION_BASE_ASSET.to_string());
            mask1_img.set_src(&PLAYER_STATION_MASK1_ASSET.to_string());
            mask2_img.set_src(&PLAYER_STATION_MASK2_ASSET.to_string());
            let trio = (base_img, mask1_img, mask2_img);
            *opt = Some(trio.clone());
            Some(trio)
        } else {
            None
        }
    })
}

#[cfg(target_arch = "wasm32")]
fn get_drone1_images() -> Option<DualMaskImages> {
    DRONE1_IMAGES.with(|cell| {
        let mut opt = cell.borrow_mut();
        if let Some(ref trio) = *opt {
            return Some(trio.clone());
        }
        if let (Ok(base_img), Ok(mask1_img), Ok(mask2_img)) = (
            web_sys::HtmlImageElement::new(),
            web_sys::HtmlImageElement::new(),
            web_sys::HtmlImageElement::new(),
        ) {
            base_img.set_src(&DRONE1_BASE_ASSET.to_string());
            mask1_img.set_src(&DRONE1_MASK1_ASSET.to_string());
            mask2_img.set_src(&DRONE1_MASK2_ASSET.to_string());
            let trio = (base_img, mask1_img, mask2_img);
            *opt = Some(trio.clone());
            Some(trio)
        } else {
            None
        }
    })
}

#[cfg(target_arch = "wasm32")]
fn get_drone_tier2_images() -> Option<DualMaskImages> {
    DRONE_TIER2_IMAGES.with(|cell| {
        let mut opt = cell.borrow_mut();
        if let Some(ref trio) = *opt {
            return Some(trio.clone());
        }
        if let (Ok(base_img), Ok(mask1_img), Ok(mask2_img)) = (
            web_sys::HtmlImageElement::new(),
            web_sys::HtmlImageElement::new(),
            web_sys::HtmlImageElement::new(),
        ) {
            base_img.set_src(&DRONE_TIER2_BASE_ASSET.to_string());
            mask1_img.set_src(&DRONE_TIER2_MASK1_ASSET.to_string());
            mask2_img.set_src(&DRONE_TIER2_MASK2_ASSET.to_string());
            let trio = (base_img, mask1_img, mask2_img);
            *opt = Some(trio.clone());
            Some(trio)
        } else {
            None
        }
    })
}

#[cfg(target_arch = "wasm32")]
fn is_image_ready(img: &web_sys::HtmlImageElement) -> bool {
    img.complete() && img.natural_width() > 0
}

#[cfg(target_arch = "wasm32")]
fn render_tinted_sprite(
    base_img: &web_sys::HtmlImageElement,
    mask_img: &web_sys::HtmlImageElement,
    tint_color: &str,
) -> Option<web_sys::HtmlCanvasElement> {
    use wasm_bindgen::JsCast;

    let window = web_sys::window()?;
    let document = window.document()?;
    let size = 512.0;

    let result_canvas: web_sys::HtmlCanvasElement =
        document.create_element("canvas").ok()?.dyn_into().ok()?;
    result_canvas.set_width(512);
    result_canvas.set_height(512);
    let result_ctx: web_sys::CanvasRenderingContext2d =
        result_canvas.get_context("2d").ok()??.dyn_into().ok()?;

    let _ =
        result_ctx.draw_image_with_html_image_element_and_dw_and_dh(base_img, 0.0, 0.0, size, size);

    let scratch_canvas: web_sys::HtmlCanvasElement =
        document.create_element("canvas").ok()?.dyn_into().ok()?;
    scratch_canvas.set_width(512);
    scratch_canvas.set_height(512);
    let scratch_ctx: web_sys::CanvasRenderingContext2d =
        scratch_canvas.get_context("2d").ok()??.dyn_into().ok()?;

    scratch_ctx.set_fill_style(&JsValue::from_str(tint_color));
    scratch_ctx.fill_rect(0.0, 0.0, size, size);

    let _ = scratch_ctx.set_global_composite_operation("destination-in");
    let _ = scratch_ctx
        .draw_image_with_html_image_element_and_dw_and_dh(mask_img, 0.0, 0.0, size, size);

    let _ = result_ctx.set_global_composite_operation("multiply");
    let _ = result_ctx.draw_image_with_html_canvas_element_and_dw_and_dh(
        &scratch_canvas,
        0.0,
        0.0,
        size,
        size,
    );

    let _ = result_ctx.set_global_composite_operation("source-over");
    result_ctx.set_global_alpha(0.35);
    let _ = result_ctx.draw_image_with_html_canvas_element_and_dw_and_dh(
        &scratch_canvas,
        0.0,
        0.0,
        size,
        size,
    );
    result_ctx.set_global_alpha(1.0);

    Some(result_canvas)
}

#[cfg(target_arch = "wasm32")]
fn render_dual_tinted_sprite(
    base_img: &web_sys::HtmlImageElement,
    mask1_img: &web_sys::HtmlImageElement,
    mask2_img: &web_sys::HtmlImageElement,
    color1: &str,
    color2: &str,
) -> Option<web_sys::HtmlCanvasElement> {
    use wasm_bindgen::JsCast;

    let window = web_sys::window()?;
    let document = window.document()?;
    let size = 512.0;

    let result_canvas: web_sys::HtmlCanvasElement =
        document.create_element("canvas").ok()?.dyn_into().ok()?;
    result_canvas.set_width(512);
    result_canvas.set_height(512);
    let result_ctx: web_sys::CanvasRenderingContext2d =
        result_canvas.get_context("2d").ok()??.dyn_into().ok()?;

    let _ =
        result_ctx.draw_image_with_html_image_element_and_dw_and_dh(base_img, 0.0, 0.0, size, size);

    let scratch_canvas: web_sys::HtmlCanvasElement =
        document.create_element("canvas").ok()?.dyn_into().ok()?;
    scratch_canvas.set_width(512);
    scratch_canvas.set_height(512);
    let scratch_ctx: web_sys::CanvasRenderingContext2d =
        scratch_canvas.get_context("2d").ok()??.dyn_into().ok()?;

    // 1. Primary Tint (Mask 1)
    scratch_ctx.set_fill_style(&JsValue::from_str(color1));
    scratch_ctx.fill_rect(0.0, 0.0, size, size);
    let _ = scratch_ctx.set_global_composite_operation("destination-in");
    let _ = scratch_ctx
        .draw_image_with_html_image_element_and_dw_and_dh(mask1_img, 0.0, 0.0, size, size);

    let _ = result_ctx.set_global_composite_operation("multiply");
    let _ = result_ctx.draw_image_with_html_canvas_element_and_dw_and_dh(
        &scratch_canvas,
        0.0,
        0.0,
        size,
        size,
    );
    let _ = result_ctx.set_global_composite_operation("source-over");
    result_ctx.set_global_alpha(0.35);
    let _ = result_ctx.draw_image_with_html_canvas_element_and_dw_and_dh(
        &scratch_canvas,
        0.0,
        0.0,
        size,
        size,
    );
    result_ctx.set_global_alpha(1.0);

    // 2. Secondary Tint (Mask 2)
    let _ = scratch_ctx.set_global_composite_operation("source-over");
    scratch_ctx.clear_rect(0.0, 0.0, size, size);
    scratch_ctx.set_fill_style(&JsValue::from_str(color2));
    scratch_ctx.fill_rect(0.0, 0.0, size, size);
    let _ = scratch_ctx.set_global_composite_operation("destination-in");
    let _ = scratch_ctx
        .draw_image_with_html_image_element_and_dw_and_dh(mask2_img, 0.0, 0.0, size, size);

    let _ = result_ctx.set_global_composite_operation("multiply");
    let _ = result_ctx.draw_image_with_html_canvas_element_and_dw_and_dh(
        &scratch_canvas,
        0.0,
        0.0,
        size,
        size,
    );
    let _ = result_ctx.set_global_composite_operation("source-over");
    result_ctx.set_global_alpha(0.35);
    let _ = result_ctx.draw_image_with_html_canvas_element_and_dw_and_dh(
        &scratch_canvas,
        0.0,
        0.0,
        size,
        size,
    );
    result_ctx.set_global_alpha(1.0);

    Some(result_canvas)
}

#[cfg(target_arch = "wasm32")]
fn get_tinted_asteroid_canvas(model: u8, resource_type: u8) -> Option<web_sys::HtmlCanvasElement> {
    let key = (model % 2, resource_type);
    ASTEROID_CANVAS_CACHE.with(|cache_cell| {
        let mut cache = cache_cell.borrow_mut();
        if let Some(canvas) = cache.get(&key) {
            return Some(canvas.clone());
        }

        let (base_img, mask_img) = get_asteroid_images(model as usize)?;
        if !is_image_ready(&base_img) || !is_image_ready(&mask_img) {
            return None;
        }

        let color = match resource_type {
            3 => "#7b1fa2", // Titanium (Purple/Gold)
            2 => "#00838f", // Ice (Cyan)
            1 => "#455a64", // Iron (Slate)
            _ => "#8d6e63", // Generic (Brownish Ore)
        };

        let canvas = render_tinted_sprite(&base_img, &mask_img, color)?;
        cache.insert(key, canvas.clone());
        Some(canvas)
    })
}

#[cfg(target_arch = "wasm32")]
fn get_tinted_player_station_canvas(hue1: u16, hue2: u16) -> Option<web_sys::HtmlCanvasElement> {
    let key = (hue1, hue2);
    PLAYER_STATION_CANVAS_CACHE.with(|cache_cell| {
        let mut cache = cache_cell.borrow_mut();
        if let Some(canvas) = cache.get(&key) {
            return Some(canvas.clone());
        }

        let (base_img, mask1_img, mask2_img) = get_player_station_images()?;
        if !is_image_ready(&base_img) || !is_image_ready(&mask1_img) || !is_image_ready(&mask2_img)
        {
            return None;
        }

        let color1 = format!("hsl({}, 90%, 55%)", hue1);
        let color2 = format!("hsl({}, 90%, 55%)", hue2);
        let canvas =
            render_dual_tinted_sprite(&base_img, &mask1_img, &mask2_img, &color1, &color2)?;
        cache.insert(key, canvas.clone());
        Some(canvas)
    })
}

#[cfg(target_arch = "wasm32")]
fn get_tinted_drone1_canvas(hue1: u16, hue2: u16) -> Option<web_sys::HtmlCanvasElement> {
    let key = (hue1, hue2);
    DRONE1_CANVAS_CACHE.with(|cache_cell| {
        let mut cache = cache_cell.borrow_mut();
        if let Some(canvas) = cache.get(&key) {
            return Some(canvas.clone());
        }

        let (base_img, mask1_img, mask2_img) = get_drone1_images()?;
        if !is_image_ready(&base_img) || !is_image_ready(&mask1_img) || !is_image_ready(&mask2_img)
        {
            return None;
        }

        let color1 = format!("hsl({}, 90%, 55%)", hue1);
        let color2 = format!("hsl({}, 90%, 55%)", hue2);
        let canvas =
            render_dual_tinted_sprite(&base_img, &mask1_img, &mask2_img, &color1, &color2)?;
        cache.insert(key, canvas.clone());
        Some(canvas)
    })
}

#[cfg(target_arch = "wasm32")]
fn get_tinted_drone_tier2_canvas(hue1: u16, hue2: u16) -> Option<web_sys::HtmlCanvasElement> {
    let key = (hue1, hue2);
    DRONE_TIER2_CANVAS_CACHE.with(|cache_cell| {
        let mut cache = cache_cell.borrow_mut();
        if let Some(canvas) = cache.get(&key) {
            return Some(canvas.clone());
        }

        let (base_img, mask1_img, mask2_img) = get_drone_tier2_images()?;
        if !is_image_ready(&base_img) || !is_image_ready(&mask1_img) || !is_image_ready(&mask2_img)
        {
            return None;
        }

        let color1 = format!("hsl({}, 90%, 55%)", hue1);
        let color2 = format!("hsl({}, 90%, 55%)", hue2);
        let canvas =
            render_dual_tinted_sprite(&base_img, &mask1_img, &mask2_img, &color1, &color2)?;
        cache.insert(key, canvas.clone());
        Some(canvas)
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
            let (hue1, hue2) = player_map
                .get(&base.owner)
                .map_or((200, (200 + 180) % 360), |p| {
                    (p.color_hue, p.secondary_color_hue)
                });
            draw_base_station(ctx, base, hue1, hue2, anim_time);
        }

        let margin = 100.0;
        let cull_min_x = min_x - margin;
        let cull_max_x = max_x + margin;
        let cull_min_y = min_y - margin;
        let cull_max_y = max_y + margin;

        let mut asteroid_pos_map = HashMap::new();

        let targeted_asteroids: std::collections::HashSet<u32> =
            ships.iter().filter_map(|s| s.target_asteroid_id).collect();

        // 5. Draw Asteroids (Culled to viewport)
        let mut sorted_asteroids: Vec<&Asteroid> = asteroids.iter().collect();
        sorted_asteroids.sort_by(|a, b| {
            b.radius
                .partial_cmp(&a.radius)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.asteroid_id.cmp(&b.asteroid_id))
        });

        for asteroid in sorted_asteroids {
            let ax = asteroid.position.x as f64;
            let ay = asteroid.position.y as f64;
            asteroid_pos_map.insert(asteroid.asteroid_id, (ax, ay));

            if ax >= cull_min_x && ax <= cull_max_x && ay >= cull_min_y && ay <= cull_max_y {
                let show_ore = targeted_asteroids.contains(&asteroid.asteroid_id);
                draw_asteroid(ctx, asteroid, (ax, ay), anim_time, show_ore);
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
        let mut sorted_ships: Vec<&Ship> = ships.iter().collect();
        sorted_ships.sort_by_key(|s| s.ship_id);
        for ship in sorted_ships {
            let (hue1, hue2) = player_map
                .get(&ship.owner)
                .map_or((180, (180 + 180) % 360), |p| {
                    (p.color_hue, p.secondary_color_hue)
                });
            draw_ship(ctx, ship, hue1, hue2, anim_time);
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
fn draw_base_station(
    ctx: &CanvasRenderingContext2d,
    base: &BaseStation,
    hue1: u16,
    hue2: u16,
    _time: f64,
) {
    let x = base.position.x as f64;
    let y = base.position.y as f64;
    let radius = base.signal_radius as f64;

    // Signal range circular boundary
    ctx.begin_path();
    let _ = ctx.arc(x, y, radius, 0.0, std::f64::consts::TAU);
    ctx.set_stroke_style(&JsValue::from_str(&format!(
        "hsla({}, 85%, 65%, 0.12)",
        hue1
    )));
    ctx.set_line_width(2.0);
    ctx.stroke();

    // Base Station Hub
    let _ = ctx.save();
    let _ = ctx.translate(x, y);

    if let Some(canvas) = get_tinted_player_station_canvas(hue1, hue2) {
        let station_size = 54.0;
        let _ = ctx.draw_image_with_html_canvas_element_and_dw_and_dh(
            &canvas,
            -station_size / 2.0,
            -station_size / 2.0,
            station_size,
            station_size,
        );
    } else {
        // Outer ring
        ctx.begin_path();
        let _ = ctx.arc(0.0, 0.0, 24.0, 0.0, std::f64::consts::TAU);
        ctx.set_fill_style(&JsValue::from_str("#0f1b29"));
        ctx.fill();
        ctx.set_stroke_style(&JsValue::from_str(&format!("hsl({}, 90%, 60%)", hue1)));
        ctx.set_line_width(3.0);
        ctx.stroke();
    }

    // Inner icon / ore count below hub
    ctx.set_font("bold 11px sans-serif");
    ctx.set_text_align("center");
    ctx.set_fill_style(&JsValue::from_str("#ffffff"));
    let _ = ctx.fill_text(&format!("{} Erz", base.stored_ore), 0.0, 36.0);

    let _ = ctx.restore();
}

#[cfg(target_arch = "wasm32")]
fn draw_asteroid(
    ctx: &CanvasRenderingContext2d,
    ast: &Asteroid,
    (x, y): (f64, f64),
    time: f64,
    show_ore: bool,
) {
    let r = ast.radius as f64;
    let model = (ast.asteroid_id % 2) as u8;

    if let Some(canvas) = get_tinted_asteroid_canvas(model, ast.resource_type) {
        let diameter = r * 2.0;
        let spin_speed = ((ast.asteroid_id % 7) as f64 - 3.0) * 0.04 + 0.08;
        let angle = ((ast.asteroid_id as f64 * 1.618) + time * spin_speed)
            .rem_euclid(std::f64::consts::TAU);

        let _ = ctx.save();
        let _ = ctx.translate(x, y);
        let _ = ctx.rotate(angle);
        let _ = ctx.draw_image_with_html_canvas_element_and_dw_and_dh(
            &canvas,
            -diameter / 2.0,
            -diameter / 2.0,
            diameter,
            diameter,
        );
        let _ = ctx.restore();
    } else {
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
    }

    if show_ore {
        ctx.set_font("bold 10px sans-serif");
        ctx.set_text_align("center");
        ctx.set_fill_style(&JsValue::from_str("#ffffff"));
        let _ = ctx.fill_text(&format!("{}", ast.remaining_ore), x, y + r + 12.0);
    }
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
fn draw_ship(ctx: &CanvasRenderingContext2d, ship: &Ship, hue1: u16, hue2: u16, time: f64) {
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
        ctx.set_fill_style(&JsValue::from_str(&format!("hsl({}, 100%, 70%)", hue1)));
        ctx.fill();
    }

    let drone_canvas = if ship.ship_tier >= 2 {
        get_tinted_drone_tier2_canvas(hue1, hue2)
    } else {
        get_tinted_drone1_canvas(hue1, hue2)
    };

    if let Some(canvas) = drone_canvas {
        let size = if ship.ship_tier >= 2 { 30.0 } else { 24.0 };
        let _ = ctx.draw_image_with_html_canvas_element_and_dw_and_dh(
            &canvas,
            -size / 2.0,
            -size / 2.0,
            size,
            size,
        );
    } else {
        // Fallback Ship Hull (triangle)
        ctx.begin_path();
        ctx.move_to(14.0, 0.0);
        ctx.line_to(-10.0, -9.0);
        ctx.line_to(-6.0, 0.0);
        ctx.line_to(-10.0, 9.0);
        ctx.close_path();

        ctx.set_fill_style(&JsValue::from_str(&format!("hsl({}, 75%, 45%)", hue1)));
        ctx.fill();
        ctx.set_stroke_style(&JsValue::from_str(&format!("hsl({}, 95%, 70%)", hue1)));
        ctx.set_line_width(1.5);
        ctx.stroke();
    }

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
