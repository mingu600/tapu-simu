# Data Module Documentation

The data module implements comprehensive Pokemon data management with Pokemon Showdown integration, generation-aware loading, and efficient repository patterns. It provides the data foundation for all battle calculations and team building.

## Architecture Overview

The data module consists of six main components:
- **Data Types**: Internal and Pokemon Showdown data structures
- **Repository System**: Type-safe data storage and access patterns
- **Pokemon Showdown Integration**: Direct PS JSON format support
- **Generation Management**: Generation-specific data loading and change tracking
- **Random Team Generation**: Format-specific team building
- **Data Conversion**: Type-safe conversion between data formats

## Data Types

### Internal Types (`types.rs`)

Engine-optimized data structures for battle calculations.

**Core Battle Types:**
```rust
pub struct EngineBaseStats {
    pub hp: i16,
    pub attack: i16,
    pub defense: i16,
    pub special_attack: i16,
    pub special_defense: i16,
    pub speed: i16,
}

pub struct EnginePokemonData {
    pub id: SpeciesId,
    pub name: String,
    pub types: Vec<PokemonType>,
    pub base_stats: EngineBaseStats,
    pub abilities: Vec<String>,
    pub weight: f32,
    pub height: f32,
}
```

**Nature System:**
```rust
pub enum Nature {
    Hardy, Lonely, Brave, Adamant, Naughty,
    Bold, Docile, Relaxed, Impish, Lax,
    // ... complete nature enum
}

impl Nature {
    pub fn get_stat_modifier(&self, stat: &str) -> f32 {
        // 1.1x for boosted stats, 0.9x for reduced stats, 1.0x for neutral
    }
}
```

**Type Effectiveness:**
```rust
pub struct TypeEffectiveness {
    effectiveness_chart: HashMap<(PokemonType, PokemonType), f32>,
}

impl TypeEffectiveness {
    pub fn get_effectiveness(&self, attacking_type: PokemonType, defending_type: PokemonType) -> f32 {
        // Returns 0.0, 0.25, 0.5, 1.0, 2.0, or 4.0
    }
}
```

### Pokemon Showdown Types (`showdown_types.rs`)

Direct representations of Pokemon Showdown data formats.

**Move Data:**
```rust
pub struct MoveData {
    pub id: String,
    pub name: String,
    pub type_: PokemonType,
    pub category: String,
    pub power: Option<u16>,
    pub accuracy: Option<u8>,
    pub pp: u8,
    pub priority: i8,
    pub target: MoveTarget,
    pub flags: MoveFlags,
    pub secondary_effects: Option<SecondaryEffect>,
}
```

**Pokemon Data:**
```rust
pub struct PokemonData {
    pub id: String,
    pub name: String,
    pub types: Vec<PokemonType>,
    pub base_stats: BaseStats,
    pub abilities: AbilitySet,
    pub height_m: f32,
    pub weight_kg: f32,
    pub color: String,
    pub evolution_stage: u8,
    pub evolution_line: Vec<String>,
    pub forms: Vec<String>,
}
```

**Move Targeting:**
```rust
pub enum MoveTarget {
    Normal,                    // Single adjacent target
    Self_,                     // User only
    AllAdjacentFoes,          // All adjacent opponents
    AllAdjacent,              // All adjacent Pokemon
    AllAllies,                // All ally Pokemon
    All,                      // All Pokemon on field
    Any,                      // Any Pokemon on field
    Scripted,                 // Special targeting (Counter, etc.)
    RandomNormal,             // Random adjacent opponent
    // ... complete targeting system
}
```

## Repository System (`repositories/`)

Type-safe repository pattern with optimized lookups and comprehensive error handling.

### Type-Safe Identifiers (`types/identifiers.rs`)

Normalized identifiers prevent lookup errors and ensure consistency.

```rust
pub struct SpeciesId(String);
pub struct MoveId(String);
pub struct ItemId(String);
pub struct AbilityId(String);
pub struct TypeId(String);

impl SpeciesId {
    pub fn from_name(name: &str) -> Self {
        Self(normalize_name(name))
    }
}

// normalize_name removes spaces, hyphens, apostrophes, dots and converts to lowercase
fn normalize_name(name: &str) -> String {
    name.chars()
        .filter(|c| !matches!(c, ' ' | '-' | '\'' | '.'))
        .map(|c| c.to_lowercase().to_string())
        .collect()
}
```

### Repository Implementation Pattern

Each repository follows a consistent pattern for data storage and access.

**Pokemon Repository (`pokemon_repository.rs`):**
```rust
pub struct PokemonRepository {
    pokemon_by_id: HashMap<SpeciesId, PokemonData>,
    name_index: HashMap<String, SpeciesId>,
}

impl PokemonRepository {
    pub fn find_by_id(&self, id: &SpeciesId) -> Result<&PokemonData, DataError> {
        self.pokemon_by_id.get(id)
            .ok_or_else(|| DataError::PokemonNotFound(id.clone()))
    }

    pub fn find_by_name(&self, name: &str) -> Result<&PokemonData, DataError> {
        let normalized_name = normalize_name(name);
        
        // Try name index first (O(1) lookup)
        if let Some(id) = self.name_index.get(&normalized_name) {
            return self.find_by_id(id);
        }
        
        // Fallback to linear search for edge cases
        self.pokemon_by_id.values()
            .find(|pokemon| normalize_name(&pokemon.name) == normalized_name)
            .ok_or_else(|| DataError::PokemonNotFound(SpeciesId::from_name(name)))
    }
}
```

**Move Repository (`move_repository.rs`):**
- Identical pattern with move-specific data
- Additional lookup by move category and type
- Generation-aware move availability checking

**Item Repository (`item_repository.rs`):**
- Item data management with type categorization
- Berry, gem, and plate special handling
- Generation-specific item availability

**Ability Repository (`ability_repository.rs`):**
- Ability data with effect descriptions
- Generation-specific ability availability
- Ability interaction metadata

### Composite Repository (`mod.rs`)

Central access point combining all repositories.

```rust
pub struct GameDataRepository {
    pub pokemon: PokemonRepository,
    pub moves: MoveRepository,
    pub items: ItemRepository,
    pub abilities: AbilityRepository,
}

impl GameDataRepository {
    pub fn new() -> Result<Self, DataError> {
        Ok(Self {
            pokemon: PokemonRepository::load()?,
            moves: MoveRepository::load()?,
            items: ItemRepository::load()?,
            abilities: AbilityRepository::load()?,
        })
    }

    pub fn get_repository_stats(&self) -> RepositoryStats {
        RepositoryStats {
            pokemon_count: self.pokemon.len(),
            move_count: self.moves.len(),
            item_count: self.items.len(),
            ability_count: self.abilities.len(),
        }
    }
}
```

## Pokemon Showdown Integration

### Data Conversion (`conversion.rs`)

Type-safe conversion between Pokemon Showdown and engine formats.

**Move Conversion:**
```rust
pub fn convert_move_data(ps_move: &MoveData) -> EngineMoveData {
    EngineMoveData {
        id: MoveId::from_name(&ps_move.name),
        name: ps_move.name.clone(),
        type_: ps_move.type_,
        category: parse_move_category(&ps_move.category),
        power: ps_move.power.unwrap_or(0),
        accuracy: ps_move.accuracy.unwrap_or(100),
        pp: ps_move.pp,
        priority: ps_move.priority,
        target: ps_move.target,
        contact: ps_move.flags.contact,
        protect: ps_move.flags.protect,
        // ... additional field conversions
    }
}
```

**Pokemon Conversion:**
```rust
pub fn convert_pokemon_data(ps_pokemon: &PokemonData) -> EnginePokemonData {
    EnginePokemonData {
        id: SpeciesId::from_name(&ps_pokemon.name),
        name: ps_pokemon.name.clone(),
        types: ps_pokemon.types.clone(),
        base_stats: EngineBaseStats {
            hp: ps_pokemon.base_stats.hp as i16,
            attack: ps_pokemon.base_stats.atk as i16,
            defense: ps_pokemon.base_stats.def as i16,
            special_attack: ps_pokemon.base_stats.spa as i16,
            special_defense: ps_pokemon.base_stats.spd as i16,
            speed: ps_pokemon.base_stats.spe as i16,
        },
        abilities: ps_pokemon.abilities.get_all_abilities(),
        weight: ps_pokemon.weight_kg,
        height: ps_pokemon.height_m,
    }
}
```

### Target Resolution

Move targeting system with multi-format support.

```rust
impl MoveTarget {
    pub fn get_default_targets(&self, user_position: BattlePosition, format: &BattleFormat) -> Vec<BattlePosition> {
        match self {
            MoveTarget::Normal => {
                // Format-aware opponent targeting
                user_position.opponent_positions(format)
                    .into_iter()
                    .take(1)
                    .collect()
            }
            MoveTarget::AllAdjacentFoes => {
                // All adjacent opponents based on format
                user_position.adjacent_opponent_positions(format)
            }
            MoveTarget::AllAdjacent => {
                // All adjacent Pokemon (allies and opponents)
                user_position.adjacent_positions(format)
            }
            // ... complete targeting logic
        }
    }

    pub fn is_spread_move(&self) -> bool {
        matches!(self, 
            MoveTarget::AllAdjacentFoes | 
            MoveTarget::AllAdjacent | 
            MoveTarget::All
        )
    }
}
```

## Generation Management (`generation_loader.rs`)

Generation-specific data loading with change tracking and fallback mechanisms.

### Generation Repository Structure

```rust
pub struct GenerationMoveData {
    pub generation: u8,
    pub moves: HashMap<MoveId, MoveData>,
    pub changes: HashMap<MoveId, Vec<MoveChange>>,
}

pub struct GenerationPokemonData {
    pub generation: u8,
    pub pokemon: HashMap<SpeciesId, PokemonData>,
    pub changes: HashMap<SpeciesId, Vec<PokemonChange>>,
}

pub struct GenerationItemData {
    pub generation: u8,
    pub items: HashMap<ItemId, ItemData>,
    pub changes: HashMap<ItemId, Vec<ItemChange>>,
}
```

### Change Tracking System

```rust
pub struct MoveChange {
    pub generation: u8,
    pub field: String,
    pub old_value: serde_json::Value,
    pub new_value: serde_json::Value,
    pub reason: Option<String>,
}

pub struct PokemonChange {
    pub generation: u8,
    pub field: String,
    pub old_value: serde_json::Value,
    pub new_value: serde_json::Value,
    pub reason: Option<String>,
}
```

### Generation-Aware Data Access

```rust
impl GenerationLoader {
    pub fn get_move_data(&self, move_id: &MoveId, generation: u8) -> Result<&MoveData, DataError> {
        // Try current generation first
        if let Some(gen_data) = self.generations.get(&generation) {
            if let Some(move_data) = gen_data.moves.get(move_id) {
                return Ok(move_data);
            }
        }
        
        // Fallback to earlier generations
        for gen in (1..generation).rev() {
            if let Some(gen_data) = self.generations.get(&gen) {
                if let Some(move_data) = gen_data.moves.get(move_id) {
                    return Ok(move_data);
                }
            }
        }
        
        Err(DataError::MoveNotFound(move_id.clone()))
    }

    pub fn get_move_changes(&self, move_id: &MoveId, generation: u8) -> Vec<&MoveChange> {
        self.generations.get(&generation)
            .and_then(|gen_data| gen_data.changes.get(move_id))
            .map(|changes| changes.iter().collect())
            .unwrap_or_default()
    }
}
```

## Random Team Generation (`random_team_loader.rs`)

Format-specific team building with Pokemon Showdown random battle compatibility.

### Team Loading System

```rust
pub struct RandomTeamLoader {
    teams_by_format: HashMap<String, Vec<RandomBattleTeam>>,
    rng: StdRng,
}

impl RandomTeamLoader {
    pub fn load_team(&mut self, format: &str) -> Result<Vec<Pokemon>, DataError> {
        let teams = self.teams_by_format.get(format)
            .ok_or_else(|| DataError::FormatNotFound(format.to_string()))?;
        
        let team_data = teams.choose(&mut self.rng)
            .ok_or_else(|| DataError::NoTeamsAvailable)?;
        
        team_data.pokemon.iter()
            .map(|pokemon_set| self.convert_pokemon_set(pokemon_set))
            .collect()
    }

    fn convert_pokemon_set(&self, set: &RandomBattlePokemonSet) -> Result<Pokemon, DataError> {
        let pokemon_data = self.data_repository.pokemon.find_by_name(&set.species)?;
        
        // Calculate stats with proper formulas
        let stats = self.calculate_stats(pokemon_data, &set.evs, &set.ivs, set.level, &set.nature)?;
        
        // Convert moves
        let moves = set.moves.iter()
            .map(|move_name| self.convert_move(move_name))
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(Pokemon {
            species: pokemon_data.name.clone(),
            level: set.level,
            stats,
            moves,
            nature: set.nature,
            ability: set.ability.clone(),
            item: set.item.clone(),
            gender: set.gender,
            // ... additional fields
        })
    }
}
```

### Stat Calculation System

```rust
impl RandomTeamLoader {
    fn calculate_stats(
        &self,
        pokemon_data: &PokemonData,
        evs: &StatSpread,
        ivs: &StatSpread,
        level: u8,
        nature: &Nature,
    ) -> Result<StatSpread, DataError> {
        let mut stats = StatSpread::default();
        
        // HP calculation (different formula)
        stats.hp = if pokemon_data.base_stats.hp == 1 {
            1  // Shedinja special case
        } else {
            ((2 * pokemon_data.base_stats.hp + ivs.hp + evs.hp / 4) * level / 100 + level + 10) as u16
        };
        
        // Other stats calculation
        for stat in &["attack", "defense", "special_attack", "special_defense", "speed"] {
            let base_stat = pokemon_data.base_stats.get_stat(stat);
            let iv = ivs.get_stat(stat);
            let ev = evs.get_stat(stat);
            
            let calculated_stat = (2 * base_stat + iv + ev / 4) * level / 100 + 5;
            let nature_modifier = nature.get_stat_modifier(stat);
            
            stats.set_stat(stat, (calculated_stat as f32 * nature_modifier) as u16);
        }
        
        Ok(stats)
    }
}
```

### EV/IV Optimization

```rust
impl RandomTeamLoader {
    fn optimize_evs_ivs(&self, pokemon_set: &mut RandomBattlePokemonSet) {
        // Smogon Random Battle rules
        
        // Special attackers get 0 Attack IV to minimize Foul Play damage
        if self.is_special_attacker(pokemon_set) {
            pokemon_set.ivs.attack = 0;
        }
        
        // Trick Room teams get 0 Speed IV
        if self.is_trick_room_team(pokemon_set) {
            pokemon_set.ivs.speed = 0;
        }
        
        // HP IV optimization for specific Pokemon
        if self.needs_hp_optimization(pokemon_set) {
            pokemon_set.ivs.hp = self.calculate_optimal_hp_iv(pokemon_set);
        }
    }
}
```

## Error Handling

Comprehensive error handling with detailed context and recovery strategies.

```rust
#[derive(Debug, thiserror::Error)]
pub enum DataError {
    #[error("Pokemon not found: {0:?}")]
    PokemonNotFound(SpeciesId),
    
    #[error("Move not found: {0:?}")]
    MoveNotFound(MoveId),
    
    #[error("Item not found: {0:?}")]
    ItemNotFound(ItemId),
    
    #[error("Ability not found: {0:?}")]
    AbilityNotFound(AbilityId),
    
    #[error("Format not found: {0}")]
    FormatNotFound(String),
    
    #[error("File I/O error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("Invalid data: {0}")]
    InvalidData(String),
}
```

## Integration Points

The data module integrates with:
- **Core Module**: Provides Pokemon, move, and item data for battle state
- **Engine Module**: Supplies data for damage calculation and move effects
- **Builders Module**: Provides data validation for team and battle building
- **Testing Module**: Supplies test data and validation utilities