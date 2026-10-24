pub mod audio;
pub mod canvas;
pub mod module_bindings;

use ::dioxus::prelude::*;
use audio::SoundEffects;
use canvas::render_space_scene;
use module_bindings::admin_set_phase_reducer::admin_set_phase;
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
        "SELECT * FROM base_station",
        "SELECT * FROM ship",
        "SELECT * FROM asteroid",
        "SELECT * FROM game_event",
        "SELECT * FROM total_players",
    ]);

    // Play sound effects when new game events occur
    let ctx = use_spacetimedb_context();
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

    rsx! {
        div {
            style: "width: 100vw; height: 100vh; margin: 0; padding: 0; background-color: #050814; color: #f0f4f8; font-family: system-ui, -apple-system, sans-serif; display: flex; flex-direction: column; overflow: hidden; user-select: none;",
            HeaderBar {}
            div {
                style: "flex: 1; display: flex; flex-direction: row; position: relative; overflow: hidden;",
                SpaceViewport {}
                SidebarOverlay {}
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
fn SpaceViewport() -> Element {
    let ctx = use_spacetimedb_context();
    let mut resize_trigger = use_signal(|| 0u32);

    use_effect(move || {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(w) = web_sys::window() {
                let closure = wasm_bindgen::closure::Closure::<dyn FnMut()>::new(move || {
                    resize_trigger += 1;
                });
                let _ =
                    w.add_event_listener_with_callback("resize", closure.as_ref().unchecked_ref());
                closure.forget();
            }
        }
    });

    // Trigger canvas re-render on periodic timer, resize, or data updates
    use_effect(move || {
        let _ = resize_trigger();
        #[cfg(target_arch = "wasm32")]
        {
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
                                        let base_stations = (ctx.tables.base_station)();
                                        let asteroids = (ctx.tables.asteroid)();
                                        let ships = (ctx.tables.ship)();

                                        let time = js_sys::Date::now() / 1000.0;

                                        render_space_scene(
                                            &ctx_2d,
                                            canvas.width() as f64,
                                            canvas.height() as f64,
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
                        }
                    }
                }
            }
        }
    });

    rsx! {
        div {
            style: "flex: 1; height: 100%; position: relative; background: #070b19; display: flex; align-items: center; justify-content: center; overflow: hidden;",
            canvas {
                id: "space-canvas",
                style: "width: 100%; height: 100%; display: block;",
            }
        }
    }
}

#[component]
fn SidebarOverlay() -> Element {
    let ctx = use_spacetimedb_context();
    let mut players = (ctx.tables.player)();
    let _ships = (ctx.tables.ship)();
    let mut events = (ctx.tables.game_event)();

    // Sort players by score descending
    players.sort_by(|a, b| b.score.cmp(&a.score));

    // Sort events by event_id descending (latest on top)
    events.sort_by(|a, b| b.event_id.cmp(&a.event_id));
    let recent_events: Vec<_> = events.into_iter().take(12).collect();

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
                    if players.is_empty() {
                        div { style: "color: #64748b; font-size: 13px;", "Noch keine Schüler beigetreten..." }
                    }
                    for (rank, player) in players.iter().enumerate() {
                        div {
                            key: "{player.identity}",
                            style: "display: flex; align-items: center; justify-content: space-between; padding: 8px 12px; background: rgba(255, 255, 255, 0.04); border-radius: 6px; border-left: 4px solid hsl({player.color_hue}, 90%, 55%);",
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
