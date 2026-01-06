use helix_config::{ConfigError, ConfigLoader};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct DecisionConfig {
    #[serde(default = "default_strict")]
    pub strict: bool,
}

impl Default for DecisionConfig {
    fn default() -> Self {
        Self {
            strict: default_strict(),
        }
    }
}

const fn default_strict() -> bool {
    true
}

pub fn load_config() -> Result<DecisionConfig, ConfigError> {
    ConfigLoader::new("helix-decisions").load()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = DecisionConfig::default();
        assert!(config.strict);
    }
}
