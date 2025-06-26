# Core Module Documentation

The core module implements Tapu Simu's fundamental battle architecture with clean separation of concerns and explicit multi-format support. Every component is designed with position-based targeting and format awareness as first-class features.

## Architecture Overview

The core module provides the foundational types and systems for Pokemon battle simulation:
- **Battle Orchestration**: Environment management and turn flow
- **Format Definition**: Multi-format support with position-based targeting  
- **State Management**: Immutable battle state with instruction-based mutations
- **Move Selection**: Explicit targeting system for all battle formats
- **Instruction System**: Atomic state modifications with rollback support

## Key Components

### Battle Environment (`battle_environment.rs`)

The battle orchestration layer managing turn flow, player AI, and battle outcomes.

**Core Types:**
- `Player` trait: AI interface with `choose_move()` and `name()` methods
- `RandomPlayer`: Selects random valid moves
- `FirstMovePlayer`: Always chooses the first available move
- `DamageMaximizer`: Attempts to maximize damage output
- `BattleEnvironment`: Main battle orchestrator
- `BattleResult`: Complete battle outcome with turn-by-turn history
- `TurnInfo`: Individual turn data including moves, damage, and state changes

**Battle Execution:**
- `run_battle_from_state()`: Execute single battle from initial state
- `run_parallel_battles_with_states()`: Thread-safe parallel execution
- `sample_instruction_index()`: Probabilistic instruction sampling for damage rolls

**Logging & Export:**
- Complete Showdown replay export with format-specific adaptations
- Team statistics tracking (damage dealt, healing, etc.)
- Move history and turn progression logging

### Battle Format (`battle_format.rs`)

Format definition and position management system supporting Singles, Doubles, VGC, and Triples.

**Format Types:**
- `FormatType`: Singles (1), Doubles (2), VGC (2), Triples (3) with active Pokemon counts
- `BattleFormat`: Complete format specification with generation, clauses, and ban lists
- Built-in formats: Gen 1-9 OU, VGC 2023-2024, Random Battle variants

**Position System:**
- `BattlePosition`: Explicit position addressing with `SideReference` and `slot: u8`
- `SideReference`: Type-safe side identification (SideOne, SideTwo)
- `all_positions()`: Generate all valid positions for current format
- `ally_position()`: Get ally position for doubles/triples
- `opponent_positions()`: Get all opponent positions based on format

**Format Features:**
- **Clauses**: Sleep, Species, Item, Evasion, OHKO bans
- **Ban Lists**: Species, moves, items, abilities with generation-specific enforcement
- **Spread Moves**: Format-aware spread move detection
- **Position Validation**: Ensure targeting legality for format constraints

### Move Choice (`move_choice.rs`)

Explicit move selection system with comprehensive targeting support.

**Move Choice Types:**
```rust
pub enum MoveChoice {
    Move(MoveIndex, Vec<BattlePosition>),     // Standard move with targets
    MoveTera(MoveIndex, PokemonType, Vec<BattlePosition>), // Gen 9+ Terastallization
    Switch(PokemonIndex),                      // Switch to party Pokemon
    None,                                      // No action (forced switch, etc.)
}
```

**Type Safety:**
- `MoveIndex`: Type-safe move slot addressing (M0, M1, M2, M3)
- `PokemonIndex`: Type-safe team position addressing (P0-P5)
- `PokemonType`: Complete type system with Gen 9 Terastallization support

**Targeting Features:**
- Multi-target support through `Vec<BattlePosition>`
- Spread move detection via `is_spread_move()` and `affects_allies()`
- Target validation against battle state and format constraints
- Human-readable logging with position-aware formatting

### Pokemon State (`pokemon_state.rs`)

Complete Pokemon battle representation with context-aware stat calculations.

**Core Types:**
- `Pokemon`: Full battle Pokemon with stats, moves, status, and volatile conditions
- `Move`: Battle move with PP, accuracy, targeting, category, and priority
- `DamageInfo`: Turn-based damage tracking for counter moves

**Stat System:**
- Base stats with nature modifications
- Stat boosts (-6 to +6) with tier-based multipliers
- `get_effective_speed()`: Context-aware speed with Trick Room, paralysis, items
- Weather, item, and ability stat modifications
- IV/EV integration with level-based calculations

**Battle Integration:**
- Substitute health tracking and volatile status duration
- Status condition management (Major: Burn/Paralysis/etc., Volatile: 60+ types)
- Item consumption tracking and ability suppression
- Terastallization state with type changing mechanics (Gen 9+)

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

### Battle State (`battle_state/`)

Decomposed state management with immutable design and instruction-based mutations.

#### Main State (`mod.rs`)

**BattleState Structure:**
- `format: BattleFormat`: Current battle format and rules
- `sides: [BattleSide; 2]`: Both player sides with teams and active Pokemon
- `field: FieldConditions`: Weather, terrain, and global effects
- `turn: TurnState`: Turn tracking with move order and damage history

**State Operations:**
- Immutable state with instruction-based mutations
- Multi-format team initialization from data or pre-built Pokemon
- `is_battle_over()` and `get_winner()`: Battle completion detection
- `get_all_options()`: Format-aware move option generation

#### Field Conditions (`field.rs`)

**Weather System:**
- `WeatherState`: Current weather with source position and turn duration
- Auto-expiration after 5 turns (8 with items)
- Weather effects on damage, accuracy, and healing

**Terrain System:**  
- `TerrainState`: Active terrain with source and duration tracking
- Priority modification, status prevention, and damage boosts
- Auto-expiration after 5 turns (8 with items)

**Global Effects:**
- `TrickRoomState`: Speed inversion with 5-turn duration
- `GravityState`: Flying immunity removal and accuracy boost
- Turn-based decrementation with automatic cleanup

#### Side Management (`side.rs`)

**BattleSide Structure:**
- `team: Vec<Pokemon>`: Complete team (6 Pokemon)
- `active_positions: BTreeMap<u8, usize>`: Slot-to-team mapping
- `volatile_status: SideVolatileStatus`: Side-wide effects
- `damage_dealt: DamageDealt`: Counter move tracking

**Active Pokemon Management:**
- Format-aware active Pokemon indexing
- Switch validation against fainted Pokemon
- Bench Pokemon availability checking
- Wish and Future Sight effect scheduling

### Instruction System (`instructions/`)

Atomic state modification system with comprehensive rollback support.

#### Instruction Architecture

**BattleInstruction Enum:**
```rust
pub enum BattleInstruction {
    Pokemon(PokemonInstruction),    // Pokemon-specific changes
    Field(FieldInstruction),        // Field condition changes  
    Status(StatusInstruction),      // Status effect changes
    Stats(StatsInstruction),        // Stat modification changes
}
```

**BattleInstructions:**
- Probabilistic instruction collections with damage roll sampling
- `affected_positions: Vec<BattlePosition>`: Position tracking for all effects
- Previous state storage for complete undo support
- Instruction probability distribution for damage variance

#### Pokemon Instructions (`pokemon.rs`)

**Damage & Healing:**
- `Damage(BattlePosition, u16)`: Direct HP reduction
- `Heal(BattlePosition, u16)`: HP restoration with max HP clamping
- `SetHP(BattlePosition, u16)`: Absolute HP setting

**State Changes:**
- `Faint(BattlePosition)`: Pokemon fainting with cleanup
- `Switch(BattlePosition, PokemonIndex)`: Position switching
- `ChangeAbility(BattlePosition, String)`: Ability modification
- `ChangeItem(BattlePosition, String)`: Item changes
- `ChangeTypes(BattlePosition, Vec<PokemonType>)`: Type modification

**Special Mechanics:**
- `CreateSubstitute(BattlePosition, u16)`: Substitute creation with HP cost
- `DamageSubstitute(BattlePosition, u16)`: Substitute damage
- `BreakSubstitute(BattlePosition)`: Substitute destruction

#### Field Instructions (`field.rs`)

**Weather Control:**
- `SetWeather(WeatherState)`: Weather establishment with source tracking
- `ClearWeather`: Weather removal
- `EndWeather`: Turn-based weather expiration

**Terrain Control:**
- `SetTerrain(TerrainState)`: Terrain establishment
- `ClearTerrain`: Terrain removal  
- `EndTerrain`: Turn-based terrain expiration

**Global Effects:**
- `SetTrickRoom(TrickRoomState)`: Speed inversion activation
- `SetGravity(GravityState)`: Gravity effect activation
- Turn-based decrementation instructions

#### Status Instructions (`status.rs`)

**Major Status:**
- `ApplyMajorStatus(BattlePosition, String)`: Primary status conditions
- `ClearMajorStatus(BattlePosition)`: Status removal
- Burn, Paralysis, Sleep, Freeze, Poison, Bad Poison support

**Volatile Status:**
- `ApplyVolatileStatus(BattlePosition, String, Option<u8>)`: Temporary effects
- `ClearVolatileStatus(BattlePosition, String)`: Effect removal
- 60+ volatile status types with duration tracking

**Move Effects:**
- `SetPP(BattlePosition, MoveIndex, u8)`: PP modification
- `DisableMove(BattlePosition, MoveIndex, u8)`: Move disabling
- Sleep turn tracking and wake-up mechanics

#### Stats Instructions (`stats.rs`)

**Stat Boosts:**
- `BoostStat(BattlePosition, String, i8)`: Stat stage modification
- `SetStatBoost(BattlePosition, String, i8)`: Absolute boost setting
- -6 to +6 clamping with tier-based multiplier system

**Advanced Stats:**
- `ModifyStatRaw(BattlePosition, String, u16)`: Raw stat changes
- `CopyStatBoosts(BattlePosition, BattlePosition)`: Boost copying
- `SwapStatBoosts(BattlePosition, BattlePosition)`: Boost swapping
- `InvertStatBoosts(BattlePosition)`: Boost sign inversion

## Component Integration

1. **BattleEnvironment** orchestrates turns using **BattleState** and **Player** implementations
2. **BattleState** contains **BattleSide**s, **FieldConditions**, and **TurnState**  
3. **MoveChoice** selections are validated and resolved through the **targeting** system
4. **Instructions** atomically modify state components with rollback support
5. **BattleFormat** constrains all targeting, validation, and move generation
6. **Pokemon** state integrates with field conditions for effective stat calculations