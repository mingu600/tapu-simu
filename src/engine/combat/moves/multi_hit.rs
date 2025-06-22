//! Multi-hit move implementations

//! 
//! This module handles all multi-hit moves with proper probability distributions
//! and hit count mechanics. Multi-hit moves can hit 2-5 times with different
//! probabilities depending on the generation and move specifics.

use crate::core::battle_state::{Pokemon, BattleState};
use crate::core::instructions::{PokemonStatus};
use crate::core::instructions::{
    BattleInstruction, BattleInstructions, PokemonInstruction,
};
use crate::core::battle_format::BattlePosition;
use crate::generation::GenerationMechanics;
use crate::engine::combat::type_effectiveness::{TypeChart, PokemonType};
use crate::data::showdown_types::MoveData;

// =============================================================================
// MULTI-HIT MOVE FUNCTIONS
// =============================================================================

/// Apply multi-hit move effects with proper probability branching
/// Multi-hit moves like Bullet Seed, Rock Blast, etc. hit 2-5 times with specific probabilities
pub fn apply_multi_hit_move(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    // Standard multi-hit probability distribution (2-5 hits)
    // Gen 1-4: Equal probability for each hit count (25% each)
    // Gen 5+: 35% for 2 hits, 35% for 3 hits, 15% for 4 hits, 15% for 5 hits
    let hit_probabilities = if generation.generation.number() >= 5 {
        vec![
            (2, 35.0), // 2 hits: 35%
            (3, 35.0), // 3 hits: 35% 
            (4, 15.0), // 4 hits: 15%
            (5, 15.0), // 5 hits: 15%
        ]
    } else {
        vec![
            (2, 25.0), // 2 hits: 25%
            (3, 25.0), // 3 hits: 25%
            (4, 25.0), // 4 hits: 25%
            (5, 25.0), // 5 hits: 25%
        ]
    };
    
    // Check for effects that modify hit count (Loaded Dice, Skill Link)
    let user_pokemon = state.get_pokemon_at_position(user_position);
    let force_max_hits = user_pokemon.map_or(false, |pokemon| {
        // Check for Loaded Dice item
        if let Some(ref item) = pokemon.item {
            if item.to_lowercase() == "loaded dice" || item.to_lowercase() == "loadeddice" {
                return true;
            }
        }
        
        // Check for Skill Link ability
        if pokemon.ability.to_lowercase() == "skill link" || pokemon.ability.to_lowercase() == "skilllink" {
            return true;
        }
        
        false
    });
    
    // Handle special cases for specific moves
    let hit_distribution = match move_data.name.to_lowercase().as_str() {
        "doubleslap" | "double slap" | "bonemerang" => {
            // These moves always hit exactly 2 times (not affected by Loaded Dice/Skill Link)
            vec![(2, 100.0)]
        }
        "tripleaxel" | "triple axel" | "triplekick" | "triple kick" => {
            // These moves always hit exactly 3 times (not affected by Loaded Dice/Skill Link)
            vec![(3, 100.0)]
        }
        "beatup" | "beat up" => {
            // Beat Up hits once per conscious party member (not affected by Loaded Dice/Skill Link)
            // For now, assume standard multi-hit
            hit_probabilities
        }
        _ => {
            if force_max_hits {
                // Loaded Dice or Skill Link: always hit maximum (5 times)
                vec![(5, 100.0)]
            } else {
                hit_probabilities
            }
        }
    };
    
    // Generate instructions for each possible hit count
    for (hit_count, probability) in hit_distribution {
        if probability > 0.0 {
            let hit_instructions = generate_multi_hit_instructions(
                state, 
                move_data, 
                user_position, 
                target_positions, 
                hit_count, 
                generation
            );
            
            instructions.push(BattleInstructions::new(probability, hit_instructions));
        }
    }
    
    if instructions.is_empty() {
        instructions.push(BattleInstructions::new(100.0, vec![]));
    }
    
    instructions
}

/// Generate the actual damage instructions for a multi-hit move
fn generate_multi_hit_instructions(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    hit_count: i32,
    generation: &GenerationMechanics,
) -> Vec<BattleInstruction> {
    let mut instructions = Vec::new();
    
    // For each hit, calculate damage
    for hit_number in 1..=hit_count {
        for &target_position in target_positions {
            // Calculate damage for this hit
            let damage = calculate_multi_hit_damage(
                state, 
                move_data, 
                user_position, 
                target_position, 
                hit_number, 
                generation
            );
            
            if damage > 0 {
                instructions.push(BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: target_position,
                amount: damage,
                previous_hp: Some(0), // This should be set to actual previous HP
            }));
            }
        }
    }
    
    instructions
}

/// Calculate damage for a single hit of a multi-hit move
fn calculate_multi_hit_damage(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_position: BattlePosition,
    hit_number: i32,
    generation: &GenerationMechanics,
) -> i16 {
    // Get attacking Pokemon
    let attacker = state
        .get_pokemon_at_position(user_position)
        .expect("Attacker position should be valid");

    // Get defending Pokemon
    let defender = state
        .get_pokemon_at_position(target_position)
        .expect("Target position should be valid");

    // Check for type immunities first
    if is_immune_to_move_type(&move_data.move_type, defender) {
        return 0;
    }

    // Check for ability immunities
    if is_immune_due_to_ability(move_data, defender) {
        return 0;
    }

    // Calculate base damage for each hit
    // Handle special cases for moves with increasing power per hit
    let effective_power = match move_data.name.to_lowercase().as_str() {
        "tripleaxel" | "triple axel" => {
            // Triple Axel: 20/40/60 base power for hits 1/2/3
            match hit_number {
                1 => 20,
                2 => 40,
                3 => 60,
                _ => move_data.base_power.max(20) as i16,
            }
        }
        "triplekick" | "triple kick" => {
            // Triple Kick: 10/20/30 base power for hits 1/2/3
            match hit_number {
                1 => 10,
                2 => 20,
                3 => 30,
                _ => move_data.base_power.max(10) as i16,
            }
        }
        _ => move_data.base_power.max(0) as i16,
    };

    // Create a modified move data with the correct power for this hit
    let mut modified_move_data = move_data.clone();
    modified_move_data.base_power = effective_power as u16;

    let base_damage = super::super::damage_calc::calculate_damage_with_positions(
        state,
        attacker,
        defender,
        &modified_move_data,
        false, // Not a critical hit for base calculation
        1.0,   // Full damage roll
        1,     // Single target for each hit
        user_position,
        target_position,
    );
    
    base_damage
}

/// Check if a Pokemon is immune to a move type (e.g., Ghost immune to Normal/Fighting)
fn is_immune_to_move_type(move_type: &str, defender: &crate::core::battle_state::Pokemon) -> bool {
    use crate::engine::combat::type_effectiveness::{PokemonType, TypeChart};

    // Use a basic type chart for now - in full implementation this would use generation-specific charts
    let type_chart = TypeChart::new(9); // Gen 9 type chart
    let attacking_type = PokemonType::from_str(move_type).unwrap_or(PokemonType::Normal);
    
    let defender_type1 = PokemonType::from_str(&defender.types[0]).unwrap_or(PokemonType::Normal);
    let defender_type2 = if defender.types.len() > 1 {
        PokemonType::from_str(&defender.types[1]).unwrap_or(defender_type1)
    } else {
        defender_type1
    };

    let type_effectiveness = type_chart.calculate_damage_multiplier(
        attacking_type,
        (defender_type1, defender_type2),
        None,
        None,
    );

    // If type effectiveness is 0, the Pokemon is immune
    type_effectiveness == 0.0
}

/// Check if a Pokemon is immune due to ability (e.g., Levitate vs Ground)
fn is_immune_due_to_ability(move_data: &MoveData, defender: &crate::core::battle_state::Pokemon) -> bool {
    use crate::engine::mechanics::abilities::ability_provides_immunity;
    
    ability_provides_immunity(defender.ability.as_str(), &move_data.move_type)
}

// =============================================================================
// INDIVIDUAL MULTI-HIT MOVE FUNCTIONS (All use apply_multi_hit_move)
// =============================================================================

/// Double Slap - Always hits exactly 2 times
pub fn apply_double_slap(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    apply_multi_hit_move(state, move_data, user_position, target_positions, generation)
}

/// Comet Punch - Hits 2-5 times with standard distribution
pub fn apply_comet_punch(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    apply_multi_hit_move(state, move_data, user_position, target_positions, generation)
}

/// Fury Attack - Hits 2-5 times with standard distribution
pub fn apply_fury_attack(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    apply_multi_hit_move(state, move_data, user_position, target_positions, generation)
}

/// Pin Missile - Hits 2-5 times with standard distribution
pub fn apply_pin_missile(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    apply_multi_hit_move(state, move_data, user_position, target_positions, generation)
}

/// Barrage - Hits 2-5 times with standard distribution
pub fn apply_barrage(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    apply_multi_hit_move(state, move_data, user_position, target_positions, generation)
}

/// Spike Cannon - Hits 2-5 times with standard distribution
pub fn apply_spike_cannon(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    apply_multi_hit_move(state, move_data, user_position, target_positions, generation)
}

/// Bonemerang - Always hits exactly 2 times
pub fn apply_bonemerang(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    apply_multi_hit_move(state, move_data, user_position, target_positions, generation)
}

/// Bullet Seed - Hits 2-5 times with standard distribution
pub fn apply_bullet_seed(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    apply_multi_hit_move(state, move_data, user_position, target_positions, generation)
}

/// Icicle Shard - Hits 2-5 times with standard distribution
pub fn apply_icicle_shard(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    apply_multi_hit_move(state, move_data, user_position, target_positions, generation)
}

/// Rock Blast - Hits 2-5 times with standard distribution
pub fn apply_rock_blast(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    apply_multi_hit_move(state, move_data, user_position, target_positions, generation)
}

/// Tail Slap - Hits 2-5 times with standard distribution
pub fn apply_tail_slap(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    apply_multi_hit_move(state, move_data, user_position, target_positions, generation)
}

/// Beat Up - Hits once per conscious party member
pub fn apply_beat_up(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    apply_multi_hit_move(state, move_data, user_position, target_positions, generation)
}

/// Arm Thrust - Hits 2-5 times with standard distribution
pub fn apply_arm_thrust(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    apply_multi_hit_move(state, move_data, user_position, target_positions, generation)
}

/// Triple Axel - Always hits exactly 3 times with increasing power (20/40/60)
pub fn apply_triple_axel(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    apply_multi_hit_move(state, move_data, user_position, target_positions, generation)
}

/// Triple Kick - Always hits exactly 3 times with increasing power (10/20/30)
pub fn apply_triple_kick(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    apply_multi_hit_move(state, move_data, user_position, target_positions, generation)
}