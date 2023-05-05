use figment::providers::{Format, Toml};
use figment::Figment;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct GoogleConfig {
    pub client_id: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub google: GoogleConfig,
}

pub fn read(path: &str) -> Result<Config, String> {
    let config: Config = Figment::new()
        .merge(Toml::file(path))
        .extract()
        .map_err(|e| e.to_string())?;

    if config.google.client_id.is_empty() {
        return Err("Must include a twitch client id".to_string());
    }

    Ok(config)
}
