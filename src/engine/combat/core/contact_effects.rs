//! Centralized contact effects system
//!
//! This module handles all contact-based effects that occur after a move hits,
//! including ability triggers, item effects, and contact damage. This eliminates
//! the need to manually implement contact checking in every move.

use crate::core::battle_format::BattlePosition;
use crate::core::battle_state::BattleState;
use crate::core::instructions::{BattleInstruction, PokemonInstruction, PokemonStatus, StatusInstruction, StatsInstruction, Stat};
use crate::data::showdown_types::MoveData;
use super::status_system::{apply_status_effect, StatusApplication};
use crate::types::{Abilities, StatBoostArray};
use std::collections::HashMap;

/// Apply all contact effects that should occur after a move hits
///
/// This centralized function handles:
/// - Rocky Helmet, Static, Flame Body, etc.
/// - Ability triggers (Mummy, Cursed Body)
/// - Item effects (Red Card, Eject Button)
pub fn apply_contact_effects(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_position: BattlePosition,
    damage_dealt: i16,
) -> Vec<BattleInstruction> {
    let mut instructions = Vec::new();

    // Only apply contact effects if the move makes contact
    if !move_data.flags.contains_key("contact") {
        return instructions;
    }

    let user = state.get_pokemon_at_position(user_position);
    let target = state.get_pokemon_at_position(target_position);

    // Apply contact ability effects
    instructions.extend(apply_contact_abilities(state, user_position, target_position, damage_dealt));

    // Apply contact item effects
    instructions.extend(apply_contact_items(state, user_position, target_position, damage_dealt));

    instructions
}

/// Apply ability-based contact effects
fn apply_contact_abilities(
    state: &BattleState,
    user_position: BattlePosition,
    target_position: BattlePosition,
    damage_dealt: i16,
) -> Vec<BattleInstruction> {
    let mut instructions = Vec::new();
    let target = state.get_pokemon_at_position(target_position);

    // Check if target exists
    let target = match target {
        Some(target) => target,
        None => return instructions, // No target, no contact effects
    };

    // Only trigger if target is alive and damage was dealt
    if target.hp == 0 || damage_dealt == 0 {
        return instructions;
    }

    match target.ability {
        Abilities::STATIC => {
            // 30% chance to paralyze the attacker
            let status_app = StatusApplication {
                status: PokemonStatus::Paralysis,
                target: user_position,
                chance: 30.0,
                duration: None,
            };
            let result = apply_status_effect(state, status_app);
            if let Some(instruction) = result.instruction {
                instructions.push(instruction);
            }
        }
        Abilities::FLAMEBODY => {
            // 30% chance to burn the attacker
            let status_app = StatusApplication {
                status: PokemonStatus::Burn,
                target: user_position,
                chance: 30.0,
                duration: None,
            };
            let result = apply_status_effect(state, status_app);
            if let Some(instruction) = result.instruction {
                instructions.push(instruction);
            }
        }
        Abilities::POISONPOINT => {
            // 30% chance to poison the attacker
            let status_app = StatusApplication {
                status: PokemonStatus::Poison,
                target: user_position,
                chance: 30.0,
                duration: None,
            };
            let result = apply_status_effect(state, status_app);
            if let Some(instruction) = result.instruction {
                instructions.push(instruction);
            }
        }
        Abilities::ROUGHSKIN => {
            // Deal 1/8 max HP damage to the attacker
            let user = state.get_pokemon_at_position(user_position);
            if let Some(user) = user {
                let damage = user.stats.hp / 8;
                instructions.push(BattleInstruction::Pokemon(PokemonInstruction::Damage {
                    target: user_position,
                    amount: damage,
                    previous_hp: None,
                }));
            }
        }
        Abilities::IRONBARBS => {
            // Deal 1/8 max HP damage to the attacker
            let user = state.get_pokemon_at_position(user_position);
            if let Some(user) = user {
                let damage = user.stats.hp / 8;
                instructions.push(BattleInstruction::Pokemon(PokemonInstruction::Damage {
                    target: user_position,
                    amount: damage,
                    previous_hp: None,
                }));
            }
        }
        Abilities::GOOEY | Abilities::TANGLINGHAIR => {
            // Lower the attacker's speed by 1 stage
            let mut stat_changes = StatBoostArray::default();
            stat_changes.insert(Stat::Speed, -1);
            let previous_boosts = if let Some(pokemon) = state.get_pokemon_at_position(user_position) {
                pokemon.stat_boosts.to_hashmap()
            } else {
                std::collections::HashMap::new()
            };
            instructions.push(BattleInstruction::Stats(StatsInstruction::BoostStats {
                target: user_position,
                stat_changes: stat_changes.to_hashmap(),
                previous_boosts,
            }));
        }
        Abilities::MUMMY => {
            // Change the attacker's ability to Mummy
            instructions.push(BattleInstruction::Pokemon(PokemonInstruction::ChangeAbility {
                target: user_position,
                new_ability: Abilities::MUMMY,
                previous_ability: state.get_pokemon_at_position(user_position)
                    .map(|p| p.ability),
            }));
        }
        Abilities::CURSEDBODY => {
            // 30% chance to disable the last used move
            use rand::Rng;
            let mut rng = rand::thread_rng();
            if rng.gen_range(0.0..100.0) < 30.0 {
                // For now, just add a placeholder instruction
                // Full disable implementation would require move tracking
                if let Some(user_pokemon) = state.get_pokemon_at_position(user_position) {
                    instructions.push(BattleInstruction::Pokemon(PokemonInstruction::Message {
                        message: format!("{}'s move was disabled by Cursed Body!", user_pokemon.species),
                        affected_positions: vec![user_position],
                    }));
                }
            }
        }
        _ => {
            // No contact ability effect
        }
    }

    instructions
}

/// Apply item-based contact effects
fn apply_contact_items(
    state: &BattleState,
    user_position: BattlePosition,
    target_position: BattlePosition,
    damage_dealt: i16,
) -> Vec<BattleInstruction> {
    let mut instructions = Vec::new();
    let target = state.get_pokemon_at_position(target_position);

    // Check if target exists
    let target = match target {
        Some(target) => target,
        None => return instructions, // No target, no item effects
    };

    // Only trigger if target is alive and damage was dealt
    if target.hp == 0 || damage_dealt == 0 {
        return instructions;
    }

    if let Some(ref item) = target.item {
        match *item {
            crate::types::Items::ROCKYHELMET => {
                // Deal 1/6 max HP damage to the attacker
                let user = state.get_pokemon_at_position(user_position);
                if let Some(user) = user {
                    let damage = user.stats.hp / 6;
                    instructions.push(BattleInstruction::Pokemon(PokemonInstruction::Damage {
                        target: user_position,
                        amount: damage,
                        previous_hp: None,
                    }));
                }
            }
            crate::types::Items::STICKYBARB => {
                // Transfer the Sticky Barb to the attacker
                instructions.push(BattleInstruction::Pokemon(PokemonInstruction::ItemTransfer {
                    from: target_position,
                    to: user_position,
                    item: "stickybarb".to_string(),
                    previous_from_item: target.item.as_ref().map(|i| i.as_str().to_string()),
                    previous_to_item: None, // TODO: Get actual previous item
                }));
            }
            crate::types::Items::REDCARD => {
                // Force the attacker to switch out (in formats that allow it)
                if state.format.allows_switching() {
                    instructions.push(BattleInstruction::Pokemon(PokemonInstruction::ForceSwitch {
                        target: user_position,
                        source: Some(target_position),
                        previous_can_switch: true, // TODO: Get actual previous state
                    }));
                }
            }
            crate::types::Items::EJECTBUTTON => {
                // Force the target to switch out if it took damage
                if state.format.allows_switching() {
                    instructions.push(BattleInstruction::Pokemon(PokemonInstruction::ForceSwitch {
                        target: target_position,
                        source: Some(user_position),
                        previous_can_switch: true, // TODO: Get actual previous state
                    }));
                }
            }
            _ => {
                // No contact item effect
            }
        }
    }

    instructions
}

/// Check if a move should trigger contact effects
pub fn should_trigger_contact_effects(move_data: &MoveData) -> bool {
    move_data.flags.contains_key("contact")
}

/// Apply contact effects for a specific ability
pub fn apply_specific_contact_ability(
    state: &BattleState,
    ability: &str,
    user_position: BattlePosition,
    target_position: BattlePosition,
) -> Vec<BattleInstruction> {
    let mut instructions = Vec::new();

    match ability.to_lowercase().as_str() {
        "effectspore" => {
            // 30% chance to inflict a random status (poison, paralysis, or sleep)
            use rand::Rng;
            let mut rng = rand::thread_rng();
            if rng.gen_range(0.0..100.0) < 30.0 {
                let status = match rng.gen_range(0..3) {
                    0 => PokemonStatus::Poison,
                    1 => PokemonStatus::Paralysis,
                    _ => PokemonStatus::Sleep,
                };

                let status_app = StatusApplication {
                    status,
                    target: user_position,
                    chance: 100.0, // Already passed the 30% check
                    duration: None,
                };
                let result = apply_status_effect(state, status_app);
                if let Some(instruction) = result.instruction {
                    instructions.push(instruction);
                }
            }
        }
        "aftermath" => {
            // Deal 1/4 max HP damage if the Pokemon faints from the contact move
            let target_check = state.get_pokemon_at_position(target_position);
            if let Some(target_check) = target_check {
                if target_check.hp == 0 {
                    let user = state.get_pokemon_at_position(user_position);
                    if let Some(user) = user {
                        let damage = user.stats.hp / 4;
                        instructions.push(BattleInstruction::Pokemon(PokemonInstruction::Damage {
                            target: user_position,
                            amount: damage,
                            previous_hp: None,
                        }));
                    }
                }
            }
        }
        _ => {
            // No specific implementation for this ability
        }
    }

    instructions
}

/// Apply recoil damage for moves that have recoil
pub fn apply_recoil_damage(
    state: &BattleState,
    user_position: BattlePosition,
    damage_dealt: i16,
    recoil_fraction: f32,
) -> Vec<BattleInstruction> {
    let mut instructions = Vec::new();
    
    if damage_dealt > 0 {
        let recoil_damage = ((damage_dealt as f32) * recoil_fraction) as i16;
        if recoil_damage > 0 {
            instructions.push(BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: user_position,
                amount: recoil_damage,
                previous_hp: None,
            }));
        }
    }

    instructions
}

/// Apply healing based on damage dealt (for draining moves)
pub fn apply_drain_healing(
    state: &BattleState,
    user_position: BattlePosition,
    damage_dealt: i16,
    drain_fraction: f32,
) -> Vec<BattleInstruction> {
    let mut instructions = Vec::new();
    
    if damage_dealt > 0 {
        let heal_amount = ((damage_dealt as f32) * drain_fraction) as i16;
        if heal_amount > 0 {
            instructions.push(BattleInstruction::Pokemon(PokemonInstruction::Heal {
                target: user_position,
                amount: heal_amount,
                previous_hp: None, // TODO: Get actual previous HP
            }));
        }
    }

    instructions
}