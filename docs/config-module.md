# Config Module Documentation

The config module provides comprehensive configuration management for Tapu Simu with file-based and environment-based configuration loading, validation, and builder patterns for flexible setup.

## Architecture Overview

The config module consists of four main components:
- **Configuration Structures**: Hierarchical configuration organization
- **Builder Pattern**: Fluent API for configuration construction
- **File Management**: JSON-based configuration persistence
- **Environment Integration**: Environment variable override support

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
- Serde support for JSON serialization/deserialization
- Intelligent defaults for all configuration options
- Built-in validation with detailed error reporting

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
- Debug logging disabled by default
- File size limit of 100MB
- Output to stdout unless file specified
- Individual component logging controls

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

**Intelligent Defaults:**
- Parallel battles enabled by default
- Auto-detection of thread count (0 = CPU count)
- Caching enabled with 256MB limit
- Optimized for common usage patterns

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

**Battle Control Features:**
- Turn limit protection (default 1000 turns)
- Strict format validation for competitive accuracy
- Damage randomization for authentic Pokemon mechanics
- Deterministic battles via seed control
- Undo support for instruction replay

## Configuration Builder (`ConfigBuilder`)

Fluent API builder pattern for constructing configurations with validation.

### Builder Construction

**Basic Usage:**
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
    pub fn data_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.config.data_path = path.into();
        self
    }
}
```

**Logging Configuration:**
```rust
impl ConfigBuilder {
    pub fn debug(mut self, enable: bool) -> Self {
        self.config.logging.debug = enable;
        self
    }

    pub fn log_instructions(mut self, enable: bool) -> Self {
        self.config.logging.log_instructions = enable;
        self
    }

    pub fn log_file(mut self, path: impl Into<PathBuf>) -> Self {
        self.config.logging.log_file = Some(path.into());
        self
    }
}
```

**Performance Configuration:**
```rust
impl ConfigBuilder {
    pub fn parallel_battles(mut self, enable: bool) -> Self {
        self.config.performance.parallel_battles = enable;
        self
    }

    pub fn thread_count(mut self, count: usize) -> Self {
        self.config.performance.thread_count = count;
        self
    }

    pub fn max_cache_size(mut self, size_mb: u64) -> Self {
        self.config.performance.max_cache_size_mb = size_mb;
        self
    }
}
```

**Battle Configuration:**
```rust
impl ConfigBuilder {
    pub fn max_turns(mut self, turns: u32) -> Self {
        self.config.battle.default_max_turns = turns;
        self
    }

    pub fn damage_randomization(mut self, enable: bool) -> Self {
        self.config.battle.damage_randomization = enable;
        self
    }

    pub fn random_seed(mut self, seed: u64) -> Self {
        self.config.battle.random_seed = Some(seed);
        self
    }
}
```

### Builder Validation

**Build with Validation:**
```rust
impl ConfigBuilder {
    pub fn build(self) -> ConfigResult<Config> {
        self.config.validate()?;
        Ok(self.config)
    }

    pub fn build_unchecked(self) -> Config {
        self.config
    }
}
```

## File Management

### Configuration Persistence

**File Loading:**
```rust
impl Config {
    /// Load configuration from a file
    pub fn load(path: impl AsRef<Path>) -> ConfigResult<Self> {
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

## Environment Integration

### Environment Variable Support

**Supported Environment Variables:**
- `TAPU_SIMU_DATA_PATH`: Override data directory path
- `TAPU_SIMU_DEBUG`: Enable debug logging
- `TAPU_SIMU_LOG_INSTRUCTIONS`: Enable instruction logging
- `TAPU_SIMU_LOG_FILE`: Set log file path
- `TAPU_SIMU_THREADS`: Set thread count
- `TAPU_SIMU_MAX_TURNS`: Set maximum turns per battle
- `TAPU_SIMU_SEED`: Set random seed for deterministic battles

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

        // Performance settings
        if let Ok(thread_count) = std::env::var("TAPU_SIMU_THREADS") {
            config.performance.thread_count = thread_count.parse().unwrap_or(0);
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

        self.validate()?;
        Ok(self)
    }
}
```

## Validation System

### Comprehensive Validation

**Configuration Validation:**
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

**Environment Priority:**
```rust
// Pure environment configuration
let config = Config::from_env()?;

// Fallback pattern
let config = Config::load("tapu-simu.json")
    .or_else(|_| Config::from_env())
    .unwrap_or_else(|_| Config::default());
```

## Integration Points

The config module integrates with:
- **Simulator Module**: Primary configuration consumer
- **Data Module**: Data path and caching configuration
- **Engine Module**: Battle behavior configuration
- **Testing Module**: Deterministic testing via seed control