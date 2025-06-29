# Core Module Documentation

The core module implements Tapu Simu's fundamental battle architecture with clean separation of concerns and explicit multi-format support. Every component is designed with position-based targeting, format awareness, and comprehensive type safety as first-class features.

## Architecture Overview

The core module (`src/core/`) provides the foundational types and systems for Pokemon battle simulation:

- **Battle Orchestration** - Environment management and turn flow (`battle_environment.rs`)
- **Format Definition** - Multi-format support with position-based targeting (`battle_format.rs`)
- **State Management** - Immutable battle state with instruction-based mutations (`battle_state/`)
- **Move Selection** - Explicit targeting system with type-safe move references (`move_choice.rs`)
- **Instruction System** - Atomic state modifications with comprehensive rollback support (`instructions/`)
- **Targeting System** - Unified targeting resolution and validation (`targeting.rs`)

## Core Components

### Battle Format (`battle_format.rs`)

Format definition and position management system supporting Singles, Doubles, VGC, and Triples.

**Format Types:**
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FormatType {
    Singles,    // 1 active Pokemon per side
    Doubles,    // 2 active Pokemon per side  
    Vgc,        // 2 active Pokemon per side with VGC rules
    Triples,    // 3 active Pokemon per side (deprecated in modern Pokemon)
}

impl FormatType {
    pub fn active_pokemon_count(&self) -> usize {
        match self {
            FormatType::Singles => 1,
            FormatType::Doubles | FormatType::Vgc => 2,
            FormatType::Triples => 3,
        }
    }

    pub fn supports_spread_moves(&self) -> bool {
        match self {
            FormatType::Singles => false,
            FormatType::Doubles | FormatType::Vgc | FormatType::Triples => true,
        }
    }
}
```

**Position System:**
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BattlePosition {
    pub side: SideReference,
    pub slot: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SideReference {
    SideOne,
    SideTwo,
}
```

**BattleFormat Features:**
- Complete format specification with generation, clauses, and ban lists
- Built-in formats: Gen 1-9 OU, VGC 2023-2024, Random Battle variants
- Clause system: Sleep, Species, Item, Evasion, OHKO bans
- Ban list management for species, moves, items, abilities
- Format-aware spread move detection and validation

### Move Choice (`move_choice.rs`)

Explicit move selection system with comprehensive targeting support and type safety.

**Move Choice Types:**
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MoveChoice {
    /// Standard move with explicit targeting
    Move {
        move_index: MoveIndex,
        target_positions: Vec<BattlePosition>,
    },
    /// Gen 9+ Terastallization moves
    MoveTera {
        move_index: MoveIndex,
        target_positions: Vec<BattlePosition>,
        tera_type: PokemonType,
    },
    /// Switch to party Pokemon by index
    Switch(PokemonIndex),
    /// No action (speed calculations, forced moves)
    None,
}
```

**Type Safety System:**
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MoveIndex {
    M0, M1, M2, M3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PokemonIndex {
    P0, P1, P2, P3, P4, P5,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PokemonType {
    Normal, Fire, Water, Electric, Grass, Ice, Fighting, Poison,
    Ground, Flying, Psychic, Bug, Rock, Ghost, Dragon, Dark,
    Steel, Fairy, // Gen 9+ includes all 18 types for Terastallization
}
```

**Move Choice Features:**
- Multi-target support through `Vec<BattlePosition>`
- Type-safe move slot addressing with `MoveIndex` enum
- Team position addressing with `PokemonIndex` enum
- Gen 9+ Terastallization support with type specification
- Move validation against battle state and format constraints
- Human-readable logging with position-aware formatting

### Battle State (`battle_state/`)

Decomposed state management with immutable design and instruction-based mutations.

#### Main State (`mod.rs`)

**BattleState Structure:**
```rust
#[derive(Clone, Serialize)]
pub struct BattleState {
    /// The battle format determining rules and active Pokemon count
    pub format: BattleFormat,
    /// The two battle sides (always exactly 2)
    pub sides: [BattleSide; 2],
    /// Field conditions affecting the entire battlefield
    pub field: FieldConditions,
    /// Turn-related state information
    pub turn_info: TurnState,
    /// Generation-specific data repository
    #[serde(skip)]
    pub generation_repo: Arc<GenerationRepository>,
    /// Game data repository  
    #[serde(skip)]
    pub game_data_repo: Arc<GameDataRepository>,
}
```

**State Operations:**
- Immutable state design with instruction-based mutations
- Multi-format initialization with comprehensive format support
- Battle completion detection and winner determination
- Format-aware move option generation
- Repository pattern for data access separation

#### Field Conditions (`field.rs`)

**Weather System:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherState {
    pub weather: Weather,
    pub source_position: Option<BattlePosition>,
    pub turns_remaining: Option<u8>,
}
```

**Terrain System:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainState {
    pub terrain: Terrain,
    pub source_position: Option<BattlePosition>,
    pub turns_remaining: Option<u8>,
}
```

**Global Effects:**
- `TrickRoomState`: Speed inversion with 5-turn duration
- `GravityState`: Flying immunity removal and accuracy boost
- Turn-based decrementation with automatic cleanup
- Weather effects on damage, accuracy, and healing
- Terrain effects on priority, status prevention, and damage boosts

#### Side Management (`side.rs`)

**BattleSide Structure:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleSide {
    /// Complete team (up to 6 Pokemon)
    pub team: Vec<Pokemon>,
    /// Mapping from battle slot to team index
    pub active_positions: BTreeMap<u8, usize>,
    /// Side-wide volatile status effects
    pub volatile_status: SideVolatileStatus,
    /// Damage tracking for counter moves
    pub damage_dealt: DamageDealt,
}
```

**Active Pokemon Management:**
- Format-aware active Pokemon indexing
- Switch validation against fainted Pokemon
- Bench Pokemon availability checking
- Wish and Future Sight effect scheduling

#### Pokemon State (`pokemon.rs`)

**Pokemon Structure:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pokemon {
    // Core identity
    pub species: PokemonName,
    pub level: u8,
    pub types: Vec<String>,
    
    // Battle stats
    pub stats: StatSpread,
    pub stat_boosts: StatBoostArray,
    
    // Moves and abilities
    pub moves: [Option<Move>; 4],
    pub ability: Option<Abilities>,
    pub item: Option<Items>,
    
    // Battle state
    pub hp: i16,
    pub status: Option<PokemonStatus>,
    pub volatile_status: HashMap<VolatileStatus, u8>,
    
    // Gen 9+ Terastallization
    pub terastallized: bool,
    pub tera_type: Option<PokemonType>,
}
```

**Pokemon Features:**
- Complete battle Pokemon with stats, moves, status, and volatile conditions
- Context-aware stat calculations with field condition integration
- Status condition management (Major and Volatile status types)
- Item consumption tracking and ability suppression
- Terastallization state with type changing mechanics (Gen 9+)
- Type-safe ability and item references using strongly-typed enums

### Targeting System (`targeting.rs`)

Unified targeting resolution and validation for all move types.

**Core Functions:**
- `resolve_targets()`: Convert Pokemon Showdown targets to position lists
- `validate_targets()`: Ensure targeting legality for move type and battle state
- `auto_resolve_targets()`: Automatic target resolution for AI players

**Target Types Supported:**
- **Single Targets**: Normal, Adjacent, Any, Self
- **Multi Targets**: AllAdjacentFoes, AllAdjacent, AllAllies, All
- **Special Targets**: Scripted (Counter), RandomNormal, Allies

**Format Integration:**
- Spread move handling with format-specific behavior
- Active position validation against fainted Pokemon
- Default targeting with format-aware fallbacks
- Ally position detection for doubles/triples

### Instruction System (`instructions/`)

Domain-grouped atomic state modification system with comprehensive rollback support.

#### Instruction Architecture

**BattleInstruction Enum:**
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BattleInstruction {
    /// Pokemon-related instructions (damage, healing, fainting, switching)
    Pokemon(PokemonInstruction),
    /// Field-related instructions (weather, terrain, global effects)
    Field(FieldInstruction),
    /// Status-related instructions (status conditions, volatile statuses)
    Status(StatusInstruction),
    /// Stats-related instructions (boosts, raw stat changes)
    Stats(StatsInstruction),
}
```

**BattleInstructions Collection:**
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BattleInstructions {
    /// Probability of this instruction set occurring (0.0 to 100.0)
    pub probability: f64,
    /// All instructions in this collection
    pub instructions: Vec<BattleInstruction>,
    /// All positions affected by these instructions
    pub affected_positions: Vec<BattlePosition>,
}
```

#### Pokemon Instructions (`pokemon.rs`)

**Damage & Healing:**
```rust
pub enum PokemonInstruction {
    Damage {
        target: BattlePosition,
        amount: i16,
        previous_hp: Option<i16>,
    },
    Heal {
        target: BattlePosition,
        amount: i16,
        previous_hp: Option<i16>,
    },
    MultiTargetDamage {
        target_damages: Vec<(BattlePosition, i16)>,
        previous_hps: Vec<(BattlePosition, Option<i16>)>,
    },
    // ... more variants
}
```

**State Changes:**
- `Faint`: Pokemon fainting with full state preservation
- `Switch`: Position switching with team indexing
- `ChangeAbility`: Type-safe ability changes with rollback
- `ChangeItem`: Type-safe item changes with rollback
- `ChangeType`: Type modification with rollback support

**Terastallization & Form Changes:**
- `ToggleTerastallized`: Gen 9+ Terastallization with type specification
- `FormeChange`: Pokemon forme changes with rollback
- `ChangeSubstituteHealth`: Substitute health tracking

#### Field Instructions (`field.rs`)

**Weather & Terrain Control:**
```rust
pub enum FieldInstruction {
    SetWeather(WeatherState),
    ClearWeather,
    EndWeather,
    SetTerrain(TerrainState),
    ClearTerrain,
    EndTerrain,
    // ... more variants
}
```

**Global Effects:**
- `SetTrickRoom`: Speed inversion activation
- `SetGravity`: Gravity effect activation
- Turn-based decrementation instructions
- Side condition management

#### Status Instructions (`status.rs`)

**Status Conditions:**
```rust
pub enum StatusInstruction {
    Apply {
        target: BattlePosition,
        status: PokemonStatus,
        duration: Option<u8>,
        previous_status: Option<PokemonStatus>,
        previous_duration: Option<u8>,
    },
    Remove {
        target: BattlePosition,
        status: PokemonStatus,
        previous_duration: Option<u8>,
    },
    // ... more variants
}
```

**Volatile Status Management:**
- `ApplyVolatile`: Temporary effects with duration tracking
- `RemoveVolatile`: Effect removal with rollback support
- `ChangeVolatileDuration`: Duration modification

**Move and PP Management:**
- `DecrementPP`: PP reduction with undo support
- `DisableMove`: Move disabling with duration tracking
- `SetLastUsedMove`: Last move tracking with type safety

#### Stats Instructions (`stats.rs`)

**Stat Boosts:**
```rust
pub enum StatsInstruction {
    BoostStats {
        target: BattlePosition,
        stat_changes: StatBoosts,
        previous_boosts: StatBoosts,
    },
    ClearBoosts {
        target: BattlePosition,
        previous_boosts: StatBoostArray,
    },
    // ... more variants
}
```

**Raw Stat Changes:**
- Direct attack, defense, special attack, special defense, speed modifications
- All with previous value storage for rollback support

**Advanced Stat Operations:**
- `SwapStats`: Stat swapping between Pokemon with rollback
- `CopyBoosts`: Copy stat boosts between Pokemon
- `InvertBoosts`: Boost sign inversion with rollback

### Battle Environment (`battle_environment.rs`)

High-level battle orchestration system managing turn order, player interactions, and battle progression.

**Core Components:**
- `Player` trait: Interface for AI players with move selection
- Battle orchestration: Turn management, instruction generation/application
- Player implementations: RandomPlayer, FirstMovePlayer, DamageMaximizer
- Parallel execution: Multi-threaded battle running with state management
- Comprehensive logging: Battle state serialization, Showdown export format