pub mod module_bindings;
pub mod radar;

use ::dioxus::prelude::*;
use ::dioxus::signals::SyncSignal;
use module_bindings::command_ship_reducer::command_ship;
use module_bindings::deploy_base_station_reducer::deploy_base_station;
use module_bindings::dioxus::{
    ConnectionState, use_spacetimedb_context, use_spacetimedb_context_provider, use_subscription,
};
use module_bindings::register_player_reducer::register_player;
use module_bindings::{BaseStation, Player, Ship, ShipCommand, Vec2};
use radar::render_player_radar;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use web_sys::HtmlCanvasElement;

fn distance(a: &Vec2, b: &Vec2) -> f32 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    (dx * dx + dy * dy).sqrt()
}

fn main() {
    ::dioxus::launch(App);
}

#[component]
fn App() -> Element {
    // 1. Read token from localStorage if available
    let saved_token = {
        #[cfg(target_arch = "wasm32")]
        {
            web_sys::window()
                .and_then(|w| w.local_storage().ok().flatten())
                .and_then(|s| s.get_item("spacetimedb_token").ok().flatten())
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            None::<String>
        }
    };

    // 2. Resolve WebSocket endpoint: prefer 127.0.0.1 over localhost to prevent IPv6 [::1] connection issues
    let ws_url = {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(w) = web_sys::window() {
                let hostname = w.location().hostname().unwrap_or_default();
                if hostname.is_empty() || hostname == "localhost" {
                    "ws://127.0.0.1:3000".to_string()
                } else {
                    format!("ws://{}:3000", hostname)
                }
            } else {
                "ws://127.0.0.1:3000".to_string()
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            "ws://127.0.0.1:3000".to_string()
        }
    };

    let _stdb_ctx = use_spacetimedb_context_provider(&ws_url, "server", saved_token);

    // 3. Subscribe to table queries so client receives live rows & updates
    use_subscription(&[
        "SELECT * FROM config",
        "SELECT * FROM player",
        "SELECT * FROM base_station",
        "SELECT * FROM ship",
        "SELECT * FROM asteroid",
        "SELECT * FROM game_event",
    ]);

    let ctx = use_spacetimedb_context();
    let state = (ctx.state)();
    let conn_error = (ctx.error)();

    // Persist token upon connection
    use_effect(move || {
        #[cfg(target_arch = "wasm32")]
        {
            let current_state = (ctx.state)();
            if let ConnectionState::Connected(_, ref token) = current_state {
                if let Some(w) = web_sys::window() {
                    if let Ok(Some(storage)) = w.local_storage() {
                        let _ = storage.set_item("spacetimedb_token", token);
                    }
                }
            }
        }
    });

    // Auto-clear invalid token on fatal error so user doesn't get stuck in Error state
    use_effect(move || {
        let current_state = (ctx.state)();
        if current_state == ConnectionState::Error {
            #[cfg(target_arch = "wasm32")]
            {
                if let Some(w) = web_sys::window() {
                    if let Ok(Some(storage)) = w.local_storage() {
                        if storage
                            .get_item("spacetimedb_token")
                            .ok()
                            .flatten()
                            .is_some()
                        {
                            let _ = storage.remove_item("spacetimedb_token");
                            let _ = w.location().reload();
                        }
                    }
                }
            }
        }
    });

    let my_identity = match state {
        ConnectionState::Connected(id, _) => Some(id),
        _ => None,
    };

    let players = (ctx.tables.player)();
    let bases = (ctx.tables.base_station)();
    let ships = (ctx.tables.ship)();

    let my_player = my_identity.and_then(|id| players.iter().find(|p| p.identity == id).cloned());
    let my_base = my_identity.and_then(|id| bases.iter().find(|b| b.owner == id).cloned());
    let my_ships: Vec<Ship> = my_identity.map_or(Vec::new(), |id| {
        ships.iter().filter(|s| s.owner == id).cloned().collect()
    });

    let reg_err_msg = use_signal_sync(|| None::<String>);
    let reg_is_submitting = use_signal_sync(|| false);

    let on_logout = move |_| {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(w) = web_sys::window() {
                if let Ok(Some(storage)) = w.local_storage() {
                    let _ = storage.remove_item("spacetimedb_token");
                }
                let _ = w.location().reload();
            }
        }
    };

    rsx! {
        div {
            style: "width: 100vw; height: 100vh; margin: 0; padding: 0; background-color: #060b18; color: #f0f4f8; font-family: system-ui, -apple-system, sans-serif; display: flex; flex-direction: column; overflow: hidden; user-select: none;",
            // Top Nav
            header {
                style: "height: 52px; background: rgba(10, 16, 32, 0.95); border-bottom: 1px solid rgba(80, 120, 200, 0.25); display: flex; align-items: center; justify-content: space-between; padding: 0 20px; z-index: 10;",
                div {
                    style: "display: flex; align-items: center; gap: 12px;",
                    span { style: "font-size: 20px;", "🛸" }
                    h1 {
                        style: "margin: 0; font-size: 17px; font-weight: 700; color: #00e5ff; letter-spacing: 0.5px;",
                        "SCHÜLER-COCKPIT"
                    }
                    span {
                        style: "font-size: 11px; padding: 2px 8px; border-radius: 4px; background: rgba(0, 229, 255, 0.1); color: #00e5ff; border: 1px solid rgba(0, 229, 255, 0.25);",
                        "MEGA-STATION"
                    }
                }
                div {
                    style: "display: flex; align-items: center; gap: 14px; font-size: 13px;",
                    if let Some(ref p) = my_player {
                        div {
                            style: "display: flex; align-items: center; gap: 8px; background: rgba(255, 255, 255, 0.05); padding: 4px 12px; border-radius: 20px; border: 1px solid rgba(255, 255, 255, 0.1);",
                            span { style: "width: 10px; height: 10px; border-radius: 50%; background: hsl({p.color_hue}, 90%, 55%); box-shadow: 0 0 6px hsl({p.color_hue}, 90%, 55%);" }
                            span { style: "font-weight: 600; color: #f1f5f9;", "{p.name}" }
                            span { style: "color: #ffd700; font-size: 12px; font-weight: 700;", "({p.score} Pkt)" }
                        }
                    }
                    span {
                        style: "padding: 3px 10px; border-radius: 10px; font-size: 11px; font-weight: 600; background: rgba(255, 255, 255, 0.08);",
                        match state {
                            ConnectionState::Connected(_, _) => rsx! { span { style: "color: #00e676;", "● ONLINE" } },
                            ConnectionState::Connecting => rsx! { span { style: "color: #ffd700;", "○ VERBINDE..." } },
                            ConnectionState::Reconnecting { attempt, .. } => rsx! { span { style: "color: #ffd700;", "○ RETRY #{attempt}" } },
                            _ => rsx! { span { style: "color: #ff5252;", "○ OFFLINE" } },
                        }
                    }
                    button {
                        style: "background: rgba(239, 68, 68, 0.15); border: 1px solid rgba(239, 68, 68, 0.4); color: #fca5a5; padding: 4px 10px; border-radius: 6px; font-size: 11px; font-weight: 600; cursor: pointer; transition: all 0.2s;",
                        onclick: on_logout,
                        title: "Löscht den lokal gespeicherten Token und verbindet neu",
                        "🔄 Token reset"
                    }
                }
            }

            // Connection Error Alert Banner if offline / disconnected
            if !matches!(state, ConnectionState::Connected(_, _)) {
                div {
                    style: "background: rgba(220, 38, 38, 0.25); border-bottom: 1px solid rgba(239, 68, 68, 0.5); padding: 8px 20px; font-size: 12px; color: #fecaca; display: flex; align-items: center; justify-content: space-between; z-index: 9;",
                    div {
                        style: "display: flex; align-items: center; gap: 8px;",
                        span { "⚠️" }
                        span {
                            match state {
                                ConnectionState::Connecting => "Verbinde mit SpacetimeDB-Server (ws://127.0.0.1:3000)...".to_string(),
                                ConnectionState::Reconnecting { attempt, delay_ms } => format!("Verbindungsaufbau wird wiederholt (Versuch #{attempt}, in {delay_ms}ms)..."),
                                ConnectionState::Error => {
                                    if let Some(ref err) = conn_error {
                                        format!("Verbindungsfehler: {err} — Klicke rechts auf 'Token löschen & Neu verbinden'.")
                                    } else {
                                        "Verbindung fehlgeschlagen — Klicke rechts auf 'Token löschen & Neu verbinden'.".to_string()
                                    }
                                }
                                _ => "Nicht mit SpacetimeDB verbunden.".to_string(),
                            }
                        }
                    }
                    button {
                        style: "background: #b91c1c; border: none; color: #fff; padding: 4px 12px; border-radius: 4px; font-size: 11px; font-weight: 700; cursor: pointer;",
                        onclick: on_logout,
                        "Token löschen & Neu verbinden"
                    }
                }
            }

            // Main Content Area based on Stage
            main {
                style: "flex: 1; display: flex; overflow: hidden; position: relative;",
                if my_player.is_none() {
                    RegistrationScreen {
                        total_registered: players.len(),
                        err_msg: reg_err_msg,
                        is_submitting: reg_is_submitting,
                    }
                } else if my_base.is_none() {
                    BuilderNavigationScreen {
                        player: my_player.unwrap(),
                        builder_ship: my_ships.into_iter().find(|s| s.ship_tier == 0),
                    }
                } else {
                    BaseDashboardScreen {
                        player: my_player.unwrap(),
                        base: my_base.unwrap(),
                        ships: my_ships,
                    }
                }
            }
        }
    }
}

// ==========================================
// Stage 1: Megastation Hangar & Player Registration
// ==========================================

#[component]
fn RegistrationScreen(
    total_registered: usize,
    mut err_msg: SyncSignal<Option<String>>,
    mut is_submitting: SyncSignal<bool>,
) -> Element {
    let ctx = use_spacetimedb_context();
    let mut name_input = use_signal(String::new);
    let mut color_hue = use_signal(|| 180u16);

    let mut submit_registration = move || {
        let name = name_input().trim().to_string();
        if name.is_empty() {
            err_msg.set(Some(
                "Bitte gib ein Rufzeichen / einen Namen für deinen Piloten ein!".to_string(),
            ));
            return;
        }

        if let Some(conn) = (ctx.connection)() {
            is_submitting.set(true);
            let hue = color_hue();
            let res = conn
                .reducers
                .register_player_then(name, hue, move |_ctx, result| {
                    is_submitting.set(false);
                    match result {
                        Ok(Ok(())) => {
                            err_msg.set(None);
                        }
                        Ok(Err(reducer_err)) => {
                            err_msg.set(Some(format!("Fehler: {reducer_err}")));
                        }
                        Err(internal_err) => {
                            err_msg.set(Some(format!("Verbindungsfehler: {internal_err}")));
                        }
                    }
                });

            if let Err(e) = res {
                is_submitting.set(false);
                err_msg.set(Some(format!("Sende-Fehler: {e}")));
            }
        } else {
            err_msg.set(Some(
                "Noch nicht mit dem SpacetimeDB-Server verbunden... Bitte warten.".to_string(),
            ));
        }
    };

    let on_submit = move |_evt: MouseEvent| {
        submit_registration();
    };

    let on_keydown = move |e: KeyboardEvent| {
        if e.key() == Key::Enter {
            submit_registration();
        }
    };

    rsx! {
        div {
            style: "flex: 1; display: flex; align-items: center; justify-content: center; background: radial-gradient(circle at center, #0f1c3f 0%, #060b18 80%); padding: 24px; box-sizing: border-box;",
            div {
                style: "max-width: 900px; width: 100%; display: grid; grid-template-columns: 1fr 1.1fr; background: rgba(15, 23, 42, 0.95); border: 1px solid rgba(80, 120, 200, 0.3); border-radius: 16px; box-shadow: 0 16px 48px rgba(0,0,0,0.6); overflow: hidden;",

                // Left Column: Megastations Hangar & Ship Preview
                div {
                    style: "background: linear-gradient(180deg, rgba(30, 41, 59, 0.8) 0%, rgba(15, 23, 42, 0.9) 100%); border-right: 1px solid rgba(80, 120, 200, 0.2); padding: 32px; display: flex; flex-direction: column; justify-content: space-between;",
                    div {
                        div {
                            style: "display: inline-flex; align-items: center; gap: 6px; padding: 4px 10px; border-radius: 6px; background: rgba(0, 229, 255, 0.15); border: 1px solid rgba(0, 229, 255, 0.3); font-size: 11px; font-weight: 700; color: #00e5ff; letter-spacing: 0.5px; margin-bottom: 12px;",
                            "🏛️ MEGA-STATION • HANGAR 04"
                        }
                        h2 {
                            style: "margin: 0 0 8px 0; font-size: 20px; font-weight: 700; color: #f1f5f9;",
                            "Bauschiff-Startbucht"
                        }
                        p {
                            style: "margin: 0 0 24px 0; font-size: 13px; color: #94a3b8; line-height: 1.5;",
                            "Dein erstes Raumfahrzeug steht im Andock-Hangar bereit. Fliege ins All hinaus und errichte die Heimatbasis für deinen Drohnenschwarm!"
                        }

                        // Ship visual preview card
                        div {
                            style: "background: #090e1a; border: 1px solid rgba(80, 120, 200, 0.25); border-radius: 10px; padding: 20px; text-align: center; margin-bottom: 20px; position: relative;",
                            // Animated SVG preview of Builder Ship with user's selected hue
                            svg {
                                width: "160",
                                height: "120",
                                view_box: "0 0 160 120",
                                style: "margin: 0 auto; filter: drop-shadow(0 0 12px hsl({color_hue}, 90%, 50%));",
                                // Thruster glow
                                polygon {
                                    points: "50,60 30,55 20,60 30,65",
                                    fill: "#00e5ff",
                                    opacity: "0.8",
                                }
                                // Solar panels / wings
                                rect {
                                    x: "65",
                                    y: "25",
                                    width: "8",
                                    height: "70",
                                    fill: "#38bdf8",
                                    stroke: "#0284c7",
                                    stroke_width: "1.5",
                                    rx: "2",
                                }
                                // Ship Main Hull
                                polygon {
                                    points: "115,60 55,40 68,60 55,80",
                                    fill: "hsl({color_hue}, 85%, 52%)",
                                    stroke: "#ffffff",
                                    stroke_width: "2",
                                }
                                // Cockpit glass
                                circle {
                                    cx: "95",
                                    cy: "60",
                                    r: "6",
                                    fill: "#ffffff",
                                    opacity: "0.9",
                                }
                            }
                            div {
                                style: "font-size: 13px; font-weight: 700; color: #e2e8f0; margin-top: 8px;",
                                "Tier-0 Bauschiff (Mk.I)"
                            }
                            div {
                                style: "font-size: 11px; color: #64748b; margin-top: 2px;",
                                "Ausgerüstet mit mobilem Stations-Entfaltungsmodul"
                            }
                        }

                        // Tech Specs
                        div {
                            style: "font-size: 12px; color: #94a3b8; display: flex; flex-direction: column; gap: 6px; background: rgba(0, 0, 0, 0.2); padding: 12px; border-radius: 8px;",
                            div { style: "display: flex; justify-content: space-between;", span { "Start-Sektor:" } span { style: "color: #cbd5e1; font-weight: 600;", "Mega-Station (1600, 1000)" } }
                            div { style: "display: flex; justify-content: space-between;", span { "Antrieb:" } span { style: "color: #cbd5e1; font-weight: 600;", "Ionen-Triebwerk (150 px/s)" } }
                            div { style: "display: flex; justify-content: space-between;", span { "Mindest-Bauabstand:" } span { style: "color: #00e676; font-weight: 600;", "250 Einheiten" } }
                        }
                    }

                    div {
                        style: "font-size: 11px; color: #64748b; margin-top: 16px; text-align: center;",
                        "Bereits registrierte Piloten im Sektor: "
                        strong { style: "color: #00e5ff;", "{total_registered}" }
                    }
                }

                // Right Column: Registration Form
                div {
                    style: "padding: 36px 32px; display: flex; flex-direction: column; justify-content: center;",
                    h2 {
                        style: "margin: 0 0 6px 0; font-size: 22px; font-weight: 700; color: #00e5ff;",
                        "Pilotenausweis ausstellen"
                    }
                    p {
                        style: "margin: 0 0 24px 0; font-size: 14px; color: #94a3b8; line-height: 1.4;",
                        "Wähle dein Rufzeichen und deine Team-Farbe, um die Startfreigabe aus dem Hangar zu erhalten."
                    }

                    // Name input
                    div {
                        style: "margin-bottom: 20px;",
                        label { style: "display: block; font-size: 13px; font-weight: 600; color: #cbd5e1; margin-bottom: 6px;", "Dein Rufzeichen / Pilot-Name:" }
                        input {
                            style: "width: 100%; box-sizing: border-box; background: #0f172a; border: 1px solid #334155; color: #fff; padding: 12px 14px; border-radius: 8px; font-size: 15px; outline: none; transition: border-color 0.2s;",
                            placeholder: "z.B. Nova, Orion, Alex...",
                            value: "{name_input}",
                            oninput: move |e| name_input.set(e.value()),
                            onkeydown: on_keydown,
                        }
                    }

                    // Color picker
                    div {
                        style: "margin-bottom: 24px;",
                        label { style: "display: block; font-size: 13px; font-weight: 600; color: #cbd5e1; margin-bottom: 8px;", "Team- & Schiffsfarbe (Farbton):" }
                        div {
                            style: "display: flex; align-items: center; gap: 14px;",
                            input {
                                r#type: "range",
                                min: "0",
                                max: "359",
                                value: "{color_hue}",
                                style: "flex: 1; cursor: pointer; accent-color: hsl({color_hue}, 90%, 55%); height: 8px;",
                                oninput: move |e| {
                                    if let Ok(val) = e.value().parse::<u16>() {
                                        color_hue.set(val);
                                    }
                                }
                            }
                            span {
                                style: "width: 36px; height: 36px; border-radius: 8px; background: hsl({color_hue}, 90%, 55%); border: 2px solid #ffffff; box-shadow: 0 0 12px hsl({color_hue}, 90%, 55%); flex-shrink: 0;"
                            }
                        }
                        div {
                            style: "font-size: 11px; color: #64748b; margin-top: 6px;",
                            "Deine Heimatbasis und alle Drohnen werden in dieser Farbe markiert."
                        }
                    }

                    if let Some(ref err) = err_msg() {
                        div {
                            style: "margin-bottom: 20px; padding: 10px 14px; background: rgba(239, 68, 68, 0.15); border: 1px solid #ef4444; border-radius: 8px; color: #fca5a5; font-size: 13px; line-height: 1.4;",
                            "⚠️ {err}"
                        }
                    }

                    button {
                        disabled: is_submitting(),
                        style: if is_submitting() {
                            "width: 100%; background: #0369a1; color: #94a3b8; border: none; padding: 14px; border-radius: 8px; font-size: 16px; font-weight: 700; cursor: wait;"
                        } else {
                            "width: 100%; background: #0284c7; hover:background: #0369a1; color: #fff; border: none; padding: 14px; border-radius: 8px; font-size: 16px; font-weight: 700; cursor: pointer; transition: all 0.2s; box-shadow: 0 4px 16px rgba(2, 132, 199, 0.4);"
                        },
                        onclick: on_submit,
                        if is_submitting() {
                            "⏳ Registriere Pilot..."
                        } else {
                            "🚀 Bauschiff starten"
                        }
                    }
                }
            }
        }
    }
}

// ==========================================
// Stage 2: Builder Ship Navigation & Base Deployment
// ==========================================

#[component]
fn BuilderNavigationScreen(player: Player, builder_ship: Option<Ship>) -> Element {
    let ctx = use_spacetimedb_context();
    let configs = (ctx.tables.config)();
    let config = configs.first().cloned();

    let mut target_marker = use_signal(|| None::<Vec2>);

    let ship = match builder_ship {
        Some(s) => s,
        None => {
            return rsx! {
                div {
                    style: "flex: 1; display: flex; align-items: center; justify-content: center; background: #060b18; color: #94a3b8;",
                    div {
                        style: "text-align: center; padding: 32px; background: rgba(15, 23, 42, 0.8); border-radius: 12px; border: 1px solid rgba(80, 120, 200, 0.3);",
                        div { style: "font-size: 32px; margin-bottom: 12px;", "⏳" }
                        h3 { style: "margin: 0 0 8px 0; color: #00e5ff;", "Bauschiff wird bereitgestellt..." }
                        p { "Verbindung mit SpacetimeDB-Hangar aktiv." }
                    }
                }
            };
        }
    };

    let mega_pos = config.as_ref().map_or(
        Vec2 {
            x: 1600.0,
            y: 1000.0,
        },
        |c| c.megastation_pos.clone(),
    );
    let dist_to_megastation = distance(&ship.position, &mega_pos);
    let can_deploy = dist_to_megastation >= 250.0;

    let p_identity = player.identity;
    let p_hue = player.color_hue;

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

    // Trigger canvas radar render whenever tables change or resize occurs
    use_effect(move || {
        let _ = resize_trigger();
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(w) = web_sys::window() {
                if let Some(doc) = w.document() {
                    if let Some(canvas_el) = doc.get_element_by_id("radar-canvas") {
                        if let Ok(canvas) = canvas_el.dyn_into::<HtmlCanvasElement>() {
                            let rect = canvas.get_bounding_client_rect();
                            let css_w = rect.width();
                            let css_h = rect.height();
                            if css_w > 0.0 && css_h > 0.0 {
                                let dpr = w.device_pixel_ratio().max(1.0);
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
                                        let asteroids = (ctx.tables.asteroid)();
                                        let ships = (ctx.tables.ship)();
                                        let current_ship = ships
                                            .into_iter()
                                            .find(|s| s.owner == p_identity && s.ship_tier == 0);

                                        if let Some(ref s) = current_ship {
                                            let ships_slice = [s.clone()];
                                            let time = js_sys::Date::now() / 1000.0;
                                            render_player_radar(
                                                &ctx_2d,
                                                css_w,
                                                css_h,
                                                dpr,
                                                s.position.clone(),
                                                p_hue,
                                                configs.first(),
                                                &ships_slice,
                                                None,
                                                &asteroids,
                                                target_marker(),
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
        }
    });

    let on_canvas_click = move |evt: ::dioxus::prelude::MouseEvent| {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(w) = web_sys::window() {
                if let Some(doc) = w.document() {
                    if let Some(canvas_el) = doc.get_element_by_id("radar-canvas") {
                        if let Ok(canvas) = canvas_el.dyn_into::<HtmlCanvasElement>() {
                            let rect = canvas.get_bounding_client_rect();
                            if rect.width() > 0.0 && rect.height() > 0.0 {
                                let coords = evt.client_coordinates();
                                let css_x = coords.x - rect.left();
                                let css_y = coords.y - rect.top();

                                // Canvas center corresponds to ship position
                                let rel_x = css_x - (rect.width() / 2.0);
                                let rel_y = css_y - (rect.height() / 2.0);

                                let target_world = Vec2 {
                                    x: ship.position.x + rel_x as f32,
                                    y: ship.position.y + rel_y as f32,
                                };

                                target_marker.set(Some(target_world.clone()));

                                if let Some(conn) = (ctx.connection)() {
                                    let _ = conn.reducers.command_ship(
                                        ship.ship_id,
                                        ShipCommand::FlyTo(target_world),
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    };

    let on_deploy = move |_| {
        if can_deploy {
            if let Some(conn) = (ctx.connection)() {
                let _ = conn.reducers.deploy_base_station(ship.ship_id);
            }
        }
    };

    rsx! {
        div {
            style: "flex: 1; display: flex; flex-direction: row; height: 100%; overflow: hidden;",

            // Radar Canvas Area (Left)
            div {
                style: "flex: 1; height: 100%; position: relative; background: #050a16; display: flex; align-items: center; justify-content: center; overflow: hidden;",
                canvas {
                    id: "radar-canvas",
                    style: "width: 100%; height: 100%; display: block; cursor: crosshair;",
                    onclick: on_canvas_click,
                }
                div {
                    style: "position: absolute; bottom: 20px; left: 24px; background: rgba(10, 16, 32, 0.85); padding: 8px 16px; border-radius: 8px; font-size: 13px; color: #94a3b8; border: 1px solid rgba(80, 120, 200, 0.25); display: flex; align-items: center; gap: 8px;",
                    span { "💡" }
                    span { "Klicke mit der Maus auf das Radar, um dein Bauschiff zum Zielpunkt zu steuern." }
                }
            }

            // Mission Instructions & Deploy Sidebar (Right)
            aside {
                style: "width: 360px; height: 100%; background: #0a1020; border-left: 1px solid rgba(80, 120, 200, 0.2); padding: 24px; box-sizing: border-box; display: flex; flex-direction: column; justify-content: space-between;",
                div {
                    div {
                        style: "display: inline-flex; align-items: center; gap: 6px; padding: 4px 10px; border-radius: 6px; background: rgba(0, 229, 255, 0.15); border: 1px solid rgba(0, 229, 255, 0.3); font-size: 11px; font-weight: 700; color: #00e5ff; letter-spacing: 0.5px; margin-bottom: 12px;",
                        "PHASE 0: KOLONISATION"
                    }
                    h2 {
                        style: "margin: 0 0 12px 0; font-size: 20px; font-weight: 700; color: #00e5ff;",
                        "🎯 Mission: Basenbau"
                    }
                    p {
                        style: "margin: 0 0 20px 0; font-size: 14px; color: #94a3b8; line-height: 1.5;",
                        "Fliege mindestens "
                        strong { style: "color: #f1f5f9;", "250 Einheiten" }
                        " von der zentralen Mega-Station weg, um einen sicheren Bauplatz für deine persönliche Heimatbasis zu beanspruchen."
                    }

                    // Distance status card
                    div {
                        style: "padding: 16px; background: rgba(255, 255, 255, 0.04); border-radius: 8px; border: 1px solid rgba(80, 120, 200, 0.15); margin-bottom: 20px;",
                        div { style: "font-size: 11px; font-weight: 600; color: #8da2c0; margin-bottom: 6px; letter-spacing: 0.5px;", "ABSTAND ZUR MEGA-STATION:" }
                        div {
                            style: "font-size: 24px; font-weight: 700; color: if can_deploy { \"#00e676\" } else { \"#ff5252\" };",
                            "{dist_to_megastation as u32} / 250 Einheiten"
                        }
                        div {
                            style: "margin-top: 8px; font-size: 12px;",
                            if can_deploy {
                                span { style: "color: #00e676; font-weight: 600;", "✔ Bauplatz freigegeben! Heimatbasis kann hier errichtet werden." }
                            } else {
                                span { style: "color: #ff5252;", "✖ Zu nah an der Mega-Station (Sperrzone)." }
                            }
                        }
                    }

                    // Ship Stats
                    div {
                        style: "padding: 16px; background: rgba(255, 255, 255, 0.02); border-radius: 8px; font-size: 13px; color: #94a3b8; display: flex; flex-direction: column; gap: 8px;",
                        div { style: "display: flex; justify-content: space-between;", span { "Position:" } span { style: "color: #cbd5e1; font-mono;", "({ship.position.x as i32}, {ship.position.y as i32})" } }
                        div { style: "display: flex; justify-content: space-between;", span { "Geschwindigkeit:" } span { style: "color: #cbd5e1;", "{ship.speed as i32} / {ship.max_speed as i32} px/s" } }
                        div { style: "display: flex; justify-content: space-between;", span { "Energie:" } span { style: "color: #00e5ff;", "{ship.energy as u32} / {ship.max_energy as u32}" } }
                    }
                }

                // Deploy Button
                div {
                    button {
                        disabled: !can_deploy,
                        style: if can_deploy {
                            "width: 100%; background: #059669; hover:background: #047857; color: #fff; border: none; padding: 14px; border-radius: 8px; font-size: 16px; font-weight: 700; cursor: pointer; box-shadow: 0 4px 16px rgba(5, 150, 105, 0.4); transition: all 0.2s;"
                        } else {
                            "width: 100%; background: #334155; color: #64748b; border: none; padding: 14px; border-radius: 8px; font-size: 16px; font-weight: 700; cursor: not-allowed;"
                        },
                        onclick: on_deploy,
                        "🏛️ Heimatbasis hier errichten"
                    }
                    div {
                        style: "font-size: 11px; color: #64748b; margin-top: 8px; text-align: center;",
                        if can_deploy {
                            "Wandelt das Bauschiff in deine Heimatbasis um"
                        } else {
                            "Erst ab 250m Abstand zur Megastation aktivierbar"
                        }
                    }
                }
            }
        }
    }
}

// ==========================================
// Stage 3: Home Base Dashboard, Local Radar & Token Export (API Mode)
// ==========================================

#[component]
fn BaseDashboardScreen(player: Player, base: BaseStation, ships: Vec<Ship>) -> Element {
    let ctx = use_spacetimedb_context();
    let state = (ctx.state)();
    let mut copied = use_signal(|| false);

    let token = match state {
        ConnectionState::Connected(_, ref t) => t.clone(),
        _ => "".to_string(),
    };

    let p_hue = player.color_hue;
    let base_pos = base.position.clone();

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

    // Render local base radar
    use_effect(move || {
        let _ = resize_trigger();
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(w) = web_sys::window() {
                if let Some(doc) = w.document() {
                    if let Some(canvas_el) = doc.get_element_by_id("base-radar-canvas") {
                        if let Ok(canvas) = canvas_el.dyn_into::<HtmlCanvasElement>() {
                            let rect = canvas.get_bounding_client_rect();
                            let css_w = rect.width();
                            let css_h = rect.height();
                            if css_w > 0.0 && css_h > 0.0 {
                                let dpr = w.device_pixel_ratio().max(1.0);
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
                                        let asteroids = (ctx.tables.asteroid)();
                                        let bases = (ctx.tables.base_station)();
                                        let current_ships = (ctx.tables.ship)();

                                        let my_current_base =
                                            bases.iter().find(|b| b.owner == player.identity);
                                        let my_current_ships: Vec<Ship> = current_ships
                                            .into_iter()
                                            .filter(|s| s.owner == player.identity)
                                            .collect();

                                        let time = js_sys::Date::now() / 1000.0;

                                        render_player_radar(
                                            &ctx_2d,
                                            css_w,
                                            css_h,
                                            dpr,
                                            base_pos.clone(),
                                            p_hue,
                                            configs.first(),
                                            &my_current_ships,
                                            my_current_base,
                                            &asteroids,
                                            None,
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

    let copy_token = {
        let t = token.clone();
        move |_| {
            #[cfg(target_arch = "wasm32")]
            {
                if let Some(w) = web_sys::window() {
                    let nav = w.navigator();
                    let clipboard = nav.clipboard();
                    let _ = clipboard.write_text(&t);
                    copied.set(true);
                }
            }
        }
    };

    rsx! {
        div {
            style: "flex: 1; display: flex; flex-direction: row; height: 100%; overflow: hidden; background: #070d1d;",

            // Left / Main: Local Sensor Radar
            div {
                style: "flex: 1; height: 100%; position: relative; background: #050a16; display: flex; align-items: center; justify-content: center; overflow: hidden;",
                canvas {
                    id: "base-radar-canvas",
                    style: "width: 100%; height: 100%; display: block;",
                }
                div {
                    style: "position: absolute; top: 16px; left: 20px; display: flex; gap: 10px;",
                    div {
                        style: "background: rgba(10, 16, 32, 0.9); padding: 8px 14px; border-radius: 8px; font-size: 12px; color: #00e5ff; border: 1px solid rgba(0, 229, 255, 0.3); display: flex; align-items: center; gap: 8px;",
                        span { "📡" }
                        span { style: "font-weight: 700;", "LOKALES RADAR" }
                        span { style: "color: #94a3b8;", "• Funkbereich: {base.signal_radius as u32} px" }
                    }
                }
                div {
                    style: "position: absolute; bottom: 16px; left: 20px; background: rgba(10, 16, 32, 0.85); padding: 8px 14px; border-radius: 6px; font-size: 12px; color: #94a3b8; border: 1px solid rgba(80, 120, 200, 0.2);",
                    "ℹ️ Drohnenflotte wird über die SpacetimeDB-API per Rust-Code in Zed gesteuert."
                }
            }

            // Right Column: Controls, Token Export & Fleet Status
            aside {
                style: "width: 440px; height: 100%; background: #0a1020; border-left: 1px solid rgba(80, 120, 200, 0.2); padding: 24px; box-sizing: border-box; display: flex; flex-direction: column; gap: 20px; overflow-y: auto;",

                // Header Notification
                div {
                    style: "background: rgba(15, 23, 42, 0.95); border: 1px solid rgba(80, 120, 200, 0.3); border-radius: 12px; padding: 20px;",
                    div {
                        style: "display: inline-flex; align-items: center; gap: 6px; padding: 4px 10px; border-radius: 6px; background: rgba(0, 230, 118, 0.15); border: 1px solid rgba(0, 230, 118, 0.3); font-size: 11px; font-weight: 700; color: #00e676; letter-spacing: 0.5px; margin-bottom: 10px;",
                        "✔ BASIS ETABLIERT"
                    }
                    h2 {
                        style: "margin: 0 0 8px 0; font-size: 18px; font-weight: 700; color: #ffd700;",
                        "🎉 Heimatbasis aktiv!"
                    }
                    p {
                        style: "margin: 0; font-size: 13px; color: #94a3b8; line-height: 1.4;",
                        "Die manuelle Maussteuerung ist beendet. Steuere deine Flotte ab jetzt über "
                        strong { style: "color: #00e5ff;", "Rust-Code im Zed-Editor" }
                        " via SpacetimeDB!"
                    }
                }

                // Token One-Click-Copy Box
                div {
                    style: "background: #0f172a; border: 1px solid #334155; border-radius: 10px; padding: 18px;",
                    label {
                        style: "display: block; font-size: 11px; font-weight: 700; color: #8da2c0; margin-bottom: 8px; letter-spacing: 0.5px;",
                        "DEIN PERSÖNLICHER API-IDENTITÄTS-TOKEN:"
                    }
                    div {
                        style: "display: flex; gap: 8px;",
                        input {
                            readonly: true,
                            style: "flex: 1; background: #020617; border: 1px solid #1e293b; color: #38bdf8; padding: 10px 12px; border-radius: 6px; font-family: monospace; font-size: 12px; outline: none;",
                            value: "{token}",
                        }
                        button {
                            style: if copied() {
                                "background: #059669; color: #fff; border: none; padding: 10px 16px; border-radius: 6px; font-weight: 700; font-size: 13px; cursor: pointer; transition: all 0.2s;"
                            } else {
                                "background: #0284c7; hover:background: #0369a1; color: #fff; border: none; padding: 10px 16px; border-radius: 6px; font-weight: 700; font-size: 13px; cursor: pointer; transition: all 0.2s;"
                            },
                            onclick: copy_token,
                            if copied() { "✔ Kopiert!" } else { "📋 Kopieren" }
                        }
                    }
                    div {
                        style: "font-size: 11px; color: #64748b; margin-top: 8px;",
                        "Füge diesen Token in die " code { style: "color: #38bdf8;", "config.toml" } " deines Zed-Bot-Projekts ein."
                    }
                }

                // Station Stats Card
                div {
                    style: "background: rgba(15, 23, 42, 0.95); border: 1px solid rgba(80, 120, 200, 0.25); border-radius: 12px; padding: 18px;",
                    h3 { style: "margin: 0 0 14px 0; font-size: 15px; font-weight: 700; color: #00e5ff;", "🏠 Stationsstatus" }
                    div {
                        style: "display: grid; grid-template-columns: 1fr 1fr; gap: 10px;",
                        div {
                            style: "padding: 10px; background: rgba(255, 255, 255, 0.03); border-radius: 6px;",
                            div { style: "font-size: 11px; color: #8da2c0;", "GELAGERTES ERZ" }
                            div { style: "font-size: 20px; font-weight: 700; color: #ffd700;", "{base.stored_ore}" }
                        }
                        div {
                            style: "padding: 10px; background: rgba(255, 255, 255, 0.03); border-radius: 6px;",
                            div { style: "font-size: 11px; color: #8da2c0;", "ENERGIE" }
                            div { style: "font-size: 20px; font-weight: 700; color: #00e5ff;", "{base.stored_energy}" }
                        }
                        div {
                            style: "padding: 10px; background: rgba(255, 255, 255, 0.03); border-radius: 6px;",
                            div { style: "font-size: 11px; color: #8da2c0;", "FUNK-REICHWEITE" }
                            div { style: "font-size: 18px; font-weight: 700; color: #38bdf8;", "{base.signal_radius as u32} px" }
                        }
                        div {
                            style: "padding: 10px; background: rgba(255, 255, 255, 0.03); border-radius: 6px;",
                            div { style: "font-size: 11px; color: #8da2c0;", "GESAMTPUNKTE" }
                            div { style: "font-size: 18px; font-weight: 700; color: #00e676;", "{player.score}" }
                        }
                    }
                    div {
                        style: "margin-top: 10px; font-size: 11px; color: #64748b; text-align: right;",
                        "Position: ({base.position.x as i32}, {base.position.y as i32}) • Level {base.level}"
                    }
                }

                // Zed Steps Instructions
                div {
                    style: "background: rgba(15, 23, 42, 0.95); border: 1px solid rgba(80, 120, 200, 0.25); border-radius: 12px; padding: 18px;",
                    h3 { style: "margin: 0 0 10px 0; font-size: 14px; font-weight: 700; color: #00e5ff;", "🚀 Nächste Schritte in Zed:" }
                    ol {
                        style: "margin: 0; padding-left: 20px; font-size: 13px; color: #cbd5e1; display: flex; flex-direction: column; gap: 6px; line-height: 1.4;",
                        li { "Öffne das Projekt über den Launcher (" strong { "uhlaunch" } ") in Zed." }
                        li { "Füge den kopierten Token in die " code { style: "background: #1e293b; color: #38bdf8; padding: 2px 4px; border-radius: 4px;", "config.toml" } " ein." }
                        li { "Schreibe deine Bot-Logik in " code { style: "background: #1e293b; color: #38bdf8; padding: 2px 4px; border-radius: 4px;", "src/ai.rs" } "." }
                        li { "Starte deinen Drohnenschwarm mit: " code { style: "background: #1e293b; color: #00e676; padding: 2px 4px; border-radius: 4px;", "cargo run" } }
                    }
                }

                // Active Ships Fleet List
                div {
                    style: "background: rgba(15, 23, 42, 0.95); border: 1px solid rgba(80, 120, 200, 0.25); border-radius: 12px; padding: 18px;",
                    h3 { style: "margin: 0 0 12px 0; font-size: 15px; font-weight: 700; color: #00e5ff;", "🚀 Deine Flotte ({ships.len()} Schiffe)" }
                    div {
                        style: "display: flex; flex-direction: column; gap: 8px;",
                        for s in ships {
                            div {
                                key: "{s.ship_id}",
                                style: "padding: 10px 12px; background: rgba(255, 255, 255, 0.03); border-radius: 6px; border-left: 3px solid #00e5ff;",
                                div {
                                    style: "display: flex; justify-content: space-between; margin-bottom: 4px;",
                                    span { style: "font-weight: 700; font-size: 13px;", "Drohne #{s.ship_id} (Tier {s.ship_tier})" }
                                    span {
                                        style: if s.is_stranded {
                                            "color: #ef4444; font-weight: 700; font-size: 11px;"
                                        } else if s.is_mining {
                                            "color: #00e5ff; font-weight: 700; font-size: 11px;"
                                        } else {
                                            "color: #00e676; font-size: 11px;"
                                        },
                                        if s.is_stranded {
                                            "GESTRANDET"
                                        } else if s.is_mining {
                                            "BAUT AB ⛏️"
                                        } else {
                                            "BEREIT (API)"
                                        }
                                    }
                                }
                                div {
                                    style: "font-size: 11px; color: #94a3b8; display: flex; justify-content: space-between;",
                                    span { "Fracht: {s.cargo_ore}/{s.max_cargo} Erz" }
                                    span { "Energie: {s.energy as u32}/{s.max_energy as u32}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
