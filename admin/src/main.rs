pub mod audio;
pub mod canvas;
pub mod module_bindings;

use ::dioxus::prelude::*;
use audio::SoundEffects;
use canvas::render_space_scene;
use module_bindings::admin_set_phase_reducer::admin_set_phase;
use module_bindings::authenticate_admin_reducer::authenticate_admin;
use module_bindings::dioxus::{
    ConnectionState, use_spacetimedb_context, use_spacetimedb_context_provider, use_subscription,
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use web_sys::HtmlCanvasElement;

fn main() {
    ::dioxus::launch(App);
}

#[component]
fn App() -> Element {
    // 1. Initialize SpacetimeDB connection to local server module
    let _stdb_ctx =
        use_spacetimedb_context_provider("ws://127.0.0.1:3000", "server", None::<String>);
    use_subscription(&[
        "SELECT * FROM config",
        "SELECT * FROM player",
        "SELECT * FROM admin_all_bases",
        "SELECT * FROM admin_all_ships",
        "SELECT * FROM admin_all_asteroids",
        "SELECT * FROM game_event",
        "SELECT * FROM total_players",
    ]);

    let ctx = use_spacetimedb_context();

    // Auto-authenticate as admin upon connecting
    use_effect(move || {
        let state = (ctx.state)();
        if matches!(state, ConnectionState::Connected(_, _)) {
            if let Some(conn) = (ctx.connection)() {
                let _ = conn.reducers.authenticate_admin("admin".to_string());
            }
        }
    });
    use_effect(move || {
        let events = (ctx.tables.game_event)();
        if let Some(last) = events.last() {
            if last.event_type == "BASE_DEPLOYED" || last.event_type == "SHIP_BUILT" {
                SoundEffects::play_deposit_chime();
            } else if last.event_type == "SHIP_LOST" {
                SoundEffects::play_distress_alarm();
            }
        }
    });

    let camera_target = use_signal(|| None::<(f64, f64)>);

    rsx! {
        div {
            style: "width: 100vw; height: 100vh; margin: 0; padding: 0; background-color: #050814; color: #f0f4f8; font-family: system-ui, -apple-system, sans-serif; display: flex; flex-direction: column; overflow: hidden; user-select: none;",
            HeaderBar {}
            div {
                style: "flex: 1; display: flex; flex-direction: row; position: relative; overflow: hidden;",
                SpaceViewport { camera_target }
                SidebarOverlay { camera_target }
            }
        }
    }
}

#[component]
fn HeaderBar() -> Element {
    let ctx = use_spacetimedb_context();
    let configs = (ctx.tables.config)();
    let config = configs.first();

    let state = (ctx.state)();
    let is_connected = matches!(state, ConnectionState::Connected(_, _));

    let current_level = config.map_or(1, |c| c.level);
    let is_paused = config.map_or(false, |c| c.paused);

    let toggle_pause = move |_| {
        if let Some(conn) = (ctx.connection)() {
            let next_paused = !is_paused;
            let _ = conn.reducers.admin_set_phase(current_level, next_paused);
        }
    };

    let next_level = move |_| {
        if let Some(conn) = (ctx.connection)() {
            let _ = conn
                .reducers
                .admin_set_phase(current_level.saturating_add(1), is_paused);
        }
    };

    let prev_level = move |_| {
        if let Some(conn) = (ctx.connection)() {
            let _ = conn
                .reducers
                .admin_set_phase(current_level.saturating_sub(1).max(1), is_paused);
        }
    };

    rsx! {
        header {
            style: "height: 56px; background: rgba(10, 16, 32, 0.95); border-bottom: 1px solid rgba(80, 120, 200, 0.25); display: flex; align-items: center; justify-content: space-between; padding: 0 24px; z-index: 10;",
            div {
                style: "display: flex; align-items: center; gap: 16px;",
                h1 {
                    style: "margin: 0; font-size: 20px; font-weight: 700; letter-spacing: 1px; color: #00e5ff;",
                    "🚀 PROGRAMMIERSPIEL (RUST SPACE)"
                }
                span {
                    style: "padding: 4px 10px; border-radius: 12px; font-size: 12px; font-weight: 600; background: rgba(255, 255, 255, 0.08);",
                    if is_connected {
                        span { style: "color: #00e676;", "● SERVER VERBUNDEN" }
                    } else {
                        span { style: "color: #ff5252;", "○ VERBINDE..." }
                    }
                }
            }

            // Phase and Admin Controls
            div {
                style: "display: flex; align-items: center; gap: 20px;",
                div {
                    style: "font-size: 15px; font-weight: 600;",
                    span { style: "color: #8da2c0; margin-right: 8px;", "UNTERRICHTS-PHASE:" }
                    span { style: "color: #ffd700;", "Level {current_level}" }
                }

                // Control buttons
                div {
                    style: "display: flex; gap: 8px;",
                    button {
                        style: "background: #1e293b; color: #fff; border: 1px solid #334155; padding: 6px 14px; border-radius: 6px; cursor: pointer; font-weight: 600;",
                        onclick: prev_level,
                        "◀ Zurück"
                    }
                    button {
                        style: if is_paused {
                            "background: #15803d; color: #fff; border: none; padding: 6px 16px; border-radius: 6px; cursor: pointer; font-weight: 700;"
                        } else {
                            "background: #b91c1c; color: #fff; border: none; padding: 6px 16px; border-radius: 6px; cursor: pointer; font-weight: 700;"
                        },
                        onclick: toggle_pause,
                        if is_paused { "▶ Spiel Fortsetzen" } else { "⏸ Spiel Pausieren" }
                    }
                    button {
                        style: "background: #1e293b; color: #fff; border: 1px solid #334155; padding: 6px 14px; border-radius: 6px; cursor: pointer; font-weight: 600;",
                        onclick: next_level,
                        "Vor ▶"
                    }
                }
            }
        }
    }
}

#[component]
fn SpaceViewport(camera_target: Signal<Option<(f64, f64)>>) -> Element {
    let ctx = use_spacetimedb_context();

    let mut cam_x = use_signal(|| 0.0f64);
    let mut cam_y = use_signal(|| 0.0f64);
    let mut zoom = use_signal(|| 0.45f64);
    let mut is_dragging = use_signal(|| false);
    let mut drag_start = use_signal(|| (0.0f64, 0.0f64));

    // Handle external jump target (e.g. from clicking player in scoreboard)
    use_effect(move || {
        if let Some((tx, ty)) = *camera_target.read() {
            cam_x.set(tx);
            cam_y.set(ty);
        }
    });

    // Continuously render scene via request_animation_frame with interpolation
    use_effect(move || {
        #[cfg(target_arch = "wasm32")]
        {
            use std::cell::RefCell;
            use std::rc::Rc;
            use wasm_bindgen::JsCast;
            use wasm_bindgen::closure::Closure;

            let anim_handle: Rc<RefCell<Option<Closure<dyn FnMut()>>>> =
                Rc::new(RefCell::new(None));
            let anim_handle_clone = anim_handle.clone();

            let render_step = Rc::new(move || -> bool {
                if let Some(window) = web_sys::window() {
                    if let Some(document) = window.document() {
                        if let Some(canvas_el) = document.get_element_by_id("space-canvas") {
                            if let Ok(canvas) = canvas_el.dyn_into::<HtmlCanvasElement>() {
                                let rect = canvas.get_bounding_client_rect();
                                let css_w = rect.width();
                                let css_h = rect.height();
                                if css_w > 0.0 && css_h > 0.0 {
                                    let dpr = window.device_pixel_ratio().max(1.0);
                                    let target_w = (css_w * dpr).round() as u32;
                                    let target_h = (css_h * dpr).round() as u32;

                                    if canvas.width() != target_w || canvas.height() != target_h {
                                        canvas.set_width(target_w);
                                        canvas.set_height(target_h);
                                    }

                                    if let Ok(Some(ctx_2d)) = canvas.get_context("2d") {
                                        if let Ok(ctx_2d) =
                                            ctx_2d.dyn_into::<web_sys::CanvasRenderingContext2d>()
                                        {
                                            let configs = (ctx.tables.config)();
                                            let players = (ctx.tables.player)();
                                            let base_stations = (ctx.tables.admin_all_bases)();
                                            let asteroids = (ctx.tables.admin_all_asteroids)();
                                            let ships = (ctx.tables.admin_all_ships)();

                                            let time = js_sys::Date::now() / 1000.0;

                                            render_space_scene(
                                                &ctx_2d,
                                                canvas.width() as f64,
                                                canvas.height() as f64,
                                                *cam_x.read(),
                                                *cam_y.read(),
                                                *zoom.read(),
                                                configs.first(),
                                                &players,
                                                &base_stations,
                                                &asteroids,
                                                &ships,
                                                time,
                                            );
                                        }
                                    }
                                }
                                return true;
                            }
                        }
                    }
                }
                false
            });

            *anim_handle_clone.borrow_mut() = Some(Closure::new({
                let anim_handle = anim_handle.clone();
                let render_step = render_step.clone();
                move || {
                    if render_step() {
                        if let Some(w) = web_sys::window() {
                            if let Some(ref closure) = *anim_handle.borrow() {
                                let _ = w.request_animation_frame(closure.as_ref().unchecked_ref());
                            }
                        }
                    }
                }
            }));

            if let Some(w) = web_sys::window() {
                if let Some(ref closure) = *anim_handle_clone.borrow() {
                    let _ = w.request_animation_frame(closure.as_ref().unchecked_ref());
                }
            }
        }
    });

    let current_x = *cam_x.read();
    let current_y = *cam_y.read();
    let current_zoom = *zoom.read();

    rsx! {
        div {
            style: if *is_dragging.read() {
                "flex: 1; height: 100%; position: relative; background: #070b19; display: flex; align-items: center; justify-content: center; overflow: hidden; cursor: grabbing;"
            } else {
                "flex: 1; height: 100%; position: relative; background: #070b19; display: flex; align-items: center; justify-content: center; overflow: hidden; cursor: grab;"
            },
            onmousedown: move |evt| {
                is_dragging.set(true);
                drag_start.set((evt.client_coordinates().x, evt.client_coordinates().y));
            },
            onmousemove: move |evt| {
                if *is_dragging.read() {
                    let (sx, sy) = *drag_start.read();
                    let cx = evt.client_coordinates().x;
                    let cy = evt.client_coordinates().y;
                    let dx = cx - sx;
                    let dy = cy - sy;
                    drag_start.set((cx, cy));
                    let z = *zoom.read();
                    let cur_x = *cam_x.read();
                    let cur_y = *cam_y.read();
                    cam_x.set(cur_x - dx / z);
                    cam_y.set(cur_y - dy / z);
                }
            },
            onmouseup: move |_| {
                is_dragging.set(false);
            },
            onmouseleave: move |_| {
                is_dragging.set(false);
            },
            onwheel: move |evt| {
                let delta_y = evt.delta().strip_units().y;
                let factor = if delta_y > 0.0 { 0.85 } else { 1.15 };
                let cur_z = *zoom.read();
                let new_z = (cur_z * factor).clamp(0.04, 3.5);
                zoom.set(new_z);
            },
            canvas {
                id: "space-canvas",
                style: "width: 100%; height: 100%; display: block;",
            }

            // HUD Overlay: Coordinates, Zoom %, and Re-center controls
            div {
                style: "position: absolute; bottom: 20px; left: 20px; display: flex; gap: 10px; align-items: center; background: rgba(10, 16, 32, 0.88); backdrop-filter: blur(8px); padding: 8px 14px; border-radius: 8px; border: 1px solid rgba(80, 120, 200, 0.3); font-size: 13px; font-family: monospace; z-index: 5; pointer-events: auto;",
                span {
                    style: "color: #8da2c0;",
                    "POS: "
                }
                span {
                    style: "color: #00e5ff; font-weight: 700; width: 140px;",
                    "[{current_x.round() as i32}, {current_y.round() as i32}]"
                }
                span {
                    style: "color: #8da2c0; margin-left: 8px;",
                    "ZOOM: "
                }
                span {
                    style: "color: #ffd700; font-weight: 700; width: 50px;",
                    "{(current_zoom * 100.0).round() as i32}%"
                }
                button {
                    style: "background: #1e293b; color: #fff; border: 1px solid #334155; padding: 4px 8px; border-radius: 4px; cursor: pointer; font-size: 12px; margin-left: 6px;",
                    onclick: move |_| {
                        let cur_z = *zoom.read();
                        let new_z = (cur_z * 1.25).clamp(0.04, 3.5);
                        zoom.set(new_z);
                    },
                    "+"
                }
                button {
                    style: "background: #1e293b; color: #fff; border: 1px solid #334155; padding: 4px 8px; border-radius: 4px; cursor: pointer; font-size: 12px;",
                    onclick: move |_| {
                        let cur_z = *zoom.read();
                        let new_z = (cur_z * 0.8).clamp(0.04, 3.5);
                        zoom.set(new_z);
                    },
                    "-"
                }
                button {
                    style: "background: #0284c7; color: #fff; border: none; padding: 4px 10px; border-radius: 4px; cursor: pointer; font-weight: 600; font-size: 12px; margin-left: 6px;",
                    onclick: move |_| {
                        cam_x.set(0.0);
                        cam_y.set(0.0);
                        zoom.set(0.45);
                    },
                    "⌂ Mega-Station"
                }
            }
        }
    }
}

#[component]
fn SidebarOverlay(mut camera_target: Signal<Option<(f64, f64)>>) -> Element {
    let ctx = use_spacetimedb_context();
    let mut players = (ctx.tables.player)();
    let ships = (ctx.tables.admin_all_ships)();
    let bases = (ctx.tables.admin_all_bases)();
    let mut events = (ctx.tables.game_event)();

    // Sort players by score descending
    players.sort_by(|a, b| b.score.cmp(&a.score));

    // Sort events by event_id descending (latest on top)
    events.sort_by(|a, b| b.event_id.cmp(&a.event_id));
    let recent_events: Vec<_> = events.into_iter().take(12).collect();

    // Precompute player item positions before rsx!
    let player_items: Vec<_> = players
        .into_iter()
        .enumerate()
        .map(|(rank, player)| {
            let p_identity = player.identity;
            let p_base = bases.iter().find(|b| b.owner == p_identity);
            let p_pos = p_base
                .map(|b| (b.position.x as f64, b.position.y as f64))
                .or_else(|| {
                    ships
                        .iter()
                        .find(|s| s.owner == p_identity)
                        .map(|s| (s.position.x as f64, s.position.y as f64))
                });
            (rank, player, p_pos)
        })
        .collect();

    rsx! {
        aside {
            style: "width: 380px; height: 100%; background: rgba(10, 16, 32, 0.88); border-left: 1px solid rgba(80, 120, 200, 0.2); backdrop-filter: blur(8px); display: flex; flex-direction: column; overflow: hidden;",

            // Scoreboard Section
            div {
                style: "padding: 16px 20px; border-bottom: 1px solid rgba(80, 120, 200, 0.15);",
                h2 {
                    style: "margin: 0 0 12px 0; font-size: 16px; font-weight: 700; color: #ffd700; letter-spacing: 0.5px;",
                    "🏆 RANGLISTE (ERZ-PUNKTE)"
                }
                div {
                    style: "display: flex; flex-direction: column; gap: 8px; max-height: 280px; overflow-y: auto;",
                    if player_items.is_empty() {
                        div { style: "color: #64748b; font-size: 13px;", "Noch keine Schüler beigetreten..." }
                    }
                    for (rank, player, p_pos) in player_items {
                        div {
                            key: "{player.identity}",
                            style: "display: flex; align-items: center; justify-content: space-between; padding: 8px 12px; background: rgba(255, 255, 255, 0.04); border-radius: 6px; border-left: 4px solid hsl({player.color_hue}, 90%, 55%); border-image: linear-gradient(to bottom, hsl({player.color_hue}, 90%, 55%), hsl({player.secondary_color_hue}, 90%, 55%)) 1 100%; cursor: pointer; transition: background 0.15s ease;",
                            title: "Klicken, um Kamera auf Basis/Schiff zu zentrieren",
                            onclick: move |_| {
                                if let Some((px, py)) = p_pos {
                                    camera_target.set(Some((px, py)));
                                }
                            },
                            div {
                                style: "display: flex; align-items: center; gap: 10px;",
                                span { style: "font-weight: 700; color: #8da2c0; font-size: 13px; width: 20px;", "#{rank + 1}" }
                                span { style: "font-weight: 600; font-size: 14px;", "{player.name}" }
                            }
                            div {
                                style: "display: flex; align-items: center; gap: 8px;",
                                span { style: "font-weight: 700; color: #ffd700; font-size: 14px;", "{player.score} Pkt" }
                            }
                        }
                    }
                }
            }

            // Live Events Feed Section
            div {
                style: "flex: 1; padding: 16px 20px; display: flex; flex-direction: column; overflow: hidden;",
                h2 {
                    style: "margin: 0 0 12px 0; font-size: 16px; font-weight: 700; color: #00e5ff; letter-spacing: 0.5px;",
                    "📡 LIVE-SEKTOR-LOG"
                }
                div {
                    style: "flex: 1; display: flex; flex-direction: column; gap: 8px; overflow-y: auto; padding-right: 4px;",
                    if recent_events.is_empty() {
                        div { style: "color: #64748b; font-size: 13px;", "Warte auf Sektorereignisse..." }
                    }
                    for ev in recent_events {
                        div {
                            key: "{ev.event_id}",
                            style: "padding: 8px 10px; background: rgba(0, 0, 0, 0.35); border-radius: 6px; font-size: 12px; border-left: 3px solid #00e5ff;",
                            div {
                                style: "font-weight: 700; color: #8da2c0; font-size: 10px; margin-bottom: 2px;",
                                "{ev.event_type}"
                            }
                            div {
                                style: "color: #e2e8f0; line-height: 1.3;",
                                "{ev.message}"
                            }
                        }
                    }
                }
            }
        }
    }
}
