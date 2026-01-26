pub mod config;
pub mod entity;
pub mod id;
pub mod index;
pub mod markdown;
pub mod paths;
pub mod repo;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[doc(hidden)]
pub mod __private {
    pub use serde;
}

// Re-export commonly used items from submodules for convenience
pub use config::{
    ConfigError, ConfigLoader, EmbeddingConfig, GitHubConfig, IxchelConfig, SharedConfig,
    StorageConfig, detect_github_token, find_project_config_dir, ixchel_config_dir,
    ixchel_data_dir, ixchel_home, ixchel_log_dir, ixchel_state_dir, load_config,
    load_shared_config,
};
pub use id::{
    IdError, id_from_key, id_from_key_with_length, id_from_parts, id_from_parts_with_length,
    id_random, id_random_with_length, parse_id,
};
