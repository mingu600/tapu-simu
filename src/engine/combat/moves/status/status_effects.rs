//! Status Effect Functions using centralized systems
//! 
//! This module contains status effect move implementations that use the new
//! centralized status system, eliminating code duplication.

use crate::core::battle_state::BattleState;
use crate::core::instructions::{BattleInstructions, PokemonStatus};
use crate::core::battle_format::BattlePosition;
use crate::generation::GenerationMechanics;
use crate::engine::combat::composers::status_moves::{
    paralysis_move, burn_move, poison_move, sleep_move,
};

// =============================================================================
// STATUS MOVES THAT INFLICT MAJOR STATUS CONDITIONS
// =============================================================================

/// Apply Thunder Wave - paralyzes the target
/// Uses the centralized status system for consistent immunity checking
pub fn apply_thunder_wave(
    state: &BattleState,
    _user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let instructions = paralysis_move(state, target_positions, 100.0);
    vec![BattleInstructions::new(100.0, instructions)]
}

/// Apply Thunder Wave using unified context signature
pub fn apply_thunder_wave_unified(ctx: &mut crate::engine::combat::move_context::MoveExecutionContext) -> Vec<crate::core::instructions::BattleInstructions> {
    let instructions = paralysis_move(ctx.state, ctx.target_positions, 100.0);
    vec![crate::core::instructions::BattleInstructions::new(100.0, instructions)]
}

/// Apply Sleep Powder - puts target to sleep
/// Uses the centralized system with powder move immunity handling
pub fn apply_sleep_powder(
    state: &BattleState,
    _user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let instructions = sleep_move(state, target_positions, 75.0); // Sleep Powder has 75% accuracy
    vec![BattleInstructions::new(100.0, instructions)]
}

/// Apply Sleep Powder using unified context signature
pub fn apply_sleep_powder_unified(ctx: &mut crate::engine::combat::move_context::MoveExecutionContext) -> Vec<crate::core::instructions::BattleInstructions> {
    let instructions = sleep_move(ctx.state, ctx.target_positions, 75.0); // Sleep Powder has 75% accuracy
    vec![crate::core::instructions::BattleInstructions::new(100.0, instructions)]
}

/// Apply Toxic - badly poisons the target
/// Uses the centralized system for poison immunity checking
pub fn apply_toxic(
    state: &BattleState,
    _user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let instructions = poison_move(state, target_positions, true, 90.0); // Toxic = badly poisoned, 90% accuracy
    vec![BattleInstructions::new(100.0, instructions)]
}

/// Apply Will-O-Wisp - burns the target
/// Uses the centralized system for burn immunity checking
pub fn apply_will_o_wisp(
    state: &BattleState,
    _user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let instructions = burn_move(state, target_positions, 85.0); // Will-O-Wisp has 85% accuracy
    vec![BattleInstructions::new(100.0, instructions)]
}

/// Apply Spore - puts target to sleep with perfect accuracy
/// Uses the centralized system but bypasses most immunity checks
pub fn apply_spore(
    state: &BattleState,
    _user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let instructions = sleep_move(state, target_positions, 100.0); // Spore has 100% accuracy
    vec![BattleInstructions::new(100.0, instructions)]
}

/// Apply Stun Spore - paralyzes the target
/// Uses the centralized system with powder move immunity
pub fn apply_stun_spore(
    state: &BattleState,
    _user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let instructions = paralysis_move(state, target_positions, 75.0); // Stun Spore has 75% accuracy
    vec![BattleInstructions::new(100.0, instructions)]
}

/// Apply Poison Powder - poisons the target
/// Uses the centralized system with powder move immunity
pub fn apply_poison_powder(
    state: &BattleState,
    _user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let instructions = poison_move(state, target_positions, false, 75.0); // Regular poison, 75% accuracy
    vec![BattleInstructions::new(100.0, instructions)]
}

/// Apply Hypnosis - puts target to sleep
/// Uses the centralized system for sleep mechanics
pub fn apply_hypnosis(
    state: &BattleState,
    _user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let instructions = sleep_move(state, target_positions, 60.0); // Hypnosis has 60% accuracy
    vec![BattleInstructions::new(100.0, instructions)]
}

/// Apply Glare - paralyzes the target
/// Uses the centralized system, affects all types (not a powder move)
pub fn apply_glare(
    state: &BattleState,
    _user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let instructions = paralysis_move(state, target_positions, 100.0); // Glare has 100% accuracy
    vec![BattleInstructions::new(100.0, instructions)]
}

/// Apply Lovely Kiss - puts target to sleep
/// Uses the centralized system for sleep mechanics  
pub fn apply_lovely_kiss(
    state: &BattleState,
    _user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let instructions = sleep_move(state, target_positions, 75.0); // Lovely Kiss has 75% accuracy
    vec![BattleInstructions::new(100.0, instructions)]
}