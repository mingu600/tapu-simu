# Config Module Documentation

The config module provides comprehensive configuration management for Tapu Simu with hierarchical configuration structures, fluent builder patterns, JSON-based persistence, and environment variable override support.

## Architecture Overview

The config module (`src/config.rs`) implements a sophisticated configuration system with four main components:

- **Configuration Structures** - Hierarchical configuration organization with type safety
- **Builder Pattern** - Fluent API for configuration construction with validation
- **File Management** - JSON-based configuration persistence with error handling
- **Environment Integration** - Environment variable override support with intelligent defaults

## Configuration Structures

### Main Configuration (`Config`)

**Core Structure:**
```rust
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
```

**Key Features:**
- Hierarchical organization with specialized configuration sections
- Full serde support for JSON serialization/deserialization
- Intelligent default values for all configuration options
- Built-in validation with detailed error reporting
- Clone and Debug implementations for development workflow

### Logging Configuration (`LoggingConfig`)

**Comprehensive Logging Control:**
```rust
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
```

**Default Settings:**
```rust
impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            debug: false,                 // Production-safe
            log_instructions: false,      // Minimal logging by default
            log_moves: false,            // Optional battle logging
            log_damage: false,           // Optional damage logging
            max_file_size_mb: 100,       // 100MB rotation limit
            log_file: None,              // stdout by default
        }
    }
}
```

**Logging Control Features:**
- Granular logging control for different battle components
- File rotation support with configurable size limits
- Optional file output with fallback to stdout
- Debug mode for development workflows

### Performance Configuration (`PerformanceConfig`)

**Performance Tuning Options:**
```rust
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
```

**Intelligent Performance Defaults:**
```rust
impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            parallel_battles: true,       // Enable parallelism by default
            thread_count: 0,             // Auto-detect CPU cores
            cache_moves: true,           // Cache frequently used move data
            cache_pokemon: true,         // Cache Pokemon data
            max_cache_size_mb: 256,      // 256MB cache limit
        }
    }
}
```

**Performance Features:**
- Auto-detection of available CPU cores
- Configurable caching for frequently accessed data
- Memory-bounded caching with configurable limits
- Parallel battle execution control

### Battle Configuration (`BattleConfig`)

**Battle Behavior Control:**
```rust
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
```

**Battle Control Defaults:**
```rust
impl Default for BattleConfig {
    fn default() -> Self {
        Self {
            default_max_turns: 1000,         // Prevent infinite loops
            strict_format_validation: true,   // Competitive accuracy
            damage_randomization: true,       // Authentic mechanics
            random_seed: None,               // Non-deterministic by default
            enable_undo: true,               // Development-friendly
        }
    }
}
```

**Battle Control Features:**
- Turn limit protection against infinite battles
- Strict format validation for competitive accuracy
- Damage randomization for authentic Pokemon mechanics
- Deterministic battle support via seed control
- Instruction undo support for development and testing

## Configuration Builder (`ConfigBuilder`)

Fluent API builder pattern for constructing validated configurations.

### Builder Construction

**Core Builder Structure:**
```rust
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
}
```

**Basic Usage Pattern:**
```rust
let config = Config::builder()
    .data_path("/path/to/data")
    .debug(true)
    .log_instructions(true)
    .parallel_battles(true)
    .thread_count(4)
    .max_turns(500)
    .build()?;
```

### Builder Methods

**Data Configuration:**
```rust
impl ConfigBuilder {
    /// Set the data path
    pub fn data_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.config.data_path = path.into();
        self
    }
}
```

**Logging Configuration:**
```rust
impl ConfigBuilder {
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
}
```

**Performance Configuration:**
```rust
impl ConfigBuilder {
    /// Enable parallel battles
    pub fn parallel_battles(mut self, enable: bool) -> Self {
        self.config.performance.parallel_battles = enable;
        self
    }

    /// Set thread count (0 for auto-detection)
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
}
```

**Battle Configuration:**
```rust
impl ConfigBuilder {
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

    /// Set random seed for deterministic battles
    pub fn random_seed(mut self, seed: u64) -> Self {
        self.config.battle.random_seed = Some(seed);
        self
    }

    /// Enable undo support
    pub fn enable_undo(mut self, enable: bool) -> Self {
        self.config.battle.enable_undo = enable;
        self
    }
}
```

### Builder Validation

**Validated Build Process:**
```rust
impl ConfigBuilder {
    /// Build the configuration with validation
    pub fn build(self) -> ConfigResult<Config> {
        self.config.validate()?;
        Ok(self.config)
    }

    /// Build the configuration without validation
    pub fn build_unchecked(self) -> Config {
        self.config
    }
}
```

## File Management

### Configuration Persistence

**File Loading Methods:**
```rust
impl Config {
    /// Load configuration from a file
    pub fn load(path: impl AsRef<Path>) -> ConfigResult<Self> {
        Self::from_file(path)
    }

    /// Load configuration from a file (explicit method)
    pub fn from_file(path: impl AsRef<Path>) -> ConfigResult<Self> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::FileNotFound { path: path.to_path_buf() })?;
        
        let config: Config = serde_json::from_str(&content)?;
        Ok(config)
    }
}
```

**File Saving:**
```rust
impl Config {
    /// Save configuration to a file
    pub fn save_to_file(&self, path: impl AsRef<Path>) -> ConfigResult<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path.as_ref(), content)
            .map_err(|e| ConfigError::FileNotFound { path: path.as_ref().to_path_buf() })?;
        Ok(())
    }
}
```

### Default Data Path Discovery

**Intelligent Path Resolution:**
```rust
impl Config {
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
}
```

**Path Discovery Strategy:**
1. Check `{current_dir}/data/ps-extracted`
2. Fallback to `data/ps-extracted` relative path
3. Allow validation to catch missing paths later

## Environment Integration

### Environment Variable Support

**Supported Environment Variables:**
- `TAPU_SIMU_DATA_PATH` - Override data directory path
- `TAPU_SIMU_DEBUG` - Enable debug logging
- `TAPU_SIMU_LOG_INSTRUCTIONS` - Enable instruction logging
- `TAPU_SIMU_LOG_FILE` - Set log file path
- `TAPU_SIMU_THREADS` - Set thread count
- `TAPU_SIMU_MAX_TURNS` - Set maximum turns per battle
- `TAPU_SIMU_SEED` - Set random seed for deterministic battles

### Environment Loading

**Complete Environment Configuration:**
```rust
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

        // Battle settings
        if let Ok(max_turns) = std::env::var("TAPU_SIMU_MAX_TURNS") {
            config.battle.default_max_turns = max_turns.parse().unwrap_or(1000);
        }

        if let Ok(seed) = std::env::var("TAPU_SIMU_SEED") {
            config.battle.random_seed = seed.parse().ok();
        }

        config.validate()?;
        Ok(config)
    }
}
```

**Environment Override Pattern:**
```rust
impl Config {
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
```

## Validation System

### Comprehensive Validation

**Configuration Validation Implementation:**
```rust
impl Config {
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
}
```

**Validation Rules:**
- Data path must exist for successful validation
- Cache size must be greater than 0
- Max turns must be greater than 0
- All validations provide detailed error context

### Runtime Configuration Helpers

**Thread Count Resolution:**
```rust
impl Config {
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
```

**Runtime Helper Features:**
- Auto-detection of CPU cores when thread_count is 0
- Graceful fallback to single-threaded execution
- Integration with std::thread::available_parallelism()

## Error Handling

### Configuration Error Types

**ConfigError Definition:**
```rust
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Configuration file not found: {path}")]
    FileNotFound { path: PathBuf },
    
    #[error("Invalid configuration format")]
    InvalidFormat(#[from] serde_json::Error),
    
    #[error("Missing required configuration field: {field}")]
    MissingField { field: String },
    
    #[error("Invalid configuration value for {field}: {value}")]
    InvalidValue { field: String, value: String },
}

pub type ConfigResult<T> = Result<T, ConfigError>;
```

**Error Handling Features:**
- Detailed error context with field names and values
- JSON parsing error propagation from serde
- File system error handling with path information
- Type-safe error handling with thiserror integration

## Usage Patterns

### Basic Configuration

**Default Configuration:**
```rust
let config = Config::new();
// Uses intelligent defaults with data path discovery
```

**Builder Configuration:**
```rust
let config = Config::builder()
    .data_path("/custom/data/path")
    .debug(true)
    .log_instructions(true)
    .parallel_battles(true)
    .thread_count(8)
    .max_turns(2000)
    .random_seed(12345)
    .build()?;
```

### File-Based Configuration

**Loading and Saving:**
```rust
// Save current configuration
config.save_to_file("tapu-simu.json")?;

// Load configuration from file
let config = Config::load("tapu-simu.json")?;

// Load with environment overrides
let config = Config::load("tapu-simu.json")?
    .with_env_overrides()?;
```

### Environment-First Configuration

**Environment Priority Loading:**
```rust
// Pure environment configuration
let config = Config::from_env()?;

// Fallback pattern with graceful degradation
let config = Config::load("tapu-simu.json")
    .or_else(|_| Config::from_env())
    .unwrap_or_else(|_| Config::default());
```

### Configuration Patterns

**Development Configuration:**
```rust
let config = Config::builder()
    .debug(true)
    .log_instructions(true)
    .log_moves(true)
    .log_damage(true)
    .strict_validation(true)
    .enable_undo(true)
    .build()?;
```

**Production Configuration:**
```rust
let config = Config::builder()
    .parallel_battles(true)
    .thread_count(16)
    .cache_moves(true)
    .cache_pokemon(true)
    .max_cache_size(512)
    .build()?;
```

**Testing Configuration:**
```rust
let config = Config::builder()
    .random_seed(12345)
    .damage_randomization(false)
    .enable_undo(true)
    .max_turns(100)
    .build()?;
```