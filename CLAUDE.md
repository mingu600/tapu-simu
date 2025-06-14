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

**Core Foundation** ✅ COMPLETED
- ✅ Multi-format battle state system (`src/battle_format.rs`, `src/state.rs`)
- ✅ Position-based targeting framework (`src/battle_format.rs`)
- ✅ Format-aware instruction system (`src/instruction.rs`)
- ✅ Move choice with explicit targeting (`src/move_choice.rs`)
- ✅ Rustemon/PokeAPI data integration layer (`src/data/`)
- ✅ CLI interface with basic commands (`src/io.rs`)

**Phase 4: Advanced Battle Mechanics** ✅ COMPLETED
- ✅ **Format-Aware Targeting System** (`src/genx/format_targeting.rs`)
  - Complete move target resolution for all 16 rustemon/PokeAPI targets
  - AutoTargetingEngine for automatic target resolution
  - Format-specific targeting logic (singles vs doubles vs VGC)
- ✅ **Format Instruction Generator** (`src/genx/format_instruction_generator.rs`)
  - Spread move damage reduction (0.75x in doubles/VGC)
  - Critical hit branching with proper percentages
  - Multi-target instruction generation
- ✅ **Doubles-Specific Mechanics** (`src/genx/doubles_mechanics.rs`)
  - Follow Me/Rage Powder redirection mechanics
  - Helping Hand, Wide Guard, Quick Guard implementation
  - Ally damage calculation for spread moves
  - Position-based adjacency checking
- ✅ **Multi-Target Instruction System** (enhanced `src/instruction.rs`)
  - PositionDamageInstruction and MultiTargetDamageInstruction
  - Position-aware volatile status instructions
  - Comprehensive affected_positions tracking
- ✅ **Enhanced Instruction Generator** (`src/genx/instruction_generator.rs`)
  - **NO MORE PLACEHOLDERS** - fully functional implementation
  - Integration of all format-aware mechanics
  - Auto-targeting resolution and redirection mechanics

**Pokemon Showdown Integration** 🚧 IN PROGRESS
- ✅ PS data extraction tool with @pkmn packages
- ✅ PS-compatible type system (PSMoveTarget, PSMoveData)
- ✅ PSAutoTargetingEngine for direct PS target usage
- ✅ PS data loader with JSON parsing
- ⏳ Replace rustemon move data with PS data
- ⏳ Migrate all targeting to PS conventions
- ⏳ Extract and integrate PS item data

**Remaining Core Mechanics** ⏳ PENDING
- ⏳ Enhanced damage calculation with type effectiveness
- ⏳ Comprehensive status condition effects  
- ⏳ Weather and terrain effects
- ⏳ Ability system integration
- ⏳ Item effects implementation

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
├── instruction.rs        # Position-aware instruction system (enhanced)
├── move_choice.rs        # Explicit targeting move choices (enhanced)
├── state.rs             # Multi-format battle state with Move definitions
├── data/                # Rustemon/PokeAPI integration
│   ├── types.rs         # Engine-optimized data structures
│   ├── conversion.rs    # Rustemon → Engine conversions
│   ├── rustemon_client.rs # API client wrapper
│   ├── move_factory.rs  # Move data factory system
│   └── move_service.rs  # Move service layer
├── genx/                # Generation-specific mechanics (Phase 4 complete)
│   ├── instruction_generator.rs    # Main instruction coordinator
│   ├── format_instruction_generator.rs # Format-aware instruction generation  
│   ├── format_targeting.rs         # Multi-format targeting system
│   ├── doubles_mechanics.rs        # Doubles-specific mechanics
│   ├── damage_calc.rs             # Damage calculation system
│   └── move_effects.rs            # Special move effects
└── io.rs                # CLI interface
```

#### 🆕 Phase 4 Architecture Highlights

**Format-Aware Instruction Generation Flow:**
1. `GenerationXInstructionGenerator` coordinates all mechanics
2. `AutoTargetingEngine` resolves move targets automatically  
3. `FormatInstructionGenerator` handles damage and status instructions
4. `DoublesSpecificMechanics` applies doubles-only interactions
5. Redirection mechanics (Follow Me, etc.) applied in final step

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

1. **Complete Core Move Mechanics**
   - Basic damage calculation
   - Accuracy and evasion
   - Critical hits
   - Type effectiveness

2. **Status System**
   - Major status conditions
   - Volatile status effects
   - Status immunities

3. **Move Categories**
   - Physical moves
   - Special moves
   - Status moves
   - Multi-hit moves
   - Spread moves

4. **Format-Specific Mechanics**
   - Spread damage reduction
   - Ally targeting
   - Wide Guard/Quick Guard
   - Follow Me/Rage Powder

Remember: V2 is a fresh start. Build it right from the beginning with multi-format support as the foundation, not an afterthought.