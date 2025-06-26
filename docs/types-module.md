# Types Module Documentation

The types module provides type-safe wrappers, comprehensive error handling, and position management for Tapu Simu. It ensures compile-time safety, prevents common errors, and provides consistent error reporting throughout the battle simulation system.

## Architecture Overview

The types module consists of three main components:
- **Identifiers**: Type-safe wrappers for game entity identifiers
- **Errors**: Comprehensive error handling with context and categorization
- **Positions**: Type-safe position and index management

## Identifiers (`identifiers.rs`)

Type-safe identifier wrappers that prevent category confusion and ensure consistent normalization.

### Identifier Types

**Core Identifier Wrappers:**
```rust
pub struct SpeciesId(String);    // Pokemon species identifiers
pub struct MoveId(String);       // Move identifiers
pub struct AbilityId(String);    // Ability identifiers
pub struct ItemId(String);       // Item identifiers
pub struct TypeId(String);       // Pokemon type identifiers
```

### Normalization System

**Consistent Name Normalization:**
All identifiers are automatically normalized using `normalize_name()`:
- Removes spaces, hyphens, apostrophes, and dots
- Converts to lowercase ASCII
- Ensures consistent lookups across the system

```rust
impl SpeciesId {
    pub fn new(species: impl Into<String>) -> Self {
        let normalized = normalize_name(&species.into());
        Self::validate_normalized(&normalized);
        Self(normalized)
    }
}

// Examples:
// "Ho-Oh" -> "hooh"
// "Farfetch'd" -> "farfetchd" 
// "Mr. Mime" -> "mrmime"
// "Type: Null" -> "typenull"
```

### Validation and Safety

**Debug-time Validation:**
```rust
fn validate_normalized(identifier: &str) {
    debug_assert!(
        identifier == normalize_name(identifier),
        "Identifier '{}' is not properly normalized. Expected: '{}'",
        identifier,
        normalize_name(identifier)
    );
    debug_assert!(
        identifier.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()),
        "Normalized identifier '{}' contains invalid characters.",
        identifier
    );
}
```

**Type Safety Features:**
- Prevents mixing different identifier types at compile time
- Automatic conversion from strings with validation
- Consistent display formatting across the system

### Conversion and Display

**Flexible Conversion:**
```rust
impl From<String> for SpeciesId {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for SpeciesId {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl fmt::Display for SpeciesId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
```

**Access Methods:**
```rust
impl SpeciesId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
```

### Serialization Support

All identifier types support serde serialization:
```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SpeciesId(String);
```

This enables:
- JSON serialization for data persistence
- Network communication compatibility
- Configuration file support

## Errors (`errors.rs`)

Comprehensive error handling system with categorized errors and rich context.

### Error Hierarchy

**Top-Level Battle Error:**
```rust
#[derive(Debug, Error)]
pub enum BattleError {
    #[error("Invalid move choice: {reason}")]
    InvalidMoveChoice { reason: String },
    
    #[error("Pokemon {species} not found")]
    PokemonNotFound { species: SpeciesId },
    
    #[error("Invalid battle state: {reason}")]
    InvalidState { reason: String },
    
    #[error("Data loading failed")]
    DataLoad(#[from] DataError),
    
    #[error("Format validation failed")]
    FormatValidation(#[from] FormatError),
    
    #[error("Team validation failed")]
    TeamValidation(#[from] TeamError),
    
    #[error("Builder error")]
    BuilderError(#[from] crate::builders::BuilderError),
}
```

### Specialized Error Types

**Data Access Errors:**
```rust
#[derive(Debug, Error)]
pub enum DataError {
    #[error("Failed to read file: {path}")]
    FileRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    
    #[error("Invalid JSON in {file}")]
    JsonParse {
        file: String,
        #[source]
        source: serde_json::Error,
    },
    
    #[error("Species {species} not found in data")]
    SpeciesNotFound { species: SpeciesId },
    
    #[error("Data directory not found: {path}")]
    DataDirNotFound { path: PathBuf },
}
```

**Format Validation Errors:**
```rust
#[derive(Debug, Error)]
pub enum FormatError {
    #[error("Invalid format name: {name}")]
    InvalidName { name: String },
    
    #[error("Unsupported generation: {generation}")]
    UnsupportedGeneration { generation: u8 },
    
    #[error("Invalid team size: {size} (expected {expected})")]
    InvalidTeamSize { size: usize, expected: usize },
    
    #[error("Banned species: {species}")]
    BannedSpecies { species: SpeciesId },
    
    #[error("Format rule violation: {rule}")]
    RuleViolation { rule: String },
}
```

**Team Building Errors:**
```rust
#[derive(Debug, Error)]
pub enum TeamError {
    #[error("Invalid team size: {size}")]
    InvalidSize { size: usize },
    
    #[error("Duplicate species: {species}")]
    DuplicateSpecies { species: SpeciesId },
    
    #[error("Invalid Pokemon configuration: {reason}")]
    InvalidPokemon { reason: String },
    
    #[error("Random team generation failed: {reason}")]
    RandomGenerationFailed { reason: String },
}
```

**Configuration Errors:**
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
```

### Error Context and Chaining

**Source Chain Integration:**
- Uses `#[from]` for automatic error conversion
- Preserves original error context with `#[source]`
- Provides structured error information for debugging

**Rich Error Context:**
- Specific identifiers for failed lookups
- File paths for I/O errors
- Field names for validation errors
- Detailed reasons for configuration failures

### Type Aliases for Convenience

```rust
/// Type alias for common Result pattern
pub type BattleResult<T> = Result<T, BattleError>;
pub type DataResult<T> = Result<T, DataError>;
pub type FormatResult<T> = Result<T, FormatError>;
pub type TeamResult<T> = Result<T, TeamError>;
pub type ConfigResult<T> = Result<T, ConfigError>;
pub type SimulatorResult<T> = Result<T, SimulatorError>;
```

## Positions (`positions.rs`)

Type-safe position and index management with validation and bounds checking.

### Slot Index System

**Validated Team Slots:**
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SlotIndex(u8);

/// Maximum team size constant for validation
pub const MAX_TEAM_SIZE: u8 = 6;

#[derive(Debug, thiserror::Error)]
#[error("Invalid slot index {slot}: must be less than {MAX_TEAM_SIZE}")]
pub struct InvalidSlotError {
    pub slot: u8,
}
```

**Safe Construction:**
```rust
impl SlotIndex {
    /// Create a new SlotIndex with validation
    pub fn new(slot: u8) -> Result<Self, InvalidSlotError> {
        if slot < MAX_TEAM_SIZE {
            Ok(SlotIndex(slot))
        } else {
            Err(InvalidSlotError { slot })
        }
    }
    
    /// Create a SlotIndex without validation (use carefully)
    pub fn new_unchecked(slot: u8) -> Self {
        SlotIndex(slot)
    }
    
    /// Get the raw slot value as usize for indexing
    pub fn as_usize(self) -> usize {
        self.0 as usize
    }
}
```

**Conversion Safety:**
```rust
impl TryFrom<u8> for SlotIndex {
    type Error = InvalidSlotError;
    
    fn try_from(slot: u8) -> Result<Self, Self::Error> {
        Self::new(slot)
    }
}

impl TryFrom<usize> for SlotIndex {
    type Error = InvalidSlotError;
    
    fn try_from(slot: usize) -> Result<Self, Self::Error> {
        if slot < MAX_TEAM_SIZE as usize {
            Ok(SlotIndex(slot as u8))
        } else {
            Err(InvalidSlotError { slot: slot as u8 })
        }
    }
}
```

### Turn Number System

**Non-Zero Turn Numbers:**
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TurnNumber(NonZeroU32);

impl TurnNumber {
    /// Create a new TurnNumber starting from 1
    pub fn new(turn: u32) -> Option<Self> {
        NonZeroU32::new(turn).map(TurnNumber)
    }
    
    /// Create the first turn
    pub fn first() -> Self {
        TurnNumber(NonZeroU32::new(1).unwrap())
    }
    
    /// Get the next turn number
    pub fn next(self) -> Self {
        TurnNumber(NonZeroU32::new(self.0.get() + 1).unwrap())
    }
}
```

**Benefits:**
- Prevents zero turn numbers at compile time
- Automatic overflow protection
- Sequential turn progression guarantees

### Position Index System

**Battle Position Management:**
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PositionIndex(u8);

impl PositionIndex {
    /// Create a new PositionIndex
    pub fn new(position: u8) -> Self {
        PositionIndex(position)
    }
    
    /// Get the raw position value as usize for indexing
    pub fn as_usize(self) -> usize {
        self.0 as usize
    }
}
```

## Usage Patterns

### Identifier Usage

```rust
// Type-safe identifier creation
let species = SpeciesId::from("Ho-Oh");  // Automatically normalized to "hooh"
let move_id = MoveId::from("Thunder Bolt");  // Normalized to "thunderbolt"

// Safe data access
let pokemon_data = repository.find_by_id(&species)?;
let move_data = repository.find_by_move_id(&move_id)?;

// Error handling with context
match repository.find_by_id(&species) {
    Ok(data) => process_pokemon(data),
    Err(DataError::SpeciesNotFound { species }) => {
        eprintln!("Pokemon '{}' not found in database", species);
    }
}
```

### Error Handling Patterns

```rust
// Comprehensive error handling
fn load_battle_data(format: &str) -> BattleResult<BattleState> {
    let format_data = load_format(format)
        .map_err(|e| BattleError::FormatValidation(e))?;
    
    let team_one = generate_team(&format_data)
        .map_err(|e| BattleError::TeamValidation(e))?;
    
    let team_two = generate_team(&format_data)
        .map_err(|e| BattleError::TeamValidation(e))?;
    
    BattleState::new(format_data, team_one, team_two)
        .map_err(|e| BattleError::InvalidState { 
            reason: format!("Failed to create battle state: {}", e)
        })
}

// Error chain analysis
fn analyze_error(error: &BattleError) {
    match error {
        BattleError::DataLoad(data_error) => {
            match data_error {
                DataError::FileRead { path, source } => {
                    eprintln!("Failed to read {}: {}", path.display(), source);
                }
                DataError::SpeciesNotFound { species } => {
                    eprintln!("Unknown Pokemon: {}", species);
                }
            }
        }
        BattleError::FormatValidation(format_error) => {
            eprintln!("Format validation failed: {}", format_error);
        }
    }
}
```

### Position Management

```rust
// Safe slot indexing
fn get_pokemon_at_slot(team: &[Pokemon], slot: u8) -> Result<&Pokemon, InvalidSlotError> {
    let slot_index = SlotIndex::new(slot)?;
    Ok(&team[slot_index.as_usize()])
}

// Turn progression
fn next_turn(current: TurnNumber) -> TurnNumber {
    current.next()  // Cannot overflow or become zero
}

// Position-based operations
fn target_position(position: u8) -> PositionIndex {
    PositionIndex::new(position)  // Always valid
}
```