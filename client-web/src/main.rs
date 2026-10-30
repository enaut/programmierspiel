pub mod audio;
pub mod module_bindings;
pub mod oauth;
pub mod radar;
pub mod tutorial;

use ::dioxus::prelude::*;
use ::dioxus::signals::SyncSignal;
use module_bindings::advance_tutorial_step_reducer::advance_tutorial_step;
use module_bindings::build_ship_reducer::build_ship;
use module_bindings::command_ship_reducer::command_ship;
use module_bindings::deploy_base_station_reducer::deploy_base_station;
use module_bindings::dioxus::{
    ConnectionState, use_spacetimedb_context, use_spacetimedb_context_provider, use_subscription,
};
use module_bindings::register_ai_agent_reducer::register_ai_agent;
use module_bindings::register_player_reducer::register_player;
use module_bindings::revoke_ai_agent_reducer::revoke_ai_agent;
use module_bindings::update_player_colors_reducer::update_player_colors;
use module_bindings::{
    BaseStation, PerceivedAsteroid, PerceivedBase, PerceivedDrone, Player, PlayerAchievement,
    PublicRiddle, Ship, ShipCommand, Vec2,
};
use radar::{SelectedEntity, render_drone1_preview, render_player_radar};
use spacetimedb_sdk::Identity;
use tutorial::QuestDrawer;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use web_sys::HtmlCanvasElement;

fn distance(a: &Vec2, b: &Vec2) -> f32 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    (dx * dx + dy * dy).sqrt()
}

#[derive(Debug, Clone, Routable, PartialEq)]
pub enum Route {
    #[route("/")]
    CockpitApp {},
    #[route("/callback?:code&:state&:error&:error_description")]
    OAuthCallback {
        code: Option<String>,
        state: Option<String>,
        error: Option<String>,
        error_description: Option<String>,
    },
    #[route("/:..route")]
    PageNotFound { route: Vec<String> },
}

fn main() {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(w) = web_sys::window() {
            if let Ok(search) = w.location().search() {
                if !search.is_empty() && (search.contains("code=") || search.contains("error=")) {
                    if let Ok(Some(session)) = w.session_storage() {
                        let _ = session.set_item("pending_oauth_search", &search);
                    }
                }
            }
        }
    }
    ::dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        Router::<Route> {}
    }
}

#[component]
fn PageNotFound(route: Vec<String>) -> Element {
    let nav = use_navigator();
    use_effect(move || {
        nav.replace(Route::CockpitApp {});
    });
    rsx! {
        div {
            style: "width: 100vw; height: 100vh; background: #060b18; color: #94a3b8; display: flex; align-items: center; justify-content: center;",
            "Weiterleitung zum Cockpit..."
        }
    }
}

#[component]
fn GiteaLoginScreen() -> Element {
    let mut login_error = use_signal(|| None::<String>);
    let mut is_redirecting = use_signal(|| false);

    let on_login = move |_| {
        is_redirecting.set(true);
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(w) = web_sys::window() {
                if let Ok(hostname) = w.location().hostname() {
                    if hostname == "localhost" {
                        let port = w.location().port().unwrap_or_default();
                        let target = if port.is_empty() {
                            "http://127.0.0.1:8081/".to_string()
                        } else {
                            format!("http://127.0.0.1:{}/", port)
                        };
                        let _ = w.location().set_href(&target);
                        return;
                    }
                }

                if let Ok(Some(session)) = w.session_storage() {
                    let verifier = oauth::generate_code_verifier();
                    let challenge = oauth::generate_code_challenge(&verifier);
                    let state = oauth::generate_state();

                    let _ = session.set_item(oauth::KEY_PKCE_VERIFIER, &verifier);
                    let _ = session.set_item(oauth::KEY_OAUTH_STATE, &state);

                    let redirect_uri = format!("{}/callback", oauth::get_current_origin());
                    let auth_url =
                        oauth::build_authorization_url(&redirect_uri, &state, &challenge);

                    if let Err(e) = w.location().set_href(&auth_url) {
                        is_redirecting.set(false);
                        login_error.set(Some(format!("Weiterleitungsfehler: {e:?}")));
                    }
                    return;
                }
            }
            is_redirecting.set(false);
            login_error.set(Some("Browser-Speicher nicht verfügbar.".to_string()));
        }
    };

    rsx! {
        div {
            style: "width: 100vw; height: 100vh; margin: 0; padding: 0; background-color: #060b18; color: #f0f4f8; font-family: system-ui, -apple-system, sans-serif; display: flex; align-items: center; justify-content: center; position: relative; overflow: hidden;",
            div {
                style: "position: absolute; width: 600px; height: 600px; border-radius: 50%; background: radial-gradient(circle, rgba(0, 229, 255, 0.08) 0%, rgba(59, 130, 246, 0.03) 50%, transparent 70%); pointer-events: none; filter: blur(40px);"
            }

            div {
                style: "position: relative; z-index: 2; max-width: 480px; width: 90%; background: rgba(15, 23, 42, 0.95); border: 1px solid rgba(80, 120, 200, 0.35); border-radius: 16px; padding: 40px; box-shadow: 0 20px 50px rgba(0,0,0,0.7); text-align: center; backdrop-filter: blur(12px);",

                div {
                    style: "width: 64px; height: 64px; border-radius: 16px; background: rgba(0, 229, 255, 0.1); border: 1px solid rgba(0, 229, 255, 0.3); display: flex; align-items: center; justify-content: center; font-size: 32px; margin: 0 auto 20px auto; box-shadow: 0 0 20px rgba(0, 229, 255, 0.2);",
                    "🛸"
                }

                div {
                    style: "display: inline-block; padding: 3px 12px; border-radius: 20px; background: rgba(0, 229, 255, 0.12); color: #00e5ff; font-size: 11px; font-weight: 700; letter-spacing: 1px; margin-bottom: 12px; border: 1px solid rgba(0, 229, 255, 0.25);",
                    "PROGRAMMIERSPIEL • PILOTEN-LOGIN"
                }

                h1 {
                    style: "margin: 0 0 10px 0; font-size: 24px; font-weight: 800; color: #f8fafc; letter-spacing: -0.5px;",
                    "Willkommen auf der Megastation"
                }

                p {
                    style: "margin: 0 0 28px 0; font-size: 14px; line-height: 1.6; color: #94a3b8;",
                    "Melde dich mit deinem Schul-Account auf "
                    span { style: "color: #38bdf8; font-weight: 600;", "git.uhle.cloud" }
                    " an, um dein Raumschiff freizuschalten und deine Heimatbasis zu errichten."
                }

                if let Some(ref err) = login_error() {
                    div {
                        style: "background: rgba(239, 68, 68, 0.15); border: 1px solid rgba(239, 68, 68, 0.4); border-radius: 8px; padding: 10px 14px; margin-bottom: 20px; font-size: 12px; color: #fca5a5; text-align: left;",
                        "⚠️ {err}"
                    }
                }

                button {
                    style: "width: 100%; display: flex; align-items: center; justify-content: center; gap: 10px; background: linear-gradient(135deg, #0284c7 0%, #0369a1 100%); color: #ffffff; border: none; padding: 14px 20px; border-radius: 10px; font-size: 15px; font-weight: 700; cursor: pointer; transition: all 0.2s; box-shadow: 0 4px 16px rgba(2, 132, 199, 0.4);",
                    disabled: is_redirecting(),
                    onclick: on_login,
                    if is_redirecting() {
                        span { "Verbinde mit Gitea..." }
                    } else {
                        span { style: "font-size: 18px;", "🔑" }
                        span { "Mit Gitea anmelden" }
                    }
                }

                div {
                    style: "margin-top: 24px; padding-top: 16px; border-top: 1px solid rgba(255, 255, 255, 0.08); font-size: 11px; color: #64748b; line-height: 1.5;",
                    "OAuth2 & OpenID Connect Authentifizierung über "
                    code { style: "color: #94a3b8;", "https://git.uhle.cloud" }
                }
            }
        }
    }
}

#[component]
fn OAuthCallback(
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
) -> Element {
    let mut status_msg = use_signal(|| "Authentifiziere mit Gitea...".to_string());
    let mut err_msg = use_signal(|| None::<String>);

    use_hook(|| {
        let code_prop = code.clone();
        let state_prop = state.clone();
        let err_prop = error.clone();
        let desc_prop = error_description.clone();

        spawn(async move {
            #[cfg(target_arch = "wasm32")]
            {
                let window = match web_sys::window() {
                    Some(w) => w,
                    None => {
                        err_msg.set(Some("Kein Browserfenster gefunden.".to_string()));
                        return;
                    }
                };

                let session = window.session_storage().ok().flatten();
                let backup_search = session
                    .as_ref()
                    .and_then(|s| s.get_item("pending_oauth_search").ok().flatten())
                    .unwrap_or_default();
                let window_search = window.location().search().unwrap_or_default();

                let combined_search = if !window_search.is_empty() {
                    window_search
                } else {
                    backup_search
                };

                let url_params = web_sys::UrlSearchParams::new_with_str(&combined_search).ok();

                let gitea_err =
                    err_prop.or_else(|| url_params.as_ref().and_then(|p| p.get("error")));
                if let Some(err) = gitea_err {
                    let desc = desc_prop
                        .or_else(|| url_params.as_ref().and_then(|p| p.get("error_description")))
                        .unwrap_or_default();
                    err_msg.set(Some(format!("OAuth-Fehler von Gitea: {err} - {desc}")));
                    return;
                }

                let auth_code = code_prop
                    .filter(|c| !c.is_empty())
                    .or_else(|| url_params.as_ref().and_then(|p| p.get("code")))
                    .filter(|c| !c.is_empty());

                let code = match auth_code {
                    Some(c) => c,
                    None => {
                        err_msg.set(Some(format!(
                            "Kein Autorisierungscode in URL gefunden. (search='{}')",
                            combined_search
                        )));
                        return;
                    }
                };

                let auth_state = state_prop
                    .or_else(|| url_params.as_ref().and_then(|p| p.get("state")))
                    .unwrap_or_default();

                let session = match session {
                    Some(s) => s,
                    None => {
                        err_msg.set(Some("SessionStorage nicht verfügbar.".to_string()));
                        return;
                    }
                };

                let saved_state = session.get_item(oauth::KEY_OAUTH_STATE).ok().flatten();
                if let Some(ref expected) = saved_state {
                    if expected != &auth_state {
                        err_msg.set(Some(
                            "CSRF-Schutz: State-Parameter stimmt nicht überein.".to_string(),
                        ));
                        return;
                    }
                }

                let verifier = session
                    .get_item(oauth::KEY_PKCE_VERIFIER)
                    .ok()
                    .flatten()
                    .unwrap_or_default();

                let redirect_uri = format!("{}/callback", oauth::get_current_origin());

                status_msg.set("Tausche Autorisierungscode gegen Token aus...".to_string());

                match oauth::exchange_code(&code, &verifier, &redirect_uri).await {
                    Ok(tokens) => {
                        status_msg.set("Anmeldung erfolgreich! Lade Cockpit...".to_string());
                        oauth::save_session(&tokens);

                        // Clear temporary PKCE and search keys
                        let _ = session.remove_item(oauth::KEY_PKCE_VERIFIER);
                        let _ = session.remove_item(oauth::KEY_OAUTH_STATE);
                        let _ = session.remove_item("pending_oauth_search");

                        // Clean reload to main page so SpacetimeDB initializes with the saved token
                        let _ = window.location().set_href("/");
                    }
                    Err(e) => {
                        err_msg.set(Some(format!("Token-Austausch fehlgeschlagen: {e}")));
                    }
                }
            }
        });
    });

    let on_retry = move |_| {
        oauth::clear_session();
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(w) = web_sys::window() {
                if let Ok(Some(session)) = w.session_storage() {
                    let _ = session.remove_item("pending_oauth_search");
                }
                let _ = w.location().set_href("/");
            }
        }
    };

    rsx! {
        div {
            style: "width: 100vw; height: 100vh; background-color: #060b18; color: #f0f4f8; font-family: system-ui, -apple-system, sans-serif; display: flex; align-items: center; justify-content: center;",
            div {
                style: "max-width: 440px; width: 90%; background: #0f172a; border: 1px solid #334155; border-radius: 12px; padding: 32px; text-align: center; box-shadow: 0 16px 40px rgba(0,0,0,0.6);",
                if let Some(ref err) = err_msg() {
                    div {
                        div { style: "font-size: 32px; margin-bottom: 12px;", "⚠️" }
                        h2 { style: "margin: 0 0 12px 0; font-size: 18px; color: #f87171;", "Anmeldung fehlgeschlagen" }
                        p { style: "font-size: 13px; color: #94a3b8; line-height: 1.5; margin-bottom: 24px;", "{err}" }
                        button {
                            style: "background: #ef4444; color: #fff; border: none; padding: 10px 20px; border-radius: 6px; font-weight: 700; cursor: pointer;",
                            onclick: on_retry,
                            "Zurück zum Login"
                        }
                    }
                } else {
                    div {
                        div { style: "font-size: 32px; margin-bottom: 12px; animation: spin 2s linear infinite;", "⏳" }
                        h2 { style: "margin: 0 0 8px 0; font-size: 18px; color: #38bdf8;", "Gitea Authentifizierung" }
                        p { style: "font-size: 13px; color: #94a3b8;", "{status_msg()}" }
                    }
                }
            }
        }
    }
}

#[component]
fn CockpitApp() -> Element {
    // 1. Read token from localStorage if available
    let saved_token = oauth::get_saved_token();

    if saved_token.is_none() {
        return rsx! {
            GiteaLoginScreen {}
        };
    }

    // 2. Resolve SpacetimeDB HTTP endpoint.
    // NOTE: In browser WASM, spacetimedb-sdk exchanges user token for a websocket token
    // via POST {host}v1/identity/websocket-token. Therefore, the URI MUST start with http:// or https://
    // and MUST end with a trailing slash so {host}v1 forms a valid URL!
    // make_uri in spacetimedb-sdk will automatically convert http:// to ws:// (or https:// to wss://)
    // for the subsequent WebSocket connection.
    let stdb_url = {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(w) = web_sys::window() {
                let hostname = w.location().hostname().unwrap_or_default();
                let protocol = w
                    .location()
                    .protocol()
                    .unwrap_or_else(|_| "http:".to_string());
                let scheme = if protocol.starts_with("https") {
                    "https"
                } else {
                    "http"
                };
                let host = if hostname.is_empty() || hostname == "localhost" {
                    "127.0.0.1"
                } else {
                    &hostname
                };
                format!("{scheme}://{host}:3000/")
            } else {
                "http://127.0.0.1:3000/".to_string()
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            "http://127.0.0.1:3000/".to_string()
        }
    };

    let _stdb_ctx = use_spacetimedb_context_provider(&stdb_url, "server", saved_token);

    // 3. Subscribe to table queries so client receives live rows & updates
    use_subscription(&[
        "SELECT * FROM config",
        "SELECT * FROM player",
        "SELECT * FROM player_achievement",
        "SELECT * FROM my_base",
        "SELECT * FROM my_ships",
        "SELECT * FROM visible_bases",
        "SELECT * FROM visible_drones",
        "SELECT * FROM visible_asteroids",
        "SELECT * FROM unidentified_objects",
        "SELECT * FROM game_event",
        "SELECT * FROM my_active_riddle",
        "SELECT * FROM my_ai_agents",
    ]);

    let ctx = use_spacetimedb_context();
    let state = (ctx.state)();
    let conn_error = (ctx.error)();

    // Log connection status and error changes to browser console for immediate visibility
    use_effect(move || {
        let current_state = (ctx.state)();
        let err = (ctx.error)();
        #[cfg(target_arch = "wasm32")]
        {
            web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&format!(
                "[SpacetimeDB] ConnectionState: {:?}, Error: {:?}",
                current_state, err
            )));
        }
    });

    // Persist token upon connection only if no session token already exists in storage
    use_effect(move || {
        #[cfg(target_arch = "wasm32")]
        {
            let current_state = (ctx.state)();
            if let ConnectionState::Connected(_, ref token) = current_state {
                if let Some(w) = web_sys::window() {
                    if let Ok(Some(storage)) = w.local_storage() {
                        if storage
                            .get_item("spacetimedb_token")
                            .ok()
                            .flatten()
                            .is_none()
                        {
                            let _ = storage.set_item("spacetimedb_token", token);
                        }
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
                    oauth::clear_session();
                    let _ = w.location().set_href("/");
                }
            }
        }
    });

    // Background token expiration check and silent renewal
    use_hook(|| {
        spawn(async move {
            #[cfg(target_arch = "wasm32")]
            loop {
                // Wait 60 seconds between checks
                let promise = js_sys::Promise::new(&mut |resolve, _| {
                    if let Some(w) = web_sys::window() {
                        let _ = w.set_timeout_with_callback_and_timeout_and_arguments_0(
                            &resolve, 60_000,
                        );
                    }
                });
                let _ = wasm_bindgen_futures::JsFuture::from(promise).await;

                if let Some(id_tok) = oauth::get_saved_token() {
                    if let Some(claims) = oauth::parse_id_token_claims(&id_tok) {
                        if let Some(exp) = claims.exp {
                            let now_secs = (js_sys::Date::now() / 1000.0) as u64;
                            if exp.saturating_sub(now_secs) < 300 {
                                if let Some(rt) = oauth::get_saved_refresh_token() {
                                    if let Ok(new_tokens) = oauth::refresh_access_token(&rt).await {
                                        oauth::save_session(&new_tokens);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });
    });

    let my_identity = match state {
        ConnectionState::Connected(id, _) => Some(id),
        _ => None,
    };

    let token = match state {
        ConnectionState::Connected(_, ref t) => t.clone(),
        _ => "".to_string(),
    };

    let players = (ctx.tables.player)();
    let my_bases = (ctx.tables.my_base)();
    let my_ships_list = (ctx.tables.my_ships)();
    let achievements = (ctx.tables.player_achievement)();
    let active_riddles = (ctx.tables.my_active_riddle)();
    let current_riddle = active_riddles.first().cloned();

    let mut show_scoreboard = use_signal(|| false);
    let mut show_heimatbasis_modal = use_signal(|| false);
    let is_tutorial_collapsed = use_signal(|| false);

    let my_player = my_identity.and_then(|id| players.iter().find(|p| p.identity == id).cloned());
    let my_base = my_bases.first().cloned();
    let my_ships: Vec<Ship> = my_ships_list;

    let my_unlocked_count = my_identity
        .map(|id| achievements.iter().filter(|a| a.player == id).count())
        .unwrap_or(0);

    let reg_err_msg = use_signal_sync(|| None::<String>);
    let reg_is_submitting = use_signal_sync(|| false);

    let on_logout = move |_| {
        oauth::clear_session();
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(w) = web_sys::window() {
                let _ = w.location().set_href("/");
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
                    if let Some(ref base) = my_base {
                        button {
                            style: "background: rgba(0, 229, 255, 0.1); hover:background: rgba(0, 229, 255, 0.2); border: 1px solid rgba(0, 229, 255, 0.35); color: #00e5ff; padding: 4px 10px; border-radius: 6px; font-size: 11px; font-weight: 700; cursor: pointer; transition: all 0.2s; display: flex; align-items: center; gap: 8px;",
                            onclick: move |_| show_heimatbasis_modal.set(!show_heimatbasis_modal()),
                            title: "Heimatbasis & Sensor-Forschungszentrum öffnen",
                            span { style: "color: #ffd700;", "⛏️ {base.stored_ore} Erz" }
                            span { style: "color: #64748b;", "|" }
                            span { style: "color: #00e5ff;", "⚡ {base.stored_energy} Energie" }
                        }
                    } else {
                        span {
                            style: "font-size: 11px; padding: 2px 8px; border-radius: 4px; background: rgba(0, 229, 255, 0.1); color: #00e5ff; border: 1px solid rgba(0, 229, 255, 0.25);",
                            "MEGA-STATION"
                        }
                    }
                }
                div {
                    style: "display: flex; align-items: center; gap: 14px; font-size: 13px;",
                    if let Some(ref p) = my_player {
                        div {
                            style: "display: flex; align-items: center; gap: 8px; background: rgba(255, 255, 255, 0.05); padding: 4px 12px; border-radius: 20px; border: 1px solid rgba(255, 255, 255, 0.1);",
                            span { style: "width: 10px; height: 10px; border-radius: 50%; background: linear-gradient(135deg, hsl({p.color_hue}, 90%, 55%) 50%, hsl({p.secondary_color_hue}, 90%, 55%) 50%); box-shadow: 0 0 6px hsl({p.color_hue}, 90%, 55%);" }
                            span { style: "font-weight: 600; color: #f1f5f9;", "{p.name}" }
                            span { style: "color: #ffd700; font-size: 12px; font-weight: 700;", "({p.score} Pkt)" }
                        }
                    }
                    if let Some(g_user) = oauth::get_saved_username() {
                        div {
                            style: "display: flex; align-items: center; gap: 6px; font-size: 12px; color: #38bdf8; background: rgba(56, 189, 248, 0.1); padding: 4px 10px; border-radius: 6px; border: 1px solid rgba(56, 189, 248, 0.25);",
                            span { "🔑" }
                            span { style: "font-weight: 600;", "{g_user}" }
                        }
                    }
                    button {
                        style: "background: rgba(250, 204, 21, 0.15); border: 1px solid rgba(250, 204, 21, 0.4); color: #fde047; padding: 4px 12px; border-radius: 6px; font-size: 11px; font-weight: 700; cursor: pointer; transition: all 0.2s; display: flex; align-items: center; gap: 6px;",
                        onclick: move |_| show_scoreboard.set(!show_scoreboard()),
                        span { "🏆" }
                        span { "Scoreboard ({my_unlocked_count}/5)" }
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
                        title: "Meldet dich von Gitea ab und löscht die Sitzung",
                        "🚪 Abmelden"
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
                                ConnectionState::Connecting => {
                                    if let Some(ref err) = conn_error {
                                        format!("Verbinde mit SpacetimeDB... [{err}]")
                                    } else {
                                        format!("Verbinde mit SpacetimeDB-Server ({stdb_url})...")
                                    }
                                }
                                ConnectionState::Reconnecting { attempt, delay_ms } => {
                                    if let Some(ref err) = conn_error {
                                        format!("Verbindung wird wiederholt (Versuch #{attempt}, in {delay_ms}ms): {err}")
                                    } else {
                                        format!("Verbindungsaufbau wird wiederholt (Versuch #{attempt}, in {delay_ms}ms)...")
                                    }
                                }
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
                        player: my_player.clone().unwrap(),
                        builder_ship: my_ships.iter().find(|s| s.ship_tier == 0).cloned(),
                        is_tutorial_collapsed,
                    }
                } else {
                    BaseDashboardScreen {
                        player: my_player.clone().unwrap(),
                        base: my_base.clone().unwrap(),
                        ships: my_ships.clone(),
                        achievements: achievements.clone(),
                        is_tutorial_collapsed,
                        on_open_heimatbasis: move |_| show_heimatbasis_modal.set(true),
                    }
                }

                // Interactive Tutorial Quest-Drawer
                if let Some(ref p) = my_player {
                    QuestDrawer {
                        step: p.tutorial_step,
                        is_collapsed: is_tutorial_collapsed,
                        events: (ctx.tables.game_event)().into_iter().filter(|e| Some(e.player) == my_identity).collect(),
                        ships: my_ships,
                        on_advance: move |target_step| {
                            if let Some(conn) = (ctx.connection)() {
                                let _ = conn.reducers.advance_tutorial_step(target_step);
                            }
                        },
                    }
                }
            }

            // Scoreboard & Achievements Modal Overlay
            if show_scoreboard() {
                ScoreboardModal {
                    my_identity,
                    players: players.clone(),
                    achievements: achievements.clone(),
                    on_close: move |_| show_scoreboard.set(false),
                }
            }

            // Heimatbasis & Sensor-Forschungszentrum Modal Overlay
            if show_heimatbasis_modal() {
                if let Some(ref b) = my_base {
                    if let Some(ref p) = my_player {
                        HeimatbasisModal {
                            player: p.clone(),
                            base: b.clone(),
                            token: token.clone(),
                            stdb_url: stdb_url.clone(),
                            current_riddle: current_riddle.clone(),
                            on_close: move |_| show_heimatbasis_modal.set(false),
                        }
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
    let gitea_user = oauth::get_saved_username();
    let pilot_name = gitea_user
        .clone()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "Pilot".to_string());
    let mut color_hue = use_signal(|| 180u16);
    let mut secondary_color_hue = use_signal(|| 0u16);

    // Render tinted drone1 sprite onto preview canvas
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
                if let Some(w) = web_sys::window() {
                    if let Some(doc) = w.document() {
                        if let Some(canvas_el) = doc.get_element_by_id("ship-preview-canvas") {
                            if let Ok(canvas) = canvas_el.dyn_into::<HtmlCanvasElement>() {
                                let css_w = 160.0;
                                let css_h = 120.0;
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
                                        render_drone1_preview(
                                            &ctx_2d,
                                            css_w,
                                            css_h,
                                            dpr,
                                            *color_hue.read(),
                                            *secondary_color_hue.read(),
                                        );
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

    let mut submit_registration = move || {
        let name = pilot_name.trim().to_string();

        if let Some(conn) = (ctx.connection)() {
            is_submitting.set(true);
            let hue = color_hue();
            let sec_hue = secondary_color_hue();
            let res =
                conn.reducers
                    .register_player_then(name, hue, sec_hue, move |_ctx, result| {
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

                        // Ship visual preview card using actual drone1 sprite
                        div {
                            style: "background: #090e1a; border: 1px solid rgba(80, 120, 200, 0.25); border-radius: 10px; padding: 20px; text-align: center; margin-bottom: 20px; position: relative;",
                            canvas {
                                id: "ship-preview-canvas",
                                style: "width: 160px; height: 120px; margin: 0 auto; display: block; filter: drop-shadow(0 0 12px hsl({color_hue}, 90%, 50%)) drop-shadow(0 0 6px hsl({secondary_color_hue}, 90%, 50%));",
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
                            div { style: "display: flex; justify-content: space-between;", span { "Start-Sektor:" } span { style: "color: #cbd5e1; font-weight: 600;", "Mega-Station (0, 0)" } }
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
                        "Wähle deine Team-Farben, um deinen Pilotenausweis zu erstellen und die Startfreigabe aus dem Hangar zu erhalten."
                    }

                    if let Some(ref gu) = gitea_user {
                        div {
                            style: "display: inline-flex; align-items: center; gap: 8px; padding: 8px 12px; border-radius: 6px; background: rgba(14, 165, 233, 0.12); border: 1px solid rgba(14, 165, 233, 0.3); font-size: 13px; color: #38bdf8; margin-bottom: 24px;",
                            span { "🔑" }
                            span { "Pilot (Gitea-Account): " strong { style: "color: #f1f5f9;", "{gu}" } }
                        }
                    }

                    // Primary Color picker
                    div {
                        style: "margin-bottom: 16px;",
                        label { style: "display: block; font-size: 13px; font-weight: 600; color: #cbd5e1; margin-bottom: 8px;", "Primärfarbe (Rumpf & Kern):" }
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
                    }

                    // Secondary Color picker
                    div {
                        style: "margin-bottom: 24px;",
                        label { style: "display: block; font-size: 13px; font-weight: 600; color: #cbd5e1; margin-bottom: 8px;", "Sekundärfarbe (Flügel & Akzente):" }
                        div {
                            style: "display: flex; align-items: center; gap: 14px;",
                            input {
                                r#type: "range",
                                min: "0",
                                max: "359",
                                value: "{secondary_color_hue}",
                                style: "flex: 1; cursor: pointer; accent-color: hsl({secondary_color_hue}, 90%, 55%); height: 8px;",
                                oninput: move |e| {
                                    if let Ok(val) = e.value().parse::<u16>() {
                                        secondary_color_hue.set(val);
                                    }
                                }
                            }
                            span {
                                style: "width: 36px; height: 36px; border-radius: 8px; background: hsl({secondary_color_hue}, 90%, 55%); border: 2px solid #ffffff; box-shadow: 0 0 12px hsl({secondary_color_hue}, 90%, 55%); flex-shrink: 0;"
                            }
                        }
                        div {
                            style: "font-size: 11px; color: #64748b; margin-top: 6px;",
                            "Deine Heimatbasis und alle Drohnen werden in diesem Farbschema markiert."
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
fn BuilderNavigationScreen(
    player: Player,
    builder_ship: Option<Ship>,
    is_tutorial_collapsed: Signal<bool>,
) -> Element {
    let ctx = use_spacetimedb_context();
    let configs = (ctx.tables.config)();
    let config = configs.first().cloned();

    let mut target_marker = use_signal(|| None::<Vec2>);
    let mut cam_offset_x = use_signal(|| 0.0f64);
    let mut cam_offset_y = use_signal(|| 0.0f64);
    let mut zoom = use_signal(|| 6.0f64);
    let mut is_dragging = use_signal(|| false);
    let mut drag_start = use_signal(|| (0.0f64, 0.0f64));
    let mut drag_distance = use_signal(|| 0.0f64);

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

    let mega_pos = config
        .as_ref()
        .map_or(Vec2 { x: 0.0, y: 0.0 }, |c| c.megastation_pos.clone());
    let dist_to_megastation = distance(&ship.position, &mega_pos);
    let visible_asteroids_list = (ctx.tables.visible_asteroids)();
    let asteroids_in_scan: Vec<_> = visible_asteroids_list
        .iter()
        .filter(|a| distance(&a.position, &ship.position) <= 300.0 && a.remaining_ore > 0)
        .collect();
    let scan_asteroid_count = asteroids_in_scan.len();
    let scan_ore_total: u32 = asteroids_in_scan.iter().map(|a| a.remaining_ore).sum();
    let has_enough_distance = dist_to_megastation >= 250.0;
    let has_enough_ore = scan_ore_total >= 400;
    let can_deploy = has_enough_distance && has_enough_ore;

    let p_identity = player.identity;
    let p_hue = player.color_hue;
    let p_sec_hue = player.secondary_color_hue;

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

    // Continuously render radar canvas via request_animation_frame with interpolation
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
                                            let players = (ctx.tables.player)();
                                            let my_ships = (ctx.tables.my_ships)();
                                            let visible_drones = (ctx.tables.visible_drones)();
                                            let visible_bases = (ctx.tables.visible_bases)();
                                            let visible_asteroids =
                                                (ctx.tables.visible_asteroids)();
                                            let unidentified_objects =
                                                (ctx.tables.unidentified_objects)();

                                            let current_ship = my_ships.into_iter().find(|s| {
                                                s.owner == p_identity && s.ship_tier == 0
                                            });

                                            if let Some(ref s) = current_ship {
                                                let ships_slice = [s.clone()];
                                                let time = js_sys::Date::now() / 1000.0;
                                                let cam_x =
                                                    s.position.x as f64 + *cam_offset_x.read();
                                                let cam_y =
                                                    s.position.y as f64 + *cam_offset_y.read();
                                                let cur_zoom = *zoom.read();

                                                render_player_radar(
                                                    &ctx_2d,
                                                    css_w,
                                                    css_h,
                                                    dpr,
                                                    cam_x,
                                                    cam_y,
                                                    cur_zoom,
                                                    s.position.clone(),
                                                    p_hue,
                                                    p_sec_hue,
                                                    configs.first(),
                                                    &ships_slice,
                                                    None,
                                                    &visible_drones,
                                                    &visible_bases,
                                                    &visible_asteroids,
                                                    &unidentified_objects,
                                                    &players,
                                                    target_marker.read().clone(),
                                                    None,
                                                    time,
                                                );
                                            }
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

    let on_deploy = move |_| {
        if can_deploy {
            if let Some(conn) = (ctx.connection)() {
                let _ = conn.reducers.deploy_base_station(ship.ship_id);
            }
        }
    };

    let ship_pos_x = ship.position.x as f64;
    let ship_pos_y = ship.position.y as f64;
    let current_cam_x = ship_pos_x + *cam_offset_x.read();
    let current_cam_y = ship_pos_y + *cam_offset_y.read();
    let current_zoom = *zoom.read();

    rsx! {
        div {
            style: "flex: 1; display: flex; flex-direction: row; height: 100%; overflow: hidden;",

            // Radar Canvas Area (Left)
            div {
                style: if *is_dragging.read() {
                    "flex: 1; height: 100%; position: relative; background: #050a16; display: flex; align-items: center; justify-content: center; overflow: hidden; cursor: grabbing;"
                } else {
                    "flex: 1; height: 100%; position: relative; background: #050a16; display: flex; align-items: center; justify-content: center; overflow: hidden; cursor: crosshair;"
                },
                onmousedown: move |evt| {
                    is_dragging.set(true);
                    drag_start.set((evt.client_coordinates().x, evt.client_coordinates().y));
                    drag_distance.set(0.0);
                },
                onmousemove: move |evt| {
                    if *is_dragging.read() {
                        let (sx, sy) = *drag_start.read();
                        let cx = evt.client_coordinates().x;
                        let cy = evt.client_coordinates().y;
                        let dx = cx - sx;
                        let dy = cy - sy;
                        let cur_dist = *drag_distance.read();
                        drag_distance.set(cur_dist + dx.hypot(dy));
                        drag_start.set((cx, cy));
                        let z = *zoom.read();
                        let cur_x = *cam_offset_x.read();
                        let cur_y = *cam_offset_y.read();
                        cam_offset_x.set(cur_x - dx / z);
                        cam_offset_y.set(cur_y - dy / z);
                    }
                },
                onmouseup: move |evt| {
                    let was_dragging = *is_dragging.read();
                    is_dragging.set(false);
                    if was_dragging && *drag_distance.read() < 6.0 {
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

                                                let rel_x = css_x - (rect.width() / 2.0);
                                                let rel_y = css_y - (rect.height() / 2.0);

                                                let z = *zoom.read();
                                                let target_world = Vec2 {
                                                    x: (ship.position.x as f64 + *cam_offset_x.read() + rel_x / z) as f32,
                                                    y: (ship.position.y as f64 + *cam_offset_y.read() + rel_y / z) as f32,
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
                    }
                },
                onmouseleave: move |_| {
                    is_dragging.set(false);
                },
                onwheel: move |evt| {
                    let delta_y = evt.delta().strip_units().y;
                    let factor = if delta_y > 0.0 { 0.85 } else { 1.15 };
                    let cur_z = *zoom.read();
                    zoom.set((cur_z * factor).clamp(0.02, 10.0));
                },
                canvas {
                    id: "radar-canvas",
                    style: "width: 100%; height: 100%; display: block;",
                }

                // HUD Overlay: Camera Info & Center Controls
                div {
                    style: if is_tutorial_collapsed() {
                        "position: absolute; bottom: 20px; left: 24px; transition: left 0.2s ease; display: flex; gap: 8px; align-items: center; background: rgba(10, 16, 32, 0.9); backdrop-filter: blur(8px); padding: 8px 14px; border-radius: 8px; border: 1px solid rgba(80, 120, 200, 0.3); font-size: 12px; font-family: monospace; z-index: 5; pointer-events: auto;"
                    } else {
                        "position: absolute; bottom: 20px; left: 432px; transition: left 0.2s ease; display: flex; gap: 8px; align-items: center; background: rgba(10, 16, 32, 0.9); backdrop-filter: blur(8px); padding: 8px 14px; border-radius: 8px; border: 1px solid rgba(80, 120, 200, 0.3); font-size: 12px; font-family: monospace; z-index: 5; pointer-events: auto;"
                    },
                    span { style: "color: #8da2c0;", "POS: " }
                    span { style: "color: #00e5ff; font-weight: 700;", "[{current_cam_x.round() as i32}, {current_cam_y.round() as i32}]" }
                    span { style: "color: #8da2c0; margin-left: 6px;", "ZOOM: " }
                    span { style: "color: #ffd700; font-weight: 700;", "{(current_zoom * 100.0).round() as i32}%" }
                    button {
                        style: "background: #1e293b; color: #fff; border: 1px solid #334155; padding: 2px 7px; border-radius: 4px; cursor: pointer;",
                        onclick: move |_| {
                            let cur_z = *zoom.read();
                            zoom.set((cur_z * 1.25).clamp(0.08, 6.0));
                        },
                        "+"
                    }
                    button {
                        style: "background: #1e293b; color: #fff; border: 1px solid #334155; padding: 2px 7px; border-radius: 4px; cursor: pointer;",
                        onclick: move |_| {
                            let cur_z = *zoom.read();
                            zoom.set((cur_z * 0.8).clamp(0.08, 6.0));
                        },
                        "-"
                    }
                    button {
                        style: "background: #0284c7; color: #fff; border: none; padding: 3px 8px; border-radius: 4px; cursor: pointer; font-weight: 600; margin-left: 4px;",
                        onclick: move |_| {
                            cam_offset_x.set(0.0);
                            cam_offset_y.set(0.0);
                            zoom.set(0.85);
                        },
                        "⌂ Schiff"
                    }
                    button {
                        style: "background: #1e293b; color: #38bdf8; border: 1px solid #0284c7; padding: 3px 8px; border-radius: 4px; cursor: pointer; font-weight: 600;",
                        onclick: move |_| {
                            cam_offset_x.set(-ship_pos_x);
                            cam_offset_y.set(-ship_pos_y);
                        },
                        "⌂ Mega-Station"
                    }
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
                        "Fliege aus dem roten Sicherheitskreis ("
                        strong { style: "color: #f1f5f9;", "mind. 250 Einheiten" }
                        " Abstand zur Mega-Station, bis der Kreis grün wird) zu einem Standort mit mindestens "
                        strong { style: "color: #ffd700;", "400 Erz" }
                        " im Asteroiden-Scanradius."
                    }

                    // Distance status card
                    div {
                        style: "padding: 16px; background: rgba(255, 255, 255, 0.04); border-radius: 8px; border: 1px solid rgba(80, 120, 200, 0.15); margin-bottom: 20px;",
                        div { style: "font-size: 11px; font-weight: 600; color: #8da2c0; margin-bottom: 6px; letter-spacing: 0.5px;", "ABSTAND ZUR MEGA-STATION:" }
                        div {
                            style: if has_enough_distance {
                                "font-size: 24px; font-weight: 700; color: #00e676;"
                            } else {
                                "font-size: 24px; font-weight: 700; color: #ff5252;"
                            },
                            "{dist_to_megastation as u32} / 250 Einheiten"
                        }
                        div {
                            style: "margin-top: 8px; font-size: 12px;",
                            if can_deploy {
                                span { style: "color: #00e676; font-weight: 600;", "✔ Bauplatz freigegeben! Heimatbasis kann hier errichtet werden." }
                            } else if !has_enough_distance {
                                span { style: "color: #ff5252;", "✖ Zu nah an der Mega-Station (innerhalb des roten Kreises)." }
                            } else {
                                span { style: "color: #fbbf24;", "⚠️ Außerhalb des Kreises, aber noch zu wenig Erz im Scanradius ({scan_ore_total} / 400 Erz)." }
                            }
                        }
                    }

                    // Ship Stats & Scan Radius Resources
                    div {
                        style: "padding: 16px; background: rgba(255, 255, 255, 0.02); border-radius: 8px; font-size: 13px; color: #94a3b8; display: flex; flex-direction: column; gap: 8px;",
                        div { style: "display: flex; justify-content: space-between;", span { "Position:" } span { style: "color: #cbd5e1; font-mono;", "({ship.position.x as i32}, {ship.position.y as i32})" } }
                        div { style: "display: flex; justify-content: space-between;", span { "Geschwindigkeit:" } span { style: "color: #cbd5e1;", "{ship.speed as i32} / {ship.max_speed as i32} px/s" } }
                        div { style: "display: flex; justify-content: space-between;", span { "Energie:" } span { style: "color: #00e5ff;", "{ship.energy as u32} / {ship.max_energy as u32}" } }
                        div {
                            style: "display: flex; justify-content: space-between; padding-top: 8px; margin-top: 4px; border-top: 1px solid rgba(255, 255, 255, 0.08);",
                            span { "Erz im Scanradius (300m):" }
                            span {
                                style: if has_enough_ore { "color: #00e676; font-weight: 700;" } else { "color: #fbbf24; font-weight: 700;" },
                                "{scan_ore_total} / 400 Erz ({scan_asteroid_count} Ast.)"
                            }
                        }
                        if !has_enough_ore {
                            div {
                                style: "font-size: 11px; color: #fbbf24; line-height: 1.35;",
                                "⚠️ Mindestens 400 Erz im 300m-Scanradius erforderlich, damit deine Drohnen genug Rohstoffe finden."
                            }
                        }
                    }
                }

                // Deploy Button
                div {
                    button {
                        disabled: !can_deploy,
                        style: if can_deploy {
                            if player.tutorial_step == 2 {
                                "width: 100%; background: #059669; hover:background: #047857; color: #fff; border: 2px solid #00e5ff; padding: 14px; border-radius: 8px; font-size: 16px; font-weight: 700; cursor: pointer; box-shadow: 0 0 24px rgba(0, 229, 255, 0.8); transition: all 0.2s;"
                            } else {
                                "width: 100%; background: #059669; hover:background: #047857; color: #fff; border: none; padding: 14px; border-radius: 8px; font-size: 16px; font-weight: 700; cursor: pointer; box-shadow: 0 4px 16px rgba(5, 150, 105, 0.4); transition: all 0.2s;"
                            }
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
                        } else if !has_enough_distance {
                            "Erst außerhalb des Kreises (≥ 250m) & ab 400 Erz im Scanradius aktivierbar"
                        } else {
                            "Noch mindestens {400u32.saturating_sub(scan_ore_total)} Erz im 300m-Scanradius benötigt"
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
fn BaseDashboardScreen(
    player: Player,
    base: BaseStation,
    ships: Vec<Ship>,
    achievements: Vec<PlayerAchievement>,
    is_tutorial_collapsed: Signal<bool>,
    on_open_heimatbasis: EventHandler<()>,
) -> Element {
    let ctx = use_spacetimedb_context();
    let state = (ctx.state)();
    let mut copied = use_signal(|| false);
    let mut selected_target = use_signal(|| None::<SelectedEntity>);
    let mut cam_offset_x = use_signal(|| 0.0f64);
    let mut cam_offset_y = use_signal(|| 0.0f64);
    let mut zoom = use_signal(|| 0.85f64);
    let mut is_dragging = use_signal(|| false);
    let mut drag_start = use_signal(|| (0.0f64, 0.0f64));
    let mut drag_distance = use_signal(|| 0.0f64);

    // Periodic tick to refresh elapsed time reactively (for 30s API active window)
    let mut time_tick = use_signal(|| 0u32);
    use_effect(move || {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(w) = web_sys::window() {
                let closure = wasm_bindgen::closure::Closure::<dyn FnMut()>::new(move || {
                    time_tick += 1;
                });
                let _ = w.set_interval_with_callback_and_timeout_and_arguments_0(
                    closure.as_ref().unchecked_ref(),
                    1000,
                );
                closure.forget();
            }
        }
    });

    let _ = time_tick();
    let _is_api_recently_active = player.last_api_activity.as_ref().map_or(false, |ts| {
        let ts_sec = (ts.to_micros_since_unix_epoch() / 1_000_000) as f64;
        let now_sec = js_sys::Date::now() / 1000.0;
        (now_sec - ts_sec) <= 30.0
    });

    let active_riddles = (ctx.tables.my_active_riddle)();
    let current_riddle = active_riddles.first().cloned();

    let token = match state {
        ConnectionState::Connected(_, ref t) => t.clone(),
        _ => "".to_string(),
    };

    let p_hue = player.color_hue;
    let p_sec_hue = player.secondary_color_hue;
    let base_pos = base.position.clone();
    let cost_t1 = (base.drone_scanner_level.saturating_sub(1) as u32) * 100
        + base.drone_scanner_progress as u32
        + 1;
    let cost_t2 = (base.base_scanner_level.saturating_sub(1) as u32) * 100
        + base.base_scanner_progress as u32
        + 1;
    let cost_t3 = (base.radar_horizon_level.saturating_sub(1) as u32) * 100
        + base.radar_horizon_progress as u32
        + 1;

    let cost_t1_text = if base.drone_scanner_level >= 5 {
        "Maximalstufe erreicht".to_string()
    } else {
        format!("Kosten: {} Erz", cost_t1)
    };
    let cost_t2_text = if base.base_scanner_level >= 5 {
        "Maximalstufe erreicht".to_string()
    } else {
        format!("Kosten: {} Erz", cost_t2)
    };
    let cost_t3_text = if base.radar_horizon_level >= 5 {
        "Maximalstufe erreicht".to_string()
    } else {
        format!("Kosten: {} Erz", cost_t3)
    };

    // Determine current research target (active riddle track, or highest progress unmaxed track)
    let (
        target_track_name,
        target_track_icon,
        target_level,
        target_progress,
        target_cost_text,
        target_color,
    ) = if let Some(ref r) = current_riddle {
        match r.track {
            1 => (
                "Drohnen-Scan (Sortieren)",
                "🛸",
                base.drone_scanner_level,
                base.drone_scanner_progress,
                cost_t1_text.clone(),
                "linear-gradient(90deg, #0284c7, #00e5ff)",
            ),
            2 => (
                "Basen & Asteroiden",
                "☄️",
                base.base_scanner_level,
                base.base_scanner_progress,
                cost_t2_text.clone(),
                "linear-gradient(90deg, #0369a1, #38bdf8)",
            ),
            _ => (
                "Radar-Horizont",
                "📡",
                base.radar_horizon_level,
                base.radar_horizon_progress,
                cost_t3_text.clone(),
                "linear-gradient(90deg, #7e22ce, #c084fc)",
            ),
        }
    } else {
        let p1 = if base.drone_scanner_level < 5 {
            base.drone_scanner_progress
        } else {
            0
        };
        let p2 = if base.base_scanner_level < 5 {
            base.base_scanner_progress
        } else {
            0
        };
        let p3 = if base.radar_horizon_level < 5 {
            base.radar_horizon_progress
        } else {
            0
        };
        if p3 > p2 && p3 > p1 {
            (
                "Radar-Horizont",
                "📡",
                base.radar_horizon_level,
                base.radar_horizon_progress,
                cost_t3_text.clone(),
                "linear-gradient(90deg, #7e22ce, #c084fc)",
            )
        } else if p2 > p1 {
            (
                "Basen & Asteroiden",
                "☄️",
                base.base_scanner_level,
                base.base_scanner_progress,
                cost_t2_text.clone(),
                "linear-gradient(90deg, #0369a1, #38bdf8)",
            )
        } else {
            (
                "Drohnen-Scan (Sortieren)",
                "🛸",
                base.drone_scanner_level,
                base.drone_scanner_progress,
                cost_t1_text.clone(),
                "linear-gradient(90deg, #0284c7, #00e5ff)",
            )
        }
    };

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

    let b_center_pos = base_pos.clone();
    // Continuously render local base radar via request_animation_frame with interpolation
    use_effect(move || {
        let b_center = b_center_pos.clone();
        #[cfg(target_arch = "wasm32")]
        {
            use std::cell::RefCell;
            use std::rc::Rc;
            use wasm_bindgen::JsCast;
            use wasm_bindgen::closure::Closure;

            let anim_handle: Rc<RefCell<Option<Closure<dyn FnMut()>>>> =
                Rc::new(RefCell::new(None));
            let anim_handle_clone = anim_handle.clone();

            let render_step = Rc::new({
                let b_center = b_center.clone();
                move || -> bool {
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

                                        if canvas.width() != target_w || canvas.height() != target_h
                                        {
                                            canvas.set_width(target_w);
                                            canvas.set_height(target_h);
                                        }

                                        if let Ok(Some(ctx_2d)) = canvas.get_context("2d") {
                                            if let Ok(ctx_2d) = ctx_2d
                                                .dyn_into::<web_sys::CanvasRenderingContext2d>(
                                            ) {
                                                let configs = (ctx.tables.config)();
                                                let players = (ctx.tables.player)();
                                                let my_current_bases = (ctx.tables.my_base)();
                                                let my_current_ships = (ctx.tables.my_ships)();
                                                let visible_drones = (ctx.tables.visible_drones)();
                                                let visible_bases = (ctx.tables.visible_bases)();
                                                let visible_asteroids =
                                                    (ctx.tables.visible_asteroids)();
                                                let unidentified_objects =
                                                    (ctx.tables.unidentified_objects)();

                                                let my_current_base = my_current_bases.first();
                                                let time = js_sys::Date::now() / 1000.0;
                                                let cam_x =
                                                    b_center.x as f64 + *cam_offset_x.read();
                                                let cam_y =
                                                    b_center.y as f64 + *cam_offset_y.read();
                                                let cur_zoom = *zoom.read();

                                                render_player_radar(
                                                    &ctx_2d,
                                                    css_w,
                                                    css_h,
                                                    dpr,
                                                    cam_x,
                                                    cam_y,
                                                    cur_zoom,
                                                    b_center.clone(),
                                                    p_hue,
                                                    p_sec_hue,
                                                    configs.first(),
                                                    &my_current_ships,
                                                    my_current_base,
                                                    &visible_drones,
                                                    &visible_bases,
                                                    &visible_asteroids,
                                                    &unidentified_objects,
                                                    &players,
                                                    None,
                                                    selected_target(),
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
                }
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

    let b_pos_x = base.position.x as f64;
    let b_pos_y = base.position.y as f64;
    let current_cam_x = b_pos_x + *cam_offset_x.read();
    let current_cam_y = b_pos_y + *cam_offset_y.read();
    let current_zoom = *zoom.read();

    let mut sorted_fleet_ships = ships.clone();
    sorted_fleet_ships.sort_by(|a, b| {
        b.ship_tier
            .cmp(&a.ship_tier)
            .then_with(|| a.ship_id.cmp(&b.ship_id))
    });

    rsx! {
        div {
            style: "flex: 1; display: flex; flex-direction: row; height: 100%; overflow: hidden; background: #070d1d;",

            // Left / Main: Local Sensor Radar
            div {
                style: if *is_dragging.read() {
                    "flex: 1; height: 100%; position: relative; background: #050a16; display: flex; align-items: center; justify-content: center; overflow: hidden; cursor: grabbing;"
                } else {
                    "flex: 1; height: 100%; position: relative; background: #050a16; display: flex; align-items: center; justify-content: center; overflow: hidden; cursor: crosshair;"
                },
                onmousedown: move |evt| {
                    is_dragging.set(true);
                    drag_start.set((evt.client_coordinates().x, evt.client_coordinates().y));
                    drag_distance.set(0.0);
                },
                onmousemove: move |evt| {
                    if *is_dragging.read() {
                        let (sx, sy) = *drag_start.read();
                        let cx = evt.client_coordinates().x;
                        let cy = evt.client_coordinates().y;
                        let dx = cx - sx;
                        let dy = cy - sy;
                        let cur_dist = *drag_distance.read();
                        drag_distance.set(cur_dist + dx.hypot(dy));
                        drag_start.set((cx, cy));
                        let z = *zoom.read();
                        let cur_x = *cam_offset_x.read();
                        let cur_y = *cam_offset_y.read();
                        cam_offset_x.set(cur_x - dx / z);
                        cam_offset_y.set(cur_y - dy / z);
                    }
                },
                onmouseup: move |evt| {
                    let was_dragging = *is_dragging.read();
                    is_dragging.set(false);
                    if was_dragging && *drag_distance.read() < 6.0 {
                        #[cfg(target_arch = "wasm32")]
                        {
                            if let Some(w) = web_sys::window() {
                                if let Some(doc) = w.document() {
                                    if let Some(canvas_el) = doc.get_element_by_id("base-radar-canvas") {
                                        if let Ok(canvas) = canvas_el.dyn_into::<HtmlCanvasElement>() {
                                            let rect = canvas.get_bounding_client_rect();
                                            if rect.width() > 0.0 && rect.height() > 0.0 {
                                                let coords = evt.client_coordinates();
                                                let css_x = coords.x - rect.left();
                                                let css_y = coords.y - rect.top();

                                                let rel_x = css_x - (rect.width() / 2.0);
                                                let rel_y = css_y - (rect.height() / 2.0);

                                                let my_current_bases = (ctx.tables.my_base)();
                                                let my_current_ships = (ctx.tables.my_ships)();
                                                let visible_drones = (ctx.tables.visible_drones)();
                                                let visible_bases = (ctx.tables.visible_bases)();
                                                let visible_asteroids = (ctx.tables.visible_asteroids)();

                                                let b_pos = my_current_bases
                                                    .first()
                                                    .map(|b| b.position.clone())
                                                    .unwrap_or(base_pos.clone());

                                                let z = *zoom.read();
                                                let cam_x = b_pos.x as f64 + *cam_offset_x.read();
                                                let cam_y = b_pos.y as f64 + *cam_offset_y.read();

                                                let click_world_x = cam_x + rel_x / z;
                                                let click_world_y = cam_y + rel_y / z;

                                                let mut best_candidate: Option<(f64, SelectedEntity)> = None;

                                                // 1. Heimatbasis
                                                let dist_home = ((click_world_x - b_pos.x as f64).powi(2) + (click_world_y - b_pos.y as f64).powi(2)).sqrt() * z;
                                                if dist_home <= 34.0 {
                                                    best_candidate = Some((dist_home, SelectedEntity::Heimatbasis));
                                                }

                                                // 2. My ships
                                                for s in &my_current_ships {
                                                    let dist = ((click_world_x - s.position.x as f64).powi(2) + (click_world_y - s.position.y as f64).powi(2)).sqrt() * z;
                                                    if dist <= 28.0 {
                                                        if best_candidate.as_ref().map_or(true, |(d, _)| dist < *d) {
                                                            best_candidate = Some((dist, SelectedEntity::MyShip(s.ship_id)));
                                                        }
                                                    }
                                                }

                                                // 3. Foreign drones
                                                for d in &visible_drones {
                                                    if !my_current_ships.iter().any(|s| s.ship_id == d.ship_id) {
                                                        let dist = ((click_world_x - d.position.x as f64).powi(2) + (click_world_y - d.position.y as f64).powi(2)).sqrt() * z;
                                                        if dist <= 28.0 {
                                                            if best_candidate.as_ref().map_or(true, |(d, _)| dist < *d) {
                                                                best_candidate = Some((dist, SelectedEntity::ForeignDrone(d.ship_id)));
                                                            }
                                                        }
                                                    }
                                                }

                                                // 4. Asteroids
                                                for ast in &visible_asteroids {
                                                    let dist = ((click_world_x - ast.position.x as f64).powi(2) + (click_world_y - ast.position.y as f64).powi(2)).sqrt();
                                                    let tolerance = ((ast.radius as f64) * 0.75).max(12.0) + (16.0 / z.min(1.0));
                                                    if dist <= tolerance {
                                                        let dist_screen = dist * z;
                                                        if best_candidate.as_ref().map_or(true, |(d, _)| dist_screen < *d) {
                                                            best_candidate = Some((dist_screen, SelectedEntity::Asteroid(ast.asteroid_id)));
                                                        }
                                                    }
                                                }

                                                // 5. Foreign bases
                                                for fb in &visible_bases {
                                                    if my_current_bases.first().map_or(true, |b| b.base_id != fb.base_id) {
                                                        let dist = ((click_world_x - fb.position.x as f64).powi(2) + (click_world_y - fb.position.y as f64).powi(2)).sqrt() * z;
                                                        if dist <= 32.0 {
                                                            if best_candidate.as_ref().map_or(true, |(d, _)| dist < *d) {
                                                                best_candidate = Some((dist, SelectedEntity::ForeignBase(fb.base_id)));
                                                            }
                                                        }
                                                    }
                                                }

                                                if let Some((_, entity)) = best_candidate {
                                                    if entity == SelectedEntity::Heimatbasis {
                                                        on_open_heimatbasis.call(());
                                                    }
                                                    selected_target.set(Some(entity));
                                                } else {
                                                    selected_target.set(None);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
                onmouseleave: move |_| {
                    is_dragging.set(false);
                },
                onwheel: move |evt| {
                    let delta_y = evt.delta().strip_units().y;
                    let factor = if delta_y > 0.0 { 0.85 } else { 1.15 };
                    let cur_z = *zoom.read();
                    zoom.set((cur_z * factor).clamp(0.08, 6.0));
                },
                canvas {
                    id: "base-radar-canvas",
                    style: "width: 100%; height: 100%; display: block;",
                }
                div {
                    style: if is_tutorial_collapsed() {
                        "position: absolute; bottom: 64px; left: 20px; transition: left 0.2s ease; background: rgba(10, 16, 32, 0.85); padding: 6px 12px; border-radius: 6px; font-size: 11px; color: #94a3b8; border: 1px solid rgba(80, 120, 200, 0.2); pointer-events: none;"
                    } else {
                        "position: absolute; bottom: 64px; left: 432px; transition: left 0.2s ease; background: rgba(10, 16, 32, 0.85); padding: 6px 12px; border-radius: 6px; font-size: 11px; color: #94a3b8; border: 1px solid rgba(80, 120, 200, 0.2); pointer-events: none;"
                    },
                    "ℹ️ Ziehen zum Bewegen • Scrollen zum Zoomen • Klick wählt Ziel"
                }

                // HUD Overlay: Camera Info & Center Controls
                div {
                    style: if is_tutorial_collapsed() {
                        "position: absolute; bottom: 16px; left: 20px; transition: left 0.2s ease; display: flex; gap: 8px; align-items: center; background: rgba(10, 16, 32, 0.9); backdrop-filter: blur(8px); padding: 8px 14px; border-radius: 8px; border: 1px solid rgba(80, 120, 200, 0.3); font-size: 12px; font-family: monospace; z-index: 5; pointer-events: auto;"
                    } else {
                        "position: absolute; bottom: 16px; left: 432px; transition: left 0.2s ease; display: flex; gap: 8px; align-items: center; background: rgba(10, 16, 32, 0.9); backdrop-filter: blur(8px); padding: 8px 14px; border-radius: 8px; border: 1px solid rgba(80, 120, 200, 0.3); font-size: 12px; font-family: monospace; z-index: 5; pointer-events: auto;"
                    },
                    span { style: "color: #8da2c0;", "POS: " }
                    span { style: "color: #00e5ff; font-weight: 700;", "[{current_cam_x.round() as i32}, {current_cam_y.round() as i32}]" }
                    span { style: "color: #8da2c0; margin-left: 6px;", "ZOOM: " }
                    span { style: "color: #ffd700; font-weight: 700;", "{(current_zoom * 100.0).round() as i32}%" }
                    button {
                        style: "background: #1e293b; color: #fff; border: 1px solid #334155; padding: 2px 7px; border-radius: 4px; cursor: pointer;",
                        onclick: move |_| {
                            let cur_z = *zoom.read();
                            zoom.set((cur_z * 1.25).clamp(0.08, 6.0));
                        },
                        "+"
                    }
                    button {
                        style: "background: #1e293b; color: #fff; border: 1px solid #334155; padding: 2px 7px; border-radius: 4px; cursor: pointer;",
                        onclick: move |_| {
                            let cur_z = *zoom.read();
                            zoom.set((cur_z * 0.8).clamp(0.08, 6.0));
                        },
                        "-"
                    }
                    button {
                        style: "background: #0284c7; color: #fff; border: none; padding: 3px 8px; border-radius: 4px; cursor: pointer; font-weight: 600; margin-left: 4px;",
                        onclick: move |_| {
                            cam_offset_x.set(0.0);
                            cam_offset_y.set(0.0);
                            zoom.set(0.85);
                        },
                        "⌂ Basis"
                    }
                    button {
                        style: "background: #334155; color: #e2e8f0; border: none; padding: 3px 8px; border-radius: 4px; cursor: pointer; font-weight: 600;",
                        onclick: move |_| {
                            let cur_bx = base.position.x as f64;
                            let cur_by = base.position.y as f64;
                            cam_offset_x.set(-cur_bx);
                            cam_offset_y.set(-cur_by);
                            zoom.set(0.5);
                        },
                        "⌂ Mega-Station"
                    }
                }

                // Floating Target Inspector HUD
                if let Some(target) = selected_target() {
                    TargetInspectorHud {
                        target,
                        base_pos: base.position.clone(),
                        ships: ships.clone(),
                        visible_asteroids: (ctx.tables.visible_asteroids)(),
                        visible_drones: (ctx.tables.visible_drones)(),
                        visible_bases: (ctx.tables.visible_bases)(),
                        players: (ctx.tables.player)(),
                        is_tutorial_collapsed,
                        on_open_heimatbasis: move |_| on_open_heimatbasis.call(()),
                        on_close: move |_| selected_target.set(None),
                    }
                }
            }

            // Right Column: Controls, Current Research Target & Fleet Status
            aside {
                style: "width: 340px; height: 100%; background: #0a1020; border-left: 1px solid rgba(80, 120, 200, 0.2); padding: 20px; box-sizing: border-box; display: flex; flex-direction: column; gap: 16px; overflow-y: auto;",

                // Notification & Token Box: Shown until API connects or during Tutorial Steps <= 4
                if !player.has_api_connected || player.tutorial_step <= 4 {
                    div {
                        style: "background: rgba(15, 23, 42, 0.95); border: 1px solid rgba(80, 120, 200, 0.3); border-radius: 12px; padding: 16px;",
                        div {
                            style: "display: inline-flex; align-items: center; gap: 6px; padding: 4px 8px; border-radius: 6px; background: rgba(0, 230, 118, 0.15); border: 1px solid rgba(0, 230, 118, 0.3); font-size: 11px; font-weight: 700; color: #00e676; letter-spacing: 0.5px; margin-bottom: 8px;",
                            "✔ BASIS ETABLIERT"
                        }
                        h2 {
                            style: "margin: 0 0 6px 0; font-size: 16px; font-weight: 700; color: #ffd700;",
                            "🎉 Heimatbasis aktiv!"
                        }
                        p {
                            style: "margin: 0; font-size: 12px; color: #94a3b8; line-height: 1.4;",
                            "Steuere deine Flotte ab jetzt über "
                            strong { style: "color: #00e5ff;", "Rust-Code im Zed-Editor" }
                            " via SpacetimeDB!"
                        }
                    }

                    // Token Box: Shown until API connects
                    {
                        let bot_token = {
                            #[cfg(target_arch = "wasm32")]
                            {
                                oauth::get_local_storage()
                                    .and_then(|s| s.get_item("base_ai_token").ok().flatten())
                                    .unwrap_or_else(|| token.clone())
                            }
                            #[cfg(not(target_arch = "wasm32"))]
                            {
                                token.clone()
                            }
                        };
                        let is_perm = bot_token != token;
                        let copy_bot_token = {
                            let t = bot_token.clone();
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
                                style: if player.tutorial_step == 3 {
                                    "background: #0f172a; border: 2px solid #00e5ff; box-shadow: 0 0 24px rgba(0, 229, 255, 0.4); border-radius: 10px; padding: 14px;"
                                } else {
                                    "background: #0f172a; border: 1px solid #334155; border-radius: 10px; padding: 14px;"
                                },
                                div {
                                    style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;",
                                    div {
                                        style: "font-size: 11px; font-weight: 700; color: #8da2c0; letter-spacing: 0.5px;",
                                        if is_perm { "🤖 DEIN PERMANENTER BASE-AI TOKEN:" } else { "DEIN PERSÖNLICHER API-TOKEN:" }
                                    }
                                    if player.tutorial_step == 3 {
                                        span { style: "font-size: 10px; color: #00e5ff; font-weight: 700; animation: pulse 1s infinite;", "👉 HIER KOPIEREN" }
                                    } else if is_perm {
                                        span { style: "font-size: 10px; color: #00e676; font-weight: 600;", "✔ Permanent (kein 60s Timeout)" }
                                    }
                                }
                                div {
                                    style: "display: flex; gap: 8px;",
                                    input {
                                        readonly: true,
                                        style: "flex: 1; background: #020617; border: 1px solid #1e293b; color: #38bdf8; padding: 8px 10px; border-radius: 6px; font-family: monospace; font-size: 11px; outline: none; min-width: 0;",
                                        value: "{bot_token}",
                                    }
                                    button {
                                        style: if copied() {
                                            "background: #059669; color: #fff; border: none; padding: 8px 12px; border-radius: 6px; font-weight: 700; font-size: 12px; cursor: pointer; transition: all 0.2s;"
                                        } else {
                                            "background: #0284c7; hover:background: #0369a1; color: #fff; border: none; padding: 8px 12px; border-radius: 6px; font-weight: 700; font-size: 12px; cursor: pointer; transition: all 0.2s;"
                                        },
                                        onclick: copy_bot_token,
                                        if copied() { "✔" } else { "📋" }
                                    }
                                }
                                div {
                                    style: "font-size: 11px; color: #64748b; margin-top: 6px;",
                                    "In " code { style: "color: #38bdf8;", "config.toml" } " deines Zed-Projekts einfügen (oder in Heimatbasis verwalten)."
                                }
                            }
                        }
                    }
                }

                // Current Research Target Card
                div {
                    style: "background: rgba(15, 23, 42, 0.95); border: 1px solid rgba(0, 229, 255, 0.35); border-radius: 12px; padding: 16px; box-shadow: 0 4px 16px rgba(0, 229, 255, 0.08); display: flex; flex-direction: column; gap: 10px;",
                    div {
                        style: "display: flex; justify-content: space-between; align-items: center;",
                        div {
                            style: "font-size: 11px; font-weight: 700; color: #8da2c0; letter-spacing: 0.5px;",
                            "FORSCHUNGSZIEL"
                        }
                        span {
                            style: "font-size: 10px; font-weight: 700; padding: 2px 6px; border-radius: 4px; background: rgba(0, 229, 255, 0.15); color: #00e5ff;",
                            "Stufe {target_level}/5"
                        }
                    }
                    div {
                        style: "display: flex; align-items: center; gap: 8px;",
                        span { style: "font-size: 18px;", "{target_track_icon}" }
                        div {
                            style: "font-weight: 700; font-size: 13px; color: #f1f5f9;",
                            "{target_track_name}"
                        }
                    }
                    div {
                        style: "width: 100%; height: 8px; background: #0f172a; border-radius: 4px; overflow: hidden; position: relative;",
                        div {
                            style: "height: 100%; width: {target_progress}%; background: {target_color}; border-radius: 4px; transition: width 0.3s;"
                        }
                    }
                    div {
                        style: "display: flex; justify-content: space-between; font-size: 10px; color: #64748b;",
                        span { "Fortschritt: {target_progress}%" }
                        span { "{target_cost_text}" }
                    }
                    if let Some(ref r) = current_riddle {
                        div {
                            style: "padding: 6px 10px; background: rgba(250, 204, 21, 0.1); border: 1px solid rgba(250, 204, 21, 0.3); border-radius: 6px; font-size: 11px; color: #fef08a;",
                            "⏳ Rätsel aktiv: Stufe {r.target_level} • In Zed lösen"
                        }
                    }
                    button {
                        style: "margin-top: 4px; background: rgba(0, 229, 255, 0.1); hover:background: rgba(0, 229, 255, 0.2); border: 1px solid rgba(0, 229, 255, 0.3); color: #00e5ff; padding: 8px 12px; border-radius: 6px; font-size: 11px; font-weight: 700; cursor: pointer; transition: all 0.2s; display: flex; align-items: center; justify-content: center; gap: 6px;",
                        onclick: move |_| on_open_heimatbasis.call(()),
                        span { "🏛️" }
                        span { "Forschungszentrum öffnen" }
                    }
                }

                // Active Ships Fleet List & Emergency Rebuild
                div {
                    style: "background: rgba(15, 23, 42, 0.95); border: 1px solid rgba(80, 120, 200, 0.25); border-radius: 12px; padding: 16px; display: flex; flex-direction: column; gap: 12px;",
                    div {
                        style: "display: flex; justify-content: space-between; align-items: center;",
                        h3 { style: "margin: 0; font-size: 14px; font-weight: 700; color: #00e5ff;", "🚀 Deine Flotte ({ships.len()} Schiffe)" }
                    }
                    if ships.is_empty() {
                        div {
                            style: "background: rgba(239, 68, 68, 0.12); border: 1px solid rgba(239, 68, 68, 0.4); border-radius: 10px; padding: 14px; text-align: center; display: flex; flex-direction: column; gap: 8px;",
                            div {
                                style: "display: flex; align-items: center; justify-content: center; gap: 6px; color: #fca5a5; font-size: 13px; font-weight: 700;",
                                span { "🚨" }
                                span { "Keine Drohnen aktiv!" }
                            }
                            p {
                                style: "margin: 0; font-size: 11px; color: #cbd5e1; line-height: 1.4;",
                                "Deine Heimatbasis stellt sofort eine kostenlose Notfall-Drohne bereit."
                            }
                            button {
                                style: "background: #dc2626; hover:background: #b91c1c; color: #fff; border: none; padding: 10px 14px; border-radius: 6px; font-size: 12px; font-weight: 700; cursor: pointer; transition: all 0.2s; box-shadow: 0 4px 12px rgba(220, 38, 38, 0.4);",
                                onclick: move |_| {
                                    if let Some(conn) = (ctx.connection)() {
                                        let _ = conn.reducers.build_ship(1);
                                    }
                                },
                                "🛸 Notfall-Drohne (Kostenlos)"
                            }
                        }
                    } else {
                        div {
                            style: "display: flex; flex-direction: column; gap: 8px;",
                            for s in sorted_fleet_ships {
                                div {
                                    key: "{s.ship_id}",
                                    style: "padding: 10px 12px; background: rgba(255, 255, 255, 0.03); border-radius: 6px; border-left: 3px solid #00e5ff; cursor: pointer;",
                                    onclick: {
                                        let sid = s.ship_id;
                                        move |_| selected_target.set(Some(SelectedEntity::MyShip(sid)))
                                    },
                                    div {
                                        style: "display: flex; justify-content: space-between; margin-bottom: 4px;",
                                        span { style: "font-weight: 700; font-size: 12px;", "Drohne #{s.ship_id} (Tier {s.ship_tier})" }
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
}

// ==========================================
// Floating Target Inspector HUD
// ==========================================

#[component]
fn TargetInspectorHud(
    target: SelectedEntity,
    base_pos: Vec2,
    ships: Vec<Ship>,
    visible_asteroids: Vec<PerceivedAsteroid>,
    visible_drones: Vec<PerceivedDrone>,
    visible_bases: Vec<PerceivedBase>,
    players: Vec<Player>,
    is_tutorial_collapsed: Signal<bool>,
    on_open_heimatbasis: EventHandler<()>,
    on_close: EventHandler<()>,
) -> Element {
    let ctx = use_spacetimedb_context();
    let my_ai_agents_list = (ctx.tables.my_ai_agents)();
    let conn_opt = (ctx.connection)();
    let mut ship_bot_id_input = use_signal(|| String::new());
    let mut ship_bot_err = use_signal(|| Option::<String>::None);

    let player_map: std::collections::HashMap<_, _> =
        players.iter().map(|p| (p.identity, p)).collect();

    rsx! {
        div {
            style: if is_tutorial_collapsed() {
                "position: absolute; bottom: 56px; left: 20px; transition: left 0.2s ease; width: 280px; background: rgba(10, 16, 32, 0.94); backdrop-filter: blur(10px); border: 1px solid rgba(0, 229, 255, 0.4); border-radius: 12px; padding: 14px; box-shadow: 0 8px 32px rgba(0,0,0,0.7); z-index: 5; display: flex; flex-direction: column; gap: 10px;"
            } else {
                "position: absolute; bottom: 56px; left: 432px; transition: left 0.2s ease; width: 280px; background: rgba(10, 16, 32, 0.94); backdrop-filter: blur(10px); border: 1px solid rgba(0, 229, 255, 0.4); border-radius: 12px; padding: 14px; box-shadow: 0 8px 32px rgba(0,0,0,0.7); z-index: 5; display: flex; flex-direction: column; gap: 10px;"
            },
            // Title Bar
            div {
                style: "display: flex; justify-content: space-between; align-items: center; border-bottom: 1px solid rgba(80, 120, 200, 0.2); padding-bottom: 8px;",
                div {
                    style: "font-size: 11px; font-weight: 700; color: #00e5ff; letter-spacing: 0.5px; display: flex; align-items: center; gap: 6px;",
                    span { "🎯" }
                    span { "RADAR-INSPEKTOR" }
                }
                button {
                    style: "background: transparent; border: none; color: #94a3b8; hover:color: #fff; font-size: 13px; cursor: pointer; padding: 0 4px;",
                    onclick: move |_| on_close.call(()),
                    "✕"
                }
            }

            match target {
                SelectedEntity::Asteroid(id) => {
                    let ast = visible_asteroids.iter().find(|a| a.asteroid_id == id);
                    if let Some(ast) = ast {
                        let (res_name, res_color) = match ast.resource_type {
                            3 => ("Titan", "#a855f7"),
                            2 => ("Eis", "#06b6d4"),
                            1 => ("Eisen", "#94a3b8"),
                            _ => ("Erz", "#78716c"),
                        };
                        let dist = distance(&ast.position, &base_pos);
                        let vel = (ast.velocity.x * ast.velocity.x + ast.velocity.y * ast.velocity.y).sqrt();
                        rsx! {
                            div {
                                style: "display: flex; justify-content: space-between; align-items: center;",
                                span { style: "font-weight: 700; font-size: 14px; color: #f1f5f9;", "☄️ Asteroid #{ast.asteroid_id}" }
                                span { style: "font-size: 11px; font-weight: 700; padding: 2px 8px; border-radius: 4px; background: rgba(255, 255, 255, 0.08); color: {res_color}; border: 1px solid {res_color};", "{res_name}" }
                            }
                            div {
                                style: "display: grid; grid-template-columns: 1fr 1fr; gap: 8px; font-size: 11px; margin-top: 4px;",
                                div {
                                    style: "padding: 8px; background: rgba(255, 255, 255, 0.03); border-radius: 6px;",
                                    div { style: "color: #8da2c0;", "VERBL. ERZ" }
                                    div { style: "font-size: 14px; font-weight: 700; color: #ffd700;", "{ast.remaining_ore}" }
                                }
                                div {
                                    style: "padding: 8px; background: rgba(255, 255, 255, 0.03); border-radius: 6px;",
                                    div { style: "color: #8da2c0;", "RADIUS" }
                                    div { style: "font-size: 14px; font-weight: 700; color: #f1f5f9;", "{ast.radius as u32} m" }
                                }
                                div {
                                    style: "padding: 8px; background: rgba(255, 255, 255, 0.03); border-radius: 6px;",
                                    div { style: "color: #8da2c0;", "BASIS-DISTANZ" }
                                    div { style: "font-size: 13px; font-weight: 700; color: #38bdf8;", "{dist as u32} px" }
                                }
                                div {
                                    style: "padding: 8px; background: rgba(255, 255, 255, 0.03); border-radius: 6px;",
                                    div { style: "color: #8da2c0;", "DRIFT" }
                                    div { style: "font-size: 13px; font-weight: 700; color: #94a3b8;", "{vel as u32} px/s" }
                                }
                            }
                        }
                    } else {
                        rsx! {
                            div { style: "padding: 8px; font-size: 12px; color: #fca5a5;", "⚠️ Asteroid #{id} nicht mehr in Sensorreichweite." }
                        }
                    }
                }
                SelectedEntity::MyShip(id) => {
                    let s = ships.iter().find(|s| s.ship_id == id);
                    if let Some(s) = s {
                        let status_text = if s.is_stranded {
                            ("🚨 GESTRANDET", "#ef4444")
                        } else if s.is_mining {
                            ("⛏️ BAUT AB", "#00e5ff")
                        } else {
                            ("● BEREIT", "#00e676")
                        };
                        rsx! {
                            div {
                                style: "display: flex; justify-content: space-between; align-items: center;",
                                span { style: "font-weight: 700; font-size: 14px; color: #f1f5f9;", "🛸 Drohne #{s.ship_id} (Tier {s.ship_tier})" }
                                span { style: "font-size: 10px; font-weight: 700; color: {status_text.1};", "{status_text.0}" }
                            }
                            div {
                                style: "display: flex; flex-direction: column; gap: 6px; font-size: 11px; margin-top: 4px;",
                                div {
                                    div { style: "display: flex; justify-content: space-between; margin-bottom: 2px;",
                                        span { style: "color: #8da2c0;", "Frachtladung" }
                                        span { style: "color: #ffd700; font-weight: 600;", "{s.cargo_ore} / {s.max_cargo} Erz" }
                                    }
                                    div { style: "width: 100%; height: 5px; background: #0f172a; border-radius: 3px; overflow: hidden;",
                                        div { style: "height: 100%; width: {((s.cargo_ore as f32 / s.max_cargo.max(1) as f32) * 100.0) as u32}%; background: #ffd700;" }
                                    }
                                }
                                div {
                                    div { style: "display: flex; justify-content: space-between; margin-bottom: 2px;",
                                        span { style: "color: #8da2c0;", "Energie" }
                                        span { style: "color: #00e5ff; font-weight: 600;", "{s.energy as u32} / {s.max_energy as u32}" }
                                    }
                                    div { style: "width: 100%; height: 5px; background: #0f172a; border-radius: 3px; overflow: hidden;",
                                        div { style: "height: 100%; width: {((s.energy / s.max_energy.max(1.0)) * 100.0) as u32}%; background: #00e5ff;" }
                                    }
                                }
                                div {
                                    style: "display: flex; justify-content: space-between; color: #94a3b8; margin-top: 4px;",
                                    span { "Geschwindigkeit: {s.speed as u32} px/s" }
                                    span { "Kurs: {((s.heading_rad.to_degrees() as i32 + 360) % 360)}°" }
                                }
                                div {
                                    style: "color: #64748b; font-size: 10px;",
                                    "Position: ({s.position.x as i32}, {s.position.y as i32})"
                                }

                                // Ship Bot / AI Pairing Section
                                {
                                    let active_ship_agent = my_ai_agents_list
                                        .iter()
                                        .find(|a| (a.role == 1 || a.role == 2) && a.bound_entity_id == id && !a.is_revoked)
                                        .cloned();

                                    rsx! {
                                        div {
                                            style: "margin-top: 8px; padding-top: 8px; border-top: 1px solid rgba(80, 120, 200, 0.2); display: flex; flex-direction: column; gap: 6px;",
                                            if let Some(ref agent) = active_ship_agent {
                                                {
                                                    let agent_id = agent.agent_identity;
                                                    let conn_opt = conn_opt.clone();
                                                    rsx! {
                                                        div {
                                                            style: "display: flex; justify-content: space-between; align-items: center;",
                                                            span { style: "font-weight: 700; color: #00e5ff; font-size: 11px;", "🤖 {agent.label}" }
                                                            button {
                                                                style: "background: rgba(239, 68, 68, 0.15); hover:background: rgba(239, 68, 68, 0.28); color: #ef4444; border: 1px solid rgba(239, 68, 68, 0.4); padding: 2px 6px; border-radius: 4px; font-size: 10px; cursor: pointer;",
                                                                onclick: move |_| {
                                                                    if let Some(ref conn) = conn_opt {
                                                                        let _ = conn.reducers.revoke_ai_agent(agent_id);
                                                                    }
                                                                },
                                                                "🛑 Entkoppeln"
                                                            }
                                                        }
                                                        div {
                                                            style: "font-family: monospace; font-size: 9px; color: #38bdf8; word-break: break-all; background: rgba(0,0,0,0.3); padding: 4px 6px; border-radius: 4px;",
                                                            "{agent.agent_identity.to_hex()}"
                                                        }
                                                    }
                                                }
                                            } else {
                                                {
                                                    let conn_opt = conn_opt.clone();
                                                    let sid = id;
                                                    let is_carrier = s.ship_tier >= 2;
                                                    rsx! {
                                                        span { style: "font-weight: 700; color: #38bdf8; font-size: 11px;", "🤖 Schiffs-KI binden" }
                                                        div {
                                                            style: "display: flex; gap: 4px;",
                                                            input {
                                                                placeholder: "Bot-Identität (Hex)...",
                                                                style: "flex: 1; background: #020617; border: 1px solid #1e293b; color: #f1f5f9; padding: 4px 6px; border-radius: 4px; font-family: monospace; font-size: 10px; min-width: 0;",
                                                                value: "{ship_bot_id_input}",
                                                                oninput: move |e| ship_bot_id_input.set(e.value()),
                                                            }
                                                            button {
                                                                style: "background: #0284c7; hover:background: #0369a1; color: #fff; border: none; padding: 4px 8px; border-radius: 4px; font-weight: 700; font-size: 10px; cursor: pointer; white-space: nowrap;",
                                                                onclick: move |_| {
                                                                    let id_str = ship_bot_id_input().trim().to_string();
                                                                    if id_str.is_empty() {
                                                                        ship_bot_err.set(Some("Identität eingeben".to_string()));
                                                                        return;
                                                                    }
                                                                    match Identity::from_hex(&id_str) {
                                                                        Ok(agent_id) => {
                                                                            if let Some(ref conn) = conn_opt {
                                                                                let role = if is_carrier { 1 } else { 2 };
                                                                                let label = format!("Schiff #{} KI", sid);
                                                                                if let Err(e) = conn.reducers.register_ai_agent(agent_id, role, sid, label) {
                                                                                    ship_bot_err.set(Some(e.to_string()));
                                                                                } else {
                                                                                    ship_bot_id_input.set(String::new());
                                                                                    ship_bot_err.set(None);
                                                                                }
                                                                            }
                                                                        }
                                                                        Err(_) => {
                                                                            ship_bot_err.set(Some("Ungültige Hex-Identität".to_string()));
                                                                        }
                                                                    }
                                                                },
                                                                "🔗 Binden"
                                                            }
                                                        }
                                                        if let Some(ref err) = ship_bot_err() {
                                                            div { style: "color: #f87171; font-size: 10px;", "{err}" }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        rsx! {
                            div { style: "padding: 8px; font-size: 12px; color: #fca5a5;", "⚠️ Drohne #{id} nicht gefunden." }
                        }
                    }
                }
                SelectedEntity::ForeignDrone(id) => {
                    let d = visible_drones.iter().find(|d| d.ship_id == id);
                    if let Some(d) = d {
                        let pilot = player_map.get(&d.owner);
                        let p_name = pilot.map_or("Unbekannt".to_string(), |p| p.name.clone());
                        let p_hue = pilot.map_or(180, |p| p.color_hue);
                        let p_sec_hue = pilot.map_or((180 + 180) % 360, |p| p.secondary_color_hue);
                        let dist = distance(&d.position, &base_pos);
                        rsx! {
                            div {
                                style: "display: flex; justify-content: space-between; align-items: center;",
                                span { style: "font-weight: 700; font-size: 14px; color: #f1f5f9;", "🛸 Drohne #{d.ship_id}" }
                                span { style: "font-size: 11px; color: #38bdf8;", "Tier {d.ship_tier}" }
                            }
                            div {
                                style: "display: flex; align-items: center; gap: 6px; font-size: 12px; margin-top: 4px;",
                                span { style: "width: 10px; height: 10px; border-radius: 50%; background: linear-gradient(135deg, hsl({p_hue}, 90%, 55%) 50%, hsl({p_sec_hue}, 90%, 55%) 50%); box-shadow: 0 0 6px hsl({p_hue}, 90%, 55%);" }
                                span { style: "color: #cbd5e1; font-weight: 600;", "{p_name}" }
                            }
                            div {
                                style: "display: flex; justify-content: space-between; font-size: 11px; color: #94a3b8; margin-top: 6px;",
                                span { if d.is_mining { "Status: ⛏️ Baut Erz ab" } else { "Status: Im Flug" } }
                                span { "{d.speed as u32} px/s" }
                            }
                            div {
                                style: "font-size: 11px; color: #64748b; margin-top: 2px;",
                                "Distanz zu dir: {dist as u32} px"
                            }
                        }
                    } else {
                        rsx! {
                            div { style: "padding: 8px; font-size: 12px; color: #fca5a5;", "⚠️ Fremde Drohne #{id} außer Reichweite." }
                        }
                    }
                }
                SelectedEntity::ForeignBase(id) => {
                    let b = visible_bases.iter().find(|b| b.base_id == id);
                    if let Some(b) = b {
                        let pilot = player_map.get(&b.owner);
                        let p_name = pilot.map_or("Fremde Basis".to_string(), |p| format!("Basis ({})", p.name));
                        let p_hue = pilot.map_or(200, |p| p.color_hue);
                        let p_sec_hue = pilot.map_or((200 + 180) % 360, |p| p.secondary_color_hue);
                        let dist = distance(&b.position, &base_pos);
                        rsx! {
                            div {
                                style: "display: flex; align-items: center; gap: 8px;",
                                span { style: "width: 12px; height: 12px; border-radius: 50%; background: linear-gradient(135deg, hsl({p_hue}, 90%, 55%) 50%, hsl({p_sec_hue}, 90%, 55%) 50%); border: 1.5px solid #fff; box-shadow: 0 0 6px hsl({p_hue}, 90%, 55%);" }
                                span { style: "font-weight: 700; font-size: 14px; color: hsl({p_hue}, 90%, 65%);", "{p_name}" }
                            }
                            div {
                                style: "font-size: 11px; color: #94a3b8; margin-top: 4px;",
                                "Position: ({b.position.x as i32}, {b.position.y as i32})"
                            }
                            div {
                                style: "font-size: 11px; color: #38bdf8; margin-top: 2px;",
                                "Distanz zu deiner Basis: {dist as u32} px"
                            }
                        }
                    } else {
                        rsx! {
                            div { style: "padding: 8px; font-size: 12px; color: #fca5a5;", "⚠️ Basis #{id} außer Reichweite." }
                        }
                    }
                }
                SelectedEntity::Heimatbasis => {
                    rsx! {
                        div {
                            style: "display: flex; flex-direction: column; gap: 8px;",
                            div {
                                style: "display: flex; align-items: center; gap: 8px;",
                                span { style: "font-size: 18px;", "🏛️" }
                                div {
                                    div { style: "font-weight: 700; font-size: 14px; color: #00e5ff;", "Deine Heimatbasis" }
                                    div { style: "font-size: 10px; color: #94a3b8;", "Sektor-Zentrale" }
                                }
                            }
                            button {
                                style: "background: #0284c7; hover:background: #0369a1; color: #fff; border: none; padding: 8px 12px; border-radius: 6px; font-weight: 700; font-size: 12px; cursor: pointer; transition: all 0.2s; margin-top: 4px; display: flex; align-items: center; justify-content: center; gap: 6px;",
                                onclick: move |_| on_open_heimatbasis.call(()),
                                span { "🔍" }
                                span { "Station & Forschung öffnen" }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ==========================================
// Heimatbasis & Sensor-Forschungszentrum Modal
// ==========================================

#[component]
fn HeimatbasisModal(
    player: Player,
    base: BaseStation,
    token: String,
    stdb_url: String,
    current_riddle: Option<PublicRiddle>,
    on_close: EventHandler<()>,
) -> Element {
    let ctx = use_spacetimedb_context();
    let my_ai_agents_list = (ctx.tables.my_ai_agents)();
    let active_base_agent = my_ai_agents_list
        .iter()
        .find(|a| a.role == 0 && a.bound_entity_id == base.base_id && !a.is_revoked)
        .cloned();

    let mut input_identity = use_signal(|| String::new());
    let mut input_label = use_signal(|| "Heimatbasis AI".to_string());
    let mut bind_error = use_signal(|| Option::<String>::None);
    let mut copied = use_signal(|| false);

    let mut edit_primary_hue = use_signal(move || player.color_hue);
    let mut edit_secondary_hue = use_signal(move || player.secondary_color_hue);
    let mut is_saving_colors = use_signal(|| false);
    let mut save_color_msg = use_signal(|| Option::<String>::None);

    let is_api_recently_active = player.last_api_activity.as_ref().map_or(false, |ts| {
        let ts_sec = (ts.to_micros_since_unix_epoch() / 1_000_000) as f64;
        let now_sec = js_sys::Date::now() / 1000.0;
        (now_sec - ts_sec) <= 30.0
    });

    let bind_base_ai = {
        let base_id = base.base_id;
        let conn_opt = (ctx.connection)();
        move |_| {
            let id_str = input_identity().trim().to_string();
            if id_str.is_empty() {
                bind_error.set(Some("Bitte gib eine Bot-Identität ein.".to_string()));
                return;
            }
            match Identity::from_hex(&id_str) {
                Ok(agent_id) => {
                    if let Some(ref conn) = conn_opt {
                        let label = input_label().trim().to_string();
                        let final_label = if label.is_empty() {
                            "Heimatbasis AI".to_string()
                        } else {
                            label
                        };
                        match conn
                            .reducers
                            .register_ai_agent(agent_id, 0, base_id, final_label)
                        {
                            Ok(_) => {
                                input_identity.set(String::new());
                                bind_error.set(None);
                            }
                            Err(e) => {
                                bind_error.set(Some(format!("Fehler beim Registrieren: {e}")));
                            }
                        }
                    }
                }
                Err(_) => {
                    bind_error.set(Some("Ungültige Identität! Erwartet 64-Zeichen Hexadezimal-String aus der Bot-Konsole.".to_string()));
                }
            }
        }
    };

    let revoke_base_ai = {
        let conn_opt = (ctx.connection)();
        let agent_opt = active_base_agent.clone();
        move |_| {
            if let (Some(conn), Some(agent)) = (&conn_opt, &agent_opt) {
                let _ = conn.reducers.revoke_ai_agent(agent.agent_identity);
            }
        }
    };

    let cost_t1 = (base.drone_scanner_level.saturating_sub(1) as u32) * 100
        + base.drone_scanner_progress as u32
        + 1;
    let cost_t2 = (base.base_scanner_level.saturating_sub(1) as u32) * 100
        + base.base_scanner_progress as u32
        + 1;
    let cost_t3 = (base.radar_horizon_level.saturating_sub(1) as u32) * 100
        + base.radar_horizon_progress as u32
        + 1;

    let cost_t1_text = if base.drone_scanner_level >= 5 {
        "Maximalstufe erreicht".to_string()
    } else {
        format!("Kosten: {} Erz", cost_t1)
    };
    let cost_t2_text = if base.base_scanner_level >= 5 {
        "Maximalstufe erreicht".to_string()
    } else {
        format!("Kosten: {} Erz", cost_t2)
    };
    let cost_t3_text = if base.radar_horizon_level >= 5 {
        "Maximalstufe erreicht".to_string()
    } else {
        format!("Kosten: {} Erz", cost_t3)
    };

    rsx! {
        div {
            style: "position: fixed; inset: 0; background: rgba(3, 7, 18, 0.85); backdrop-filter: blur(6px); display: flex; align-items: center; justify-content: center; z-index: 1000; padding: 20px;",
            onclick: move |_| on_close.call(()),
            div {
                style: "background: #0b1329; border: 1px solid rgba(0, 229, 255, 0.35); border-radius: 16px; width: 100%; max-width: 760px; max-height: 90vh; overflow-y: auto; box-shadow: 0 24px 64px rgba(0,0,0,0.8); display: flex; flex-direction: column;",
                onclick: move |e| e.stop_propagation(),

                // Header
                div {
                    style: "padding: 20px 24px; border-bottom: 1px solid rgba(80, 120, 200, 0.2); display: flex; justify-content: space-between; align-items: center; background: rgba(15, 23, 42, 0.6);",
                    div {
                        style: "display: flex; align-items: center; gap: 12px;",
                        span { style: "font-size: 24px;", "🏛️" }
                        div {
                            h2 { style: "margin: 0; font-size: 18px; font-weight: 700; color: #00e5ff;", "Heimatbasis & Sensor-Forschungszentrum" }
                            div { style: "font-size: 11px; color: #94a3b8; margin-top: 2px;", "Stufe {base.level} • Sektor-Position: ({base.position.x as i32}, {base.position.y as i32})" }
                        }
                    }
                    button {
                        style: "background: rgba(255, 255, 255, 0.08); hover:background: rgba(255, 255, 255, 0.2); border: none; color: #cbd5e1; font-size: 16px; width: 32px; height: 32px; border-radius: 50%; cursor: pointer; display: flex; align-items: center; justify-content: center;",
                        onclick: move |_| on_close.call(()),
                        "✕"
                    }
                }

                // Body
                div {
                    style: "padding: 24px; display: flex; flex-direction: column; gap: 20px;",

                    // 1. Bot & AI Identity Management Panel
                    div {
                        style: "background: #0f172a; border: 1px solid rgba(0, 229, 255, 0.25); border-radius: 12px; padding: 18px;",
                        div {
                            style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;",
                            div {
                                style: "display: flex; align-items: center; gap: 8px;",
                                span { style: "font-size: 16px;", "🤖" }
                                span { style: "font-size: 14px; font-weight: 700; color: #00e5ff;", "Heimatbasis-AI Steuerung" }
                                if active_base_agent.is_some() {
                                    span { style: "font-size: 10px; padding: 2px 6px; border-radius: 4px; background: rgba(0, 230, 118, 0.15); color: #00e676; border: 1px solid rgba(0, 230, 118, 0.3); font-weight: 600;", "✔ Gekoppelt" }
                                } else {
                                    span { style: "font-size: 10px; padding: 2px 6px; border-radius: 4px; background: rgba(250, 204, 21, 0.15); color: #fde047; border: 1px solid rgba(250, 204, 21, 0.3); font-weight: 600;", "Wartet auf Bot-Kopplung" }
                                }
                            }
                            div {
                                style: "display: flex; align-items: center; gap: 8px;",
                                if player.has_api_connected {
                                    span { style: "color: #00e676; font-size: 12px;", "●" }
                                    span { style: "font-size: 11px; color: #cbd5e1;", "API aktiv" }
                                    if is_api_recently_active {
                                        span { style: "font-size: 10px; padding: 1px 5px; border-radius: 3px; background: rgba(0, 230, 118, 0.15); color: #00e676; font-weight: 600;", "Aktiv" }
                                    }
                                } else {
                                    span { style: "color: #94a3b8; font-size: 12px;", "○" }
                                    span { style: "font-size: 11px; color: #94a3b8;", "Keine Bot-Aktivität" }
                                }
                            }
                        }

                        if let Some(ref agent) = active_base_agent {
                            {
                                let agent_hex = agent.agent_identity.to_hex();
                                let a_hex_copy = agent_hex.clone();
                                rsx! {
                                    div {
                                        style: "display: flex; flex-direction: column; gap: 10px; background: rgba(0, 229, 255, 0.04); border: 1px solid rgba(0, 229, 255, 0.15); border-radius: 8px; padding: 12px;",
                                        div {
                                            style: "display: flex; justify-content: space-between; align-items: center;",
                                            div {
                                                span { style: "font-size: 12px; font-weight: 700; color: #f1f5f9;", "{agent.label}" }
                                                span { style: "font-size: 11px; color: #94a3b8; margin-left: 8px;", "(Basis-KI: steuert alle Drohnen in Basisreichweite)" }
                                            }
                                            button {
                                                style: "background: rgba(239, 68, 68, 0.15); hover:background: rgba(239, 68, 68, 0.28); color: #ef4444; border: 1px solid rgba(239, 68, 68, 0.4); padding: 4px 10px; border-radius: 5px; font-size: 11px; font-weight: 600; cursor: pointer;",
                                                onclick: revoke_base_ai,
                                                "🛑 Kopplung aufheben"
                                            }
                                        }
                                        div {
                                            style: "display: flex; gap: 8px; align-items: center;",
                                            span { style: "font-size: 11px; color: #64748b;", "Identität:" }
                                            input {
                                                readonly: true,
                                                style: "flex: 1; background: #020617; border: 1px solid #1e293b; color: #38bdf8; padding: 6px 10px; border-radius: 5px; font-family: monospace; font-size: 11px; outline: none;",
                                                value: "{agent_hex}",
                                            }
                                            button {
                                                style: if copied() {
                                                    "background: #059669; color: #fff; border: none; padding: 6px 12px; border-radius: 5px; font-weight: 700; font-size: 11px; cursor: pointer;"
                                                } else {
                                                    "background: #0284c7; hover:background: #0369a1; color: #fff; border: none; padding: 6px 12px; border-radius: 5px; font-weight: 700; font-size: 11px; cursor: pointer;"
                                                },
                                                onclick: move |_| {
                                                    #[cfg(target_arch = "wasm32")]
                                                    if let Some(w) = web_sys::window() {
                                                        let _ = w.navigator().clipboard().write_text(&a_hex_copy);
                                                        copied.set(true);
                                                    }
                                                },
                                                if copied() { "✔ Kopiert" } else { "📋 Kopieren" }
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            div {
                                style: "display: flex; flex-direction: column; gap: 10px;",
                                div {
                                    style: "font-size: 11px; color: #94a3b8; line-height: 1.4;",
                                    "Starte deinen Bot mit " code { style: "color: #38bdf8;", "cargo run" } ". Kopiere die in der Bot-Konsole ausgegebene " strong { "Identität" } " hier hinein, um den Bot dauerhaft an deine Heimatbasis zu binden."
                                }
                                div {
                                    style: "display: flex; gap: 8px;",
                                    input {
                                        placeholder: "Bot-Identität eingeben (64-Zeichen Hex aus Konsole)...",
                                        style: "flex: 2; background: #020617; border: 1px solid #1e293b; color: #f1f5f9; padding: 8px 12px; border-radius: 6px; font-family: monospace; font-size: 11px; outline: none;",
                                        value: "{input_identity}",
                                        oninput: move |e| input_identity.set(e.value()),
                                    }
                                    input {
                                        placeholder: "Bezeichnung (z.B. Heimatbasis AI)",
                                        style: "flex: 1; background: #020617; border: 1px solid #1e293b; color: #f1f5f9; padding: 8px 12px; border-radius: 6px; font-size: 11px; outline: none;",
                                        value: "{input_label}",
                                        oninput: move |e| input_label.set(e.value()),
                                    }
                                    button {
                                        style: "background: #0284c7; hover:background: #0369a1; color: #fff; border: none; padding: 8px 16px; border-radius: 6px; font-weight: 700; font-size: 12px; cursor: pointer; white-space: nowrap; transition: all 0.2s;",
                                        onclick: bind_base_ai,
                                        "🔗 Bot binden"
                                    }
                                }
                            }
                        }

                        if let Some(ref err) = bind_error() {
                            div {
                                style: "color: #f87171; font-size: 11px; margin-top: 8px;",
                                "Fehler: {err}"
                            }
                        }
                    }

                    // 2. Stationsstatus Section
                    div {
                        style: "background: rgba(15, 23, 42, 0.95); border: 1px solid rgba(80, 120, 200, 0.25); border-radius: 12px; padding: 18px;",
                        h3 { style: "margin: 0 0 14px 0; font-size: 15px; font-weight: 700; color: #00e5ff;", "🏠 Stationsstatus" }
                        div {
                            style: "display: grid; grid-template-columns: repeat(4, 1fr); gap: 10px;",
                            div {
                                style: "padding: 12px; background: rgba(255, 255, 255, 0.03); border-radius: 6px;",
                                div { style: "font-size: 11px; color: #8da2c0;", "GELAGERTES ERZ" }
                                div { style: "font-size: 20px; font-weight: 700; color: #ffd700; margin-top: 2px;", "{base.stored_ore}" }
                            }
                            div {
                                style: "padding: 12px; background: rgba(255, 255, 255, 0.03); border-radius: 6px;",
                                div { style: "font-size: 11px; color: #8da2c0;", "ENERGIE" }
                                div { style: "font-size: 20px; font-weight: 700; color: #00e5ff; margin-top: 2px;", "{base.stored_energy}" }
                            }
                            div {
                                style: "padding: 12px; background: rgba(255, 255, 255, 0.03); border-radius: 6px;",
                                div { style: "font-size: 11px; color: #8da2c0;", "FUNK-REICHWEITE" }
                                div { style: "font-size: 20px; font-weight: 700; color: #38bdf8; margin-top: 2px;", "{base.signal_radius as u32} px" }
                            }
                            div {
                                style: "padding: 12px; background: rgba(255, 255, 255, 0.03); border-radius: 6px;",
                                div { style: "font-size: 11px; color: #8da2c0;", "GESAMTPUNKTE" }
                                div { style: "font-size: 20px; font-weight: 700; color: #00e676; margin-top: 2px;", "{player.score}" }
                            }
                        }
                    }

                    // 3. Sensor-Forschungszentrum Section
                    div {
                        style: "background: rgba(15, 23, 42, 0.95); border: 1px solid rgba(0, 229, 255, 0.35); border-radius: 12px; padding: 18px;",
                        div {
                            style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 6px;",
                            h3 { style: "margin: 0; font-size: 15px; font-weight: 700; color: #00e5ff;", "🧪 Sensor-Forschungszentrum" }
                            span {
                                style: "font-size: 10px; font-weight: 700; padding: 2px 6px; border-radius: 4px; background: rgba(239, 68, 68, 0.2); color: #fca5a5; border: 1px solid rgba(239, 68, 68, 0.4);",
                                "⚠️ 50% Reset-Risiko"
                            }
                        }
                        p {
                            style: "margin: 0 0 14px 0; font-size: 11px; color: #94a3b8; line-height: 1.3;",
                            "100 Rätsel pro Stufe • Steigende Erzkosten • Falsche Antworten bergen 50% Durchbrenn-Risiko!"
                        }

                        // Track 1
                        div {
                            style: "margin-bottom: 12px; background: rgba(255, 255, 255, 0.02); padding: 12px; border-radius: 8px; border: 1px solid rgba(80, 120, 200, 0.15);",
                            div {
                                style: "display: flex; justify-content: space-between; font-size: 12px; margin-bottom: 4px;",
                                span { style: "color: #cbd5e1; font-weight: 600;", "🛸 Drohnen-Scan (Sortieren)" }
                                span { style: "color: #00e5ff; font-weight: 700;", "Stufe {base.drone_scanner_level}/5 ({150 + (base.drone_scanner_level.saturating_sub(1) as u32 * 50)} px)" }
                            }
                            div {
                                style: "width: 100%; height: 8px; background: #0f172a; border-radius: 4px; overflow: hidden; position: relative;",
                                div {
                                    style: "height: 100%; width: {base.drone_scanner_progress}%; background: linear-gradient(90deg, #0284c7, #00e5ff); border-radius: 4px; transition: width 0.3s;"
                                }
                            }
                            div {
                                style: "display: flex; justify-content: space-between; font-size: 10px; color: #64748b; margin-top: 3px;",
                                span { "Fortschritt: {base.drone_scanner_progress}%" }
                                span { "{cost_t1_text}" }
                            }
                        }

                        // Track 2
                        div {
                            style: "margin-bottom: 12px; background: rgba(255, 255, 255, 0.02); padding: 12px; border-radius: 8px; border: 1px solid rgba(80, 120, 200, 0.15);",
                            div {
                                style: "display: flex; justify-content: space-between; font-size: 12px; margin-bottom: 4px;",
                                span { style: "color: #cbd5e1; font-weight: 600;", "☄️ Basen- & Asteroiden (Distanz)" }
                                span { style: "color: #38bdf8; font-weight: 700;", "Stufe {base.base_scanner_level}/5 ({300 + (base.base_scanner_level.saturating_sub(1) as u32 * 100)} px)" }
                            }
                            div {
                                style: "width: 100%; height: 8px; background: #0f172a; border-radius: 4px; overflow: hidden; position: relative;",
                                div {
                                    style: "height: 100%; width: {base.base_scanner_progress}%; background: linear-gradient(90deg, #0369a1, #38bdf8); border-radius: 4px; transition: width 0.3s;"
                                }
                            }
                            div {
                                style: "display: flex; justify-content: space-between; font-size: 10px; color: #64748b; margin-top: 3px;",
                                span { "Fortschritt: {base.base_scanner_progress}%" }
                                span { "{cost_t2_text}" }
                            }
                        }

                        // Track 3
                        div {
                            style: "margin-bottom: 12px; background: rgba(255, 255, 255, 0.02); padding: 12px; border-radius: 8px; border: 1px solid rgba(80, 120, 200, 0.15);",
                            div {
                                style: "display: flex; justify-content: space-between; font-size: 12px; margin-bottom: 4px;",
                                span { style: "color: #cbd5e1; font-weight: 600;", "📡 Radar-Horizont (Denoise)" }
                                span { style: "color: #a855f7; font-weight: 700;", "Stufe {base.radar_horizon_level}/5 ({600 + (base.radar_horizon_level.saturating_sub(1) as u32 * 150)} px)" }
                            }
                            div {
                                style: "width: 100%; height: 8px; background: #0f172a; border-radius: 4px; overflow: hidden; position: relative;",
                                div {
                                    style: "height: 100%; width: {base.radar_horizon_progress}%; background: linear-gradient(90deg, #7e22ce, #c084fc); border-radius: 4px; transition: width 0.3s;"
                                }
                            }
                            div {
                                style: "display: flex; justify-content: space-between; font-size: 10px; color: #64748b; margin-top: 3px;",
                                span { "Fortschritt: {base.radar_horizon_progress}%" }
                                span { "{cost_t3_text}" }
                            }
                        }

                        // Active Riddle Status
                        if let Some(ref r) = current_riddle {
                            div {
                                style: "padding: 10px 14px; background: rgba(250, 204, 21, 0.1); border: 1px solid rgba(250, 204, 21, 0.3); border-radius: 6px; font-size: 12px; color: #fef08a;",
                                "⏳ Aktives Rätsel: Track {r.track} (Ziel-Stufe {r.target_level}) • Wird in Zed gelöst."
                            }
                        }
                    }

                    // 3. Team- & Flottenlackierung Panel
                    div {
                        style: "background: #0f172a; border: 1px solid rgba(80, 120, 200, 0.25); border-radius: 12px; padding: 18px;",
                        div {
                            style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 14px;",
                            div {
                                style: "display: flex; align-items: center; gap: 8px;",
                                span { style: "font-size: 16px;", "🎨" }
                                span { style: "font-size: 14px; font-weight: 700; color: #00e5ff;", "Schiffs- & Stationslackierung" }
                            }
                            div {
                                style: "display: flex; align-items: center; gap: 8px;",
                                span {
                                    style: "width: 24px; height: 24px; border-radius: 50%; background: linear-gradient(135deg, hsl({edit_primary_hue}, 90%, 55%) 50%, hsl({edit_secondary_hue}, 90%, 55%) 50%); border: 2px solid #ffffff; box-shadow: 0 0 10px hsl({edit_primary_hue}, 90%, 55%);"
                                }
                            }
                        }

                        div {
                            style: "display: flex; flex-direction: column; gap: 14px;",

                            // Primary Hue Slider
                            div {
                                div { style: "display: flex; justify-content: space-between; font-size: 12px; color: #cbd5e1; margin-bottom: 6px;",
                                    span { "Primärfarbe (Rumpf & Kern):" }
                                    span { style: "font-weight: 700; color: hsl({edit_primary_hue}, 90%, 55%);", "{edit_primary_hue}°" }
                                }
                                div { style: "display: flex; align-items: center; gap: 10px;",
                                    input {
                                        r#type: "range",
                                        min: "0",
                                        max: "359",
                                        value: "{edit_primary_hue}",
                                        style: "flex: 1; cursor: pointer; accent-color: hsl({edit_primary_hue}, 90%, 55%); height: 6px;",
                                        oninput: move |e| {
                                            if let Ok(v) = e.value().parse::<u16>() {
                                                edit_primary_hue.set(v);
                                            }
                                        }
                                    }
                                }
                            }

                            // Secondary Hue Slider
                            div {
                                div { style: "display: flex; justify-content: space-between; font-size: 12px; color: #cbd5e1; margin-bottom: 6px;",
                                    span { "Sekundärfarbe (Flügel & Akzente):" }
                                    span { style: "font-weight: 700; color: hsl({edit_secondary_hue}, 90%, 55%);", "{edit_secondary_hue}°" }
                                }
                                div { style: "display: flex; align-items: center; gap: 10px;",
                                    input {
                                        r#type: "range",
                                        min: "0",
                                        max: "359",
                                        value: "{edit_secondary_hue}",
                                        style: "flex: 1; cursor: pointer; accent-color: hsl({edit_secondary_hue}, 90%, 55%); height: 6px;",
                                        oninput: move |e| {
                                            if let Ok(v) = e.value().parse::<u16>() {
                                                edit_secondary_hue.set(v);
                                            }
                                        }
                                    }
                                }
                            }

                            div {
                                style: "display: flex; justify-content: space-between; align-items: center; margin-top: 6px;",
                                div {
                                    if let Some(ref msg) = save_color_msg() {
                                        span { style: "font-size: 12px; color: #00e676; font-weight: 600;", "{msg}" }
                                    }
                                }
                                button {
                                    style: "padding: 8px 18px; border-radius: 6px; background: linear-gradient(135deg, #0284c7, #0369a1); border: 1px solid #38bdf8; color: #ffffff; font-size: 12px; font-weight: 700; cursor: pointer; transition: all 0.2s;",
                                    disabled: is_saving_colors(),
                                    onclick: move |_| {
                                        if let Some(conn) = (ctx.connection)() {
                                            is_saving_colors.set(true);
                                            let h1 = edit_primary_hue();
                                            let h2 = edit_secondary_hue();
                                            let res = conn.reducers.update_player_colors(h1, h2);
                                            is_saving_colors.set(false);
                                            if res.is_ok() {
                                                save_color_msg.set(Some("✔ Farbschema aktualisiert!".to_string()));
                                            } else {
                                                save_color_msg.set(Some("⚠️ Fehler beim Speichern".to_string()));
                                            }
                                        }
                                    },
                                    if is_saving_colors() { "Speichern..." } else { "Farbschema speichern" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ==========================================
// Scoreboard & Achievements Modal
// ==========================================

#[component]
fn ScoreboardModal(
    my_identity: Option<Identity>,
    players: Vec<Player>,
    achievements: Vec<PlayerAchievement>,
    on_close: EventHandler<()>,
) -> Element {
    let mut sorted_players = players;
    sorted_players.sort_by(|a, b| b.score.cmp(&a.score));

    let milestones = [
        (
            "BASE_DEPLOYED",
            "🏛️",
            "Pionier",
            "Heimatbasis im All errichtet",
        ),
        (
            "FIRST_ORE",
            "⛏️",
            "Asteroiden-Schürfer",
            "Erstes Erz abgebaut & geliefert",
        ),
        (
            "FIRST_RESEARCH",
            "🧪",
            "Akademiker",
            "Erstes Telemetrie-Forschungsrätsel gelöst",
        ),
        (
            "SCANNER_UPGRADE",
            "📡",
            "Scharfe Augen",
            "Scanner-Zweig auf Stufe 2 ausgebaut",
        ),
        (
            "FLEET_EXPANDED",
            "🛸",
            "Flottenadmiral",
            "Flotte auf mehrere Drohnen vergrößert",
        ),
    ];

    rsx! {
        div {
            style: "position: fixed; inset: 0; background: rgba(3, 7, 18, 0.85); backdrop-filter: blur(6px); display: flex; align-items: center; justify-content: center; z-index: 1000; padding: 20px;",
            onclick: move |_| on_close.call(()),
            div {
                style: "background: #0b1329; border: 1px solid rgba(80, 120, 200, 0.4); border-radius: 16px; width: 100%; max-width: 820px; max-height: 90vh; overflow-y: auto; box-shadow: 0 24px 64px rgba(0,0,0,0.8); display: flex; flex-direction: column;",
                onclick: move |e| e.stop_propagation(),

                // Header
                div {
                    style: "padding: 20px 24px; border-bottom: 1px solid rgba(80, 120, 200, 0.2); display: flex; justify-content: space-between; align-items: center; background: rgba(15, 23, 42, 0.6);",
                    div {
                        style: "display: flex; align-items: center; gap: 10px;",
                        span { style: "font-size: 24px;", "🏆" }
                        h2 { style: "margin: 0; font-size: 18px; font-weight: 700; color: #ffd700;", "Sektor-Scoreboard & Erfolge" }
                    }
                    button {
                        style: "background: rgba(255, 255, 255, 0.08); hover:background: rgba(255, 255, 255, 0.2); border: none; color: #cbd5e1; font-size: 16px; width: 32px; height: 32px; border-radius: 50%; cursor: pointer; display: flex; align-items: center; justify-content: center;",
                        onclick: move |_| on_close.call(()),
                        "✕"
                    }
                }

                // Body
                div {
                    style: "padding: 24px; display: flex; flex-direction: column; gap: 24px;",

                    // Leaderboard Section
                    div {
                        h3 { style: "margin: 0 0 12px 0; font-size: 15px; font-weight: 700; color: #00e5ff;", "📊 Piloten-Rangliste" }
                        div {
                            style: "background: rgba(15, 23, 42, 0.6); border: 1px solid rgba(80, 120, 200, 0.2); border-radius: 10px; overflow: hidden;",
                            table {
                                style: "width: 100%; border-collapse: collapse; font-size: 13px; text-align: left;",
                                thead {
                                    style: "background: rgba(255, 255, 255, 0.03); color: #8da2c0; font-size: 11px; text-transform: uppercase; letter-spacing: 0.5px;",
                                    tr {
                                        th { style: "padding: 10px 16px; width: 60px;", "Rang" }
                                        th { style: "padding: 10px 16px;", "Pilot" }
                                        th { style: "padding: 10px 16px; text-align: right;", "Punkte" }
                                        th { style: "padding: 10px 16px; text-align: center; width: 140px;", "Erfolge" }
                                    }
                                }
                                tbody {
                                    for (rank, p) in sorted_players.iter().enumerate() {
                                        {
                                            let is_me = my_identity.map_or(false, |id| id == p.identity);
                                            let p_unlocked = achievements.iter().filter(|a| a.player == p.identity).count();
                                            rsx! {
                                                tr {
                                                    key: "{p.identity}",
                                                    style: if is_me {
                                                        "background: rgba(0, 229, 255, 0.08); border-bottom: 1px solid rgba(80, 120, 200, 0.15);"
                                                    } else {
                                                        "border-bottom: 1px solid rgba(80, 120, 200, 0.1);"
                                                    },
                                                    td {
                                                        style: "padding: 12px 16px; font-weight: 700;",
                                                        match rank {
                                                            0 => rsx! { span { style: "color: #ffd700; font-size: 14px;", "🥇 #1" } },
                                                            1 => rsx! { span { style: "color: #cbd5e1; font-size: 14px;", "🥈 #2" } },
                                                            2 => rsx! { span { style: "color: #cd7f32; font-size: 14px;", "🥉 #3" } },
                                                            _ => rsx! { span { style: "color: #64748b;", "#{rank + 1}" } },
                                                        }
                                                    }
                                                    td {
                                                        style: "padding: 12px 16px;",
                                                        div {
                                                            style: "display: flex; align-items: center; gap: 8px;",
                                                            span { style: "width: 10px; height: 10px; border-radius: 50%; background: linear-gradient(135deg, hsl({p.color_hue}, 90%, 55%) 50%, hsl({p.secondary_color_hue}, 90%, 55%) 50%); box-shadow: 0 0 6px hsl({p.color_hue}, 90%, 55%);" }
                                                            span { style: "font-weight: 600; color: #f1f5f9;", "{p.name}" }
                                                            if is_me {
                                                                span { style: "font-size: 10px; padding: 1px 6px; border-radius: 4px; background: rgba(0, 229, 255, 0.2); color: #00e5ff; font-weight: 700;", "Du" }
                                                            }
                                                        }
                                                    }
                                                    td {
                                                        style: "padding: 12px 16px; text-align: right; font-weight: 700; color: #ffd700; font-mono;",
                                                        "{p.score} Pkt"
                                                    }
                                                    td {
                                                        style: "padding: 12px 16px; text-align: center;",
                                                        span {
                                                            style: "display: inline-flex; align-items: center; gap: 4px; padding: 2px 8px; border-radius: 12px; font-size: 11px; font-weight: 700; background: rgba(250, 204, 21, 0.1); color: #fde047; border: 1px solid rgba(250, 204, 21, 0.3);",
                                                            "⭐ {p_unlocked} / 5"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Personal Achievements Section
                    div {
                        h3 { style: "margin: 0 0 12px 0; font-size: 15px; font-weight: 700; color: #00e5ff;", "⭐ Deine Meilensteine & Erfolge" }
                        div {
                            style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 12px;",
                            for (key, icon, title, desc) in milestones {
                                {
                                    let is_unlocked = my_identity.map_or(false, |id| {
                                        achievements.iter().any(|a| a.player == id && a.achievement_key == key)
                                    });
                                    rsx! {
                                        div {
                                            key: "{key}",
                                            style: if is_unlocked {
                                                "background: linear-gradient(135deg, rgba(250, 204, 21, 0.08) 0%, rgba(15, 23, 42, 0.9) 100%); border: 1px solid rgba(250, 204, 21, 0.4); border-radius: 10px; padding: 14px; box-shadow: 0 4px 12px rgba(250, 204, 21, 0.06);"
                                            } else {
                                                "background: rgba(15, 23, 42, 0.5); border: 1px dashed rgba(255, 255, 255, 0.12); border-radius: 10px; padding: 14px; opacity: 0.55;"
                                            },
                                            div {
                                                style: "display: flex; align-items: center; justify-content: space-between; margin-bottom: 6px;",
                                                span { style: "font-size: 24px;", "{icon}" }
                                                if is_unlocked {
                                                    span { style: "font-size: 10px; font-weight: 700; padding: 2px 6px; border-radius: 4px; background: rgba(0, 230, 118, 0.2); color: #00e676; border: 1px solid rgba(0, 230, 118, 0.4);", "✔ FREIGESCHALTET" }
                                                } else {
                                                    span { style: "font-size: 10px; font-weight: 700; padding: 2px 6px; border-radius: 4px; background: rgba(255, 255, 255, 0.05); color: #94a3b8; border: 1px solid rgba(255, 255, 255, 0.1);", "🔒 GESPERRT" }
                                                }
                                            }
                                            div { style: "font-size: 13px; font-weight: 700; color: if is_unlocked { \"#ffd700\" } else { \"#cbd5e1\" }; margin-bottom: 4px;", "{title}" }
                                            div { style: "font-size: 11px; color: #94a3b8; line-height: 1.3;", "{desc}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
