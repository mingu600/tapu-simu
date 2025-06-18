# Instruction System Documentation

## Overview

The tapu-simu instruction system is a position-aware, format-agnostic architecture that handles all battle mechanics through atomic, reversible operations. The system supports probabilistic outcomes, multi-target operations, and complete undo functionality.

## Core Components

### 1. Instruction Types (`src/instruction.rs`)

The system defines a comprehensive `Instruction` enum with 40+ instruction types covering all battle mechanics:

```rust
pub enum Instruction {
    // Damage and healing
    PositionDamage(PositionDamageInstruction),
    PositionHeal(PositionHealInstruction),
    MultiTargetDamage(MultiTargetDamageInstruction),
    
    // Status effects
    ApplyStatus(ApplyStatusInstruction),
    RemoveStatus(RemoveStatusInstruction),
    BoostStats(BoostStatsInstruction),
    UnboostStats(UnboostStatsInstruction),
    
    // Move mechanics
    DisableMove(DisableMoveInstruction),
    DecrementPP(DecrementPPInstruction),
    RestoreLastUsedMove(RestoreLastUsedMoveInstruction),
    
    // Field effects
    ChangeWeather(ChangeWeatherInstruction),
    ChangeTerrain(ChangeTerrainInstruction),
    ToggleTrickRoom(ToggleTrickRoomInstruction),
    ToggleGravity(ToggleGravityInstruction),
    
    // Pokemon management
    Switch(SwitchInstruction),
    Faint(FaintInstruction),
    ChangeForm(ChangeFormInstruction),
    ChangeType(ChangeTypeInstruction),
    
    // Abilities and items
    ChangeAbility(ChangeAbilityInstruction),
    ToggleAbility(ToggleAbilityInstruction),
    RemoveItem(RemoveItemInstruction),
    GiveItem(GiveItemInstruction),
    
    // And many more...
}
```

#### Key Features:
- **Position Awareness**: Every instruction tracks affected positions via `affected_positions()` method
- **Undo Support**: Instructions store previous values for complete reversibility
- **Multi-Target Support**: Handles spread moves and multiple target scenarios
- **Format Agnostic**: Works across Singles, Doubles, and other formats

### 2. Probabilistic Instructions (`StateInstructions`)

The system uses `StateInstructions` to handle multiple possible outcomes:

```rust
pub struct StateInstructions {
    percentage: f32,
    instruction: Vec<Instruction>,
}
```

This enables probabilistic branching for:
- Critical hits (5% vs 95% probability)
- Move accuracy (hit vs miss)
- Secondary effects (30% chance to burn, etc.)

### 3. Instruction Generation (`src/genx/instruction_generator.rs`)

The instruction generation process follows this flow:

```rust
pub fn generate_instructions(
    &self,
    state: &mut State,
    side_one_choice: &MoveChoice,
    side_two_choice: &MoveChoice,
) -> Vec<StateInstructions>
```

#### Generation Steps:
1. **Auto-targeting Resolution**: Resolves unspecified targets based on battle format
2. **Move Ordering**: Determines execution order using priority, speed, and switch mechanics
3. **Single Move Processing**: Generates instruction sets for each move
4. **Probability Combination**: Creates all possible outcome combinations
5. **Format-Specific Mechanics**: Applies redirection, spread move damage reduction

#### Example Generation:
```rust
// A move with potential critical hit generates two instruction sets:
vec![
    StateInstructions::new(95.0, vec![normal_damage_instruction]),
    StateInstructions::new(5.0, vec![critical_hit_instruction]),
]
```

### 4. Instruction Processing (`src/state.rs`)

Instructions are applied to the battle state through:

```rust
pub fn apply_instructions(&mut self, instructions: &[Instruction]) {
    for instruction in instructions {
        self.apply_instruction(instruction);
    }
}

pub fn apply_instruction(&mut self, instruction: &Instruction) {
    match instruction {
        Instruction::PositionDamage(instr) => {
            self.apply_position_damage(instr.target_position, instr.damage_amount);
        }
        Instruction::ApplyStatus(instr) => {
            self.apply_status(instr.target_position, instr.status);
        }
        // ... handles all instruction types
    }
}
```

The state modification methods handle the actual battle state changes, including:
- Pokemon HP/PP management
- Status condition application
- Stat boost/unboost tracking
- Field effect management
- Pokemon switching and fainting

### 5. Undo System (`src/instruction_undo.rs`)

The system provides complete instruction reversal:

```rust
pub fn reverse_instructions(&mut self, instructions: &[Instruction]) {
    // Process instructions in REVERSE order
    for instruction in instructions.iter().rev() {
        self.reverse_instruction(instruction);
    }
}

pub fn reverse_instruction(&mut self, instruction: &Instruction) {
    match instruction {
        Instruction::PositionDamage(instr) => {
            // Restore previous HP value
            if let Some(previous_hp) = instr.previous_hp {
                self.set_pokemon_hp(instr.target_position, previous_hp);
            }
        }
        Instruction::ApplyStatus(instr) => {
            // Remove the applied status
            self.remove_status(instr.target_position);
        }
        // ... reverses all instruction types
    }
}
```

#### Enhanced Undo Support:
Some instructions include "WithUndo" variants that store complete previous state:

```rust
pub struct PositionDamageInstructionWithUndo {
    pub target_position: BattlePosition,
    pub damage_amount: i16,
    pub previous_hp: i16,  // Always stored for guaranteed reversal
}
```

## Complete Instruction Lifecycle

### 1. Generation Phase
- Move choices are processed by `GenerationXInstructionGenerator`
- Auto-targeting resolves unspecified targets
- Move ordering determines execution sequence
- Probabilistic instruction sets are created for different outcomes

### 2. Processing Phase
- `State::apply_instructions()` applies instruction sequences
- Battle state is modified according to instruction specifications
- Position tracking ensures format-aware mechanics

### 3. Undo Phase
- `State::reverse_instructions()` reverses applied instructions
- Instructions are processed in reverse order
- Previous state values are restored from instruction data

### 4. Position Tracking
- Every instruction implements `affected_positions()` method
- Enables format-specific mechanics (redirection, spread moves)
- Supports optimization and selective processing

## Format Awareness

The instruction system is designed to work across different battle formats:

- **Singles**: Direct targeting between two Pokemon
- **Doubles**: Multi-target moves, redirection, position-specific mechanics
- **Future Formats**: Extensible architecture for additional formats

All instructions use `BattlePosition` for targeting instead of assuming single opponents, ensuring the system works across all supported formats.

## Testing Framework

The instruction system includes comprehensive testing via `test_framework.rs`:

```rust
// Test instruction generation
let instructions = framework.test_instruction_generation(&mut state, move_choice, None);

// Verify instruction correctness
assert!(framework.verify_damage_instructions(&instructions));
assert!(framework.verify_critical_hit_branching(&instructions));

// Test undo functionality
state.apply_instruction(&instruction);
let modified_state = state.clone();
state.reverse_instruction(&instruction);
assert_eq!(state, original_state);
```

This ensures instruction generation, processing, and undo functionality work correctly across all battle scenarios.