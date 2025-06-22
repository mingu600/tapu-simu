# Tapu Simu Architecture Documentation

## Overview

Tapu Simu is a standalone, format-aware Pokémon battle simulator built from the ground up in Rust. It is designed to support multiple battle formats (Singles, Doubles, VGC, Triples) with a clean, modular architecture that prioritizes multi-format support and explicit position-based targeting.

**Key Architectural Principles:**
- **Format-First Design**: Every component assumes multi-format support
- **Position-Based Targeting**: All moves use explicit BattlePosition targeting 
- **Clean State Management**: Immutable state during instruction generation with atomic, reversible instructions
- **Modular Move Effects**: Move implementations are organized into focused modules by category
- **Generation Awareness**: All mechanics are generation-aware for proper historical accuracy

## Project Structure

```
src/
├── core/                    # Core battle abstractions and data structures
├── engine/                  # Battle mechanics implementation  
├── data/                    # Pokémon data integration (Pokemon Showdown)
├── builders/                # Builder patterns for battle setup
├── types/                   # Common type definitions and identifiers
├── ui/                      # Web interface bridge
├── testing/                 # Testing framework and utilities
├── generation.rs            # Generation-specific mechanics
├── simulator.rs             # High-level battle simulation
├── config.rs                # Configuration management
└── main.rs                  # CLI entry point
```

## Core Architecture Components

### 1. Battle State Management (`src/core/`)

#### **BattleState** (`src/core/battle_state.rs`)
The central state container that replaces the legacy monolithic State struct. Features:

- **Decomposed Structure**: Organized into logical components (sides, field, turn info)
- **Multi-format Support**: Adapts automatically to format requirements
- **Compatibility Layer**: Maintains API compatibility with legacy code during transition
- **Explicit Position Access**: All Pokemon access through BattlePosition

**Key Components:**
```rust
pub struct BattleState {
    pub format: BattleFormat,           // Format configuration
    pub sides: [BattleSide; 2],         // Two battle sides (always exactly 2)
    pub field: FieldConditions,         // Weather, terrain, global effects
    pub turn_info: TurnState,           // Turn tracking and move order
    // Legacy compatibility fields...
}
```

**Design Issues Identified:**
- Duplicated state between modern `sides` array and legacy `side_one`/`side_two` fields
- Legacy compatibility fields create maintenance burden
- Should consolidate to single state representation

#### **BattleFormat** (`src/core/battle_format.rs`)
Comprehensive format definition system supporting multiple competitive formats:

```rust
pub struct BattleFormat {
    pub name: String,                   // Display name (e.g., "Gen 9 OU")
    pub generation: Generation,         // Pokemon generation
    pub format_type: FormatType,        // Singles/Doubles/VGC/Triples  
    pub team_size: usize,               // Usually 6
    pub active_per_side: usize,         // Active Pokemon count
    pub clauses: Vec<FormatClause>,     // Sleep Clause, Species Clause, etc.
    pub ban_list: BanList,              // Banned Pokemon/moves/items/abilities
}
```

**Supported Formats:**
- `gen9_ou()`, `gen4_ou()` - Competitive singles formats
- `vgc2024()` - Official VGC tournament format  
- `doubles()` - Casual doubles format
- `gen9_random_battle()` - Random battle formats for multiple generations

#### **MoveChoice** (`src/core/move_choice.rs`)
Format-aware move selection system with explicit targeting:

```rust
pub enum MoveChoice {
    Move { 
        move_index: MoveIndex, 
        target_positions: Vec<BattlePosition> 
    },
    MoveTera { 
        move_index: MoveIndex, 
        target_positions: Vec<BattlePosition>,
        tera_type: PokemonType 
    },
    Switch(PokemonIndex),
    None,
}
```

#### **Instructions System** (`src/core/instructions/`)
Domain-grouped instruction system for atomic state modifications:

- **PokemonInstruction**: Damage, healing, switching, ability/item changes
- **FieldInstruction**: Weather, terrain, side conditions, global effects  
- **StatusInstruction**: Status conditions, volatile statuses
- **StatsInstruction**: Stat boosts, raw stat modifications

**Key Features:**
- Atomic operations with undo support via `previous_hp` fields
- Explicit position tracking via `affected_positions()`  
- Probabilistic instruction sets with percentage chances

### 2. Battle Engine (`src/engine/`)

#### **Turn Resolution** (`src/engine/turn.rs`)
Simplified turn processing that replaced complex generator hierarchy:

**Key Functions:**
- `generate_instructions()` - Main turn processing entry point
- `determine_move_order_advanced()` - Priority/speed/Pursuit handling
- `generate_move_instructions_with_enhanced_context()` - Context-aware move processing
- `end_of_turn::process_end_of_turn_effects()` - Status damage, weather effects

**Advanced Features:**
- **Context-Aware Move Generation**: Opponent move information for moves like Sucker Punch
- **Switch-Attack Interactions**: Proper handling of switches vs attacks (Pursuit mechanics)
- **Enhanced Accuracy Calculation**: Weather, abilities, items, stat stages

#### **Combat System** (`src/engine/combat/`)

**Damage Calculation** (`damage_calc.rs`):
- Generation-aware damage formulas
- Critical hit mechanics  
- Type effectiveness integration
- Spread move damage reduction (75% in multi-target scenarios)

**Move Effects** (`move_effects.rs` + `moves/`):
Comprehensive move effect system organized into 25+ focused modules:

- **Status Effects**: `thunder_wave`, `toxic`, `sleep_powder`
- **Stat Modifications**: `swords_dance`, `dragon_dance`, `nasty_plot`  
- **Healing**: `recover`, `synthesis`, `moonlight`
- **Recoil**: `double_edge`, `flare_blitz`, `brave_bird`
- **Protection**: `protect`, `detect`, `wide_guard`
- **Hazards**: `stealth_rock`, `spikes`, `toxic_spikes`
- **Variable Power**: `bolt_beak`, `fishious_rend`, `avalanche`
- **Priority Context**: `sucker_punch`, `upper_hand`, `me_first`

**Organization Strength**: Clear separation of concerns with each move category in its own module.

**Potential Issues**: Some modules may be too granular - could benefit from consolidation of closely related effects.

#### **Type Effectiveness** (`type_effectiveness.rs`)
Handles type chart interactions with generation awareness for mechanics changes over time.

### 3. Data Layer (`src/data/`)

#### **Pokemon Showdown Integration** (`src/data/ps/`)
Direct integration with Pokemon Showdown data format:

**Repository Pattern** (`repository.rs`):
```rust
pub struct Repository {
    moves: HashMap<MoveId, MoveData>,
    pokemon: HashMap<SpeciesId, PokemonData>,  
    items: HashMap<ItemId, ItemData>,
    abilities: HashMap<AbilityId, AbilityData>,
}
```

**Data Sources** (`data/ps-extracted/`):
- `moves.json` - Complete move database
- `pokemon.json` - Pokemon species data
- `items.json` - Item database  
- `moves-by-generation.json` - Generation-specific data

**Random Team Support** (`random_team_loader.rs`):
Loads pre-generated random teams for Random Battle formats from JSON files.

**Data Issues Identified:**
- Repository pattern seems incomplete (only first 100 lines reviewed)
- Conversion between PS data format and engine types could be streamlined
- Random team data dependency on external JSON files

### 4. Generation System (`src/generation.rs`)

Comprehensive generation mechanics system supporting Generations 1-9:

```rust
pub enum Generation {
    Gen1, Gen2, Gen3, Gen4, Gen5, Gen6, Gen7, Gen8, Gen9
}

pub struct GenerationMechanics {
    pub battle_mechanics: GenerationBattleMechanics,
    pub features: Vec<GenerationFeature>,
}
```

**Key Features:**
- Physical/Special split (Gen 4+)  
- Dark/Steel types (Gen 2+)
- Fairy type (Gen 6+)
- Z-Moves (Gen 7+)
- Dynamax (Gen 8)
- Terastallization (Gen 9)

### 5. User Interfaces

#### **CLI Interface** (`src/main.rs`)
Comprehensive command-line interface supporting:
- Battle simulation with AI players (random, first-move, damage-maximizer)
- Multiple battle runs with statistics
- Format validation
- Team index selection
- Verbose logging

#### **Web Interface** (`ui/` directory)
React/TypeScript frontend with:
- Battle visualization (`BattleInterface.tsx`)
- Pokemon team builder (`PokemonBuilder.tsx`)  
- Showdown team import (`ShowdownImporter.tsx`)
- Real-time instruction viewing (`InstructionViewer.tsx`)

**Interface Bridge** (`src/ui/`):
- WebSocket server (`server.rs`)
- State serialization bridge (`bridge.rs`)
- Pokemon builder integration (`pokemon_builder.rs`)

### 6. Testing Infrastructure (`src/testing.rs`)

**Basic Framework Available:**
- `TestFramework` struct for battle testing
- `ContactStatusResult` for test result handling
- `TestUtils` for creating test states

**Testing Issues Identified:**
- Framework is minimal and underdeveloped
- Limited test coverage visible in main codebase
- Could benefit from more comprehensive battle scenario testing

### 7. Builder System (`src/builders/`)

Provides builder patterns for complex object creation:
- `BattleBuilder` - Battle setup
- `FormatBuilder` - Format configuration  
- `TeamBuilder` - Team construction
- `ModernBattleBuilder` - New battle state creation

## Battle Flow Architecture

### 1. State Creation
```
BattleFormat → BattleState::new() → Team Assignment → Active Pokemon Setup
```

### 2. Move Selection  
```
Player Input → MoveChoice → Auto-targeting Resolution → Validation
```

### 3. Turn Processing
```
Move Choices → Move Order Determination → Instruction Generation → State Application → End-of-Turn Effects
```

### 4. Instruction Pipeline
```
Move Effects → Probabilistic Instructions → State Mutations → Undo Support
```

## Key Architectural Strengths

### 1. **Multi-Format Design**
- Every component built with multiple formats in mind
- Position-based targeting eliminates format-specific assumptions
- Format-aware damage calculations (spread move reductions)

### 2. **Modular Move Effects**
- Clear separation of move categories into focused modules
- Easy to add new moves within existing categories
- Consistent patterns across similar moves

### 3. **Generation Awareness**
- Historical accuracy for different Pokemon generations
- Feature flags for generation-specific mechanics
- Proper handling of type system evolution

### 4. **Instruction-Based State Management**
- Atomic, reversible operations
- Probabilistic outcomes supported natively  
- Clear separation between state query and mutation

### 5. **Data Integration**
- Direct Pokemon Showdown data usage
- Automated data extraction tools
- Comprehensive Pokemon/move/item databases

## Areas Needing Improvement

### 1. **Legacy Compatibility Overhead**
- Duplicate state fields between modern and legacy systems
- Multiple state access patterns create complexity
- **Recommendation**: Complete migration to modern BattleState, remove legacy fields

### 2. **Move Effects Organization**
- Some modules are too granular (single-move modules)
- Inconsistent naming patterns across modules
- **Recommendation**: Consolidate closely related effects, establish consistent naming

### 3. **Testing Infrastructure**
- Minimal testing framework implementation
- Limited automated battle scenario testing
- **Recommendation**: Develop comprehensive test suite with battle scenario coverage

### 4. **Data Conversion Complexity**
- Multiple data type conversions between PS and engine formats
- Repository pattern implementation appears incomplete  
- **Recommendation**: Streamline data pipeline, complete repository implementation

### 5. **Documentation**
- Limited inline documentation for complex battle mechanics
- Move effect implementations could use more detailed comments
- **Recommendation**: Add comprehensive documentation for battle mechanics and move effects

## Dependencies

**Core Dependencies:**
- `clap` (4.5.4) - CLI argument parsing
- `serde` (1.0) - JSON serialization  
- `tokio` (1.0) - Async runtime for web interface
- `axum` (0.7) - Web server framework
- `rand` (0.8.4) - Random number generation

**Architecture Decision**: No compile-time features - everything handled at runtime for flexibility.

## Future Architecture Considerations

### 1. **Performance Optimization**
- Consider caching frequently accessed Pokemon/move data
- Profile instruction generation pipeline for bottlenecks
- Optimize damage calculation for large-scale simulations

### 2. **Extension Points**
- Plugin system for custom move effects
- Configurable battle rules beyond current clause system
- Support for custom formats and team validation

### 3. **State Serialization**
- Complete battle replay system
- Network play support with state synchronization
- Battle analysis and debugging tools

### 4. **Multi-threaded Processing**  
- Parallel battle simulation (already partially implemented with `rayon`)
- Background processing for large tournament simulations
- Concurrent web interface handling

## Conclusion

Tapu Simu demonstrates a well-architected Pokemon battle simulator with strong multi-format support and clean separation of concerns. The modular design facilitates maintenance and extension, while the generation-aware mechanics ensure historical accuracy. Key areas for improvement include completing the legacy migration, consolidating the move effects system, and developing comprehensive testing infrastructure.

The architecture successfully achieves its goal of supporting multiple battle formats through position-based targeting and format-aware mechanics, making it a solid foundation for a comprehensive Pokemon battle simulation platform.