use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
pub struct BotConfig {
    #[serde(default)]
    pub auth_token: String,
    #[serde(default = "default_server_url")]
    pub server_url: String,
    #[serde(default = "default_module_name")]
    pub module_name: String,
}

fn default_server_url() -> String {
    "http://127.0.0.1:3000".to_string()
}

fn default_module_name() -> String {
    "server".to_string()
}

impl BotConfig {
    pub fn load() -> Result<Self, String> {
        let path = find_config_file()?;
        let content = fs::read_to_string(&path)
            .map_err(|e| format!("Fehler beim Lesen von {:?}: {}", path, e))?;

        let config: BotConfig = toml::from_str(&content)
            .map_err(|e| format!("Fehler beim Parsen von {:?}: {}", path, e))?;

        let token = config.auth_token.trim();
        if token.is_empty()
            || token == "DEIN_TOKEN_HIER"
            || token == "YOUR_TOKEN_HERE"
            || token.contains("PASTE_YOUR_TOKEN")
        {
            print_token_help_message(&path);
            return Err("Ungültiger oder fehlender 'auth_token' in config.toml".to_string());
        }

        Ok(config)
    }
}

fn find_config_file() -> Result<PathBuf, String> {
    let candidates = [
        Path::new("config.toml"),
        Path::new("command_center/config.toml"),
        Path::new("bot/config.toml"),
        Path::new("../config.toml"),
    ];

    for candidate in candidates {
        if candidate.exists() {
            return Ok(candidate.to_path_buf());
        }
    }

    print_missing_file_help();
    Err("Datei 'config.toml' konnte nicht gefunden werden.".to_string())
}

fn print_token_help_message(config_path: &Path) {
    eprintln!("\n{}", "=".repeat(80));
    eprintln!("🚀 [SPACE BOT FEHLER] Kein gültiger Authentifizierungs-Token konfiguriert!");
    eprintln!("{}", "-".repeat(80));
    eprintln!("Dein Bot benötigt deinen persönlichen API-Identitäts-Token aus dem Spiel.");
    eprintln!();
    eprintln!("Schritte zur Behebung:");
    eprintln!("  1. Öffne das Spiel-Dashboard im Browser (client-web).");
    eprintln!(
        "  2. Falls noch nicht geschehen: Fliege mit dem Bauschiff hinaus und baue deine Heimatbasis."
    );
    eprintln!("  3. Klicke im Dashboard auf 'DEIN PERSÖNLICHER API-IDENTITÄTS-TOKEN' (Kopieren).");
    eprintln!("  4. Öffne die Datei '{:?}' in Zed.", config_path);
    eprintln!("  5. Trage deinen Token ein:");
    eprintln!("       auth_token = \"<dein_kopierter_token>\"");
    eprintln!("  6. Speichere die Datei und starte deinen Bot erneut mit: cargo run");
    eprintln!("{}\n", "=".repeat(80));
}

fn print_missing_file_help() {
    eprintln!("\n{}", "=".repeat(80));
    eprintln!("🚀 [SPACE BOT FEHLER] Datei 'config.toml' nicht gefunden!");
    eprintln!("{}", "-".repeat(80));
    eprintln!("Erstelle bitte eine Datei namens 'config.toml' mit folgendem Inhalt:");
    eprintln!();
    eprintln!("  server_url = \"http://127.0.0.1:3000\"");
    eprintln!("  module_name = \"server\"");
    eprintln!("  auth_token = \"<dein_kopierter_token>\"");
    eprintln!("{}\n", "=".repeat(80));
}
