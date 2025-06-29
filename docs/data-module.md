# Tapu Simu Data Module - Technical Implementation Guide

The data module provides the foundational data management layer for Tapu Simu's multi-format Pokemon battle simulator. It implements sophisticated repository patterns, Pokemon Showdown integration, generation-aware data loading, and performance-optimized lookup systems.

## Core Architecture

The data module consists of seven primary components operating in a layered architecture:

1. **Data Types Layer** (`types.rs`): Core Pokemon data structures with type safety
2. **Pokemon Showdown Integration Layer** (`showdown_types.rs`): Direct PS JSON compatibility
3. **Repository Layer** (`repositories/`): Optimized data storage and retrieval
4. **Generation Management Layer** (`generation_loader.rs`): Multi-generation data handling
5. **Random Battle Layer** (`random_team_loader.rs`): Format-specific team generation
6. **Module Layer** (`mod.rs`): Public API and component integration
7. **Data Files Layer**: JSON data extracted from Pokemon Showdown

## Data Types Implementation (`types.rs`)

### Core Statistics Structure

```rust
pub struct Stats {
    pub hp: i16,
    pub attack: i16,
    pub defense: i16,
    pub special_attack: i16,
    pub special_defense: i16,
    pub speed: i16,
}
```

**Technical Details:**
- Uses `i16` for overflow protection and negative stat handling
- Implements `Default`, `Clone`, `Debug`, `PartialEq` for comprehensive functionality
- Provides arithmetic operations and display formatting

### Nature System Implementation

```rust
pub enum Nature {
    Hardy, Lonely, Brave, Adamant, Naughty,
    Bold, Docile, Relaxed, Impish, Lax,
    Timid, Hasty, Serious, Jolly, Naive,
    Modest, Mild, Quiet, Bashful, Rash,
    Calm, Gentle, Sassy, Careful, Quirky,
}
```

**Implementation Features:**
- **Stat Modifier Methods**: Individual methods for each stat (`attack_modifier()`, `defense_modifier()`, etc.)
- **Accurate Multipliers**: 1.1x boost, 0.9x reduction, 1.0x neutral
- **FromNormalizedString Trait**: Type-safe string conversion with error handling
- **Complete Coverage**: All 25 official Pokemon natures

**Nature Modifier Logic:**
```rust
impl Nature {
    pub fn attack_modifier(&self) -> f64 {
        match self {
            Nature::Lonely | Nature::Brave | Nature::Adamant | Nature::Naughty => 1.1,
            Nature::Bold | Nature::Timid | Nature::Modest | Nature::Calm => 0.9,
            _ => 1.0,
        }
    }
}
```

## Pokemon Showdown Integration (`showdown_types.rs`)

### Move Targeting System

```rust
pub enum MoveTarget {
    Normal,                    // Single adjacent opponent
    Self_,                     // User only
    AllAdjacentFoes,          // All adjacent opponents
    AllAdjacent,              // All adjacent Pokemon
    AllAllies,                // All ally Pokemon
    All,                      // All Pokemon on field
    Any,                      // Any single Pokemon
    Scripted,                 // Special script-defined targeting
    RandomNormal,             // Random adjacent opponent
    AllySide,                 // User's entire side
    FoeSide,                  // Opponent's entire side
    AllyTeam,                 // All allies including reserves
    AllOpponents,             // All opponents
    AllOpposingPokemon,       // All opponents on field
    AllPokemon,               // Every Pokemon in battle
}
```

**Multi-Format Targeting Logic:**
```rust
impl MoveTarget {
    pub fn requires_target_selection(&self, active_per_side: usize) -> bool {
        match self {
            MoveTarget::Normal | MoveTarget::Any => active_per_side > 1,
            MoveTarget::AllAdjacentFoes | MoveTarget::AllAdjacent => false,
            _ => false,
        }
    }

    pub fn get_default_targets(&self, active_per_side: usize) -> Vec<(usize, usize)> {
        match self {
            MoveTarget::Normal => vec![(1, 0)], // First opponent slot
            MoveTarget::AllAdjacentFoes => {
                // Returns all adjacent opponent positions based on format
                if active_per_side == 1 {
                    vec![(1, 0)]
                } else {
                    vec![(1, 0), (1, 1)]
                }
            }
            // ... complete targeting logic for all formats
        }
    }
}
```

### Move Data Structure

```rust
pub struct MoveData {
    pub id: String,
    pub name: String,
    pub type_: PokemonType,
    pub category: String,                    // Physical/Special/Status
    pub power: Option<u16>,
    pub accuracy: Option<u8>,
    pub pp: u8,
    pub priority: i8,
    pub target: MoveTarget,
    pub flags: MoveFlags,
    pub secondary_effects: Option<SecondaryEffect>,
    pub critical_hit_ratio: Option<u8>,
    pub flinch_chance: Option<u8>,
    pub healing: Option<HealingData>,
    pub drain: Option<DrainData>,
    pub multi_hit: Option<MultiHitData>,
    pub recoil: Option<RecoilData>,
    pub self_switch: Option<String>,
    pub weather: Option<String>,
    pub status: Option<StatusCondition>,
    pub boosts: Option<StatBoosts>,
    pub terrain: Option<String>,
    pub force_switch: bool,
    pub ohko: bool,
    pub sleep_usable: bool,
    pub z_move: Option<ZMoveData>,
    pub max_move: Option<MaxMoveData>,
    // ... 30+ total fields
}
```

**Advanced Features:**
- **Flag System**: `has_flag()` method for move property checking
- **Engine Conversion**: `to_engine_move()` for battle system integration
- **Custom Deserializers**: Type-safe JSON parsing with fallback handling
- **Secondary Effects**: Complete support for chance-based effects and conditions

### Pokemon Data Structure

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
    pub gender_ratio: GenderRatio,
    pub catch_rate: u8,
    pub base_experience: u16,
    pub growth_rate: String,
    pub egg_groups: Vec<String>,
    pub hatch_time: u16,
}
```

## Repository System (`repositories/`)

### GameDataRepository (Composite Pattern)

```rust
pub struct GameDataRepository {
    pub moves: MoveRepository,
    pub pokemon: PokemonRepository, 
    pub items: ItemRepository,
}

impl GameDataRepository {
    pub fn global() -> Arc<GameDataRepository> {
        static INSTANCE: OnceLock<Mutex<Option<Arc<GameDataRepository>>>> = OnceLock::new();
        
        let mutex = INSTANCE.get_or_init(|| Mutex::new(None));
        let mut guard = mutex.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        
        if let Some(ref instance) = *guard {
            instance.clone()
        } else {
            let instance = Arc::new(Self::new().expect("Failed to load game data"));
            *guard = Some(instance.clone());
            instance
        }
    }
}
```

**Singleton Features:**
- **Thread-Safe Access**: `OnceLock<Mutex<Option<Arc<T>>>>` pattern
- **Lazy Initialization**: Loads data only when first accessed  
- **Poison Recovery**: Handles mutex poisoning gracefully
- **Reference Counting**: Arc for shared ownership across threads

### MoveRepository Performance Optimizations

```rust
pub struct MoveRepository {
    moves: HashMap<MoveId, MoveData>,        // Primary storage
    name_index: HashMap<String, MoveId>,     // Name lookup index
}

impl MoveRepository {
    pub fn new() -> Result<Self, DataError> {
        let raw_data: HashMap<String, MoveData> = serde_json::from_str(&moves_json)?;
        
        // Pre-allocate with capacity estimation
        let capacity = raw_data.len();
        let mut moves = HashMap::with_capacity(capacity);
        let mut name_index = HashMap::with_capacity(capacity * 2); // 2x for name variations
        
        // Dual indexing construction
        for (id, move_data) in raw_data {
            let move_id = MoveId::from_id(&id);
            let normalized_name = normalize_name(&move_data.name);
            
            name_index.insert(normalized_name, move_id.clone());
            name_index.insert(id.clone(), move_id.clone()); // ID as name fallback
            moves.insert(move_id, move_data);
        }
        
        Ok(Self { moves, name_index })
    }

    pub fn find_by_name(&self, name: &str) -> Result<&MoveData, DataError> {
        let normalized = normalize_name(name);
        
        // O(1) index lookup
        if let Some(move_id) = self.name_index.get(&normalized) {
            return Ok(self.moves.get(move_id).unwrap()); // Safe unwrap - index guarantees existence
        }
        
        // O(n) fallback for edge cases
        self.moves.values()
            .find(|move_data| normalize_name(&move_data.name) == normalized)
            .ok_or_else(|| DataError::MoveNotFound(MoveId::from_name(name)))
    }
}
```

**Optimization Techniques:**
- **Dual Indexing**: Primary storage + normalized name index for O(1) lookups
- **Capacity Pre-allocation**: Prevents hash map rehashing during initialization
- **Index Guarantee**: Safe unwrapping after index lookup
- **Fallback Strategy**: Linear search for edge cases while maintaining performance

### PokemonRepository Implementation

```rust
pub struct PokemonRepository {
    pokemon: HashMap<SpeciesId, PokemonData>,
    name_index: HashMap<String, SpeciesId>,
}

impl PokemonRepository {
    fn extract_weight(data: &HashMap<String, serde_json::Value>) -> f32 {
        // PS-specific weight extraction from nested JSON
        data.get("weightkg").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32
    }
    
    pub fn find_by_weight_range(&self, min_kg: f32, max_kg: f32) -> Vec<&PokemonData> {
        self.pokemon.values()
            .filter(|pokemon| pokemon.weight_kg >= min_kg && pokemon.weight_kg <= max_kg)
            .collect()
    }
}
```

**Advanced Features:**
- **Weight Mechanics**: Special handling for fling damage and weight-based moves
- **Range Queries**: Efficient filtering for weight-based mechanics
- **Multi-Index Support**: Name, ID, and property-based lookups

### ItemRepository Specialized Methods

```rust
impl ItemRepository {
    pub fn get_fling_power(&self, item_name: &str) -> Option<u16> {
        self.find_by_name(item_name)
            .ok()
            .and_then(|item| item.fling_power)
    }
    
    pub fn is_choice_item(&self, item_name: &str) -> bool {
        matches!(
            normalize_name(item_name).as_str(),
            "choiceband" | "choicescarf" | "choicespecs"
        )
    }
    
    pub fn is_berry(&self, item_name: &str) -> bool {
        self.find_by_name(item_name)
            .map(|item| item.name.ends_with("Berry"))
            .unwrap_or(false)
    }
}
```

## Generation Management (`generation_loader.rs`)

### Multi-Generation Architecture

```rust
pub struct GenerationRepository {
    generations: HashMap<u8, GenerationData>,
    move_changes: HashMap<MoveId, Vec<MoveChange>>,
    pokemon_changes: HashMap<SpeciesId, Vec<PokemonChange>>,
    item_changes: HashMap<ItemId, Vec<ItemChange>>,
}

pub struct GenerationData {
    pub generation: u8,
    pub moves: HashMap<MoveId, MoveData>,
    pub pokemon: HashMap<SpeciesId, PokemonData>,
    pub items: HashMap<ItemId, ItemData>,
    pub metadata: GenerationMetadata,
}
```

### Generation-Aware Data Access

```rust
impl GenerationRepository {
    pub fn get_move_data(&self, move_id: &MoveId, generation: u8) -> Result<&MoveData, DataError> {
        // Current generation lookup
        if let Some(gen_data) = self.generations.get(&generation) {
            if let Some(move_data) = gen_data.moves.get(move_id) {
                return Ok(move_data);
            }
        }
        
        // Backward compatibility search
        for gen in (1..generation).rev() {
            if let Some(gen_data) = self.generations.get(&gen) {
                if let Some(move_data) = gen_data.moves.get(move_id) {
                    return Ok(move_data);
                }
            }
        }
        
        Err(DataError::MoveNotFound(move_id.clone()))
    }
    
    pub fn get_generation_stats(&self, generation: u8) -> GenerationStats {
        let gen_data = &self.generations[&generation];
        GenerationStats {
            generation,
            move_count: gen_data.moves.len(),
            pokemon_count: gen_data.pokemon.len(),
            item_count: gen_data.items.len(),
            new_moves: self.count_new_moves(generation),
            changed_moves: self.count_changed_moves(generation),
        }
    }
}
```

**Advanced Features:**
- **Fallback Logic**: Searches backward through generations for missing data
- **Change Tracking**: Complete audit trail of mechanical changes
- **Performance Optimized**: Single-pass generation validation
- **Statistical Analysis**: Generation comparison and change analysis

## Random Battle System (`random_team_loader.rs`)

### RandomPokemonSet Structure

```rust
pub struct RandomPokemonSet {
    pub species: String,
    pub level: u8,
    pub gender: Option<Gender>,
    pub ability: String,
    pub item: Option<String>,
    pub moves: Vec<String>,
    pub nature: Nature,
    pub evs: Stats,
    pub ivs: Stats,
    pub tera_type: Option<PokemonType>,  // Gen 9 Terastalization
}
```

### Battle Pokemon Conversion

```rust
impl RandomPokemonSet {
    pub fn to_battle_pokemon(&self, repository: &GameDataRepository) -> Result<Pokemon, DataError> {
        let pokemon_data = repository.pokemon.find_by_name(&self.species)?;
        
        // Accurate Pokemon stat calculation
        let stats = self.calculate_stats(pokemon_data)?;
        
        // Move conversion with error handling
        let moves = self.moves.iter()
            .take(4) // Enforce 4-move limit
            .map(|move_name| repository.moves.create_move(move_name))
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(Pokemon {
            species: pokemon_data.id.clone(),
            level: self.level,
            stats: self.apply_stat_optimizations(stats, repository),
            moves,
            nature: self.nature,
            ability: self.ability.clone(),
            item: self.item.clone(),
            gender: self.gender,
            tera_type: self.tera_type,
        })
    }
}
```

### Stat Optimization System

```rust
impl RandomPokemonSet {
    fn apply_stat_optimizations(&self, mut stats: Stats, repository: &GameDataRepository) -> Stats {
        // Physical move detection
        let has_physical_moves = self.moves.iter()
            .any(|move_name| {
                repository.moves.find_by_name(move_name)
                    .map(|move_data| move_data.category == "Physical")
                    .unwrap_or(false)
            });
        
        // Special attacker optimization (minimize Foul Play damage)
        if !has_physical_moves {
            stats.attack = stats.attack.saturating_sub(stats.attack / 4);
        }
        
        // Speed-dependent move detection
        let has_priority_moves = self.moves.iter()
            .any(|move_name| {
                repository.moves.find_by_name(move_name)
                    .map(|move_data| move_data.priority > 0)
                    .unwrap_or(false)
            });
        
        // Trick Room optimization
        if self.has_trick_room_indicator() && !has_priority_moves {
            stats.speed = stats.speed.saturating_sub(stats.speed / 2);
        }
        
        stats
    }
}
```

## Error Handling Architecture

```rust
#[derive(Debug, thiserror::Error)]
pub enum DataError {
    #[error("Move not found: {0}")]
    MoveNotFound(String),
    
    #[error("Pokemon not found: {0}")]
    PokemonNotFound(String),
    
    #[error("Item not found: {0}")]
    ItemNotFound(String),
    
    #[error("Generation {generation} not supported")]
    UnsupportedGeneration { generation: u8 },
    
    #[error("Invalid data format: {message}")]
    InvalidFormat { message: String },
    
    #[error("Repository not initialized")]
    RepositoryNotInitialized,
    
    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
}
```