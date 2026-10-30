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
        Self::load_with_path().map(|(cfg, _)| cfg)
    }

    pub fn load_with_path() -> Result<(Self, PathBuf), String> {
        let path = find_config_file()?;
        let content = fs::read_to_string(&path)
            .map_err(|e| format!("Fehler beim Lesen von {:?}: {}", path, e))?;

        let mut config: BotConfig = toml::from_str(&content)
            .map_err(|e| format!("Fehler beim Parsen von {:?}: {}", path, e))?;

        let token = config.auth_token.trim();
        if token.is_empty()
            || token == "DEIN_TOKEN_HIER"
            || token == "YOUR_TOKEN_HERE"
            || token.contains("PASTE_YOUR_TOKEN")
        {
            config.auth_token = String::new();
        }

        Ok((config, path))
    }

    /// Aktualisiert oder ergänzt `auth_token` in der angegebenen `config.toml`,
    /// ohne bestehende Kommentare oder andere Einstellungen zu überschreiben.
    pub fn save_token(path: &Path, token: &str) -> Result<(), String> {
        let content = if path.exists() {
            fs::read_to_string(path)
                .map_err(|e| format!("Fehler beim Lesen von {:?}: {}", path, e))?
        } else {
            String::new()
        };

        let new_line = format!("auth_token = \"{}\"", token);
        let mut replaced = false;
        let mut new_lines = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("auth_token") && trimmed.contains('=') {
                new_lines.push(new_line.clone());
                replaced = true;
            } else {
                new_lines.push(line.to_string());
            }
        }

        if !replaced {
            if !new_lines.is_empty() && !new_lines.last().map_or(false, |l| l.is_empty()) {
                new_lines.push(String::new());
            }
            new_lines.push(new_line);
        }

        let mut output = new_lines.join("\n");
        output.push('\n');

        fs::write(path, output)
            .map_err(|e| format!("Fehler beim Schreiben von {:?}: {}", path, e))?;

        Ok(())
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
