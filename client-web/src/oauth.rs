use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use sha2::{Digest, Sha256};

pub const DEFAULT_GITEA_URL: &str = "https://git.uhle.cloud";
pub const DEFAULT_CLIENT_ID: &str = "82cc597e-f313-4527-ab7b-16e2dfd1e78a";

pub fn gitea_url() -> String {
    option_env!("GITEA_URL")
        .unwrap_or(DEFAULT_GITEA_URL)
        .trim_end_matches('/')
        .to_string()
}

pub fn client_id() -> String {
    option_env!("GITEA_CLIENT_ID")
        .unwrap_or(DEFAULT_CLIENT_ID)
        .to_string()
}

/// Helper to percent-encode standard OAuth parameters
pub fn url_encode(input: &str) -> String {
    let mut encoded = String::with_capacity(input.len());
    for byte in input.bytes() {
        match byte {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            b' ' => {
                encoded.push_str("%20");
            }
            _ => {
                encoded.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    encoded
}

/// Generates a PKCE code verifier (cryptographically random 32 bytes URL-safe base64 encoded)
pub fn generate_code_verifier() -> String {
    let mut bytes = [0u8; 32];
    #[cfg(target_arch = "wasm32")]
    {
        let _ = getrandom::fill(&mut bytes);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        // Fallback for non-wasm compilation if needed
        for (i, b) in bytes.iter_mut().enumerate() {
            *b = (i as u8).wrapping_mul(31);
        }
    }
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Computes SHA-256 code challenge from a code verifier for PKCE
pub fn generate_code_challenge(verifier: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let hash = hasher.finalize();
    URL_SAFE_NO_PAD.encode(hash)
}

/// Generates a random state string for CSRF protection
pub fn generate_state() -> String {
    let mut bytes = [0u8; 16];
    #[cfg(target_arch = "wasm32")]
    {
        let _ = getrandom::fill(&mut bytes);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        for (i, b) in bytes.iter_mut().enumerate() {
            *b = (i as u8).wrapping_mul(17);
        }
    }
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Builds the full authorization URL for Gitea OAuth2 with OpenID Connect and PKCE
pub fn build_authorization_url(redirect_uri: &str, state: &str, code_challenge: &str) -> String {
    format!(
        "{}/login/oauth/authorize?client_id={}&redirect_uri={}&response_type=code&scope={}&code_challenge={}&code_challenge_method=S256&state={}",
        gitea_url(),
        url_encode(&client_id()),
        url_encode(redirect_uri),
        url_encode("openid profile email"),
        url_encode(code_challenge),
        url_encode(state),
    )
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    #[serde(default)]
    pub id_token: Option<String>,
    #[serde(default)]
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub expires_in: Option<u64>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Default)]
pub struct IdTokenClaims {
    pub iss: Option<String>,
    pub sub: Option<String>,
    pub preferred_username: Option<String>,
    pub name: Option<String>,
    pub email: Option<String>,
    pub exp: Option<u64>,
}

/// Parses the payload part of a JWT (without cryptographic verification, which SpacetimeDB handles server-side)
pub fn parse_id_token_claims(id_token: &str) -> Option<IdTokenClaims> {
    let parts: Vec<&str> = id_token.split('.').collect();
    if parts.len() < 2 {
        return None;
    }
    let payload_b64 = parts[1];
    // JWT base64 might need padding to be multiple of 4
    let mut padded = payload_b64.to_string();
    while padded.len() % 4 != 0 {
        padded.push('=');
    }
    // Try URL_SAFE or standard engine
    let bytes = base64::engine::general_purpose::URL_SAFE
        .decode(&padded)
        .or_else(|_| URL_SAFE_NO_PAD.decode(payload_b64))
        .ok()?;
    serde_json::from_slice::<IdTokenClaims>(&bytes).ok()
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;

/// Exchanges an authorization code for tokens via Gitea token endpoint
#[cfg(target_arch = "wasm32")]
pub async fn exchange_code(
    code: &str,
    code_verifier: &str,
    redirect_uri: &str,
) -> Result<TokenResponse, String> {
    let token_endpoint = format!("{}/login/oauth/access_token", gitea_url());

    let form_body = format!(
        "grant_type=authorization_code&client_id={}&code={}&code_verifier={}&redirect_uri={}",
        url_encode(&client_id()),
        url_encode(code),
        url_encode(code_verifier),
        url_encode(redirect_uri)
    );

    let window = web_sys::window().ok_or_else(|| "Kein Window verfügbar".to_string())?;

    let opts = web_sys::RequestInit::new();
    opts.set_method("POST");
    opts.set_body(&JsValue::from_str(&form_body));

    let headers = web_sys::Headers::new().map_err(|e| format!("{e:?}"))?;
    headers
        .set("Content-Type", "application/x-www-form-urlencoded")
        .map_err(|e| format!("{e:?}"))?;
    headers
        .set("Accept", "application/json")
        .map_err(|e| format!("{e:?}"))?;
    opts.set_headers(&headers);

    let request = web_sys::Request::new_with_str_and_init(&token_endpoint, &opts)
        .map_err(|e| format!("Fehler beim Erstellen des Requests: {e:?}"))?;

    let resp_val = wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("Netzwerkfehler beim Token-Austausch: {e:?}"))?;

    let resp: web_sys::Response = resp_val
        .dyn_into()
        .map_err(|e| format!("Unerwartete Antwort: {e:?}"))?;

    let status = resp.status();
    let text_promise = resp
        .text()
        .map_err(|e| format!("Fehler beim Lesen des Bodys: {e:?}"))?;
    let text_val = wasm_bindgen_futures::JsFuture::from(text_promise)
        .await
        .map_err(|e| format!("Fehler beim Lesen des Antworttexts: {e:?}"))?;
    let body_text = text_val.as_string().unwrap_or_default();

    if !resp.ok() {
        return Err(format!("Gitea Token-Fehler ({status}): {body_text}"));
    }

    #[cfg(target_arch = "wasm32")]
    {
        web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&format!(
            "[OAuth] Gitea token response status {status}, payload bytes: {}",
            body_text.len()
        )));
    }

    serde_json::from_str::<TokenResponse>(&body_text)
        .map_err(|e| format!("Fehler beim Parsen der Token-Antwort: {e}"))
}

/// Refreshes an expired token using the refresh_token
#[cfg(target_arch = "wasm32")]
pub async fn refresh_access_token(refresh_token: &str) -> Result<TokenResponse, String> {
    let token_endpoint = format!("{}/login/oauth/access_token", gitea_url());

    let form_body = format!(
        "grant_type=refresh_token&client_id={}&refresh_token={}",
        url_encode(&client_id()),
        url_encode(refresh_token)
    );

    let window = web_sys::window().ok_or_else(|| "Kein Window verfügbar".to_string())?;

    let opts = web_sys::RequestInit::new();
    opts.set_method("POST");
    opts.set_body(&JsValue::from_str(&form_body));

    let headers = web_sys::Headers::new().map_err(|e| format!("{e:?}"))?;
    headers
        .set("Content-Type", "application/x-www-form-urlencoded")
        .map_err(|e| format!("{e:?}"))?;
    headers
        .set("Accept", "application/json")
        .map_err(|e| format!("{e:?}"))?;
    opts.set_headers(&headers);

    let request = web_sys::Request::new_with_str_and_init(&token_endpoint, &opts)
        .map_err(|e| format!("Fehler beim Erstellen des Requests: {e:?}"))?;

    let resp_val = wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("Netzwerkfehler beim Token-Refresh: {e:?}"))?;

    let resp: web_sys::Response = resp_val
        .dyn_into()
        .map_err(|e| format!("Unerwartete Antwort: {e:?}"))?;

    let status = resp.status();
    let text_promise = resp
        .text()
        .map_err(|e| format!("Fehler beim Lesen des Bodys: {e:?}"))?;
    let text_val = wasm_bindgen_futures::JsFuture::from(text_promise)
        .await
        .map_err(|e| format!("Fehler beim Lesen des Antworttexts: {e:?}"))?;
    let body_text = text_val.as_string().unwrap_or_default();

    if !resp.ok() {
        return Err(format!(
            "Token Refresh fehlgeschlagen ({status}): {body_text}"
        ));
    }

    serde_json::from_str::<TokenResponse>(&body_text)
        .map_err(|e| format!("Fehler beim Parsen der Refresh-Antwort: {e}"))
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreateApiTokenRequest {
    pub role: Option<String>,
    pub entity_id: Option<u32>,
    pub label: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreateApiTokenResponse {
    pub identity: String,
    pub token: String,
}

/// Requests a permanent, long-lived API token for a sub-AI from the SpacetimeDB host
#[cfg(target_arch = "wasm32")]
pub async fn request_ai_token(
    stdb_url: &str,
    role: Option<&str>,
    entity_id: Option<u32>,
    label: Option<&str>,
) -> Result<CreateApiTokenResponse, String> {
    let endpoint = format!("{}/v1/identity/api-token", stdb_url.trim_end_matches('/'));
    let session_token = get_saved_token().ok_or("Nicht eingeloggt (kein Session-Token)")?;

    let payload = CreateApiTokenRequest {
        role: role.map(|s| s.to_string()),
        entity_id,
        label: label.map(|s| s.to_string()),
    };
    let json_body = serde_json::to_string(&payload)
        .map_err(|e| format!("Fehler beim Serialisieren des Requests: {e}"))?;

    let opts = web_sys::RequestInit::new();
    opts.set_method("POST");
    opts.set_body(&wasm_bindgen::JsValue::from_str(&json_body));

    let headers = web_sys::Headers::new().map_err(|e| format!("Header-Fehler: {e:?}"))?;
    headers
        .set("Content-Type", "application/json")
        .map_err(|e| format!("Header-Fehler: {e:?}"))?;
    headers
        .set("Authorization", &format!("Bearer {session_token}"))
        .map_err(|e| format!("Header-Fehler: {e:?}"))?;
    opts.set_headers(&headers);

    let request = web_sys::Request::new_with_str_and_init(&endpoint, &opts)
        .map_err(|e| format!("Netzwerkfehler beim Erstellen des Requests: {e:?}"))?;

    let window = web_sys::window().ok_or("Kein Window-Objekt verfügbar")?;
    let resp_val = wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("Netzwerkfehler beim Abrufen des AI-Tokens: {e:?}"))?;

    let resp: web_sys::Response = resp_val
        .dyn_into()
        .map_err(|e| format!("Unerwartete Antwort: {e:?}"))?;

    let status = resp.status();
    let text_promise = resp
        .text()
        .map_err(|e| format!("Fehler beim Lesen des Bodys: {e:?}"))?;
    let text_val = wasm_bindgen_futures::JsFuture::from(text_promise)
        .await
        .map_err(|e| format!("Fehler beim Lesen des Antworttexts: {e:?}"))?;
    let body_text = text_val.as_string().unwrap_or_default();

    if !resp.ok() {
        return Err(format!(
            "AI-Token Generierung fehlgeschlagen ({status}): {body_text}"
        ));
    }

    serde_json::from_str::<CreateApiTokenResponse>(&body_text)
        .map_err(|e| format!("Fehler beim Parsen der AI-Token-Antwort: {e}"))
}

// ==========================================
// Local Storage Session Management
// ==========================================

pub const KEY_STDB_TOKEN: &str = "spacetimedb_token";
pub const KEY_GITEA_ACCESS_TOKEN: &str = "gitea_access_token";
pub const KEY_GITEA_REFRESH_TOKEN: &str = "gitea_refresh_token";
pub const KEY_GITEA_USERNAME: &str = "gitea_username";
pub const KEY_PKCE_VERIFIER: &str = "oauth_code_verifier";
pub const KEY_OAUTH_STATE: &str = "oauth_state";

#[cfg(target_arch = "wasm32")]
pub fn get_local_storage() -> Option<web_sys::Storage> {
    web_sys::window().and_then(|w| w.local_storage().ok().flatten())
}

#[cfg(target_arch = "wasm32")]
pub fn get_session_storage() -> Option<web_sys::Storage> {
    web_sys::window().and_then(|w| w.session_storage().ok().flatten())
}

pub fn save_session(tokens: &TokenResponse) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(storage) = get_local_storage() {
            let has_id = tokens.id_token.is_some();
            // Prefer id_token for SpacetimeDB connection, fallback to access_token
            let stdb_tok = tokens.id_token.as_ref().unwrap_or(&tokens.access_token);
            let _ = storage.set_item(KEY_STDB_TOKEN, stdb_tok);
            let _ = storage.set_item(KEY_GITEA_ACCESS_TOKEN, &tokens.access_token);
            if let Some(ref rt) = tokens.refresh_token {
                let _ = storage.set_item(KEY_GITEA_REFRESH_TOKEN, rt);
            }
            if let Some(ref id_tok) = tokens.id_token {
                if let Some(claims) = parse_id_token_claims(id_tok) {
                    if let Some(username) = claims.preferred_username.or(claims.name).or(claims.sub)
                    {
                        let _ = storage.set_item(KEY_GITEA_USERNAME, &username);
                    }
                }
            }
            web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&format!(
                "[OAuth] Session saved: has_id_token={}, access_token_len={}, stdb_tok_len={}",
                has_id,
                tokens.access_token.len(),
                stdb_tok.len()
            )));
        }
    }
}

pub fn clear_session() {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(storage) = get_local_storage() {
            let _ = storage.remove_item(KEY_STDB_TOKEN);
            let _ = storage.remove_item(KEY_GITEA_ACCESS_TOKEN);
            let _ = storage.remove_item(KEY_GITEA_REFRESH_TOKEN);
            let _ = storage.remove_item(KEY_GITEA_USERNAME);
        }
    }
}

pub fn get_saved_token() -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        get_local_storage().and_then(|s| s.get_item(KEY_STDB_TOKEN).ok().flatten())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        None
    }
}

pub fn get_saved_username() -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        get_local_storage().and_then(|s| s.get_item(KEY_GITEA_USERNAME).ok().flatten())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        None
    }
}

pub fn get_saved_refresh_token() -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        get_local_storage().and_then(|s| s.get_item(KEY_GITEA_REFRESH_TOKEN).ok().flatten())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        None
    }
}

pub fn get_current_origin() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(w) = web_sys::window() {
            if let Ok(hostname) = w.location().hostname() {
                if hostname == "localhost" || hostname == "127.0.0.1" {
                    let port = w.location().port().unwrap_or_default();
                    if port.is_empty() {
                        return "http://127.0.0.1:8081".to_string();
                    } else {
                        return format!("http://127.0.0.1:{}", port);
                    }
                }
            }
            if let Ok(origin) = w.location().origin() {
                return origin;
            }
        }
        "http://127.0.0.1:8081".to_string()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        "http://127.0.0.1:8081".to_string()
    }
}
