use crate::types::errors::{ConfigError, ConfigResult};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Main configuration for the Tapu Simu simulator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Path to the Pokemon Showdown data directory
    pub data_path: PathBuf,
    /// Logging configuration
    pub logging: LoggingConfig,
    /// Performance configuration
    pub performance: PerformanceConfig,
    /// Battle configuration
    pub battle: BattleConfig,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Enable debug logging
    pub debug: bool,
    /// Enable instruction logging
    pub log_instructions: bool,
    /// Enable move choice logging
    pub log_moves: bool,
    /// Enable damage calculation logging
    pub log_damage: bool,
    /// Maximum log file size in MB
    pub max_file_size_mb: u64,
    /// Log file path (None for stdout)
    pub log_file: Option<PathBuf>,
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Enable parallel battle execution
    pub parallel_battles: bool,
    /// Number of threads for parallel execution (0 = auto)
    pub thread_count: usize,
    /// Enable move caching
    pub cache_moves: bool,
    /// Enable Pokemon data caching
    pub cache_pokemon: bool,
    /// Maximum cache size in MB
    pub max_cache_size_mb: u64,
}

/// Battle-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleConfig {
    /// Default maximum turns per battle
    pub default_max_turns: u32,
    /// Enable strict format validation
    pub strict_format_validation: bool,
    /// Enable damage range randomization
    pub damage_randomization: bool,
    /// Random seed (None for system entropy)
    pub random_seed: Option<u64>,
    /// Enable undo support for instructions
    pub enable_undo: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            data_path: Self::default_data_path(),
            logging: LoggingConfig::default(),
            performance: PerformanceConfig::default(),
            battle: BattleConfig::default(),
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            debug: false,
            log_instructions: false,
            log_moves: false,
            log_damage: false,
            max_file_size_mb: 100,
            log_file: None,
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            parallel_battles: true,
            thread_count: 0, // Auto-detect
            cache_moves: true,
            cache_pokemon: true,
            max_cache_size_mb: 256,
        }
    }
}

impl Default for BattleConfig {
    fn default() -> Self {
        Self {
            default_max_turns: 1000,
            strict_format_validation: true,
            damage_randomization: true,
            random_seed: None,
            enable_undo: true,
        }
    }
}

impl Config {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Load configuration from a file
    pub fn from_file(path: impl AsRef<Path>) -> ConfigResult<Self> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::FileNotFound { path: path.to_path_buf() })?;
        
        let config: Config = serde_json::from_str(&content)?;
        Ok(config)
    }

    /// Save configuration to a file
    pub fn save_to_file(&self, path: impl AsRef<Path>) -> ConfigResult<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path.as_ref(), content)
            .map_err(|e| ConfigError::FileNotFound { path: path.as_ref().to_path_buf() })?;
        Ok(())
    }

    /// Create a configuration builder
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::new()
    }

    /// Get the default data path
    fn default_data_path() -> PathBuf {
        // Try to find data directory relative to the current executable
        let mut path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        path.push("data");
        path.push("ps-extracted");
        
        // If that doesn't exist, try relative to the source directory
        if !path.exists() {
            path = PathBuf::from("data/ps-extracted");
        }
        
        path
    }

    /// Validate the configuration
    pub fn validate(&self) -> ConfigResult<()> {
        // Check if data path exists
        if !self.data_path.exists() {
            return Err(ConfigError::InvalidValue {
                field: "data_path".to_string(),
                value: self.data_path.to_string_lossy().to_string(),
            });
        }

        // Validate performance settings
        if self.performance.max_cache_size_mb == 0 {
            return Err(ConfigError::InvalidValue {
                field: "performance.max_cache_size_mb".to_string(),
                value: "0".to_string(),
            });
        }

        // Validate battle settings
        if self.battle.default_max_turns == 0 {
            return Err(ConfigError::InvalidValue {
                field: "battle.default_max_turns".to_string(),
                value: "0".to_string(),
            });
        }

        Ok(())
    }

    /// Get effective thread count (resolving 0 to actual CPU count)
    pub fn effective_thread_count(&self) -> usize {
        if self.performance.thread_count == 0 {
            std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1)
        } else {
            self.performance.thread_count
        }
    }
}

/// Builder for creating configurations
#[derive(Debug, Clone)]
pub struct ConfigBuilder {
    config: Config,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: Config::default(),
        }
    }

    /// Set the data path
    pub fn data_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.config.data_path = path.into();
        self
    }

    /// Enable debug logging
    pub fn debug(mut self, enable: bool) -> Self {
        self.config.logging.debug = enable;
        self
    }

    /// Enable instruction logging
    pub fn log_instructions(mut self, enable: bool) -> Self {
        self.config.logging.log_instructions = enable;
        self
    }

    /// Enable move logging
    pub fn log_moves(mut self, enable: bool) -> Self {
        self.config.logging.log_moves = enable;
        self
    }

    /// Enable damage logging
    pub fn log_damage(mut self, enable: bool) -> Self {
        self.config.logging.log_damage = enable;
        self
    }

    /// Set log file
    pub fn log_file(mut self, path: impl Into<PathBuf>) -> Self {
        self.config.logging.log_file = Some(path.into());
        self
    }

    /// Enable parallel battles
    pub fn parallel_battles(mut self, enable: bool) -> Self {
        self.config.performance.parallel_battles = enable;
        self
    }

    /// Set thread count (0 for auto)
    pub fn thread_count(mut self, count: usize) -> Self {
        self.config.performance.thread_count = count;
        self
    }

    /// Enable move caching
    pub fn cache_moves(mut self, enable: bool) -> Self {
        self.config.performance.cache_moves = enable;
        self
    }

    /// Enable Pokemon caching
    pub fn cache_pokemon(mut self, enable: bool) -> Self {
        self.config.performance.cache_pokemon = enable;
        self
    }

    /// Set maximum cache size
    pub fn max_cache_size(mut self, size_mb: u64) -> Self {
        self.config.performance.max_cache_size_mb = size_mb;
        self
    }

    /// Set default maximum turns
    pub fn max_turns(mut self, turns: u32) -> Self {
        self.config.battle.default_max_turns = turns;
        self
    }

    /// Enable strict format validation
    pub fn strict_validation(mut self, enable: bool) -> Self {
        self.config.battle.strict_format_validation = enable;
        self
    }

    /// Enable damage randomization
    pub fn damage_randomization(mut self, enable: bool) -> Self {
        self.config.battle.damage_randomization = enable;
        self
    }

    /// Set random seed
    pub fn random_seed(mut self, seed: u64) -> Self {
        self.config.battle.random_seed = Some(seed);
        self
    }

    /// Enable undo support
    pub fn enable_undo(mut self, enable: bool) -> Self {
        self.config.battle.enable_undo = enable;
        self
    }

    /// Build the configuration
    pub fn build(self) -> ConfigResult<Config> {
        self.config.validate()?;
        Ok(self.config)
    }

    /// Build the configuration without validation
    pub fn build_unchecked(self) -> Config {
        self.config
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Environment-based configuration loading
impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> ConfigResult<Self> {
        let mut config = Config::default();

        // Data path from environment
        if let Ok(data_path) = std::env::var("TAPU_SIMU_DATA_PATH") {
            config.data_path = PathBuf::from(data_path);
        }

        // Logging settings
        if let Ok(debug) = std::env::var("TAPU_SIMU_DEBUG") {
            config.logging.debug = debug.parse().unwrap_or(false);
        }

        if let Ok(log_instructions) = std::env::var("TAPU_SIMU_LOG_INSTRUCTIONS") {
            config.logging.log_instructions = log_instructions.parse().unwrap_or(false);
        }

        if let Ok(log_file) = std::env::var("TAPU_SIMU_LOG_FILE") {
            config.logging.log_file = Some(PathBuf::from(log_file));
        }

        // Performance settings
        if let Ok(thread_count) = std::env::var("TAPU_SIMU_THREADS") {
            config.performance.thread_count = thread_count.parse().unwrap_or(0);
        }

        if let Ok(max_turns) = std::env::var("TAPU_SIMU_MAX_TURNS") {
            config.battle.default_max_turns = max_turns.parse().unwrap_or(1000);
        }

        if let Ok(seed) = std::env::var("TAPU_SIMU_SEED") {
            config.battle.random_seed = seed.parse().ok();
        }

        config.validate()?;
        Ok(config)
    }

    /// Get configuration with environment overrides
    pub fn with_env_overrides(mut self) -> ConfigResult<Self> {
        // Apply environment overrides on top of existing config
        if let Ok(data_path) = std::env::var("TAPU_SIMU_DATA_PATH") {
            self.data_path = PathBuf::from(data_path);
        }

        if let Ok(debug) = std::env::var("TAPU_SIMU_DEBUG") {
            self.logging.debug = debug.parse().unwrap_or(self.logging.debug);
        }

        if let Ok(thread_count) = std::env::var("TAPU_SIMU_THREADS") {
            self.performance.thread_count = thread_count.parse().unwrap_or(self.performance.thread_count);
        }

        self.validate()?;
        Ok(self)
    }
}

