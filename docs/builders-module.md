# Builders Module Documentation

The builders module implements sophisticated builder patterns for constructing battle components with comprehensive validation, type safety, and fluent APIs. It provides consistent interfaces for building formats, teams, Pokemon, and complete battles.

## Architecture Overview

The builders module provides four main builders unified under common traits:
- **Format Builder**: Battle format construction with rule validation
- **Team Builder**: Hierarchical team and Pokemon building
- **Battle Builder**: Complete battle setup with player management
- **Trait System**: Standardized builder interfaces and validation

## Core Traits System (`traits.rs`)

### Primary Builder Trait

```rust
pub trait Builder<T> {
    type Error: Debug;
    fn build(self) -> Result<T, Self::Error>;
    fn validate(&self) -> Result<(), Self::Error>;
}
```

**Design Features:**
- **Consuming Build**: `build()` consumes the builder, preventing reuse after construction
- **Non-destructive Validation**: `validate()` checks state without consuming the builder
- **Type-safe Errors**: Associated error types ensure compile-time error consistency

### Extended Traits

**ValidatingBuilder:**
```rust
pub trait ValidatingBuilder<T>: Builder<T> {
    fn validate_with_context(&self, context: &mut ValidationContext) -> Result<(), Self::Error>;
    fn collect_warnings(&self) -> Vec<BuilderWarning>;
}
```

**ResettableBuilder:**
```rust
pub trait ResettableBuilder<T>: Builder<T> {
    fn reset(&mut self);
    fn is_clean(&self) -> bool;
}
```

**CloneableBuilder:**
```rust
pub trait CloneableBuilder<T>: Builder<T> + Clone {
    fn template(&self) -> Self;
}
```

### Standardized Error System

```rust
#[derive(Debug, thiserror::Error)]
pub enum BuilderError {
    #[error("Missing required field: {field}")]
    MissingRequired { field: String },
    
    #[error("Invalid value for {field}: {value:?} - {reason}")]
    InvalidValue { field: String, value: String, reason: String },
    
    #[error("Configuration conflict: {details}")]
    ConfigConflict { details: String },
    
    #[error("Data error: {0}")]
    DataError(#[from] DataError),
    
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
}
```

**Error Context Features:**
- Hierarchical field paths for debugging
- Rich error details with reasons
- External dependency error propagation
- Validation failure categorization

## Format Builder (`format.rs`)

Battle format construction with intelligent defaults and validation.

### Fluent Interface Design

```rust
impl FormatBuilder {
    pub fn new() -> Self {
        Self {
            format_type: None,
            generation: None,
            clauses: Vec::new(),
            banned_species: Vec::new(),
            banned_moves: Vec::new(),
            banned_items: Vec::new(),
            banned_abilities: Vec::new(),
            active_per_side: None,
            team_size: None,
            errors: Vec::new(),
        }
    }

    pub fn singles(mut self) -> Self {
        self.format_type = Some(FormatType::Singles);
        self.active_per_side = Some(1);
        self
    }

    pub fn doubles(mut self) -> Self {
        self.format_type = Some(FormatType::Doubles);
        self.active_per_side = Some(2);
        self
    }

    pub fn generation(mut self, gen: Generation) -> Self {
        self.generation = Some(gen);
        self
    }
}
```

### Smart Defaults and Validation

**Format Type Intelligence:**
- Automatically sets `active_per_side` based on format type
- Validates team size constraints (≤ 6, ≥ active_per_side)
- Prevents invalid format/generation combinations

**Clause Management:**
```rust
pub fn standard_clauses(mut self) -> Self {
    self.clauses.extend([
        Clause::Species,
        Clause::Item,
        Clause::Sleep,
        Clause::Evasion,
    ]);
    self
}

pub fn add_clause(mut self, clause: Clause) -> Self {
    if !self.clauses.contains(&clause) {
        self.clauses.push(clause);
    }
    self
}
```

**Preset Builders:**
```rust
impl FormatBuilder {
    pub fn gen9_ou() -> Self {
        Self::new()
            .generation(Generation::Gen9)
            .singles()
            .standard_clauses()
            .ban_species("Mewtwo")
            .ban_ability("Moody")
    }

    pub fn vgc_2024() -> Self {
        Self::new()
            .generation(Generation::Gen9)
            .doubles()
            .team_size(4)
            .restricted_series()
    }
}
```

## Team Builder (`team.rs`)

Hierarchical builder pattern supporting nested Pokemon construction.

### Context-Switching Architecture

```rust
pub struct TeamBuilder {
    data_repository: Arc<GameDataRepository>,
    pokemon: Vec<TeamPokemon>,
    current_context: Option<TeamPokemonContext>,
    format: Option<BattleFormat>,
    errors: Vec<BuilderError>,
}

pub struct TeamPokemonContext {
    pokemon: TeamPokemon,
    parent: *mut TeamBuilder,
}
```

### Fluent Pokemon Building

```rust
impl TeamBuilder {
    pub fn pokemon(mut self, species: &str) -> TeamPokemonContext {
        let pokemon_data = self.data_repository.pokemon
            .find_by_name(species)
            .expect("Valid Pokemon species");

        TeamPokemonContext {
            pokemon: TeamPokemon::new(species, pokemon_data),
            parent: &mut self as *mut _,
        }
    }
}

impl TeamPokemonContext {
    pub fn level(mut self, level: u8) -> Self {
        if level >= 1 && level <= 100 {
            self.pokemon.level = level;
        } else {
            self.add_error(BuilderError::InvalidValue {
                field: "level".to_string(),
                value: level.to_string(),
                reason: "Level must be between 1 and 100".to_string(),
            });
        }
        self
    }

    pub fn ability(mut self, ability: &str) -> Self {
        if self.pokemon.data.abilities.contains(ability) {
            self.pokemon.ability = Some(ability.to_string());
        } else {
            self.add_error(BuilderError::InvalidValue {
                field: "ability".to_string(),
                value: ability.to_string(),
                reason: "Ability not available for this Pokemon".to_string(),
            });
        }
        self
    }

    pub fn finish(self) -> TeamBuilder {
        unsafe { 
            let parent = &mut *self.parent;
            parent.pokemon.push(self.pokemon);
            parent.take_ownership()
        }
    }
}
```

### Advanced Validation

**EV/IV System:**
```rust
impl TeamPokemonContext {
    pub fn evs(mut self, hp: u8, atk: u8, def: u8, spa: u8, spd: u8, spe: u8) -> Self {
        let total = hp + atk + def + spa + spd + spe;
        
        if total > 510 {
            self.add_error(BuilderError::InvalidValue {
                field: "evs".to_string(),
                value: total.to_string(),
                reason: "Total EVs cannot exceed 510".to_string(),
            });
        }
        
        if [hp, atk, def, spa, spd, spe].iter().any(|&ev| ev > 252) {
            self.add_error(BuilderError::InvalidValue {
                field: "evs".to_string(),
                value: "individual".to_string(),
                reason: "Individual EVs cannot exceed 252".to_string(),
            });
        }
        
        self.pokemon.evs = StatSpread { hp, atk, def, spa, spd, spe };
        self
    }

    pub fn move_slot(mut self, move_name: &str) -> Self {
        if self.pokemon.moves.len() >= 4 {
            self.add_error(BuilderError::InvalidValue {
                field: "moves".to_string(),
                value: move_name.to_string(),
                reason: "Pokemon cannot have more than 4 moves".to_string(),
            });
            return self;
        }

        match self.parent.data_repository.moves.find_by_name(move_name) {
            Ok(move_data) => {
                self.pokemon.moves.push(move_data.name.clone());
            }
            Err(_) => {
                self.add_error(BuilderError::DataError(
                    DataError::MoveNotFound(MoveId::from_name(move_name))
                ));
            }
        }
        self
    }
}
```

## Battle Builder (`battle.rs`)

Comprehensive battle construction with multi-stage validation.

### Complete Battle Setup

```rust
pub struct BattleBuilder {
    format: Option<BattleFormat>,
    player_one: Option<Box<dyn Player>>,
    player_two: Option<Box<dyn Player>>,
    team_one: Option<Vec<Pokemon>>,
    team_two: Option<Vec<Pokemon>>,
    turn_limit: Option<u32>,
    timeout_ms: Option<u64>,
    data_repository: Arc<GameDataRepository>,
    errors: Vec<BuilderError>,
}

impl BattleBuilder {
    pub fn new(data: Arc<GameDataRepository>) -> Self {
        Self {
            format: None,
            player_one: None,
            player_two: None,
            team_one: None,
            team_two: None,
            turn_limit: Some(1000),
            timeout_ms: Some(30000),
            data_repository: data,
            errors: Vec::new(),
        }
    }

    pub fn format(mut self, format: BattleFormat) -> Self {
        self.format = Some(format);
        self
    }

    pub fn players(mut self, p1: Box<dyn Player>, p2: Box<dyn Player>) -> Self {
        self.player_one = Some(p1);
        self.player_two = Some(p2);
        self
    }

    pub fn teams(mut self, team1: Vec<Pokemon>, team2: Vec<Pokemon>) -> Self {
        self.team_one = Some(team1);
        self.team_two = Some(team2);
        self
    }
}
```

### Multi-Stage Validation

```rust
impl Builder<BattleEnvironment> for BattleBuilder {
    type Error = BuilderError;

    fn validate(&self) -> Result<(), Self::Error> {
        // 1. Required field validation
        if self.format.is_none() {
            return Err(BuilderError::MissingRequired {
                field: "format".to_string(),
            });
        }

        // 2. Format compatibility validation
        if let (Some(format), Some(team)) = (&self.format, &self.team_one) {
            if team.len() < format.team_size() {
                return Err(BuilderError::ConfigConflict {
                    details: format!(
                        "Team size ({}) is less than format requirement ({})",
                        team.len(),
                        format.team_size()
                    ),
                });
            }
        }

        // 3. Configuration validation
        if let Some(limit) = self.turn_limit {
            if limit == 0 {
                return Err(BuilderError::InvalidValue {
                    field: "turn_limit".to_string(),
                    value: limit.to_string(),
                    reason: "Turn limit must be greater than 0".to_string(),
                });
            }
        }

        Ok(())
    }

    fn build(mut self) -> Result<BattleEnvironment, Self::Error> {
        self.validate()?;

        // Provide default players if not specified
        let player_one = self.player_one.unwrap_or_else(|| Box::new(RandomPlayer::new()));
        let player_two = self.player_two.unwrap_or_else(|| Box::new(RandomPlayer::new()));

        // Generate random teams if not provided
        let team_one = self.team_one.unwrap_or_else(|| {
            self.generate_random_team().unwrap_or_default()
        });
        let team_two = self.team_two.unwrap_or_else(|| {
            self.generate_random_team().unwrap_or_default()
        });

        // Create battle state
        let battle_state = BattleState::new(
            self.format.unwrap(),
            team_one,
            team_two,
        )?;

        Ok(BattleEnvironment::new(
            battle_state,
            player_one,
            player_two,
            self.turn_limit,
            self.timeout_ms,
        ))
    }
}
```

### Advanced Features

**Random Team Generation:**
```rust
impl BattleBuilder {
    fn generate_random_team(&self) -> Result<Vec<Pokemon>, BuilderError> {
        let format_name = self.format.as_ref()
            .map(|f| f.format_name())
            .unwrap_or("gen9randombattle");

        let mut team_loader = RandomTeamLoader::new(self.data_repository.clone())?;
        team_loader.load_team(format_name)
            .map_err(BuilderError::DataError)
    }

    pub fn run(self) -> Result<BattleResult, BuilderError> {
        let environment = self.build()?;
        Ok(environment.run_battle())
    }
}
```

## Validation Architecture

### Multi-layered Validation Strategy

1. **Build-time Validation**: Strict checks during `build()` calls
2. **Incremental Validation**: Field-level validation during construction
3. **Format-specific Validation**: Rules based on battle format constraints
4. **Data Dependency Validation**: External data source verification

### Error Context System

```rust
pub struct ValidationContext {
    field_path: Vec<String>,
    warnings: Vec<BuilderWarning>,
    strict_mode: bool,
}

impl ValidationContext {
    pub fn push_field(&mut self, field: &str) {
        self.field_path.push(field.to_string());
    }

    pub fn current_path(&self) -> String {
        self.field_path.join(".")
    }

    pub fn add_warning(&mut self, warning: BuilderWarning) {
        self.warnings.push(warning);
    }
}
```

## Type Safety Mechanisms

### Compile-time Guarantees

**Strong Typing:**
```rust
// Identifier types prevent category mixing
pub struct SpeciesId(String);
pub struct MoveId(String);
pub struct AbilityId(String);

// Enum-based format types with associated rules
pub enum FormatType {
    Singles,   // 1 active per side
    Doubles,   // 2 active per side
    VGC,       // 2 active per side, team size 4
    Triples,   // 3 active per side
}
```

**Memory Safety:**
- Move semantics prevent builder reuse after consumption
- Repository references ensure data availability during construction
- Type-erased storage (`Box<dyn Player>`) with static dispatch where possible

### Runtime Safety

**Bounds Checking:**
```rust
pub fn level(mut self, level: u8) -> Self {
    if level >= 1 && level <= 100 {
        self.pokemon.level = level;
    } else {
        self.add_error(BuilderError::InvalidValue {
            field: "level".to_string(),
            value: level.to_string(),
            reason: "Level must be between 1 and 100".to_string(),
        });
    }
    self
}
```

## Integration Points

The builders module integrates with:
- **Data Module**: Validates against `GameDataRepository` for data consistency
- **Core Module**: Constructs `BattleState`, `BattleFormat`, and related types
- **Engine Module**: Provides validated components for battle simulation
- **Testing Module**: Supplies builders for test case construction