use crate::module_bindings::{Asteroid, BaseStation, Config, Ship, Vec2};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;
#[cfg(target_arch = "wasm32")]
use web_sys::CanvasRenderingContext2d;

pub fn render_player_radar(
    #[cfg(target_arch = "wasm32")] ctx: &CanvasRenderingContext2d,
    canvas_w: f64,
    canvas_h: f64,
    center_pos: Vec2,
    color_hue: u16,
    config: Option<&Config>,
    my_ship: Option<&Ship>,
    my_base: Option<&BaseStation>,
    asteroids: &[Asteroid],
    target_pos: Option<Vec2>,
    time: f64,
) {
    #[cfg(target_arch = "wasm32")]
    {
        // 1. Dark radar background
        ctx.set_fill_style(&JsValue::from_str("#070d1d"));
        ctx.fill_rect(0.0, 0.0, canvas_w, canvas_h);

        let cx = canvas_w / 2.0;
        let cy = canvas_h / 2.0;

        let _ = ctx.save();
        let _ = ctx.translate(cx, cy);

        // Radar range rings (relative to center)
        ctx.set_stroke_style(&JsValue::from_str("rgba(0, 229, 255, 0.1)"));
        ctx.set_line_width(1.0);
        for r in [100.0, 200.0, 300.0, 400.0, 500.0] {
            ctx.begin_path();
            let _ = ctx.arc(0.0, 0.0, r, 0.0, std::f64::consts::TAU);
            ctx.stroke();
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

        // Draw Mega-Station
        let mega_x = config.map_or(1600.0, |c| c.megastation_pos.x as f64) - center_x;
        let mega_y = config.map_or(1000.0, |c| c.megastation_pos.y as f64) - center_y;

        // Safe zone boundary
        ctx.begin_path();
        let _ = ctx.arc(mega_x, mega_y, 250.0, 0.0, std::f64::consts::TAU);
        ctx.set_stroke_style(&JsValue::from_str("rgba(80, 200, 255, 0.25)"));
        ctx.set_line_width(2.0);
        ctx.stroke();

        // Mega-Station icon
        ctx.begin_path();
        let _ = ctx.arc(mega_x, mega_y, 28.0, 0.0, std::f64::consts::TAU);
        ctx.set_fill_style(&JsValue::from_str("#1c3b57"));
        ctx.fill();
        ctx.set_stroke_style(&JsValue::from_str("#5ce1e6"));
        ctx.set_line_width(2.0);
        ctx.stroke();

        ctx.set_font("12px sans-serif");
        ctx.set_text_align("center");
        ctx.set_fill_style(&JsValue::from_str("#e0f7fa"));
        let _ = ctx.fill_text("MEGA-STATION", mega_x, mega_y + 45.0);

        // Draw Asteroids relative to center
        for ast in asteroids {
            let ax = ast.position.x as f64 - center_x;
            let ay = ast.position.y as f64 - center_y;

            // Only draw if within radar viewport
            if ax.abs() < cx + 50.0 && ay.abs() < cy + 50.0 {
                let color = match ast.resource_type {
                    3 => "#7b1fa2",
                    2 => "#00838f",
                    1 => "#455a64",
                    _ => "#5d4037",
                };
                ctx.begin_path();
                let _ = ctx.arc(
                    ax,
                    ay,
                    (ast.radius as f64) * 0.8,
                    0.0,
                    std::f64::consts::TAU,
                );
                ctx.set_fill_style(&JsValue::from_str(color));
                ctx.fill();
            }
        }

        // Draw Target Marker if flying
        if let Some(target) = target_pos {
            let tx = target.x as f64 - center_x;
            let ty = target.y as f64 - center_y;

            let pulse = (time * 8.0).sin() * 3.0 + 12.0;
            ctx.begin_path();
            let _ = ctx.arc(tx, ty, pulse, 0.0, std::f64::consts::TAU);
            ctx.set_stroke_style(&JsValue::from_str("#00e676"));
            ctx.set_line_width(2.0);
            ctx.stroke();

            // Flight path line
            ctx.begin_path();
            ctx.move_to(0.0, 0.0);
            ctx.line_to(tx, ty);
            ctx.set_stroke_style(&JsValue::from_str("rgba(0, 230, 118, 0.4)"));
            ctx.set_line_width(1.5);
            ctx.stroke();
        }

        // Draw Base Station if exists
        if let Some(base) = my_base {
            let bx = base.position.x as f64 - center_x;
            let by = base.position.y as f64 - center_y;

            ctx.begin_path();
            let _ = ctx.arc(
                bx,
                by,
                base.signal_radius as f64,
                0.0,
                std::f64::consts::TAU,
            );
            ctx.set_stroke_style(&JsValue::from_str(&format!(
                "hsla({}, 85%, 65%, 0.2)",
                color_hue
            )));
            ctx.set_line_width(1.5);
            ctx.stroke();

            ctx.begin_path();
            let _ = ctx.arc(bx, by, 20.0, 0.0, std::f64::consts::TAU);
            ctx.set_fill_style(&JsValue::from_str(&format!("hsl({}, 90%, 55%)", color_hue)));
            ctx.fill();
        }

        // Draw Player Ship at center (0, 0)
        let heading = my_ship.map_or(0.0, |s| s.heading_rad as f64);
        let _ = ctx.save();
        let _ = ctx.rotate(heading);

        ctx.begin_path();
        ctx.move_to(16.0, 0.0);
        ctx.line_to(-12.0, -10.0);
        ctx.line_to(-8.0, 0.0);
        ctx.line_to(-12.0, 10.0);
        ctx.close_path();

        ctx.set_fill_style(&JsValue::from_str(&format!("hsl({}, 85%, 50%)", color_hue)));
        ctx.fill();
        ctx.set_stroke_style(&JsValue::from_str("#ffffff"));
        ctx.set_line_width(2.0);
        ctx.stroke();

        let _ = ctx.restore();
        let _ = ctx.restore();
    }
}
