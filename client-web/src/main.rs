pub mod module_bindings;
pub mod radar;

use ::dioxus::prelude::*;
use module_bindings::command_ship_reducer::command_ship;
use module_bindings::deploy_base_station_reducer::deploy_base_station;
use module_bindings::dioxus::{
    ConnectionState, use_spacetimedb_context, use_spacetimedb_context_provider,
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

    let _stdb_ctx = use_spacetimedb_context_provider("ws://localhost:3000", "server", saved_token);

    let ctx = use_spacetimedb_context();
    let state = (ctx.state)();

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

    rsx! {
        div {
            style: "width: 100vw; height: 100vh; margin: 0; padding: 0; background-color: #060b18; color: #f0f4f8; font-family: system-ui, -apple-system, sans-serif; display: flex; flex-direction: column; overflow: hidden; user-select: none;",
            // Top Nav
            header {
                style: "height: 52px; background: rgba(10, 16, 32, 0.95); border-bottom: 1px solid rgba(80, 120, 200, 0.25); display: flex; align-items: center; justify-content: space-between; padding: 0 20px;",
                div {
                    style: "display: flex; align-items: center; gap: 12px;",
                    span { style: "font-size: 18px;", "🛸" }
                    h1 {
                        style: "margin: 0; font-size: 17px; font-weight: 700; color: #00e5ff;",
                        "SCHÜLER-COCKPIT"
                    }
                }
                div {
                    style: "display: flex; align-items: center; gap: 16px; font-size: 13px;",
                    if let Some(ref p) = my_player {
                        span {
                            style: "display: flex; align-items: center; gap: 6px; font-weight: 600;",
                            span { style: "width: 10px; height: 10px; border-radius: 50%; background: hsl({p.color_hue}, 90%, 55%);" }
                            "{p.name} ({p.score} Pkt)"
                        }
                    }
                    span {
                        style: "padding: 3px 8px; border-radius: 10px; font-size: 11px; font-weight: 600; background: rgba(255, 255, 255, 0.08);",
                        match state {
                            ConnectionState::Connected(_, _) => rsx! { span { style: "color: #00e676;", "VERBUNDEN" } },
                            ConnectionState::Connecting => rsx! { span { style: "color: #ffd700;", "VERBINDE..." } },
                            _ => rsx! { span { style: "color: #ff5252;", "OFFLINE" } },
                        }
                    }
                }
            }

            // Main Content Area based on Stage
            main {
                style: "flex: 1; display: flex; overflow: hidden; position: relative;",
                if my_player.is_none() {
                    RegistrationScreen {}
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
// Stage 1: Player Registration
// ==========================================

#[component]
fn RegistrationScreen() -> Element {
    let ctx = use_spacetimedb_context();
    let mut name_input = use_signal(String::new);
    let mut color_hue = use_signal(|| 180u16);
    let mut err_msg = use_signal(|| None::<String>);

    let on_submit = move |_| {
        let name = name_input().trim().to_string();
        if name.is_empty() {
            err_msg.set(Some("Bitte gib einen Rufnamen ein!".to_string()));
            return;
        }

        if let Some(conn) = (ctx.connection)() {
            match conn.reducers.register_player(name, color_hue()) {
                Ok(_) => err_msg.set(None),
                Err(e) => err_msg.set(Some(format!("Fehler: {e}"))),
            }
        } else {
            err_msg.set(Some("Noch nicht mit Server verbunden...".to_string()));
        }
    };

    rsx! {
        div {
            style: "flex: 1; display: flex; align-items: center; justify-content: center; background: radial-gradient(circle at center, #0f1c3f 0%, #060b18 70%);",
            div {
                style: "width: 420px; background: rgba(15, 23, 42, 0.95); border: 1px solid rgba(80, 120, 200, 0.3); border-radius: 12px; padding: 32px; box-shadow: 0 12px 36px rgba(0,0,0,0.5);",
                h2 {
                    style: "margin: 0 0 8px 0; font-size: 22px; font-weight: 700; color: #00e5ff; text-align: center;",
                    "Ankunft an der Mega-Station"
                }
                p {
                    style: "margin: 0 0 24px 0; font-size: 14px; color: #94a3b8; text-align: center; line-height: 1.4;",
                    "Registriere deinen Piloten, besteige dein Bauschiff und suche dir einen Platz im Sektor für deine Heimatbasis!"
                }

                // Name input
                div {
                    style: "margin-bottom: 20px;",
                    label { style: "display: block; font-size: 13px; font-weight: 600; color: #cbd5e1; margin-bottom: 6px;", "Pilot / Rufzeichen:" }
                    input {
                        style: "width: 100%; box-sizing: border-box; background: #0f172a; border: 1px solid #334155; color: #fff; padding: 10px 14px; border-radius: 6px; font-size: 15px; outline: none;",
                        placeholder: "z.B. Nova, Orion, Alex...",
                        value: "{name_input}",
                        oninput: move |e| name_input.set(e.value()),
                    }
                }

                // Color picker
                div {
                    style: "margin-bottom: 24px;",
                    label { style: "display: block; font-size: 13px; font-weight: 600; color: #cbd5e1; margin-bottom: 8px;", "Team-Farbe:" }
                    div {
                        style: "display: flex; align-items: center; gap: 14px; margin-bottom: 10px;",
                        input {
                            r#type: "range",
                            min: "0",
                            max: "359",
                            value: "{color_hue}",
                            style: "flex: 1; cursor: pointer; accent-color: hsl({color_hue}, 90%, 55%);",
                            oninput: move |e| {
                                if let Ok(val) = e.value().parse::<u16>() {
                                    color_hue.set(val);
                                }
                            }
                        }
                        span {
                            style: "width: 32px; height: 32px; border-radius: 6px; background: hsl({color_hue}, 90%, 55%); border: 2px solid #ffffff; box-shadow: 0 0 10px hsl({color_hue}, 90%, 55%);"
                        }
                    }
                }

                if let Some(ref err) = err_msg() {
                    div {
                        style: "margin-bottom: 16px; padding: 8px 12px; background: rgba(239, 68, 68, 0.15); border: 1px solid #ef4444; border-radius: 6px; color: #fca5a5; font-size: 13px;",
                        "{err}"
                    }
                }

                button {
                    style: "width: 100%; background: #0284c7; hover:background: #0369a1; color: #fff; border: none; padding: 12px; border-radius: 6px; font-size: 16px; font-weight: 700; cursor: pointer; transition: background 0.2s;",
                    onclick: on_submit,
                    "🚀 Bauschiff starten"
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
                div { style: "padding: 30px; text-align: center; color: #94a3b8;", "Bauschiff wird bereitgestellt..." }
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

    // Trigger canvas radar render
    use_effect(move || {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(w) = web_sys::window() {
                if let Some(doc) = w.document() {
                    if let Some(canvas_el) = doc.get_element_by_id("radar-canvas") {
                        if let Ok(canvas) = canvas_el.dyn_into::<HtmlCanvasElement>() {
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
                                        render_player_radar(
                                            &ctx_2d,
                                            canvas.width() as f64,
                                            canvas.height() as f64,
                                            s.position.clone(),
                                            p_hue,
                                            configs.first(),
                                            Some(s),
                                            None,
                                            &asteroids,
                                            target_marker(),
                                            0.0,
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

    let on_canvas_click = move |evt: ::dioxus::prelude::MouseEvent| {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(w) = web_sys::window() {
                if let Some(doc) = w.document() {
                    if let Some(canvas_el) = doc.get_element_by_id("radar-canvas") {
                        if let Ok(canvas) = canvas_el.dyn_into::<HtmlCanvasElement>() {
                            let rect = canvas.get_bounding_client_rect();
                            let coords = evt.client_coordinates();
                            let click_x = coords.x - rect.left();
                            let click_y = coords.y - rect.top();

                            let cw = canvas.width() as f64;
                            let ch = canvas.height() as f64;

                            // Canvas center corresponds to ship position
                            let rel_x = click_x - (cw / 2.0);
                            let rel_y = click_y - (ch / 2.0);

                            let target_world = Vec2 {
                                x: ship.position.x + rel_x as f32,
                                y: ship.position.y + rel_y as f32,
                            };

                            target_marker.set(Some(target_world.clone()));

                            if let Some(conn) = (ctx.connection)() {
                                let _ = conn
                                    .reducers
                                    .command_ship(ship.ship_id, ShipCommand::FlyTo(target_world));
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
            style: "flex: 1; display: flex; flex-direction: row; height: 100%;",

            // Radar Canvas Area (Left)
            div {
                style: "flex: 1; height: 100%; position: relative; background: #050a16; display: flex; align-items: center; justify-content: center;",
                canvas {
                    id: "radar-canvas",
                    width: "1200",
                    height: "900",
                    style: "width: 100%; height: 100%; object-fit: contain; cursor: crosshair;",
                    onclick: on_canvas_click,
                }
                div {
                    style: "position: absolute; bottom: 20px; left: 24px; background: rgba(10, 16, 32, 0.85); padding: 8px 14px; border-radius: 6px; font-size: 13px; color: #94a3b8; border: 1px solid rgba(80, 120, 200, 0.2);",
                    "💡 Klicke auf die Karte, um dein Bauschiff zu steuern."
                }
            }

            // Mission Instructions & Deploy Sidebar (Right)
            aside {
                style: "width: 360px; height: 100%; background: #0a1020; border-left: 1px solid rgba(80, 120, 200, 0.2); padding: 24px; box-sizing: border-box; display: flex; flex-direction: column; justify-content: space-between;",
                div {
                    h2 {
                        style: "margin: 0 0 12px 0; font-size: 18px; font-weight: 700; color: #00e5ff;",
                        "🎯 Mission: Basenbau"
                    }
                    p {
                        style: "margin: 0 0 20px 0; font-size: 14px; color: #94a3b8; line-height: 1.4;",
                        "Fliege mindestens 250 Einheiten von der zentralen Mega-Station weg, um Platz für deine Heimatbasis zu beanspruchen."
                    }

                    // Distance status card
                    div {
                        style: "padding: 16px; background: rgba(255, 255, 255, 0.04); border-radius: 8px; margin-bottom: 20px;",
                        div { style: "font-size: 12px; color: #8da2c0; margin-bottom: 4px;", "ABSTAND ZUR MEGA-STATION:" }
                        div {
                            style: "font-size: 22px; font-weight: 700; color: if can_deploy { \"#00e676\" } else { \"#ff5252\" };",
                            "{dist_to_megastation as u32} / 250 Einheiten"
                        }
                        div {
                            style: "margin-top: 8px; font-size: 12px;",
                            if can_deploy {
                                span { style: "color: #00e676;", "✔ Sicherer Bauabstand erreicht!" }
                            } else {
                                span { style: "color: #ff5252;", "✖ Zu nah an der Mega-Station." }
                            }
                        }
                    }

                    // Ship Stats
                    div {
                        style: "font-size: 13px; color: #94a3b8; display: flex; flex-direction: column; gap: 6px;",
                        div { "Position: ({ship.position.x as i32}, {ship.position.y as i32})" }
                        div { "Geschwindigkeit: {ship.speed as i32} / {ship.max_speed as i32}" }
                        div { "Energie: {ship.energy as u32} / {ship.max_energy as u32}" }
                    }
                }

                // Deploy Button
                button {
                    disabled: !can_deploy,
                    style: if can_deploy {
                        "width: 100%; background: #059669; hover:background: #047857; color: #fff; border: none; padding: 14px; border-radius: 8px; font-size: 16px; font-weight: 700; cursor: pointer; box-shadow: 0 4px 14px rgba(5, 150, 105, 0.4);"
                    } else {
                        "width: 100%; background: #334155; color: #64748b; border: none; padding: 14px; border-radius: 8px; font-size: 16px; font-weight: 700; cursor: not-allowed;"
                    },
                    onclick: on_deploy,
                    "🏛️ Heimatbasis hier errichten"
                }
            }
        }
    }
}

// ==========================================
// Stage 3: Home Base Dashboard & Token Export (API Mode)
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
            style: "flex: 1; display: flex; padding: 32px; gap: 32px; overflow-y: auto; background: #070d1d;",

            // Left Column: Welcome & Zed Token Instructions
            div {
                style: "flex: 1; max-width: 650px;",
                div {
                    style: "background: rgba(15, 23, 42, 0.95); border: 1px solid rgba(80, 120, 200, 0.25); border-radius: 12px; padding: 28px; margin-bottom: 24px;",
                    h2 {
                        style: "margin: 0 0 10px 0; font-size: 22px; font-weight: 700; color: #ffd700;",
                        "🎉 Heimatbasis etabliert!"
                    }
                    p {
                        style: "margin: 0 0 20px 0; font-size: 14px; color: #94a3b8; line-height: 1.5;",
                        "Die manuelle Steuerung ist beendet. Ab jetzt steuerst du deine Flotte ausschließlich per "
                        strong { style: "color: #00e5ff;", "Rust-Code im Zed-Editor" }
                        " über die SpacetimeDB-API!"
                    }

                    // Token box
                    div {
                        style: "background: #0f172a; border: 1px solid #334155; border-radius: 8px; padding: 16px; margin-bottom: 20px;",
                        label { style: "display: block; font-size: 12px; font-weight: 600; color: #8da2c0; margin-bottom: 6px;", "DEIN PERSÖNLICHER IDENTITÄTS-TOKEN:" }
                        div {
                            style: "display: flex; gap: 10px;",
                            input {
                                readonly: true,
                                style: "flex: 1; background: #020617; border: 1px solid #1e293b; color: #38bdf8; padding: 8px 12px; border-radius: 6px; font-family: monospace; font-size: 13px;",
                                value: "{token}",
                            }
                            button {
                                style: "background: #0284c7; hover:background: #0369a1; color: #fff; border: none; padding: 8px 16px; border-radius: 6px; font-weight: 600; font-size: 13px; cursor: pointer;",
                                onclick: copy_token,
                                if copied() { "✔ Kopiert!" } else { "📋 Kopieren" }
                            }
                        }
                    }

                    // Instructions for Zed
                    div {
                        style: "font-size: 14px; color: #cbd5e1; line-height: 1.5;",
                        h3 { style: "margin: 0 0 8px 0; font-size: 15px; color: #00e5ff;", "Nächste Schritte in Zed (uhlaunch):" }
                        ol {
                            style: "margin: 0; padding-left: 20px; display: flex; flex-direction: column; gap: 6px;",
                            li { "Öffne das Projekt über den Launcher (" strong { "uhlaunch" } ") in Zed." }
                            li { "Füge deinen kopierten Token in die " code { style: "background: #1e293b; padding: 2px 6px; border-radius: 4px;", "config.toml" } " ein." }
                            li { "Schreibe deine Entscheidungslogik in " code { style: "background: #1e293b; padding: 2px 6px; border-radius: 4px;", "src/ai.rs" } "." }
                            li { "Starte deinen Bot im Terminal mit: " code { style: "background: #1e293b; color: #00e676; padding: 2px 6px; border-radius: 4px;", "cargo run" } }
                        }
                    }
                }
            }

            // Right Column: Station Stats & Fleet List
            div {
                style: "width: 420px; display: flex; flex-direction: column; gap: 20px;",

                // Base Station Status
                div {
                    style: "background: rgba(15, 23, 42, 0.95); border: 1px solid rgba(80, 120, 200, 0.25); border-radius: 12px; padding: 24px;",
                    h3 { style: "margin: 0 0 16px 0; font-size: 16px; font-weight: 700; color: #00e5ff;", "🏠 Stationsstatus" }
                    div {
                        style: "display: grid; grid-template-columns: 1fr 1fr; gap: 14px;",
                        div {
                            style: "padding: 12px; background: rgba(255, 255, 255, 0.03); border-radius: 6px;",
                            div { style: "font-size: 11px; color: #8da2c0;", "GELAGERTES ERZ" }
                            div { style: "font-size: 20px; font-weight: 700; color: #ffd700;", "{base.stored_ore}" }
                        }
                        div {
                            style: "padding: 12px; background: rgba(255, 255, 255, 0.03); border-radius: 6px;",
                            div { style: "font-size: 11px; color: #8da2c0;", "ENERGIE" }
                            div { style: "font-size: 20px; font-weight: 700; color: #00e5ff;", "{base.stored_energy}" }
                        }
                        div {
                            style: "padding: 12px; background: rgba(255, 255, 255, 0.03); border-radius: 6px;",
                            div { style: "font-size: 11px; color: #8da2c0;", "FUNK-REICHWEITE" }
                            div { style: "font-size: 20px; font-weight: 700; color: #38bdf8;", "{base.signal_radius as u32} px" }
                        }
                        div {
                            style: "padding: 12px; background: rgba(255, 255, 255, 0.03); border-radius: 6px;",
                            div { style: "font-size: 11px; color: #8da2c0;", "GESAMTPUNKTE" }
                            div { style: "font-size: 20px; font-weight: 700; color: #00e676;", "{player.score}" }
                        }
                    }
                }

                // Active Ships Fleet
                div {
                    style: "background: rgba(15, 23, 42, 0.95); border: 1px solid rgba(80, 120, 200, 0.25); border-radius: 12px; padding: 24px; flex: 1;",
                    h3 { style: "margin: 0 0 16px 0; font-size: 16px; font-weight: 700; color: #00e5ff;", "🚀 Deine Flotte ({ships.len()} Schiffe)" }
                    div {
                        style: "display: flex; flex-direction: column; gap: 10px;",
                        for s in ships {
                            div {
                                key: "{s.ship_id}",
                                style: "padding: 12px; background: rgba(255, 255, 255, 0.03); border-radius: 6px; border-left: 3px solid #00e5ff;",
                                div {
                                    style: "display: flex; justify-content: space-between; margin-bottom: 4px;",
                                    span { style: "font-weight: 700;", "Drohne #{s.ship_id} (Tier {s.ship_tier})" }
                                    span {
                                        style: if s.is_stranded { "color: #ef4444; font-weight: 700;" } else { "color: #00e676; font-size: 12px;" },
                                        if s.is_stranded { "GESTRANDET" } else { "BEREIT (API)" }
                                    }
                                }
                                div {
                                    style: "font-size: 12px; color: #94a3b8; display: flex; justify-content: space-between;",
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
