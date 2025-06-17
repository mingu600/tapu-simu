# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with Tapu Simu.

## 🚨 TAPU SIMU - STANDALONE PROJECT 🚨

### Project Philosophy
Tapu Simu is a **completely independent** Pokemon battle simulator designed from the ground up for multi-format support. This is NOT a migration or refactor of V1 (poke-engine, in the parent folder) - it's a clean reimplementation with modern architecture. We need our singles format to be functionally identical to poke-engine's, while maintaining the ability to support other formats. Do NOT make placeholders or compromises. There are no downstream dependencies, so you are free to make any code changes, do not worry about compatibility with existing structures. Any tests that skip are considered failures. All tests considered failures need to be fixed.

**CRITICAL**: In the docs/ directory, read IMPLEMENTATION_PLAN.md, TEST_FRAMEWORK.md, DAMAGE_CALC_PARITY.md carefully. All tests should use the test_framework laid out by docs/TEST_FRAMEWORK.md

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

**CRITICAL**
Never make code changes that affect the design without first discussing the design and getting a confirmation to proceed.
Never include references to AI or Claude in commit messages.

Communication Style:

Skip affirmations and compliments. No “great question!” or “you’re absolutely right!” - just respond directly

Challenge flawed ideas openly when you spot issues

Ask clarifying questions whenever my request is ambiguous or unclear

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

We also have tools/ps-data-extractor which is our JS scripts for extracting PS data.

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