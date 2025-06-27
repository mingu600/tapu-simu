//! # Move Prevention System
//! 
//! Implements poke-engine style move prevention checks including status conditions,
//! flinch, and other prevention effects with proper probability branching.

use crate::core::battle_format::BattlePosition;
use crate::core::battle_state::{BattleState, Pokemon};
use crate::core::move_choice::MoveChoice;
use crate::core::instructions::{PokemonStatus, VolatileStatus, BattleInstructions, BattleInstruction, PokemonInstruction, StatusInstruction};
use crate::data::showdown_types::MoveData;
use serde::{Deserialize, Serialize};

/// Reasons why a move might be prevented from being used
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MovePreventionReason {
    /// Flinch prevents normal/negative priority moves
    Flinch,
    /// Sleep prevents all moves (with wake-up chance)
    Sleep { wake_up_chance: f32 },
    /// Freeze prevents all moves (with thaw chance)
    Freeze { thaw_chance: f32 },
    /// Paralysis has 25% chance to prevent moves
    Paralysis,
    /// Confusion has 33% chance to prevent moves and cause self-damage
    Confusion { self_damage: i16 },
    /// Taunt prevents status moves
    Taunt,
    /// Torment prevents using the same move twice in a row
    Torment,
    /// Disable prevents using a specific disabled move
    Disable,
    /// Encore forces using the same move
    Encore,
    /// Choice item locks into the last used move
    ChoiceLock,
}

/// Check if a Pokemon cannot use a move due to status conditions or other effects
/// Returns Some(reason) if the move is prevented, None if the move can be used
pub fn cannot_use_move(
    pokemon: &Pokemon, 
    move_choice: &MoveChoice,
    move_data: Option<&MoveData>,
    battle_state: &BattleState,
    position: BattlePosition
) -> Option<MovePreventionReason> {
    // Only check prevention for move choices, not switches
    if move_choice.is_switch() {
        return None;
    }
    
    // Check in poke-engine order:
    
    // 1. Flinch (only for normal/negative priority moves)
    if let Some(reason) = check_flinch_prevention(pokemon, move_choice, move_data, battle_state, position) {
        return Some(reason);
    }
    
    // 2. Sleep (with wake-up chance calculation)
    if let Some(reason) = check_sleep_prevention(pokemon, battle_state, position) {
        return Some(reason);
    }
    
    // 3. Freeze (with thaw chance)
    if let Some(reason) = check_freeze_prevention(pokemon, move_choice, move_data, battle_state, position) {
        return Some(reason);
    }
    
    // 4. Paralysis (25% chance)
    if let Some(reason) = check_paralysis_prevention(pokemon, battle_state, position) {
        return Some(reason);
    }
    
    // 5. Confusion (33% chance + self-damage)
    if let Some(reason) = check_confusion_prevention(pokemon, battle_state, position) {
        return Some(reason);
    }
    
    // 6. Other move-specific prevention effects
    if let Some(reason) = check_other_prevention_effects(pokemon, move_choice, move_data, battle_state, position) {
        return Some(reason);
    }
    
    None
}

/// Check if flinch prevents move usage
/// Flinch prevents ALL moves regardless of priority - it's based on speed order, not move priority
fn check_flinch_prevention(
    pokemon: &Pokemon,
    _move_choice: &MoveChoice,
    _move_data: Option<&MoveData>,
    _battle_state: &BattleState,
    _position: BattlePosition
) -> Option<MovePreventionReason> {
    if !pokemon.volatile_statuses.contains(&VolatileStatus::Flinch) {
        return None;
    }
    
    // Flinch prevents ALL moves - the flinch check is based on whether the flinching move
    // came from a faster Pokemon, which is determined during flinch application, not here
    Some(MovePreventionReason::Flinch)
}

/// Check if sleep prevents move usage
fn check_sleep_prevention(
    pokemon: &Pokemon,
    _battle_state: &BattleState,
    _position: BattlePosition
) -> Option<MovePreventionReason> {
    if pokemon.status != PokemonStatus::Sleep {
        return None;
    }
    
    // Calculate wake-up chance based on sleep turns
    // In poke-engine: sleep lasts 1-3 turns, with higher chance to wake up each turn
    let wake_up_chance = match pokemon.status_duration {
        Some(1) => 100.0, // Always wake up after 1 turn
        Some(2) => 50.0,  // 50% chance to wake up on turn 2
        Some(3) => 33.3,  // 33% chance to wake up on turn 3
        _ => 33.3,        // Default wake-up chance
    };
    
    Some(MovePreventionReason::Sleep { wake_up_chance })
}

/// Check if freeze prevents move usage
fn check_freeze_prevention(
    pokemon: &Pokemon,
    move_choice: &MoveChoice,
    move_data: Option<&MoveData>,
    _battle_state: &BattleState,
    _position: BattlePosition
) -> Option<MovePreventionReason> {
    if pokemon.status != PokemonStatus::Freeze {
        return None;
    }
    
    // Calculate thaw chance based on move used
    let thaw_chance = if let Some(data) = move_data {
        if is_fire_type_move(&data.move_type) {
            100.0 // Fire moves always thaw
        } else {
            20.0 // 20% base thaw chance
        }
    } else if let Some(move_index) = move_choice.move_index() {
        if let Some(move_data_internal) = pokemon.get_move(move_index) {
            if is_fire_type_move(&move_data_internal.move_type) {
                100.0 // Fire moves always thaw
            } else {
                20.0 // 20% base thaw chance
            }
        } else {
            20.0
        }
    } else {
        20.0
    };
    
    Some(MovePreventionReason::Freeze { thaw_chance })
}

/// Check if paralysis prevents move usage
fn check_paralysis_prevention(
    pokemon: &Pokemon,
    _battle_state: &BattleState,
    _position: BattlePosition
) -> Option<MovePreventionReason> {
    if pokemon.status != PokemonStatus::Paralysis {
        return None;
    }
    
    // 25% chance to prevent move
    Some(MovePreventionReason::Paralysis)
}

/// Check if confusion prevents move usage and calculate self-damage
fn check_confusion_prevention(
    pokemon: &Pokemon,
    _battle_state: &BattleState,
    _position: BattlePosition
) -> Option<MovePreventionReason> {
    if !pokemon.volatile_statuses.contains(&VolatileStatus::Confusion) {
        return None;
    }
    
    // Calculate self-damage for confusion
    // Confusion self-damage uses base 40 power physical move calculation
    let self_damage = calculate_confusion_self_damage(pokemon);
    
    // 33% chance to prevent move and cause self-damage
    Some(MovePreventionReason::Confusion { self_damage })
}

/// Check other prevention effects (Taunt, Torment, Disable, etc.)
fn check_other_prevention_effects(
    pokemon: &Pokemon,
    move_choice: &MoveChoice,
    move_data: Option<&MoveData>,
    _battle_state: &BattleState,
    _position: BattlePosition
) -> Option<MovePreventionReason> {
    // Check Taunt (prevents status moves)
    if pokemon.volatile_statuses.contains(&VolatileStatus::Taunt) {
        let is_status_move = if let Some(data) = move_data {
            data.category.to_lowercase() == "status"
        } else if let Some(move_index) = move_choice.move_index() {
            if let Some(move_data_internal) = pokemon.get_move(move_index) {
                move_data_internal.category == crate::core::battle_state::MoveCategory::Status
            } else {
                false
            }
        } else {
            false
        };
        
        if is_status_move {
            return Some(MovePreventionReason::Taunt);
        }
    }
    
    // Check Torment (prevents using same move twice in a row)
    if pokemon.volatile_statuses.contains(&VolatileStatus::Torment) {
        // For now, implement basic torment check
        // TODO: Implement proper last move tracking
        // return Some(MovePreventionReason::Torment);
    }
    
    // Check Disable (prevents using a specific disabled move)
    if pokemon.volatile_statuses.contains(&VolatileStatus::Disable) {
        // TODO: Implement proper move disable tracking
        // return Some(MovePreventionReason::Disable);
    }
    
    // Check Encore (forces using the same move)
    if pokemon.volatile_statuses.contains(&VolatileStatus::Encore) {
        // TODO: Implement proper encore tracking
        // return Some(MovePreventionReason::Encore);
    }
    
    None
}

/// Calculate confusion self-damage (base 40 power physical move)
fn calculate_confusion_self_damage(pokemon: &Pokemon) -> i16 {
    // Confusion self-damage formula: base 40 power physical move
    // Damage = ((2 * Level / 5 + 2) * Power * Attack / Defense / 50 + 2) * random(85-100)
    let level = pokemon.level as f32;
    let attack = pokemon.get_effective_stat(crate::core::instructions::Stat::Attack) as f32;
    let defense = pokemon.get_effective_stat(crate::core::instructions::Stat::Defense) as f32;
    let power = 40.0; // Confusion self-damage uses base 40 power
    
    // Basic damage formula (using average roll of 92.5%)
    let damage = ((2.0 * level / 5.0 + 2.0) * power * attack / defense / 50.0 + 2.0) * 0.925;
    
    // Cap damage at current HP - 1 (confusion can't kill)
    let max_damage = pokemon.hp.saturating_sub(1);
    (damage as i16).min(max_damage)
}

/// Check if a move type is Fire-type (for freeze thaw calculation)
fn is_fire_type_move(move_type: &str) -> bool {
    move_type.to_lowercase() == "fire"
}

/// Generate prevention instructions with probability branching
pub fn generate_prevention_instructions(
    prevention: MovePreventionReason,
    position: BattlePosition,
    pokemon: &Pokemon,
) -> Vec<BattleInstructions> {
    match prevention {
        MovePreventionReason::Flinch => {
            // Flinch is deterministic - move is always prevented
            vec![BattleInstructions::new(100.0, vec![])]
        }
        
        MovePreventionReason::Sleep { wake_up_chance } => {
            let mut instructions = Vec::new();
            
            // Wake up branch
            if wake_up_chance > 0.0 {
                let wake_up_instructions = vec![
                    BattleInstruction::Status(StatusInstruction::Remove {
                        target: position,
                        status: PokemonStatus::Sleep,
                        previous_duration: pokemon.status_duration,
                    }),
                ];
                instructions.push(BattleInstructions::new(wake_up_chance, wake_up_instructions));
            }
            
            // Stay asleep branch (move prevented)
            let stay_asleep_chance = 100.0 - wake_up_chance;
            if stay_asleep_chance > 0.0 {
                instructions.push(BattleInstructions::new(stay_asleep_chance, vec![]));
            }
            
            instructions
        }
        
        MovePreventionReason::Freeze { thaw_chance } => {
            let mut instructions = Vec::new();
            
            // Thaw branch
            if thaw_chance > 0.0 {
                let thaw_instructions = vec![
                    BattleInstruction::Status(StatusInstruction::Remove {
                        target: position,
                        status: PokemonStatus::Freeze,
                        previous_duration: None,
                    }),
                ];
                instructions.push(BattleInstructions::new(thaw_chance, thaw_instructions));
            }
            
            // Stay frozen branch (move prevented)
            let stay_frozen_chance = 100.0 - thaw_chance;
            if stay_frozen_chance > 0.0 {
                instructions.push(BattleInstructions::new(stay_frozen_chance, vec![]));
            }
            
            instructions
        }
        
        MovePreventionReason::Paralysis => {
            let mut instructions = Vec::new();
            
            // Move succeeds (75% chance)
            instructions.push(BattleInstructions::new(75.0, vec![]));
            
            // Move prevented by paralysis (25% chance)
            instructions.push(BattleInstructions::new(25.0, vec![]));
            
            instructions
        }
        
        MovePreventionReason::Confusion { self_damage } => {
            let mut instructions = Vec::new();
            
            // Move succeeds (67% chance)
            let success_instructions = vec![
                BattleInstruction::Status(StatusInstruction::RemoveVolatile {
                    target: position,
                    status: VolatileStatus::Confusion,
                    previous_duration: None, // Confusion removes itself after use
                }),
            ];
            instructions.push(BattleInstructions::new(67.0, success_instructions));
            
            // Move prevented and self-damage (33% chance)
            let prevent_instructions = vec![
                BattleInstruction::Pokemon(PokemonInstruction::Damage {
                    target: position,
                    amount: self_damage,
                    previous_hp: Some(pokemon.hp),
                }),
                BattleInstruction::Status(StatusInstruction::RemoveVolatile {
                    target: position,
                    status: VolatileStatus::Confusion,
                    previous_duration: None,
                }),
            ];
            instructions.push(BattleInstructions::new(33.0, prevent_instructions));
            
            instructions
        }
        
        MovePreventionReason::Taunt => {
            // Taunt is deterministic - status moves are always prevented
            vec![BattleInstructions::new(100.0, vec![])]
        }
        
        MovePreventionReason::Torment => {
            // Torment is deterministic - repeat move is always prevented
            vec![BattleInstructions::new(100.0, vec![])]
        }
        
        MovePreventionReason::Disable => {
            // Disable is deterministic - disabled move is always prevented
            vec![BattleInstructions::new(100.0, vec![])]
        }
        
        MovePreventionReason::Encore => {
            // Encore is deterministic - wrong move is always prevented
            vec![BattleInstructions::new(100.0, vec![])]
        }
        
        MovePreventionReason::ChoiceLock => {
            // Choice lock is deterministic - wrong move is always prevented
            vec![BattleInstructions::new(100.0, vec![])]
        }
    }
}

/// Apply flinch effect with speed awareness
/// Returns true if the flinch should be applied (user is faster than target)
/// The actual flinch status should only be applied if this returns true
pub fn apply_flinch_effect(
    target_position: BattlePosition,
    user_position: BattlePosition,
    battle_state: &BattleState
) -> bool {
    // Flinch only works if user is faster than target
    // This determines whether the flinch status should be applied
    let user = battle_state.get_pokemon_at_position(user_position);
    let target = battle_state.get_pokemon_at_position(target_position);
    
    if let (Some(user_pokemon), Some(target_pokemon)) = (user, target) {
        let user_speed = user_pokemon.get_effective_stat(crate::core::instructions::Stat::Speed);
        let target_speed = target_pokemon.get_effective_stat(crate::core::instructions::Stat::Speed);
        
        user_speed > target_speed
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::battle_state::{Pokemon, MoveCategory};
    use crate::core::instructions::Stat;
    use std::collections::HashMap;

    fn create_test_pokemon() -> Pokemon {
        use crate::data::types::Stats;
        Pokemon {
            species: "Pikachu".to_string(),
            hp: 100,
            max_hp: 100,
            stats: Stats {
                hp: 100,
                attack: 100,
                defense: 100,
                special_attack: 100,
                special_defense: 100,
                speed: 100,
            },
            base_stats: Stats {
                hp: 100,
                attack: 100,
                defense: 100,
                special_attack: 100,
                special_defense: 100,
                speed: 100,
            },
            stat_boosts: HashMap::new(),
            status: PokemonStatus::None,
            status_duration: None,
            volatile_statuses: std::collections::HashSet::new(),
            volatile_status_durations: HashMap::new(),
            substitute_health: 0,
            moves: HashMap::new(),
            ability: "Static".to_string(),
            item: None,
            types: vec!["Electric".to_string()],
            level: 50,
            gender: crate::core::battle_state::Gender::Unknown,
            tera_type: None,
            is_terastallized: false,
            ability_suppressed: false,
            ability_triggered_this_turn: false,
            item_consumed: false,
            weight_kg: 6.0,
        }
    }

    #[test]
    fn test_flinch_prevention_all_moves() {
        let mut pokemon = create_test_pokemon();
        pokemon.volatile_statuses.insert(VolatileStatus::Flinch);
        
        let move_data = MoveData {
            name: "Tackle".to_string(),
            priority: 0, // Normal priority - should be prevented
            ..Default::default()
        };
        
        let move_choice = MoveChoice::Move {
            move_index: crate::core::move_choice::MoveIndex::M1,
            target_positions: vec![],
        };
        
        let result = check_flinch_prevention(
            &pokemon, 
            &move_choice, 
            Some(&move_data), 
            &BattleState::default(), 
            BattlePosition::new(crate::core::battle_format::SideReference::SideOne, 0)
        );
        
        assert_eq!(result, Some(MovePreventionReason::Flinch));
    }

    #[test]
    fn test_flinch_prevents_high_priority_moves_too() {
        let mut pokemon = create_test_pokemon();
        pokemon.volatile_statuses.insert(VolatileStatus::Flinch);
        
        let move_data = MoveData {
            name: "Quick Attack".to_string(),
            priority: 1, // High priority - should STILL be prevented by flinch
            ..Default::default()
        };
        
        let move_choice = MoveChoice::Move {
            move_index: crate::core::move_choice::MoveIndex::M1,
            target_positions: vec![],
        };
        
        let result = check_flinch_prevention(
            &pokemon, 
            &move_choice, 
            Some(&move_data), 
            &BattleState::default(), 
            BattlePosition::new(crate::core::battle_format::SideReference::SideOne, 0)
        );
        
        // Flinch prevents ALL moves regardless of priority
        assert_eq!(result, Some(MovePreventionReason::Flinch));
    }

    #[test]
    fn test_sleep_prevention() {
        let mut pokemon = create_test_pokemon();
        pokemon.status = PokemonStatus::Sleep;
        pokemon.status_duration = Some(2);
        
        let result = check_sleep_prevention(
            &pokemon, 
            &BattleState::default(), 
            BattlePosition::new(crate::core::battle_format::SideReference::SideOne, 0)
        );
        
        assert_eq!(result, Some(MovePreventionReason::Sleep { wake_up_chance: 50.0 }));
    }

    #[test]
    fn test_paralysis_prevention() {
        let mut pokemon = create_test_pokemon();
        pokemon.status = PokemonStatus::Paralysis;
        
        let result = check_paralysis_prevention(
            &pokemon, 
            &BattleState::default(), 
            BattlePosition::new(crate::core::battle_format::SideReference::SideOne, 0)
        );
        
        assert_eq!(result, Some(MovePreventionReason::Paralysis));
    }

    #[test]
    fn test_confusion_prevention() {
        let mut pokemon = create_test_pokemon();
        pokemon.volatile_statuses.insert(VolatileStatus::Confusion);
        
        let result = check_confusion_prevention(
            &pokemon, 
            &BattleState::default(), 
            BattlePosition::new(crate::core::battle_format::SideReference::SideOne, 0)
        );
        
        match result {
            Some(MovePreventionReason::Confusion { self_damage }) => {
                assert!(self_damage > 0);
                assert!(self_damage < pokemon.hp); // Can't kill
            }
            _ => panic!("Expected confusion prevention"),
        }
    }
}