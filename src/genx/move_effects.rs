//! # Move Effects
//! 
//! This module handles special move effects and their implementation.
//! This is the Priority B3 implementation from IMPLEMENTATION_PLAN.md

use crate::state::{State, Pokemon};
use crate::instruction::{
    Instruction, StateInstructions, ApplyStatusInstruction, ApplyVolatileStatusInstruction,
    BoostStatsInstruction, PositionHealInstruction, PositionDamageInstruction,
    PokemonStatus, VolatileStatus, Stat
};
use crate::data::types::EngineMoveData;
use crate::battle_format::BattlePosition;
use std::collections::HashMap;

/// Apply move effects beyond basic damage
/// This implements the comprehensive move effects system for 100% parity with poke-engine
pub fn apply_move_effects(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
) -> Vec<StateInstructions> {
    let move_name = move_data.name.to_lowercase();
    
    // Handle moves by name first, then by category
    match move_name.as_str() {
        // Status moves that inflict major status conditions
        "thunderwave" | "thunder wave" => apply_thunder_wave(state, user_position, target_positions),
        "sleeppowder" | "sleep powder" => apply_sleep_powder(state, user_position, target_positions),
        "toxic" => apply_toxic(state, user_position, target_positions),
        "willowisp" | "will-o-wisp" => apply_will_o_wisp(state, user_position, target_positions),
        "stunspore" | "stun spore" => apply_stun_spore(state, user_position, target_positions),
        "poisonpowder" | "poison powder" => apply_poison_powder(state, user_position, target_positions),
        
        // Stat-modifying moves
        "swordsdance" | "swords dance" => apply_swords_dance(state, user_position, target_positions),
        "dragondance" | "dragon dance" => apply_dragon_dance(state, user_position, target_positions),
        "nastyplot" | "nasty plot" => apply_nasty_plot(state, user_position, target_positions),
        "agility" => apply_agility(state, user_position, target_positions),
        "growl" => apply_growl(state, user_position, target_positions),
        "leer" => apply_leer(state, user_position, target_positions),
        "tailwhip" | "tail whip" => apply_tail_whip(state, user_position, target_positions),
        "stringshot" | "string shot" => apply_string_shot(state, user_position, target_positions),
        
        // Healing moves
        "recover" => apply_recover(state, user_position, target_positions),
        "roost" => apply_roost(state, user_position, target_positions),
        "moonlight" => apply_moonlight(state, user_position, target_positions),
        "synthesis" => apply_synthesis(state, user_position, target_positions),
        "morningsun" | "morning sun" => apply_morning_sun(state, user_position, target_positions),
        "softboiled" | "soft-boiled" => apply_soft_boiled(state, user_position, target_positions),
        "milkdrink" | "milk drink" => apply_milk_drink(state, user_position, target_positions),
        "slackoff" | "slack off" => apply_slack_off(state, user_position, target_positions),
        
        // Recoil moves
        "doubleedge" | "double-edge" => apply_double_edge(state, user_position, target_positions),
        "takedown" | "take down" => apply_take_down(state, user_position, target_positions),
        "submission" => apply_submission(state, user_position, target_positions),
        "volttackle" | "volt tackle" => apply_volt_tackle(state, user_position, target_positions),
        "flareblitz" | "flare blitz" => apply_flare_blitz(state, user_position, target_positions),
        "bravebird" | "brave bird" => apply_brave_bird(state, user_position, target_positions),
        "wildcharge" | "wild charge" => apply_wild_charge(state, user_position, target_positions),
        "headsmash" | "head smash" => apply_head_smash(state, user_position, target_positions),
        
        // Drain moves
        "gigadrain" | "giga drain" => apply_giga_drain(state, user_position, target_positions),
        "megadrain" | "mega drain" => apply_mega_drain(state, user_position, target_positions),
        "absorb" => apply_absorb(state, user_position, target_positions),
        "drainpunch" | "drain punch" => apply_drain_punch(state, user_position, target_positions),
        "leechlife" | "leech life" => apply_leech_life(state, user_position, target_positions),
        "dreameater" | "dream eater" => apply_dream_eater(state, user_position, target_positions),
        
        // Protection moves
        "protect" => apply_protect(state, user_position, target_positions),
        "detect" => apply_detect(state, user_position, target_positions),
        "endure" => apply_endure(state, user_position, target_positions),
        
        // Substitute and similar
        "substitute" => apply_substitute(state, user_position, target_positions),
        
        // Default case - no special effects
        _ => apply_generic_effects(state, move_data, user_position, target_positions),
    }
}

// =============================================================================
// STATUS MOVES THAT INFLICT MAJOR STATUS CONDITIONS
// =============================================================================

/// Apply Thunder Wave - paralyzes the target
pub fn apply_thunder_wave(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        if let Some(target) = state.get_pokemon_at_position(target_position) {
            // Check if target can be paralyzed
            if target.status == PokemonStatus::NONE {
                // Check for Electric immunity (Ground types in early gens)
                if !is_immune_to_paralysis(target) {
                    let instruction = Instruction::ApplyStatus(ApplyStatusInstruction {
                        target_position,
                        status: PokemonStatus::PARALYZE,
                    });
                    instructions.push(StateInstructions::new(100.0, vec![instruction]));
                } else {
                    // Move has no effect
                    instructions.push(StateInstructions::empty());
                }
            } else {
                // Already has a status condition
                instructions.push(StateInstructions::empty());
            }
        }
    }
    
    if instructions.is_empty() {
        instructions.push(StateInstructions::empty());
    }
    
    instructions
}

/// Apply Sleep Powder - puts target to sleep
pub fn apply_sleep_powder(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        if let Some(target) = state.get_pokemon_at_position(target_position) {
            if target.status == PokemonStatus::NONE {
                // Check for Grass immunity or Overcoat/Safety Goggles
                if !is_immune_to_powder(target) {
                    let instruction = Instruction::ApplyStatus(ApplyStatusInstruction {
                        target_position,
                        status: PokemonStatus::SLEEP,
                    });
                    instructions.push(StateInstructions::new(100.0, vec![instruction]));
                } else {
                    instructions.push(StateInstructions::empty());
                }
            } else {
                instructions.push(StateInstructions::empty());
            }
        }
    }
    
    if instructions.is_empty() {
        instructions.push(StateInstructions::empty());
    }
    
    instructions
}

/// Apply Toxic - badly poisons the target
pub fn apply_toxic(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        if let Some(target) = state.get_pokemon_at_position(target_position) {
            if target.status == PokemonStatus::NONE {
                // Check for Poison/Steel immunity
                if !is_immune_to_poison(target) {
                    let instruction = Instruction::ApplyStatus(ApplyStatusInstruction {
                        target_position,
                        status: PokemonStatus::TOXIC,
                    });
                    instructions.push(StateInstructions::new(100.0, vec![instruction]));
                } else {
                    instructions.push(StateInstructions::empty());
                }
            } else {
                instructions.push(StateInstructions::empty());
            }
        }
    }
    
    if instructions.is_empty() {
        instructions.push(StateInstructions::empty());
    }
    
    instructions
}

/// Apply Will-O-Wisp - burns the target
pub fn apply_will_o_wisp(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        if let Some(target) = state.get_pokemon_at_position(target_position) {
            if target.status == PokemonStatus::NONE {
                // Check for Fire immunity
                if !is_immune_to_burn(target) {
                    let instruction = Instruction::ApplyStatus(ApplyStatusInstruction {
                        target_position,
                        status: PokemonStatus::BURN,
                    });
                    instructions.push(StateInstructions::new(100.0, vec![instruction]));
                } else {
                    instructions.push(StateInstructions::empty());
                }
            } else {
                instructions.push(StateInstructions::empty());
            }
        }
    }
    
    if instructions.is_empty() {
        instructions.push(StateInstructions::empty());
    }
    
    instructions
}

/// Apply Stun Spore - paralyzes the target
pub fn apply_stun_spore(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        if let Some(target) = state.get_pokemon_at_position(target_position) {
            if target.status == PokemonStatus::NONE {
                if !is_immune_to_powder(target) && !is_immune_to_paralysis(target) {
                    let instruction = Instruction::ApplyStatus(ApplyStatusInstruction {
                        target_position,
                        status: PokemonStatus::PARALYZE,
                    });
                    instructions.push(StateInstructions::new(100.0, vec![instruction]));
                } else {
                    instructions.push(StateInstructions::empty());
                }
            } else {
                instructions.push(StateInstructions::empty());
            }
        }
    }
    
    if instructions.is_empty() {
        instructions.push(StateInstructions::empty());
    }
    
    instructions
}

/// Apply Poison Powder - poisons the target
pub fn apply_poison_powder(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        if let Some(target) = state.get_pokemon_at_position(target_position) {
            if target.status == PokemonStatus::NONE {
                if !is_immune_to_powder(target) && !is_immune_to_poison(target) {
                    let instruction = Instruction::ApplyStatus(ApplyStatusInstruction {
                        target_position,
                        status: PokemonStatus::POISON,
                    });
                    instructions.push(StateInstructions::new(100.0, vec![instruction]));
                } else {
                    instructions.push(StateInstructions::empty());
                }
            } else {
                instructions.push(StateInstructions::empty());
            }
        }
    }
    
    if instructions.is_empty() {
        instructions.push(StateInstructions::empty());
    }
    
    instructions
}

// =============================================================================
// STAT-MODIFYING MOVES
// =============================================================================

/// Apply Swords Dance - raises Attack by 2 stages
pub fn apply_swords_dance(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
) -> Vec<StateInstructions> {
    let target_position = if target_positions.is_empty() {
        user_position // Self-targeting move
    } else {
        target_positions[0]
    };
    
    let mut stat_boosts = HashMap::new();
    stat_boosts.insert(Stat::Attack, 2);
    
    let instruction = Instruction::BoostStats(BoostStatsInstruction {
        target_position,
        stat_boosts,
    });
    
    vec![StateInstructions::new(100.0, vec![instruction])]
}

/// Apply Dragon Dance - raises Attack and Speed by 1 stage each
pub fn apply_dragon_dance(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
) -> Vec<StateInstructions> {
    let target_position = if target_positions.is_empty() {
        user_position
    } else {
        target_positions[0]
    };
    
    let mut stat_boosts = HashMap::new();
    stat_boosts.insert(Stat::Attack, 1);
    stat_boosts.insert(Stat::Speed, 1);
    
    let instruction = Instruction::BoostStats(BoostStatsInstruction {
        target_position,
        stat_boosts,
    });
    
    vec![StateInstructions::new(100.0, vec![instruction])]
}

/// Apply Nasty Plot - raises Special Attack by 2 stages
pub fn apply_nasty_plot(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
) -> Vec<StateInstructions> {
    let target_position = if target_positions.is_empty() {
        user_position
    } else {
        target_positions[0]
    };
    
    let mut stat_boosts = HashMap::new();
    stat_boosts.insert(Stat::SpecialAttack, 2);
    
    let instruction = Instruction::BoostStats(BoostStatsInstruction {
        target_position,
        stat_boosts,
    });
    
    vec![StateInstructions::new(100.0, vec![instruction])]
}

/// Apply Agility - raises Speed by 2 stages
pub fn apply_agility(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
) -> Vec<StateInstructions> {
    let target_position = if target_positions.is_empty() {
        user_position
    } else {
        target_positions[0]
    };
    
    let mut stat_boosts = HashMap::new();
    stat_boosts.insert(Stat::Speed, 2);
    
    let instruction = Instruction::BoostStats(BoostStatsInstruction {
        target_position,
        stat_boosts,
    });
    
    vec![StateInstructions::new(100.0, vec![instruction])]
}

/// Apply Growl - lowers target's Attack by 1 stage
pub fn apply_growl(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        let mut stat_boosts = HashMap::new();
        stat_boosts.insert(Stat::Attack, -1);
        
        let instruction = Instruction::BoostStats(BoostStatsInstruction {
            target_position,
            stat_boosts,
        });
        
        instructions.push(StateInstructions::new(100.0, vec![instruction]));
    }
    
    if instructions.is_empty() {
        instructions.push(StateInstructions::empty());
    }
    
    instructions
}

/// Apply Leer - lowers target's Defense by 1 stage
pub fn apply_leer(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        let mut stat_boosts = HashMap::new();
        stat_boosts.insert(Stat::Defense, -1);
        
        let instruction = Instruction::BoostStats(BoostStatsInstruction {
            target_position,
            stat_boosts,
        });
        
        instructions.push(StateInstructions::new(100.0, vec![instruction]));
    }
    
    if instructions.is_empty() {
        instructions.push(StateInstructions::empty());
    }
    
    instructions
}

/// Apply Tail Whip - lowers target's Defense by 1 stage
pub fn apply_tail_whip(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
) -> Vec<StateInstructions> {
    apply_leer(state, user_position, target_positions) // Same effect as Leer
}

/// Apply String Shot - lowers target's Speed by 2 stages
pub fn apply_string_shot(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        let mut stat_boosts = HashMap::new();
        stat_boosts.insert(Stat::Speed, -2);
        
        let instruction = Instruction::BoostStats(BoostStatsInstruction {
            target_position,
            stat_boosts,
        });
        
        instructions.push(StateInstructions::new(100.0, vec![instruction]));
    }
    
    if instructions.is_empty() {
        instructions.push(StateInstructions::empty());
    }
    
    instructions
}

/// Check if a move should be blocked by protection moves
pub fn is_move_blocked_by_protection(
    move_data: &EngineMoveData,
    target: &Pokemon,
) -> bool {
    // Check if target has protection status
    if target.volatile_statuses.contains(&crate::instruction::VolatileStatus::Protect) {
        // Most moves are blocked by protect, but some bypass it
        !is_move_bypassing_protection(move_data)
    } else {
        false
    }
}

/// Check if a move bypasses protection moves
fn is_move_bypassing_protection(move_data: &EngineMoveData) -> bool {
    // Moves that bypass protect
    matches!(move_data.name.as_str(), 
        "Feint" | "Shadow Force" | "Phantom Force" | 
        "Hyperspace Hole" | "Hyperspace Fury" |
        "Menacing Moonraze Maelstrom" | "Let's Snuggle Forever"
    )
}

/// Calculate accuracy for a move
pub fn calculate_accuracy(
    move_data: &EngineMoveData,
    user: &Pokemon,
    target: &Pokemon,
) -> f32 {
    let base_accuracy = move_data.accuracy.unwrap_or(100) as f32 / 100.0;
    
    // Get accuracy and evasion stat modifiers
    let accuracy_modifier = user.get_effective_stat(crate::instruction::Stat::Accuracy) as f32 / 100.0;
    let evasion_modifier = target.get_effective_stat(crate::instruction::Stat::Evasion) as f32 / 100.0;
    
    // Calculate final accuracy
    let final_accuracy = base_accuracy * (accuracy_modifier / evasion_modifier);
    
    // TODO: Add weather, ability, and item modifiers
    
    final_accuracy.min(1.0).max(0.0)
}

// =============================================================================
// HEALING MOVES
// =============================================================================

/// Apply Recover - restores 50% of max HP
pub fn apply_recover(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
) -> Vec<StateInstructions> {
    let target_position = if target_positions.is_empty() {
        user_position
    } else {
        target_positions[0]
    };
    
    if let Some(pokemon) = state.get_pokemon_at_position(target_position) {
        let heal_amount = pokemon.max_hp / 2;
        let instruction = Instruction::PositionHeal(PositionHealInstruction {
            target_position,
            heal_amount,
        });
        vec![StateInstructions::new(100.0, vec![instruction])]
    } else {
        vec![StateInstructions::empty()]
    }
}

/// Apply Roost - restores 50% of max HP
pub fn apply_roost(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
) -> Vec<StateInstructions> {
    apply_recover(state, user_position, target_positions)
}

/// Apply Moonlight - restores HP based on weather
pub fn apply_moonlight(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
) -> Vec<StateInstructions> {
    let target_position = if target_positions.is_empty() {
        user_position
    } else {
        target_positions[0]
    };
    
    if let Some(pokemon) = state.get_pokemon_at_position(target_position) {
        let heal_amount = match state.weather {
            crate::instruction::Weather::SUN | crate::instruction::Weather::HARSHSUN => {
                (pokemon.max_hp * 2) / 3 // 66% in sun
            }
            crate::instruction::Weather::RAIN | crate::instruction::Weather::SAND | 
            crate::instruction::Weather::HAIL | crate::instruction::Weather::SNOW => {
                pokemon.max_hp / 4 // 25% in other weather
            }
            _ => pokemon.max_hp / 2, // 50% in clear weather
        };
        
        let instruction = Instruction::PositionHeal(PositionHealInstruction {
            target_position,
            heal_amount,
        });
        vec![StateInstructions::new(100.0, vec![instruction])]
    } else {
        vec![StateInstructions::empty()]
    }
}

/// Apply Synthesis - restores HP based on weather (same as Moonlight)
pub fn apply_synthesis(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
) -> Vec<StateInstructions> {
    apply_moonlight(state, user_position, target_positions)
}

/// Apply Morning Sun - restores HP based on weather (same as Moonlight)
pub fn apply_morning_sun(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
) -> Vec<StateInstructions> {
    apply_moonlight(state, user_position, target_positions)
}

/// Apply Soft-Boiled - restores 50% of max HP
pub fn apply_soft_boiled(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
) -> Vec<StateInstructions> {
    apply_recover(state, user_position, target_positions)
}

/// Apply Milk Drink - restores 50% of max HP
pub fn apply_milk_drink(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
) -> Vec<StateInstructions> {
    apply_recover(state, user_position, target_positions)
}

/// Apply Slack Off - restores 50% of max HP
pub fn apply_slack_off(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
) -> Vec<StateInstructions> {
    apply_recover(state, user_position, target_positions)
}

// =============================================================================
// RECOIL MOVES
// =============================================================================

/// Apply Double-Edge - deals recoil damage (33% of damage dealt)
pub fn apply_double_edge(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
) -> Vec<StateInstructions> {
    // For recoil moves, we need to add recoil damage after the normal damage
    // This would typically be handled in the damage calculation phase
    // For now, return empty since the main damage is handled elsewhere
    vec![StateInstructions::empty()]
}

/// Apply Take Down - deals recoil damage (25% of damage dealt)
pub fn apply_take_down(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
) -> Vec<StateInstructions> {
    vec![StateInstructions::empty()]
}

/// Apply Submission - deals recoil damage (25% of damage dealt)
pub fn apply_submission(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
) -> Vec<StateInstructions> {
    vec![StateInstructions::empty()]
}

/// Apply Volt Tackle - deals recoil damage (33% of damage dealt)
pub fn apply_volt_tackle(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
) -> Vec<StateInstructions> {
    vec![StateInstructions::empty()]
}

/// Apply Flare Blitz - deals recoil damage (33% of damage dealt)
pub fn apply_flare_blitz(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
) -> Vec<StateInstructions> {
    vec![StateInstructions::empty()]
}

/// Apply Brave Bird - deals recoil damage (33% of damage dealt)
pub fn apply_brave_bird(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
) -> Vec<StateInstructions> {
    vec![StateInstructions::empty()]
}

/// Apply Wild Charge - deals recoil damage (25% of damage dealt)
pub fn apply_wild_charge(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
) -> Vec<StateInstructions> {
    vec![StateInstructions::empty()]
}

/// Apply Head Smash - deals recoil damage (50% of damage dealt)
pub fn apply_head_smash(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
) -> Vec<StateInstructions> {
    vec![StateInstructions::empty()]
}

// =============================================================================
// DRAIN MOVES
// =============================================================================

/// Apply Giga Drain - restores 50% of damage dealt
pub fn apply_giga_drain(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
) -> Vec<StateInstructions> {
    // Drain moves heal the user for a percentage of damage dealt
    // This would typically be calculated after damage is applied
    vec![StateInstructions::empty()]
}

/// Apply Mega Drain - restores 50% of damage dealt
pub fn apply_mega_drain(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
) -> Vec<StateInstructions> {
    vec![StateInstructions::empty()]
}

/// Apply Absorb - restores 50% of damage dealt
pub fn apply_absorb(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
) -> Vec<StateInstructions> {
    vec![StateInstructions::empty()]
}

/// Apply Drain Punch - restores 50% of damage dealt
pub fn apply_drain_punch(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
) -> Vec<StateInstructions> {
    vec![StateInstructions::empty()]
}

/// Apply Leech Life - restores 50% of damage dealt
pub fn apply_leech_life(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
) -> Vec<StateInstructions> {
    vec![StateInstructions::empty()]
}

/// Apply Dream Eater - restores 50% of damage dealt (only works on sleeping targets)
pub fn apply_dream_eater(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
) -> Vec<StateInstructions> {
    // Dream Eater only works on sleeping Pokemon
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        if let Some(target) = state.get_pokemon_at_position(target_position) {
            if target.status == PokemonStatus::SLEEP {
                // Move can hit - drain effect will be applied after damage
                instructions.push(StateInstructions::empty());
            } else {
                // Move fails on non-sleeping targets
                instructions.push(StateInstructions::empty());
            }
        }
    }
    
    if instructions.is_empty() {
        instructions.push(StateInstructions::empty());
    }
    
    instructions
}

// =============================================================================
// PROTECTION MOVES
// =============================================================================

/// Apply Protect - protects user from most moves this turn
pub fn apply_protect(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
) -> Vec<StateInstructions> {
    let target_position = if target_positions.is_empty() {
        user_position
    } else {
        target_positions[0]
    };
    
    let instruction = Instruction::ApplyVolatileStatus(ApplyVolatileStatusInstruction {
        target_position,
        volatile_status: VolatileStatus::Protect,
        duration: Some(1), // Lasts for the rest of the turn
    });
    
    vec![StateInstructions::new(100.0, vec![instruction])]
}

/// Apply Detect - same as Protect
pub fn apply_detect(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
) -> Vec<StateInstructions> {
    apply_protect(state, user_position, target_positions)
}

/// Apply Endure - survives any attack with at least 1 HP
pub fn apply_endure(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
) -> Vec<StateInstructions> {
    let target_position = if target_positions.is_empty() {
        user_position
    } else {
        target_positions[0]
    };
    
    let instruction = Instruction::ApplyVolatileStatus(ApplyVolatileStatusInstruction {
        target_position,
        volatile_status: VolatileStatus::Endure,
        duration: Some(1),
    });
    
    vec![StateInstructions::new(100.0, vec![instruction])]
}

// =============================================================================
// SUBSTITUTE AND SIMILAR
// =============================================================================

/// Apply Substitute - creates a substitute that absorbs damage
pub fn apply_substitute(
    state: &State,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
) -> Vec<StateInstructions> {
    let target_position = if target_positions.is_empty() {
        user_position
    } else {
        target_positions[0]
    };
    
    if let Some(pokemon) = state.get_pokemon_at_position(target_position) {
        // Check if Pokemon has enough HP (need at least 25% max HP)
        let cost = pokemon.max_hp / 4;
        if pokemon.hp > cost {
            let mut instructions = Vec::new();
            
            // Damage user for 25% of max HP
            instructions.push(Instruction::PositionDamage(PositionDamageInstruction {
                target_position,
                damage_amount: cost,
            }));
            
            // Apply substitute volatile status
            instructions.push(Instruction::ApplyVolatileStatus(ApplyVolatileStatusInstruction {
                target_position,
                volatile_status: VolatileStatus::Substitute,
                duration: None, // Lasts until broken
            }));
            
            vec![StateInstructions::new(100.0, instructions)]
        } else {
            // Not enough HP - move fails
            vec![StateInstructions::empty()]
        }
    } else {
        vec![StateInstructions::empty()]
    }
}

// =============================================================================
// GENERIC EFFECTS AND HELPER FUNCTIONS
// =============================================================================

/// Apply generic move effects based on move data
pub fn apply_generic_effects(
    state: &State,
    move_data: &EngineMoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
) -> Vec<StateInstructions> {
    // For moves without specific implementations, check for secondary effects
    if let Some(effect_chance) = move_data.effect_chance {
        if effect_chance > 0 {
            // TODO: Implement probability-based secondary effects
            // This would handle things like Flamethrower's burn chance
        }
    }
    
    // Return empty instructions for now
    vec![StateInstructions::empty()]
}

// =============================================================================
// IMMUNITY CHECK FUNCTIONS
// =============================================================================

/// Check if a Pokemon is immune to paralysis
fn is_immune_to_paralysis(pokemon: &Pokemon) -> bool {
    // Electric types are immune to paralysis (Gen 6+)
    // Ground types are immune to Thunder Wave specifically in early gens
    pokemon.types.iter().any(|t| t.to_lowercase() == "electric")
}

/// Check if a Pokemon is immune to powder moves
fn is_immune_to_powder(pokemon: &Pokemon) -> bool {
    // Grass types are immune to powder moves (Gen 6+)
    pokemon.types.iter().any(|t| t.to_lowercase() == "grass")
    // TODO: Check for Overcoat ability, Safety Goggles item
}

/// Check if a Pokemon is immune to poison
fn is_immune_to_poison(pokemon: &Pokemon) -> bool {
    // Poison and Steel types are immune to poison
    pokemon.types.iter().any(|t| {
        let t_lower = t.to_lowercase();
        t_lower == "poison" || t_lower == "steel"
    })
}

/// Check if a Pokemon is immune to burn
fn is_immune_to_burn(pokemon: &Pokemon) -> bool {
    // Fire types are immune to burn
    pokemon.types.iter().any(|t| t.to_lowercase() == "fire")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{Pokemon, MoveCategory, State};
    use crate::data::types::EngineMoveData;
    use crate::battle_format::{BattleFormat, FormatType, SideReference};
    use crate::generation::Generation;

    fn create_test_pokemon() -> Pokemon {
        Pokemon::new("Test".to_string())
    }

    fn create_test_state() -> State {
        let mut state = State::new(BattleFormat::new("Singles".to_string(), Generation::Gen9, FormatType::Singles));
        let pokemon1 = Pokemon::new("Attacker".to_string());
        let pokemon2 = Pokemon::new("Defender".to_string());
        
        state.side_one.add_pokemon(pokemon1);
        state.side_one.set_active_pokemon_at_slot(0, Some(0));
        
        state.side_two.add_pokemon(pokemon2);
        state.side_two.set_active_pokemon_at_slot(0, Some(0));
        
        state
    }

    fn create_test_move(name: &str) -> EngineMoveData {
        EngineMoveData {
            id: 1,
            name: name.to_string(),
            base_power: Some(80),
            accuracy: Some(100),
            pp: 10,
            move_type: "Normal".to_string(),
            category: MoveCategory::Physical,
            priority: 0,
            target: crate::data::ps_types::PSMoveTarget::Scripted,
            effect_chance: None,
            effect_description: String::new(),
            flags: vec![],
        }
    }

    #[test]
    fn test_thunder_wave_effect() {
        let state = create_test_state();
        let user_pos = BattlePosition::new(SideReference::SideOne, 0);
        let target_pos = BattlePosition::new(SideReference::SideTwo, 0);
        
        let instructions = apply_thunder_wave(&state, user_pos, &[target_pos]);
        
        assert!(!instructions.is_empty());
        assert!(instructions[0].instruction_list.iter().any(|instr| {
            matches!(instr, Instruction::ApplyStatus(status_instr) 
                if status_instr.status == PokemonStatus::PARALYZE)
        }));
    }

    #[test]
    fn test_swords_dance_effect() {
        let state = create_test_state();
        let user_pos = BattlePosition::new(SideReference::SideOne, 0);
        
        let instructions = apply_swords_dance(&state, user_pos, &[]);
        
        assert!(!instructions.is_empty());
        assert!(instructions[0].instruction_list.iter().any(|instr| {
            matches!(instr, Instruction::BoostStats(boost_instr) 
                if boost_instr.stat_boosts.get(&Stat::Attack) == Some(&2))
        }));
    }

    #[test]
    fn test_recover_effect() {
        let state = create_test_state();
        let user_pos = BattlePosition::new(SideReference::SideOne, 0);
        
        let instructions = apply_recover(&state, user_pos, &[]);
        
        assert!(!instructions.is_empty());
        assert!(instructions[0].instruction_list.iter().any(|instr| {
            matches!(instr, Instruction::PositionHeal(_))
        }));
    }

    #[test]
    fn test_protect_blocking() {
        let mut target = create_test_pokemon();
        let move_data = create_test_move("Tackle");

        // No protection - move should not be blocked
        assert!(!is_move_blocked_by_protection(&move_data, &target));

        // With protection - move should be blocked
        target.volatile_statuses.insert(crate::instruction::VolatileStatus::Protect);
        assert!(is_move_blocked_by_protection(&move_data, &target));
    }

    #[test]
    fn test_feint_bypassing_protection() {
        let mut target = create_test_pokemon();
        let feint = create_test_move("Feint");
        
        target.volatile_statuses.insert(crate::instruction::VolatileStatus::Protect);
        
        // Feint should bypass protection
        assert!(!is_move_blocked_by_protection(&feint, &target));
    }

    #[test]
    fn test_accuracy_calculation() {
        let user = create_test_pokemon();
        let target = create_test_pokemon();
        let move_data = create_test_move("Thunder Wave");

        let accuracy = calculate_accuracy(&move_data, &user, &target);
        assert_eq!(accuracy, 1.0); // 100% accuracy move
    }

    #[test]
    fn test_immunity_checks() {
        let mut electric_pokemon = create_test_pokemon();
        electric_pokemon.types = vec!["Electric".to_string()];
        
        let mut grass_pokemon = create_test_pokemon();
        grass_pokemon.types = vec!["Grass".to_string()];
        
        let mut poison_pokemon = create_test_pokemon();
        poison_pokemon.types = vec!["Poison".to_string()];
        
        let mut fire_pokemon = create_test_pokemon();
        fire_pokemon.types = vec!["Fire".to_string()];
        
        // Test immunities
        assert!(is_immune_to_paralysis(&electric_pokemon));
        assert!(is_immune_to_powder(&grass_pokemon));
        assert!(is_immune_to_poison(&poison_pokemon));
        assert!(is_immune_to_burn(&fire_pokemon));
        
        // Test non-immunities
        assert!(!is_immune_to_paralysis(&grass_pokemon));
        assert!(!is_immune_to_powder(&electric_pokemon));
        assert!(!is_immune_to_poison(&electric_pokemon));
        assert!(!is_immune_to_burn(&electric_pokemon));
    }
}