# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with Tapu Simu.

## 🚨 TAPU SIMU - STANDALONE PROJECT 🚨

### Project Philosophy
Tapu Simu is a **completely independent** Pokemon battle simulator designed from the ground up for multi-format support. This is NOT a migration or refactor of V1 (poke-engine) - it's a clean reimplementation with modern architecture.

### Core Principles

1. **Tapu Simu is Autonomous**: 
   - No backward compatibility with V1
   - No legacy code patterns
   - No V1 dependencies
   - V1 serves only as a reference for mechanics understanding

2. **Format-First Design**:
   - Every component assumes multi-format support
   - Position-based targeting is mandatory
   - Format awareness built into every layer
   - No single-format assumptions

3. **Clean Architecture**:
   - State is immutable during instruction generation
   - Instructions are atomic and position-aware
   - Move choices explicitly declare targets
   - No implicit behavior

## Design Philosophy Principles

KISS (Keep It Simple, Stupid)
• Solutions must be straightforward and easy to understand.
• Avoid over-engineering or unnecessary abstraction.
• Prioritise code readability and maintainability.

YAGNI (You Aren’t Gonna Need It)
• Do not add speculative features or future-proofing unless explicitly required.
• Focus only on immediate requirements and deliverables.
• Minimise code bloat and long-term technical debt. 

**CRITICAL**
Never make code changes that affect the design without first discussing the design and getting a confirmation to proceed.
Never include references to AI or Claude in commit messages.

Communication Style:

Skip affirmations and compliments. No “great question!” or “you’re absolutely right!” - just respond directly

Challenge flawed ideas openly when you spot issues

Ask clarifying questions whenever my request is ambiguous or unclear

### 🎯 Current Implementation Status

**🎉 MAJOR MILESTONE: Pokemon Showdown Integration Complete** ✅ 
- **Fully replaced rustemon** - Complete migration to Pokemon Showdown as primary data source
- **772 moves** + **244 items** with complete battle metadata
- **Generation-specific data** - Complete Gen 1-9 support (252-777 moves per generation)
- **319 moves with change tracking** - Full historical evolution across generations
- **Production-ready synchronous API** - No async dependencies, fast local access
- **Battle-tested accuracy** - Direct Pokemon Showdown data ensures simulator-grade precision

**Core Architecture** ✅ COMPLETED
- ✅ Multi-format battle state system (`src/battle_format.rs`, `src/state.rs`)
- ✅ Position-based targeting framework with PSMoveTarget integration
- ✅ Format-aware instruction system (`src/instruction.rs`)
- ✅ Move choice with explicit targeting (`src/move_choice.rs`)
- ✅ Pokemon Showdown data integration layer (`src/data/`)
- ✅ CLI interface with basic commands (`src/io.rs`)

**Advanced Battle Mechanics** ✅ COMPLETED
- ✅ **Format-Aware Targeting** (`src/genx/format_targeting.rs`) - Complete PSMoveTarget resolution
- ✅ **Format Instruction Generator** (`src/genx/format_instruction_generator.rs`) - Spread damage, critical hits
- ✅ **Doubles-Specific Mechanics** (`src/genx/doubles_mechanics.rs`) - Redirection, ally interactions
- ✅ **Multi-Target Instructions** - Position-aware damage and status effects
- ✅ **Complete Instruction Generator** (`src/genx/instruction_generator.rs`) - Production-ready implementation

**Pokemon Showdown Data System** ✅ COMPLETED
- ✅ **PS Data Extraction** - Complete toolchain with @pkmn packages
- ✅ **PS Type System** - PSMoveTarget, PSMoveData with advanced type handling
- ✅ **Generation Repository** (`src/data/ps_generation_loader.rs`) - Historical move data access
- ✅ **PS Move Services** - Synchronous local data access replacing all async dependencies
- ✅ **PS Move Factory** - Enhanced moveset creation with engine optimizations
- ✅ **Advanced Data Types** - Complex immunity handling, Z-moves, Max moves, secondary effects

**Next Implementation Focus**
- Enhanced damage calculation with PS type effectiveness
- Status condition system using PS status data
- Weather and terrain effects with PS metadata
- Ability system integration
- Item effects implementation

### 📋 Development Guidelines

#### When implementing new features:

1. **Always think multi-format first**
   - How does this work in Singles?
   - How does this work in Doubles?
   - Are there format-specific variations?

2. **Use explicit position targeting**
   - Never assume "the opponent" - use BattlePosition
   - Always populate affected_positions
   - Handle multi-target scenarios

3. **Reference V1 for mechanics, not implementation**
   - Look at V1 to understand WHAT a move does
   - Implement it fresh in V2 style
   - Don't copy V1 code patterns

4. **Test with format variations**
   - Every mechanic should be tested in Singles and Doubles
   - Consider edge cases in each format
   - Verify position targeting works correctly

### 🏗 Architecture Overview

```
src/
├── battle_format.rs      # Format definitions and position management  
├── instruction.rs        # Position-aware instruction system
├── move_choice.rs        # Explicit targeting move choices
├── state.rs             # Multi-format battle state with Move definitions
├── data/                # Pokemon Showdown data integration
│   ├── types.rs         # Engine-optimized legacy structures
│   ├── ps_types.rs      # Pokemon Showdown data types (PSMoveData, PSMoveTarget)
│   ├── ps_conversion.rs # PS → Engine conversions
│   ├── ps_loader.rs     # PS JSON data loader
│   ├── ps_generation_loader.rs # Generation-specific data repository
│   ├── ps_move_service.rs      # Synchronous move data access
│   ├── ps_move_factory.rs      # Enhanced moveset creation
│   └── choices.rs       # Move choice utilities
├── genx/                # Advanced battle mechanics
│   ├── instruction_generator.rs    # Main instruction coordinator
│   ├── format_instruction_generator.rs # Format-aware instruction generation  
│   ├── format_targeting.rs         # Multi-format targeting system
│   ├── ps_targeting.rs            # Pokemon Showdown targeting engine
│   ├── doubles_mechanics.rs        # Doubles-specific mechanics
│   ├── damage_calc.rs             # Damage calculation system
│   └── move_effects.rs            # Special move effects
└── io.rs                # CLI interface
```

#### Key Architecture Features

**Pokemon Showdown Data Pipeline:**
1. `PSDataRepository` loads and caches all PS JSON data
2. `PSGenerationRepository` provides generation-aware move access
3. `PSMoveService` offers synchronous move lookups with caching
4. `PSAutoTargetingEngine` handles native PS target resolution

**Format-Aware Battle Flow:**
1. `GenerationXInstructionGenerator` coordinates all mechanics
2. `PSAutoTargetingEngine` resolves move targets using PS conventions
3. `FormatInstructionGenerator` generates position-aware instructions
4. `DoublesSpecificMechanics` applies format-specific interactions
5. Multi-target damage calculations with spread reduction

**Multi-Target Support:**
- `PositionDamageInstruction` for single-target moves
- `MultiTargetDamageInstruction` for spread moves  
- Automatic spread damage reduction in doubles/VGC formats
- Critical hit branching with proper percentage calculations

### 🔧 Working with Move Effects

When implementing a move effect:

```rust
// V2 Style - Always position-aware
pub fn apply_thunder_wave(
    state: &mut State,
    user_position: BattlePosition,
    target_position: BattlePosition,
) -> Vec<Instruction> {
    let mut instructions = vec![];
    
    // Check if target can be paralyzed
    if let Some(target) = state.get_pokemon_at_position(target_position) {
        if target.status == PokemonStatus::None {
            instructions.push(Instruction::ApplyStatus(ApplyStatusInstruction {
                target_position,
                status: PokemonStatus::Paralysis,
            }));
        }
    }
    
    instructions
}
```

### 🎮 Battle Flow

1. **State Creation**: Format-aware from the start
2. **Move Selection**: Players choose moves with explicit targets
3. **Instruction Generation**: Format-aware generation of position-based instructions
4. **State Application**: Instructions modify battle state
5. **Turn Resolution**: End-of-turn effects applied

### ⚠️ Common Pitfalls to Avoid

1. **Don't assume single target**: Always handle Vec<BattlePosition>
2. **Don't copy V1 patterns**: Reimplement with V2 principles
3. **Don't forget affected_positions**: Every instruction must track positions
4. **Don't ignore format differences**: Test in multiple formats

### 🧪 Testing Requirements

Every feature must include tests that:
- Verify behavior in Singles format
- Verify behavior in Doubles format
- Check position targeting correctness
- Validate affected_positions tracking
- Handle edge cases per format

### 📚 V1 Reference Usage

When referencing V1:
- ✅ Look at test cases to understand mechanics
- ✅ Read move implementations for behavior understanding
- ✅ Check damage formulas and calculations
- ❌ Don't copy code structure
- ❌ Don't maintain V1 compatibility
- ❌ Don't use V1 design patterns

### 🚀 Next Implementation Priorities

1. **Enhanced Battle Mechanics** 🔥 HIGH PRIORITY
   - Type effectiveness calculation using PS type chart data
   - Status condition system leveraging PS status metadata
   - Weather and terrain effects with PS environmental data
   - Enhanced damage calculation with PS formulas

2. **Advanced Move Systems**
   - Multi-hit moves using PS multihit data structures
   - Z-move and Max move mechanics with PS Z-crystal/Dynamax data
   - Secondary effects and status conditions from PS secondary data
   - Move flags integration (contact, sound, protect, etc.)

3. **Pokemon Stats and Abilities**
   - Ability system integration with PS ability data
   - Item effects implementation using PS item metadata
   - Stat calculation and modification systems
   - Base stat and type data integration

4. **Battle State Enhancements**
   - Turn order calculation with priority and speed
   - End-of-turn effect processing
   - Field condition management
   - Team preview and switch mechanics

### 🎯 Data Utilization Guide

**Pokemon Showdown Data Features Available:**
- **Move Flags**: Contact, sound, protect, mirror, metronome, etc.
- **Secondary Effects**: Status conditions, stat boosts, field effects
- **Drain/Recoil**: HP recovery and damage ratios
- **Complex Targeting**: Type-specific immunity overrides
- **Generation Tracking**: Historical move changes across generations
- **Z-Move Data**: Z-crystal requirements and power calculations
- **Max Move Data**: Dynamax effects and power scaling

Remember: V2 leverages Pokemon Showdown's battle-tested data for maximum accuracy.