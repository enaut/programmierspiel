use crate::module_bindings::{
    BaseStation, Config, PerceivedAsteroid, PerceivedBase, PerceivedDrone, Player, Ship,
    UnidentifiedContact, Vec2,
};
use dioxus::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;

impl Vec2 {
    pub const ZERO: Vec2 = Vec2 { x: 0.0, y: 0.0 };
    pub fn distance(&self, other: &Vec2) -> f32 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}

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
            3 => "#a855f7", // Titanium (purple)
            2 => "#06b6d4", // Ice (cyan)
            1 => "#64748b", // Iron (slate)
            _ => "#8d6e63", // Generic (brown)
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
pub(crate) fn get_tinted_drone1_canvas(hue1: u16, hue2: u16) -> Option<web_sys::HtmlCanvasElement> {
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

pub fn render_drone1_preview(
    #[cfg(target_arch = "wasm32")] ctx: &web_sys::CanvasRenderingContext2d,
    css_w: f64,
    css_h: f64,
    dpr: f64,
    hue1: u16,
    hue2: u16,
) {
    #[cfg(target_arch = "wasm32")]
    {
        ctx.clear_rect(0.0, 0.0, css_w * dpr, css_h * dpr);
        let _ = ctx.save();
        let _ = ctx.scale(dpr, dpr);
        let cx = css_w / 2.0;
        let cy = css_h / 2.0;
        let _ = ctx.translate(cx, cy);

        if let Some(canvas) = get_tinted_drone1_canvas(hue1, hue2) {
            let size = css_h.min(css_w) * 0.85;
            let _ = ctx.draw_image_with_html_canvas_element_and_dw_and_dh(
                &canvas,
                -size / 2.0,
                -size / 2.0,
                size,
                size,
            );
        }
        let _ = ctx.restore();
    }
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
    secondary_color_hue: u16,
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
        let has_builder = my_ships.iter().any(|s| s.ship_tier == 0);
        let stroke_color = if has_builder {
            let pulse = (time * 3.0).sin() * 0.5 + 0.5;
            let is_outside = my_ships.iter().any(|s| {
                s.ship_tier == 0
                    && s.position
                        .distance(&config.map_or(Vec2::ZERO, |c| c.megastation_pos.clone()))
                        >= 250.0
            });
            if is_outside {
                format!("rgba(0, 230, 118, {:.2})", 0.5 + 0.5 * pulse)
            } else {
                format!("rgba(255, 60, 60, {:.2})", 0.5 + 0.5 * pulse)
            }
        } else {
            "rgba(80, 200, 255, 0.22)".to_string()
        };
        ctx.set_stroke_style(&JsValue::from_str(&stroke_color));
        ctx.set_line_width(if has_builder { 3.0 / zoom } else { 2.0 / zoom });
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

        // Spotlight highlight for tutorial step 5 (target closest asteroid with ore)
        if let Some(first_drone) = my_ships.first() {
            if let Some(player_me) = players.iter().find(|p| p.identity == first_drone.owner) {
                if player_me.tutorial_step == 5 {
                    if let Some(target_ast) = visible_asteroids
                        .iter()
                        .filter(|a| a.remaining_ore > 0)
                        .min_by(|a, b| {
                            a.position
                                .distance(&first_drone.position)
                                .total_cmp(&b.position.distance(&first_drone.position))
                        })
                    {
                        let pulse = (time * 4.0).sin() * 0.5 + 0.5;
                        let target_radius = (target_ast.radius as f64) + 14.0 + (pulse * 8.0);
                        ctx.begin_path();
                        let _ = ctx.arc(
                            target_ast.position.x as f64,
                            target_ast.position.y as f64,
                            target_radius,
                            0.0,
                            std::f64::consts::TAU,
                        );
                        ctx.set_stroke_style(&JsValue::from_str(&format!(
                            "rgba(0, 229, 255, {:.2})",
                            0.6 + 0.4 * pulse
                        )));
                        ctx.set_line_width(2.5 / zoom);
                        ctx.stroke();
                    }
                }
            }
        }

        let mut sorted_asteroids: Vec<&PerceivedAsteroid> = visible_asteroids.iter().collect();
        sorted_asteroids.sort_by(|a, b| {
            b.radius
                .partial_cmp(&a.radius)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.asteroid_id.cmp(&b.asteroid_id))
        });

        for ast in sorted_asteroids {
            let ax = ast.position.x as f64;
            let ay = ast.position.y as f64;
            asteroid_pos_map.insert(ast.asteroid_id, (ax, ay));

            let model = (ast.asteroid_id % 2) as u8;
            let diameter = (ast.radius as f64) * 2.0;
            let is_selected = selected_entity == Some(SelectedEntity::Asteroid(ast.asteroid_id));

            if let Some(canvas) = get_tinted_asteroid_canvas(model, ast.resource_type) {
                let spin_speed = ((ast.asteroid_id % 7) as f64 - 3.0) * 0.04 + 0.08;
                let angle = ((ast.asteroid_id as f64 * 1.618) + time * spin_speed)
                    .rem_euclid(std::f64::consts::TAU);

                let _ = ctx.save();
                let _ = ctx.translate(ax, ay);
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
            }

            if is_selected {
                draw_target_reticle(ctx, ax, ay, (ast.radius as f64) * 0.85, time, "#00e5ff");
                ctx.set_font("bold 11px sans-serif");
                ctx.set_text_align("center");
                ctx.set_fill_style(&JsValue::from_str("#ffffff"));
                let _ = ctx.fill_text(
                    &format!("{} Erz", ast.remaining_ore),
                    ax,
                    ay + (ast.radius as f64) + 14.0,
                );
            }
        }

        // 5. Draw Bases
        // 5A. Foreign identified bases
        for base in visible_bases {
            let is_my_base = my_base.map_or(false, |b| b.base_id == base.base_id);
            if !is_my_base {
                let bx = base.position.x as f64;
                let by = base.position.y as f64;
                let (b_hue1, b_hue2) = player_map
                    .get(&base.owner)
                    .map_or((200, (200 + 180) % 360), |p| {
                        (p.color_hue, p.secondary_color_hue)
                    });
                let b_name = player_map
                    .get(&base.owner)
                    .map_or("Basis".to_string(), |p| format!("Basis ({})", p.name));

                if let Some(canvas) = get_tinted_player_station_canvas(b_hue1, b_hue2) {
                    let station_size = 54.0;
                    let _ = ctx.draw_image_with_html_canvas_element_and_dw_and_dh(
                        &canvas,
                        bx - station_size / 2.0,
                        by - station_size / 2.0,
                        station_size,
                        station_size,
                    );
                } else {
                    ctx.begin_path();
                    let _ = ctx.arc(bx, by, 18.0, 0.0, std::f64::consts::TAU);
                    ctx.set_fill_style(&JsValue::from_str("#0f172a"));
                    ctx.fill();
                    ctx.set_stroke_style(&JsValue::from_str(&format!("hsl({}, 90%, 55%)", b_hue1)));
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
                }

                ctx.set_font("bold 11px sans-serif");
                ctx.set_text_align("center");
                ctx.set_fill_style(&JsValue::from_str("#e2e8f0"));
                let _ = ctx.fill_text(&b_name, bx, by + 36.0);

                if selected_entity == Some(SelectedEntity::ForeignBase(base.base_id)) {
                    draw_target_reticle(ctx, bx, by, 28.0, time, "#f59e0b");
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

            if let Some(canvas) = get_tinted_player_station_canvas(color_hue, secondary_color_hue) {
                let station_size = 54.0;
                let _ = ctx.draw_image_with_html_canvas_element_and_dw_and_dh(
                    &canvas,
                    bx - station_size / 2.0,
                    by - station_size / 2.0,
                    station_size,
                    station_size,
                );
            } else {
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
            }

            ctx.set_font("bold 12px sans-serif");
            ctx.set_text_align("center");
            ctx.set_fill_style(&JsValue::from_str("#ffffff"));
            let _ = ctx.fill_text("HEIMATBASIS", bx, by + 46.0);

            if selected_entity == Some(SelectedEntity::Heimatbasis) {
                draw_target_reticle(ctx, bx, by, 28.0, time, "#00e5ff");
            }
        }

        // 6. Draw Foreign Drones
        let mut sorted_foreign_drones: Vec<&PerceivedDrone> = visible_drones.iter().collect();
        sorted_foreign_drones.sort_by_key(|d| d.ship_id);
        for d in sorted_foreign_drones {
            let is_my_drone = my_ships.iter().any(|s| s.ship_id == d.ship_id);
            if !is_my_drone {
                let dx = d.position.x as f64;
                let dy = d.position.y as f64;
                let (d_hue1, d_hue2) = player_map
                    .get(&d.owner)
                    .map_or((180, (180 + 180) % 360), |p| {
                        (p.color_hue, p.secondary_color_hue)
                    });
                let p_name = player_map
                    .get(&d.owner)
                    .map_or("".to_string(), |p| p.name.clone());

                let _ = ctx.save();
                let _ = ctx.translate(dx, dy);
                let _ = ctx.rotate(d.heading_rad as f64);

                let drone_canvas = if d.ship_tier >= 2 {
                    get_tinted_drone_tier2_canvas(d_hue1, d_hue2)
                } else {
                    get_tinted_drone1_canvas(d_hue1, d_hue2)
                };

                if let Some(canvas) = drone_canvas {
                    let size = if d.ship_tier >= 2 { 30.0 } else { 24.0 };
                    let _ = ctx.draw_image_with_html_canvas_element_and_dw_and_dh(
                        &canvas,
                        -size / 2.0,
                        -size / 2.0,
                        size,
                        size,
                    );
                } else {
                    ctx.begin_path();
                    ctx.move_to(16.0, 0.0);
                    ctx.line_to(-12.0, -10.0);
                    ctx.line_to(-6.0, 0.0);
                    ctx.line_to(-12.0, 10.0);
                    ctx.close_path();

                    ctx.set_fill_style(&JsValue::from_str(&format!("hsl({}, 85%, 50%)", d_hue1)));
                    ctx.fill();
                    ctx.set_stroke_style(&JsValue::from_str("#ffffff"));
                    ctx.set_line_width(1.5 / zoom);
                    ctx.stroke();
                }

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
        let mut sorted_my_ships: Vec<&Ship> = my_ships.iter().collect();
        sorted_my_ships.sort_by_key(|s| s.ship_id);
        for s in sorted_my_ships {
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

            let drone_canvas = if s.ship_tier >= 2 {
                get_tinted_drone_tier2_canvas(color_hue, secondary_color_hue)
            } else {
                get_tinted_drone1_canvas(color_hue, secondary_color_hue)
            };

            if let Some(canvas) = drone_canvas {
                let size = if s.ship_tier >= 2 { 30.0 } else { 24.0 };
                let _ = ctx.draw_image_with_html_canvas_element_and_dw_and_dh(
                    &canvas,
                    -size / 2.0,
                    -size / 2.0,
                    size,
                    size,
                );
            } else {
                // Fallback vector drawing
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
