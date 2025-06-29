//! Damage move composers for common patterns
//!
//! This module provides composer functions for common damage move patterns,
//! building on the core damage system to create reusable move implementations.

use crate::core::battle_format::BattlePosition;
use crate::core::battle_state::BattleState;
use crate::core::instructions::{BattleInstruction, PokemonInstruction, StatsInstruction, Stat};
use crate::data::showdown_types::MoveData;
use crate::generation::GenerationMechanics;
use super::super::core::{
    damage_system::{
        DamageCalculationContext, HitCountCalculator, execute_multi_hit_sequence,
        calculate_damage_with_effects,
    },
    status_system::{StatusApplication, VolatileStatusApplication, apply_multiple_status_effects},
    contact_effects::{apply_recoil_damage, apply_drain_healing},
    substitute_protection::{EffectType, should_block_effect, get_effect_type_for_status_instruction},
};
use crate::types::StatBoostArray;
use std::collections::HashMap;
use crate::core::instructions::{StatusInstruction, VolatileStatus};
use crate::core::instructions::VolatileStatus as VS;

/// Check if the user moved first this turn by comparing with targets
fn check_moved_first(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
) -> bool {
    // Get user Pokemon
    let user_pokemon = match state.get_pokemon_at_position(user_position) {
        Some(pokemon) => pokemon,
        None => return false,
    };
    
    let user_speed = user_pokemon.get_effective_stat(crate::core::instructions::Stat::Speed) as i32;
    
    // Check against all targets
    for &target_position in target_positions {
        if let Some(target_pokemon) = state.get_pokemon_at_position(target_position) {
            let target_speed = target_pokemon.get_effective_stat(crate::core::instructions::Stat::Speed) as i32;
            
            // If target is faster, user didn't move first
            if target_speed > user_speed {
                return false;
            }
            
            // For speed ties, we'll assume the user moved first (simplified)
            // In a real implementation, this would check the actual turn order
        }
    }
    
    true
}

/// Modifiers for damage calculations
#[derive(Debug, Clone)]
pub struct DamageModifiers {
    /// Power multiplier
    pub power_multiplier: Option<f32>,
    /// Force critical hit
    pub force_critical: bool,
    /// Secondary status effects
    pub secondary_effects: Vec<StatusApplication>,
    /// Stat changes to apply
    pub stat_changes: Option<HashMap<Stat, i8>>,
    /// Recoil damage fraction (e.g., 0.33 for 1/3 recoil)
    pub recoil_fraction: Option<f32>,
    /// Drain healing fraction (e.g., 0.5 for 50% healing)
    pub drain_fraction: Option<f32>,
}

impl Default for DamageModifiers {
    fn default() -> Self {
        Self {
            power_multiplier: None,
            force_critical: false,
            secondary_effects: Vec::new(),
            stat_changes: None,
            recoil_fraction: None,
            drain_fraction: None,
        }
    }
}

/// Standard damage move with optional secondary effects
///
/// This is the most commonly used composer for basic attacking moves.
/// Use this for moves like Tackle, Quick Attack, Thunderbolt, etc.
///
/// ## Handles automatically:
/// - Base damage calculation with type effectiveness
/// - Critical hit mechanics
/// - STAB (Same Type Attack Bonus)
/// - Accuracy checks and miss handling
/// - Secondary status effects (if specified in move data)
/// - Contact effects (abilities like Static, Rough Skin)
/// - Recoil/drain damage (if configured)
/// - Multi-target damage distribution
///
/// ## When to use:
/// - Most physical and special attacking moves
/// - Moves with simple secondary effects
/// - Moves that follow standard damage formulas
///
/// ## When NOT to use:
/// - Fixed damage moves (use `fixed_damage_move` instead)
/// - Multi-hit moves (use `multi_hit_move` instead)  
/// - Moves with complex power calculations (use condition-dependent composers)
///
/// ## Example usage:
/// ```rust
/// // For a basic move like Tackle
/// simple_damage_move(state, move_data, user, targets, DamageModifiers::default(), generation)
///
/// // For a move with recoil like Take Down
/// let modifiers = DamageModifiers { recoil_fraction: Some(0.25), ..Default::default() };
/// simple_damage_move(state, move_data, user, targets, modifiers, generation)
/// ```
pub fn simple_damage_move(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    modifiers: DamageModifiers,
    generation: &GenerationMechanics,
) -> Vec<BattleInstruction> {
    let mut instructions = Vec::new();

    for &target_position in target_positions {
        // Create damage context
        let mut context = DamageCalculationContext::new(
            move_data,
            user_position,
            target_position,
            generation.clone(),
            false, // Use deterministic damage for composed moves
        );

        // Apply modifiers
        if let Some(power_mult) = modifiers.power_multiplier {
            context = context.with_power_modifier(power_mult);
        }
        if modifiers.force_critical {
            context = context.with_force_critical();
        }

        // Calculate and apply damage
        let damage_result = calculate_damage_with_effects(state, context);
        let damage_dealt = damage_result.damage;

        if damage_dealt > 0 {
            instructions.push(BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: target_position,
                amount: damage_dealt,
                previous_hp: None, // Will be filled in by battle state
            }));

            // Apply contact effects if the move makes contact
            if move_data.flags.contains_key("contact") {
                let contact_effects = super::super::core::contact_effects::apply_contact_effects(
                    state,
                    move_data,
                    user_position,
                    target_position,
                    damage_dealt,
                );
                instructions.extend(contact_effects);
            }

            // Apply recoil damage
            if let Some(recoil_fraction) = modifiers.recoil_fraction {
                let recoil_instructions = apply_recoil_damage(
                    state,
                    user_position,
                    damage_dealt,
                    recoil_fraction,
                );
                instructions.extend(recoil_instructions);
            }

            // Apply drain healing
            if let Some(drain_fraction) = modifiers.drain_fraction {
                let drain_instructions = apply_drain_healing(
                    state,
                    user_position,
                    damage_dealt,
                    drain_fraction,
                );
                instructions.extend(drain_instructions);
            }
        }

        // Apply secondary status effects (with substitute protection)
        if !modifiers.secondary_effects.is_empty() {
            let status_instructions = apply_multiple_status_effects_with_substitute_protection(
                state,
                modifiers.secondary_effects.clone(),
                target_position,
            );
            instructions.extend(status_instructions);
        }

        // Apply stat changes
        if let Some(ref stat_changes) = modifiers.stat_changes {
            let mut non_zero_changes = StatBoostArray::default();
            for (stat, change) in stat_changes {
                if *change != 0 {
                    non_zero_changes.insert(*stat, *change);
                }
            }
            if !non_zero_changes.is_empty() {
                instructions.push(BattleInstruction::Stats(StatsInstruction::BoostStats {
                    target: target_position,
                    stat_changes: non_zero_changes.to_hashmap(),
                    previous_boosts: std::collections::HashMap::new(), // Will be filled in by battle state
                }));
            }
        }
    }

    instructions
}

/// Multi-hit move wrapper
///
/// This composer handles multi-hit moves by wrapping the core multi-hit system
/// with common patterns like hit count calculation and effect application.
pub fn multi_hit_move(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    hit_count_calculator: HitCountCalculator,
    generation: &GenerationMechanics,
) -> Vec<BattleInstruction> {
    let mut instructions = Vec::new();

    for &target_position in target_positions {
        let context = DamageCalculationContext::new(
            move_data,
            user_position,
            target_position,
            generation.clone(),
            false, // Use deterministic damage for composer moves - TODO: make this configurable
        );

        let hit_instructions = execute_multi_hit_sequence(
            state,
            context,
            match hit_count_calculator {
                HitCountCalculator::Fixed(count) => count,
                HitCountCalculator::Random { min, max } => {
                    use rand::Rng;
                    let mut rng = rand::thread_rng();
                    rng.gen_range(min..=max)
                }
                HitCountCalculator::Custom(calculator) => {
                    calculator(state, move_data, user_position)
                }
            },
            None, // No hit-specific modifiers
        );

        instructions.extend(hit_instructions);
    }

    instructions
}

/// Priority-dependent power move (like Bolt Beak, Fishious Rend)
///
/// These moves have increased power when the user moves first.
pub fn priority_dependent_power_move(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    power_multiplier: f32,
    generation: &GenerationMechanics,
) -> Vec<BattleInstruction> {
    // Check if user moved first this turn by comparing speeds and priorities
    let moved_first = check_moved_first(state, user_position, target_positions);
    
    let modifiers = DamageModifiers {
        power_multiplier: if moved_first { Some(power_multiplier) } else { None },
        ..Default::default()
    };

    simple_damage_move(
        state,
        move_data,
        user_position,
        target_positions,
        modifiers,
        generation,
    )
}

/// Condition-dependent power move (like Facade, Hex)
///
/// These moves have increased power when certain conditions are met.
pub fn condition_dependent_power_move(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    condition_check: Box<dyn Fn(&BattleState, BattlePosition, BattlePosition) -> bool>,
    power_multiplier: f32,
    generation: &GenerationMechanics,
) -> Vec<BattleInstruction> {
    let mut instructions = Vec::new();

    for &target_position in target_positions {
        let condition_met = condition_check(state, user_position, target_position);
        
        let modifiers = DamageModifiers {
            power_multiplier: if condition_met { Some(power_multiplier) } else { None },
            ..Default::default()
        };

        let move_instructions = simple_damage_move(
            state,
            move_data,
            user_position,
            &[target_position],
            modifiers,
            generation,
        );
        instructions.extend(move_instructions);
    }

    instructions
}

/// Stat-substitution move (like Body Press, Foul Play)
///
/// These moves use different stats for damage calculation.
pub fn stat_substitution_move(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    attack_stat: Stat,
    defense_stat: Option<Stat>,
    use_target_stats: bool,
    generation: &GenerationMechanics,
) -> Vec<BattleInstruction> {
    let mut instructions = Vec::new();

    for &target_position in target_positions {
        // Create damage context with stat substitutions
        let mut context = DamageCalculationContext::new(
            move_data,
            user_position,
            target_position,
            generation.clone(),
            false, // Use deterministic damage for composed moves
        );

        // Apply stat substitutions
        context = context.with_attack_stat_substitution(attack_stat);
        if let Some(def_stat) = defense_stat {
            context = context.with_defense_stat_substitution(def_stat);
        }
        if use_target_stats {
            context = context.with_target_stats();
        }

        // Calculate and apply damage
        let damage_result = calculate_damage_with_effects(state, context);
        let damage_dealt = damage_result.damage;

        if damage_dealt > 0 {
            instructions.push(BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: target_position,
                amount: damage_dealt,
                previous_hp: None, // Will be filled in by battle state
            }));

            // Apply contact effects if the move makes contact
            if move_data.flags.contains_key("contact") {
                let contact_effects = super::super::core::contact_effects::apply_contact_effects(
                    state,
                    move_data,
                    user_position,
                    target_position,
                    damage_dealt,
                );
                instructions.extend(contact_effects);
            }
        }
    }

    instructions
}

/// Recoil move (like Double-Edge, Flare Blitz)
pub fn recoil_move(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    recoil_fraction: f32,
    generation: &GenerationMechanics,
) -> Vec<BattleInstruction> {
    let modifiers = DamageModifiers {
        recoil_fraction: Some(recoil_fraction),
        ..Default::default()
    };

    simple_damage_move(
        state,
        move_data,
        user_position,
        target_positions,
        modifiers,
        generation,
    )
}

/// Draining move (like Absorb, Giga Drain)
pub fn draining_move(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    drain_fraction: f32,
    generation: &GenerationMechanics,
) -> Vec<BattleInstruction> {
    let modifiers = DamageModifiers {
        drain_fraction: Some(drain_fraction),
        ..Default::default()
    };

    simple_damage_move(
        state,
        move_data,
        user_position,
        target_positions,
        modifiers,
        generation,
    )
}

/// Move with secondary status effect (like Thunderbolt, Ice Beam)
pub fn damage_move_with_secondary_status(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    status_applications: Vec<StatusApplication>,
    generation: &GenerationMechanics,
) -> Vec<BattleInstruction> {
    let modifiers = DamageModifiers {
        secondary_effects: status_applications,
        ..Default::default()
    };

    simple_damage_move(
        state,
        move_data,
        user_position,
        target_positions,
        modifiers,
        generation,
    )
}

/// Move that always crits (like Frost Breath, Storm Throw)
pub fn always_crit_move(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstruction> {
    let modifiers = DamageModifiers {
        force_critical: true,
        ..Default::default()
    };

    simple_damage_move(
        state,
        move_data,
        user_position,
        target_positions,
        modifiers,
        generation,
    )
}

/// Move with dynamic category (like Photon Geyser)
///
/// These moves determine their category (Physical/Special) based on the user's stats.
pub fn dynamic_category_move(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    category_determiner: Box<dyn Fn(&BattleState, BattlePosition) -> crate::core::instructions::pokemon::MoveCategory>,
    generation: &GenerationMechanics,
) -> Vec<BattleInstruction> {
    // Determine the actual category to use
    let actual_category = category_determiner(state, user_position);
    
    // Create modified move data with the determined category
    let modified_move_data = MoveData {
        category: actual_category,
        ..move_data.clone()
    };
    
    // Use simple damage move with the modified category
    simple_damage_move(
        state,
        &modified_move_data,
        user_position,
        target_positions,
        DamageModifiers::default(),
        generation,
    )
}

/// Apply multiple status effects with substitute protection
/// 
/// This function checks if the target has a substitute and blocks effects accordingly.
/// Unlike the regular apply_multiple_status_effects, this version considers whether
/// the damage hit a substitute to determine if secondary effects should be blocked.
pub fn apply_multiple_status_effects_with_substitute_protection(
    state: &BattleState,
    applications: Vec<StatusApplication>,
    target_position: BattlePosition,
) -> Vec<BattleInstruction> {
    let mut instructions = Vec::new();
    
    // Check if target has substitute
    if let Some(target_pokemon) = state.get_pokemon_at_position(target_position) {
        let has_substitute = target_pokemon.volatile_statuses.contains(VS::Substitute) 
            && target_pokemon.substitute_health > 0;
        
        if has_substitute {
            // If substitute is present, block secondary effects
            // (In real implementation, we would check if damage actually hit the substitute
            // but for now we assume any damage move with substitute present hits substitute)
            return instructions; // Return empty - substitute blocks secondary effects
        }
    }
    
    // No substitute protection, apply effects normally
    apply_multiple_status_effects(state, applications)
}

/// Compose a damage move with volatile status secondary effects
pub fn damage_move_with_secondary_volatile_status(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    volatile_status_applications: Vec<VolatileStatusApplication>,
    generation: &GenerationMechanics,
) -> Vec<BattleInstruction> {
    let mut instructions = Vec::new();

    for &target_position in target_positions {
        // Create damage context
        let mut context = DamageCalculationContext::new(
            move_data,
            user_position,
            target_position,
            generation.clone(),
            false, // Use deterministic damage for composed moves
        );

        // Calculate and apply damage
        let damage_result = calculate_damage_with_effects(state, context);
        let damage_dealt = damage_result.damage;

        if damage_dealt > 0 {
            instructions.push(BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: target_position,
                amount: damage_dealt,
                previous_hp: None, // Will be filled in by battle state
            }));

            // Apply contact effects if the move makes contact
            if move_data.flags.contains_key("contact") {
                let contact_effects = super::super::core::contact_effects::apply_contact_effects(
                    state,
                    move_data,
                    user_position,
                    target_position,
                    damage_dealt,
                );
                instructions.extend(contact_effects);
            }
        }

        // Apply volatile status effects using the centralized status system
        let target_applications: Vec<VolatileStatusApplication> = volatile_status_applications
            .iter()
            .filter(|app| app.target == target_position)
            .cloned()
            .collect();

        if !target_applications.is_empty() {
            let volatile_status_instructions = super::super::core::status_system::apply_multiple_volatile_status_effects(
                state,
                target_applications,
            );
            instructions.extend(volatile_status_instructions);
        }
    }

    instructions
}