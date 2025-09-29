use serde::Deserialize;

/// The static path to the TOML config file for bot's presence data
pub const TOML_FILE: &str = if cfg!(feature = "production") {
  "presence.toml"
} else {
  "assets/presence.toml"
};

#[derive(Deserialize)]
pub struct Activity {
  pub name: String,
  pub url:  String
}

#[derive(Deserialize)]
pub struct TomlConfig {
  pub activity: Activity
}

pub fn read_config() -> TomlConfig {
  let content = std::fs::read_to_string(TOML_FILE).expect("[TomlConfig] Error loading config file");
  toml::from_str(&content).expect("[TomlConfig] Error parsing config file")
}
