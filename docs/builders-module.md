# Builders Module Documentation

The builders module implements comprehensive builder patterns for constructing battle components in Tapu Simu. It provides type-safe, validated construction of formats, teams, Pokemon, and complete battles with fluent APIs and intelligent defaults.

## Architecture Overview

The builders module (`src/builders/`) consists of five core files:

- **`traits.rs`** - Common builder traits and standardized error system
- **`format.rs`** - Battle format construction with rule validation  
- **`team.rs`** - Hierarchical team and Pokemon building with data validation
- **`battle.rs`** - Complete battle setup with multi-stage validation
- **`mod.rs`** - Module declarations and public API exports

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

**ValidatingBuilder** - Incremental validation with context:
```rust
pub trait ValidatingBuilder<T>: Builder<T> {
    fn validate_with_context(&self, context: &mut ValidationContext) -> Result<(), Self::Error>;
    fn collect_warnings(&self) -> Vec<BuilderWarning>;
}
```

**ResettableBuilder** - Stateful builders that can be reset:
```rust
pub trait ResettableBuilder<T>: Builder<T> {
    fn reset(&mut self);
    fn is_clean(&self) -> bool;
}
```

**CloneableBuilder** - Template-based builders:
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
    
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
    
    #[error("Configuration conflict: {details}")]
    ConfigConflict { details: String },
    
    #[error("Data error: {0}")]
    DataError(#[from] DataError),
}
```

**Error Context Features:**
- Hierarchical field paths for precise error location
- Rich error details with contextual reasons
- External dependency error propagation (`DataError`)
- Validation failure categorization

## Format Builder (`format.rs`)

Constructs `BattleFormat` objects with intelligent defaults and comprehensive rule validation.

### Implementation Architecture

```rust
pub struct FormatBuilder {
    format_type: Option<FormatType>,
    generation: Option<Generation>,
    clauses: Vec<Clause>,
    banned_species: Vec<String>,
    banned_moves: Vec<String>,
    banned_items: Vec<String>,
    banned_abilities: Vec<String>,
    active_per_side: Option<u8>,
    team_size: Option<u8>,
}
```

### Fluent Interface Design

**Format Type Configuration:**
```rust
impl FormatBuilder {
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

    pub fn vgc(mut self) -> Self {
        self.format_type = Some(FormatType::VGC);
        self.active_per_side = Some(2);
        self.team_size = Some(4);
        self
    }
}
```

**Intelligent Clause Management:**
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

**Ban List Management:**
```rust
pub fn ban_species(mut self, species: &str) -> Self {
    self.banned_species.push(species.to_string());
    self
}

pub fn ban_move(mut self, move_name: &str) -> Self {
    self.banned_moves.push(move_name.to_string());
    self
}
```

### Preset Builders

**Common Format Presets:**
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
            .vgc()
            .team_size(4)
    }
}
```

### Validation Logic

**Format-Specific Validation:**
- Team size constraints (≤ 6, ≥ active_per_side)
- Format/generation compatibility checking
- Clause conflict resolution
- Ban list consistency validation

## Team Builder (`team.rs`)

Hierarchical builder supporting nested Pokemon construction with comprehensive data validation.

### Architecture Components

```rust
pub struct TeamBuilder<'a> {
    data_repository: &'a GameDataRepository,
    pokemon: Vec<RandomPokemonSet>,
    format: Option<&'a BattleFormat>,
}

pub struct PokemonBuilder {
    species: String,
    level: u8,
    ability: Option<String>,
    moves: Vec<String>,
    item: Option<String>,
    nature: Option<String>,
    evs: StatSpread,
    ivs: StatSpread,
    gender: Option<Gender>,
}

pub struct TeamPokemonContext<'a> {
    pokemon_builder: PokemonBuilder,
    team_builder: &'a mut TeamBuilder<'a>,
}
```

### Context-Switching Pattern

**Seamless Team ↔ Pokemon Transitions:**
```rust
impl<'a> TeamBuilder<'a> {
    pub fn pokemon(&mut self, species: &str) -> TeamPokemonContext {
        TeamPokemonContext {
            pokemon_builder: PokemonBuilder::new(species),
            team_builder: self,
        }
    }
}

impl<'a> TeamPokemonContext<'a> {
    pub fn level(mut self, level: u8) -> Self {
        self.pokemon_builder.level = level;
        self
    }

    pub fn ability(mut self, ability: &str) -> Self {
        self.pokemon_builder.ability = Some(ability.to_string());
        self
    }

    pub fn finish(self) -> &'a mut TeamBuilder<'a> {
        let pokemon_set = self.pokemon_builder.build()
            .expect("Valid Pokemon configuration");
        self.team_builder.pokemon.push(pokemon_set);
        self.team_builder
    }
}
```

### Comprehensive Validation

**EV/IV System Validation:**
```rust
impl PokemonBuilder {
    pub fn evs(mut self, hp: u8, atk: u8, def: u8, spa: u8, spd: u8, spe: u8) -> Self {
        let total = hp as u16 + atk as u16 + def as u16 + spa as u16 + spd as u16 + spe as u16;
        
        if total > 510 {
            panic!("Total EVs cannot exceed 510, got {}", total);
        }
        
        if [hp, atk, def, spa, spd, spe].iter().any(|&ev| ev > 252) {
            panic!("Individual EVs cannot exceed 252");
        }
        
        self.evs = StatSpread { hp, atk, def, spa, spd, spe };
        self
    }
}
```

**Move Slot Management:**
```rust
pub fn move_slot(mut self, move_name: &str) -> Self {
    if self.moves.len() >= 4 {
        panic!("Pokemon cannot have more than 4 moves");
    }
    
    self.moves.push(move_name.to_string());
    self
}
```

**Data Repository Integration:**
- Species existence validation
- Move availability checking
- Ability compatibility verification
- Item legality validation

### Configuration Helpers

**EV/IV Configuration Objects:**
```rust
pub struct EVsConfig {
    pub hp: u8,
    pub atk: u8,
    pub def: u8,
    pub spa: u8,
    pub spd: u8,
    pub spe: u8,
}

impl EVsConfig {
    pub fn max_hp_atk() -> Self {
        Self { hp: 252, atk: 252, def: 4, spa: 0, spd: 0, spe: 0 }
    }
    
    pub fn max_hp_spa() -> Self {
        Self { hp: 252, atk: 0, def: 4, spa: 252, spd: 0, spe: 0 }
    }
}
```

## Battle Builder (`battle.rs`)

Complete battle construction with multi-stage validation and intelligent defaults.

### Core Architecture

```rust
pub struct BattleBuilder<'a> {
    data_repository: &'a GameDataRepository,
    generation_repository: &'a GenerationRepository,
    format: Option<BattleFormat>,
    team_one: Option<Vec<RandomPokemonSet>>,
    team_two: Option<Vec<RandomPokemonSet>>,
    player_one: Option<Box<dyn Player>>,
    player_two: Option<Box<dyn Player>>,
    config: BattleConfig,
}

pub struct BattleConfig {
    pub max_turns: u32,
    pub seed: Option<u64>,
    pub timeout_ms: Option<u64>,
}

pub struct Battle {
    pub state: BattleState,
    pub player_one: Box<dyn Player>,
    pub player_two: Box<dyn Player>,
    pub config: BattleConfig,
}
```

### Multi-Stage Validation

**Required Field Validation:**
```rust
fn validate(&self) -> Result<(), BuilderError> {
    if self.format.is_none() {
        return Err(BuilderError::MissingRequired {
            field: "format".to_string(),
        });
    }
    
    if self.team_one.is_none() {
        return Err(BuilderError::MissingRequired {
            field: "team_one".to_string(),
        });
    }
    
    if self.team_two.is_none() {
        return Err(BuilderError::MissingRequired {
            field: "team_two".to_string(),
        });
    }
    
    Ok(())
}
```

**Format Compatibility Validation:**
```rust
fn validate_format_compatibility(&self) -> Result<(), BuilderError> {
    if let (Some(format), Some(team)) = (&self.format, &self.team_one) {
        if team.len() < format.team_size as usize {
            return Err(BuilderError::ConfigConflict {
                details: format!(
                    "Team size ({}) is less than format requirement ({})",
                    team.len(),
                    format.team_size
                ),
            });
        }
    }
    Ok(())
}
```

**Configuration Validation:**
```rust
fn validate_config(&self) -> Result<(), BuilderError> {
    if self.config.max_turns == 0 {
        return Err(BuilderError::InvalidValue {
            field: "max_turns".to_string(),
            value: self.config.max_turns.to_string(),
            reason: "Max turns must be greater than 0".to_string(),
        });
    }
    Ok(())
}
```

### Smart Defaults and Auto-Generation

**Automatic Player Assignment:**
```rust
pub fn auto_players(mut self) -> Self {
    self.player_one = Some(Box::new(RandomPlayer::new()));
    self.player_two = Some(Box::new(RandomPlayer::new()));
    self
}
```

**Random Team Generation:**
```rust
pub fn random_teams(mut self) -> Result<Self, BuilderError> {
    let format_name = self.format.as_ref()
        .map(|f| f.get_name())
        .unwrap_or("gen9randombattle");
    
    let team_loader = RandomTeamLoader::new(self.data_repository)?;
    
    self.team_one = Some(team_loader.load_team(format_name)?);
    self.team_two = Some(team_loader.load_team(format_name)?);
    
    Ok(self)
}
```

**Battle Execution:**
```rust
pub fn run(self) -> Result<BattleResult, BuilderError> {
    let battle = self.build()?;
    Ok(battle.run())
}
```

### Configuration Options

**Battle Parameters:**
```rust
impl<'a> BattleBuilder<'a> {
    pub fn max_turns(mut self, turns: u32) -> Self {
        self.config.max_turns = turns;
        self
    }
    
    pub fn seed(mut self, seed: u64) -> Self {
        self.config.seed = Some(seed);
        self
    }
    
    pub fn timeout(mut self, ms: u64) -> Self {
        self.config.timeout_ms = Some(ms);
        self
    }
}
```

## Type System Integration

### Data Flow Architecture

```
GameDataRepository + GenerationRepository
    ↓
FormatBuilder → BattleFormat
    ↓
TeamBuilder → Vec<RandomPokemonSet> → Vec<Pokemon>
    ↓
BattleBuilder → Battle → BattleState + Players + Config
```

### Type Conversions

**RandomPokemonSet to Pokemon:**
```rust
impl RandomPokemonSet {
    pub fn to_battle_pokemon(
        &self,
        data_repo: &GameDataRepository,
        gen_repo: &GenerationRepository,
    ) -> Result<Pokemon, DataError> {
        let species_data = data_repo.pokemon.get(&self.species)?;
        let nature_data = data_repo.natures.get(&self.nature)?;
        
        Ok(Pokemon::new(
            species_data,
            self.level,
            self.ability.clone(),
            self.moves.clone(),
            self.item.clone(),
            nature_data,
            self.evs,
            self.ivs,
            self.gender,
        ))
    }
}
```

## Usage Patterns

### Complete Battle Construction
```rust
let battle = BattleBuilder::new(data_repo, gen_repo)
    .format(FormatBuilder::new().gen9_ou().build()?)
    .random_teams()?
    .auto_players()
    .max_turns(1000)
    .seed(42)
    .build()?;
```

### Custom Team Building
```rust
let team = TeamBuilder::new(data_repo)
    .format(&battle_format)
    .pokemon("Pikachu")
        .level(50)
        .ability("Static")
        .move_slot("Thunderbolt")
        .move_slot("Quick Attack")
        .evs(0, 252, 0, 252, 4, 0)
        .finish()
    .pokemon("Charizard")
        .level(50)
        .ability("Blaze")
        .move_slot("Flamethrower")
        .finish()
    .build()?;
```