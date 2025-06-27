//! Multi-hit move implementations using the centralized damage system
//! 
//! This module handles all multi-hit moves using the new core systems,
//! eliminating code duplication and providing consistent behavior.

use crate::core::battle_state::BattleState;
use crate::core::instructions::{BattleInstruction, BattleInstructions, StatsInstruction};
use crate::core::battle_format::{BattlePosition, SideReference};
use crate::generation::GenerationMechanics;
use crate::data::showdown_types::MoveData;
use crate::engine::combat::core::damage_system::{
    multi_hit_move, HitCountCalculator,
};
use crate::engine::combat::composers::damage_moves::{
    simple_damage_move, DamageModifiers,
};

// =============================================================================
// MULTI-HIT MOVE FUNCTIONS
// =============================================================================

/// Apply multi-hit move effects using the centralized system
/// This is now a simple wrapper around the core multi-hit system
pub fn apply_multi_hit_move(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
    branch_on_damage: bool,
) -> Vec<BattleInstructions> {
    // Determine hit count based on move and abilities/items
    let hit_count_calculator = determine_hit_count(state, move_data, user_position);
    
    // Use the centralized multi-hit system
    let instructions = multi_hit_move(
        state,
        move_data,
        user_position,
        target_positions,
        hit_count_calculator,
        generation,
        branch_on_damage,
    );
    
    // Convert to the expected return format
    vec![BattleInstructions::new(100.0, instructions)]
}

/// Determine the hit count calculator for a multi-hit move
fn determine_hit_count(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
) -> HitCountCalculator {
    let user_pokemon = state.get_pokemon_at_position(user_position);
    
    // Check for effects that modify hit count
    let force_max_hits = user_pokemon.map_or(false, |pokemon| {
        // Check for Loaded Dice item
        if let Some(ref item) = pokemon.item {
            if *item == crate::types::Items::LOADEDDICE {
                return true;
            }
        }
        
        // Check for Skill Link ability
        if pokemon.ability == crate::types::Abilities::SKILLLINK {
            return true;
        }
        
        false
    });
    
    // Handle special cases for specific moves
    let move_name = move_data.name.as_str();
    match move_name {
        "doubleslap" | "bonemerang" => HitCountCalculator::Fixed(2),
        "tripleaxel" | "triplekick" | "surgingstrikes" => HitCountCalculator::Fixed(3),
        "dragondarts" => HitCountCalculator::Fixed(2),
        "populationbomb" => {
            // Population Bomb special case - check for Wide Lens
            let has_wide_lens = user_pokemon.map_or(false, |pokemon| {
                if let Some(ref item) = pokemon.item {
                    *item == crate::types::Items::WIDELENS
                } else {
                    false
                }
            });
            
            if has_wide_lens {
                HitCountCalculator::Fixed(10)
            } else {
                HitCountCalculator::Fixed(7)
            }
        }
        "beatup" => {
            // Beat Up hits once per conscious party member
            HitCountCalculator::Custom(|state, _move_data, user_position| {
                // Count conscious party members
                let user_side_index = user_position.side.to_index();
                let conscious_count = state.sides[user_side_index]
                    .pokemon
                    .iter()
                    .filter(|p| p.hp > 0)
                    .count() as u8;
                conscious_count.max(1) // At least 1 hit
            })
        }
        _ => {
            if force_max_hits {
                HitCountCalculator::Fixed(5)
            } else {
                // Standard 2-5 hit distribution, use fixed 3 for consistency
                HitCountCalculator::Fixed(3)
            }
        }
    }
}


// =============================================================================
// SIMPLIFIED MULTI-HIT MOVE FUNCTIONS
// =============================================================================

/// All standard multi-hit moves now use the same centralized implementation
macro_rules! multi_hit_move_impl {
    ($func_name:ident) => {
        pub fn $func_name(
            state: &BattleState,
            move_data: &MoveData,
            user_position: BattlePosition,
            target_positions: &[BattlePosition],
            generation: &GenerationMechanics,
            branch_on_damage: bool,
        ) -> Vec<BattleInstructions> {
            apply_multi_hit_move(state, move_data, user_position, target_positions, generation, branch_on_damage)
        }
    };
}

// Generate all the multi-hit move functions
multi_hit_move_impl!(apply_double_slap);     // 2 hits
multi_hit_move_impl!(apply_comet_punch);     // 2-5 hits  
multi_hit_move_impl!(apply_fury_attack);     // 2-5 hits
multi_hit_move_impl!(apply_pin_missile);     // 2-5 hits
multi_hit_move_impl!(apply_barrage);         // 2-5 hits
multi_hit_move_impl!(apply_spike_cannon);    // 2-5 hits
multi_hit_move_impl!(apply_bonemerang);      // 2 hits
multi_hit_move_impl!(apply_bullet_seed);     // 2-5 hits
multi_hit_move_impl!(apply_icicle_shard);    // 2-5 hits
multi_hit_move_impl!(apply_rock_blast);      // 2-5 hits
multi_hit_move_impl!(apply_tail_slap);       // 2-5 hits
multi_hit_move_impl!(apply_beat_up);         // Variable hits
multi_hit_move_impl!(apply_arm_thrust);      // 2-5 hits
multi_hit_move_impl!(apply_triple_axel);     // 3 hits (increasing power)
multi_hit_move_impl!(apply_triple_kick);     // 3 hits (increasing power)

/// Surging Strikes - Always hits exactly 3 times (critical hit guaranteed)
/// Uses the centralized system with forced critical hits
pub fn apply_surging_strikes(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
    branch_on_damage: bool,
) -> Vec<BattleInstructions> {
    use crate::engine::combat::core::damage_system::{
        DamageCalculationContext, execute_multi_hit_sequence, HitCountCalculator,
    };
    
    let mut instructions = Vec::new();

    for &target_position in target_positions {
        // Surging Strikes always hits exactly 3 times with guaranteed critical hits
        let context = DamageCalculationContext::new(
            move_data,
            user_position,
            target_position,
            generation.clone(),
            branch_on_damage,
        ).with_force_critical(); // Force critical hits for Surging Strikes

        let hit_instructions = execute_multi_hit_sequence(
            state,
            context,
            3, // Always 3 hits
            None, // No hit-specific modifiers
        );

        instructions.extend(hit_instructions);
    }

    // Convert to the expected return format
    vec![BattleInstructions::new(100.0, instructions)]
}

/// Dragon Darts - Always hits exactly 2 times with special targeting
/// Uses the centralized system; targeting logic handled elsewhere
pub fn apply_dragon_darts(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
    branch_on_damage: bool,
) -> Vec<BattleInstructions> {
    apply_multi_hit_move(state, move_data, user_position, target_positions, generation, branch_on_damage)
}

/// Population Bomb - Special multi-hit move with accuracy mechanics
/// Uses the centralized system with custom hit count calculation
pub fn apply_population_bomb(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
    branch_on_damage: bool,
) -> Vec<BattleInstructions> {
    apply_multi_hit_move(state, move_data, user_position, target_positions, generation, branch_on_damage)
}

/// Scale Shot - Multi-hit move that also boosts Speed by 1 and lowers Defense by 1
/// Uses composers to combine multi-hit with stat changes
/// Has 90% accuracy, so includes miss chance
pub fn apply_scale_shot(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
    branch_on_damage: bool,
) -> Vec<BattleInstructions> {
    use crate::core::instructions::Stat;
    use std::collections::HashMap;
    
    // Note: Accuracy is handled by the turn engine, so we assume this is a hit
    // and generate the multi-hit damage + stat changes
    
    // Determine hit count based on move and abilities/items
    let hit_count_calculator = determine_hit_count(state, move_data, user_position);
    
    // Use the centralized multi-hit system
    let mut instructions = multi_hit_move(
        state,
        move_data,
        user_position,
        target_positions,
        hit_count_calculator,
        generation,
        branch_on_damage,
    );
    
    // Create stat changes for Scale Shot (+1 Speed, -1 Defense)
    let mut stat_changes = HashMap::new();
    stat_changes.insert(Stat::Speed, 1);
    stat_changes.insert(Stat::Defense, -1);
    
    // Add stat change instructions
    instructions.push(BattleInstruction::Stats(StatsInstruction::BoostStats {
        target: user_position,
        stat_changes,
        previous_boosts: std::collections::HashMap::new(), // Will be filled in by battle state
    }));
    
    // Return single instruction set with 100% probability (accuracy handled by turn engine)
    vec![BattleInstructions::new(100.0, instructions)]
}

