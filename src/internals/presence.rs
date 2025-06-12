use serde::{
  Deserialize,
  Serialize
};

/// The static path to the TOML config file for bot's presence data
pub const TOML_FILE: &str = if cfg!(feature = "production") {
  "presence.toml"
} else {
  "src/internals/assets/presence.toml"
};

#[derive(Serialize, Deserialize)]
pub struct Activity {
  pub name: String,
  pub url:  String
}

#[derive(Serialize, Deserialize)]
pub struct Presence {
  pub activities: Vec<Activity>
}

#[derive(Serialize, Deserialize)]
pub struct TomlConfig {
  pub presence: Presence
}

pub fn read_config() -> TomlConfig {
  let content = std::fs::read_to_string(TOML_FILE).expect("[TomlConfig] Error loading config file");
  toml::from_str(&content).expect("[TomlConfig] Error parsing config file")
}
