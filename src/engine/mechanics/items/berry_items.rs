//! # Berry Items
//!
//! All berry implementations including damage reduction, healing, and stat boost berries.
//! Berries typically activate under specific conditions and are consumed after use.

use super::{ItemModifier, StatBoosts};
use crate::engine::combat::damage_context::DamageContext;
use crate::generation::{GenerationBattleMechanics, Generation};
use crate::engine::combat::type_effectiveness::TypeChart;
use crate::types::{PokemonType, StatBoostArray};
use crate::core::battle_state::{MoveCategory, Pokemon};
use crate::core::battle_format::BattlePosition;
use crate::core::instructions::{Stat, PokemonStatus};
use crate::core::instructions::{BattleInstruction, BattleInstructions, StatusInstruction, PokemonInstruction, StatsInstruction};
use crate::types::{Items, Moves};
use std::collections::HashMap;

/// Get berry item effect if the item is a berry
pub fn get_berry_item_effect(
    item_id: &Items,
    generation: &dyn GenerationBattleMechanics,
    attacker: &Pokemon,
    defender: Option<&Pokemon>,
    move_id: &Moves,
    move_type_id: &PokemonType,
    move_category: MoveCategory,
    context: &DamageContext,
) -> Option<ItemModifier> {
    match item_id {
        // Damage Reduction Berries (18 items)
        Items::CHOPLEBERRY => Some(damage_reduction_berry_effect(PokemonType::Fighting, *move_type_id, context)),
        Items::COBABERRY => Some(damage_reduction_berry_effect(PokemonType::Flying, *move_type_id, context)),
        Items::KEBIABERRY => Some(damage_reduction_berry_effect(PokemonType::Poison, *move_type_id, context)),
        Items::SHUCABERRY => Some(damage_reduction_berry_effect(PokemonType::Ground, *move_type_id, context)),
        Items::CHARTIBERRY => Some(damage_reduction_berry_effect(PokemonType::Rock, *move_type_id, context)),
        Items::TANGABERRY => Some(damage_reduction_berry_effect(PokemonType::Bug, *move_type_id, context)),
        Items::KASIBBERRY => Some(damage_reduction_berry_effect(PokemonType::Ghost, *move_type_id, context)),
        Items::BABIRIBERRY => Some(damage_reduction_berry_effect(PokemonType::Steel, *move_type_id, context)),
        Items::OCCABERRY => Some(damage_reduction_berry_effect(PokemonType::Fire, *move_type_id, context)),
        Items::PASSHOBERRY => Some(damage_reduction_berry_effect(PokemonType::Water, *move_type_id, context)),
        Items::RINDOBERRY => Some(damage_reduction_berry_effect(PokemonType::Grass, *move_type_id, context)),
        Items::WACANBERRY => Some(damage_reduction_berry_effect(PokemonType::Electric, *move_type_id, context)),
        Items::PAYAPABERRY => Some(damage_reduction_berry_effect(PokemonType::Psychic, *move_type_id, context)),
        Items::YACHEBERRY => Some(damage_reduction_berry_effect(PokemonType::Ice, *move_type_id, context)),
        Items::HABANBERRY => Some(damage_reduction_berry_effect(PokemonType::Dragon, *move_type_id, context)),
        Items::COLBURBERRY => Some(damage_reduction_berry_effect(PokemonType::Dark, *move_type_id, context)),
        Items::ROSELIBERRY => Some(damage_reduction_berry_effect(PokemonType::Fairy, *move_type_id, context)),
        Items::CHILANBERRY => Some(chilan_berry_effect(*move_type_id)), // Special case for Normal
        
        // Healing/Status Berries (5 items)
        Items::LUMBERRY => Some(lum_berry_effect(defender)),
        Items::SITRUSBERRY => Some(sitrus_berry_effect(defender, generation)),
        Items::CHESTOBERRY => Some(chesto_berry_effect(defender)),
        Items::MIRACLEBERRY => Some(miracle_berry_effect(defender, generation)),
        Items::MINTBERRY => Some(mint_berry_effect(defender, generation)),
        
        // Stat Boost Berries (4 items)
        Items::LIECHIBERRY => Some(liechi_berry_effect(defender)),
        Items::PETAYABERRY => Some(petaya_berry_effect(defender)),
        Items::SALACBERRY => Some(salac_berry_effect(defender)),
        Items::CUSTAPBERRY => Some(custap_berry_effect(attacker)),
        
        _ => None,
    }
}

// =============================================================================
// DAMAGE REDUCTION BERRIES (18 items)
// =============================================================================

/// Standard damage reduction berry that halves super effective damage
fn damage_reduction_berry_effect(
    resisted_type: PokemonType,
    move_type: PokemonType,
    context: &DamageContext,
) -> ItemModifier {
    // Check if this move is the resisted type
    if move_type != resisted_type {
        return ItemModifier::default();
    }
    
    // Check if move is super effective
    let type_chart = TypeChart::get_cached(9); // Gen 9 type chart
    let attacking_type = move_type;
    let defender_type1 = context.defender.pokemon.types.get(0).copied().unwrap_or(PokemonType::Normal);
    let defender_type2 = context.defender.pokemon.types.get(1).copied().unwrap_or(defender_type1);
    let type_effectiveness = type_chart.calculate_damage_multiplier(
        attacking_type,
        (defender_type1, defender_type2),
        None, // No tera type
        None, // No special move name
    );
    if type_effectiveness > 1.0 {
        ItemModifier::new()
            .with_damage_multiplier(0.5)
            .with_consumed()
    } else {
        ItemModifier::default()
    }
}

/// Chilan Berry - Special case that reduces Normal-type damage regardless of effectiveness
fn chilan_berry_effect(move_type: PokemonType) -> ItemModifier {
    if move_type == PokemonType::Normal {
        ItemModifier::new()
            .with_damage_multiplier(0.5)
            .with_consumed()
    } else {
        ItemModifier::default()
    }
}

// =============================================================================
// HEALING/STATUS BERRIES (5 items)
// =============================================================================

/// Lum Berry - Cures all status conditions
fn lum_berry_effect(defender: Option<&Pokemon>) -> ItemModifier {
    if let Some(pokemon) = defender {
        if pokemon.status != PokemonStatus::None {
            ItemModifier::new().with_consumed()
        } else {
            ItemModifier::default()
        }
    } else {
        ItemModifier::default()
    }
}

/// Sitrus Berry - Heals HP when below threshold (generation-dependent)
fn sitrus_berry_effect(defender: Option<&Pokemon>, generation: &dyn GenerationBattleMechanics) -> ItemModifier {
    if let Some(pokemon) = defender {
        // Threshold based on generation: Gen 3 = 50%, Gen 4+ = 25%
        let threshold = match generation.generation() {
            Generation::Gen3 => 0.5,
            _ => 0.25,
        };
        
        let hp_percentage = pokemon.hp as f32 / pokemon.max_hp as f32;
        if hp_percentage <= threshold {
            ItemModifier::new().with_consumed()
        } else {
            ItemModifier::default()
        }
    } else {
        ItemModifier::default()
    }
}

/// Chesto Berry - Cures sleep status
fn chesto_berry_effect(defender: Option<&Pokemon>) -> ItemModifier {
    if let Some(pokemon) = defender {
        if pokemon.status == PokemonStatus::Sleep {
            ItemModifier::new().with_consumed()
        } else {
            ItemModifier::default()
        }
    } else {
        ItemModifier::default()
    }
}

/// Miracle Berry - Gen 2 exclusive, cures all status conditions
fn miracle_berry_effect(defender: Option<&Pokemon>, generation: &dyn GenerationBattleMechanics) -> ItemModifier {
    // Only available in Gen 2
    if generation.generation() != Generation::Gen2 {
        return ItemModifier::default();
    }
    
    if let Some(pokemon) = defender {
        if pokemon.status != PokemonStatus::None {
            ItemModifier::new().with_consumed()
        } else {
            ItemModifier::default()
        }
    } else {
        ItemModifier::default()
    }
}

/// Mint Berry - Gen 2 exclusive, cures sleep status
fn mint_berry_effect(defender: Option<&Pokemon>, generation: &dyn GenerationBattleMechanics) -> ItemModifier {
    // Only available in Gen 2
    if generation.generation() != Generation::Gen2 {
        return ItemModifier::default();
    }
    
    if let Some(pokemon) = defender {
        if pokemon.status == PokemonStatus::Sleep {
            ItemModifier::new().with_consumed()
        } else {
            ItemModifier::default()
        }
    } else {
        ItemModifier::default()
    }
}

// =============================================================================
// STAT BOOST BERRIES (4 items)
// =============================================================================

/// Liechi Berry - +1 Attack when HP ≤ 25%
fn liechi_berry_effect(pokemon: Option<&Pokemon>) -> ItemModifier {
    if let Some(poke) = pokemon {
        let hp_percentage = poke.hp as f32 / poke.max_hp as f32;
        if hp_percentage <= 0.25 {
            ItemModifier::new()
                .with_stat_boosts(StatBoosts::attack(1))
                .with_consumed()
        } else {
            ItemModifier::default()
        }
    } else {
        ItemModifier::default()
    }
}

/// Petaya Berry - +1 Special Attack when HP ≤ 25%
fn petaya_berry_effect(pokemon: Option<&Pokemon>) -> ItemModifier {
    if let Some(poke) = pokemon {
        let hp_percentage = poke.hp as f32 / poke.max_hp as f32;
        if hp_percentage <= 0.25 {
            ItemModifier::new()
                .with_stat_boosts(StatBoosts::special_attack(1))
                .with_consumed()
        } else {
            ItemModifier::default()
        }
    } else {
        ItemModifier::default()
    }
}

/// Salac Berry - +1 Speed when HP ≤ 25%
fn salac_berry_effect(pokemon: Option<&Pokemon>) -> ItemModifier {
    if let Some(poke) = pokemon {
        let hp_percentage = poke.hp as f32 / poke.max_hp as f32;
        if hp_percentage <= 0.25 {
            ItemModifier::new()
                .with_stat_boosts(StatBoosts {
                    attack: 0,
                    defense: 0,
                    special_attack: 0,
                    special_defense: 0,
                    speed: 1,
                    accuracy: 0,
                })
                .with_consumed()
        } else {
            ItemModifier::default()
        }
    } else {
        ItemModifier::default()
    }
}

/// Custap Berry - Provides +1 priority when HP ≤ 25%
fn custap_berry_effect(pokemon: &Pokemon) -> ItemModifier {
    let hp_percentage = pokemon.hp as f32 / pokemon.max_hp as f32;
    if hp_percentage <= 0.25 {
        ItemModifier::new()
            .with_priority_modifier(1)
            .with_consumed()
    } else {
        ItemModifier::default()
    }
}

/// Generate berry activation instructions for reactive berries
pub fn generate_berry_activation_instructions(
    item_id: &Items,
    pokemon: &Pokemon,
    position: BattlePosition,
    generation: &dyn GenerationBattleMechanics,
) -> Option<BattleInstructions> {
    match item_id {
        Items::LUMBERRY => {
            if pokemon.status != PokemonStatus::None {
                let instructions = vec![
                    BattleInstruction::Status(StatusInstruction::Remove {
                        target: position,
                        status: pokemon.status,
                        previous_duration: None,
                    }),
                    BattleInstruction::Pokemon(PokemonInstruction::ChangeItem {
                        target: position,
                        new_item: None,
                        previous_item: Some(crate::types::Items::LUMBERRY),
                    })
                ];
                Some(BattleInstructions::new(100.0, instructions))
            } else {
                None
            }
        },
        
        Items::SITRUSBERRY => {
            let threshold = match generation.generation() {
                Generation::Gen3 => 0.5,
                _ => 0.25,
            };
            
            let hp_percentage = pokemon.hp as f32 / pokemon.max_hp as f32;
            if hp_percentage <= threshold {
                // Heal 1/4 of max HP
                let heal_amount = pokemon.max_hp / 4;
                
                let instructions = vec![
                    BattleInstruction::Pokemon(PokemonInstruction::Heal {
                        target: position,
                        amount: heal_amount,
                        previous_hp: Some(pokemon.hp),
                    }),
                    BattleInstruction::Pokemon(PokemonInstruction::ChangeItem {
                        target: position,
                        new_item: None,
                        previous_item: Some(crate::types::Items::SITRUSBERRY),
                    })
                ];
                Some(BattleInstructions::new(100.0, instructions))
            } else {
                None
            }
        },
        
        Items::CHESTOBERRY => {
            if pokemon.status == PokemonStatus::Sleep {
                let instructions = vec![
                    BattleInstruction::Status(StatusInstruction::Remove {
                        target: position,
                        status: PokemonStatus::Sleep,
                        previous_duration: None,
                    }),
                    BattleInstruction::Pokemon(PokemonInstruction::ChangeItem {
                        target: position,
                        new_item: None,
                        previous_item: Some(crate::types::Items::CHESTOBERRY),
                    })
                ];
                Some(BattleInstructions::new(100.0, instructions))
            } else {
                None
            }
        },
        
        Items::LIECHIBERRY | Items::PETAYABERRY | Items::SALACBERRY | Items::GANLONBERRY | Items::APICOTBERRY => {
            let hp_percentage = pokemon.hp as f32 / pokemon.max_hp as f32;
            if hp_percentage <= 0.25 {
                let stat = match item_id {
                    Items::LIECHIBERRY => Stat::Attack,
                    Items::PETAYABERRY => Stat::SpecialAttack,
                    Items::SALACBERRY => Stat::Speed,
                    Items::GANLONBERRY => Stat::Defense,
                    Items::APICOTBERRY => Stat::SpecialDefense,
                    _ => return None, // Should not happen, but return None instead of panicking
                };
                
                let mut stat_changes = StatBoostArray::default();
                stat_changes.insert(stat, 1);
                
                let instructions = vec![
                    BattleInstruction::Stats(StatsInstruction::BoostStats {
                        target: position,
                        stat_changes: stat_changes.to_hashmap(),
                        previous_boosts: std::collections::HashMap::new(),
                    }),
                    BattleInstruction::Pokemon(PokemonInstruction::ChangeItem {
                        target: position,
                        new_item: None,
                        previous_item: Some(*item_id),
                    })
                ];
                Some(BattleInstructions::new(100.0, instructions))
            } else {
                None
            }
        },
        
        Items::LANSATBERRY => {
            let hp_percentage = pokemon.hp as f32 / pokemon.max_hp as f32;
            if hp_percentage <= 0.25 {
                // Lansat Berry applies Focus Energy effect (increases critical hit ratio)
                let instructions = vec![
                    BattleInstruction::Status(StatusInstruction::ApplyVolatile {
                        target: position,
                        status: crate::core::instructions::VolatileStatus::FocusEnergy,
                        duration: None,
                        previous_had_status: false,
                        previous_duration: None,
                    }),
                    BattleInstruction::Pokemon(PokemonInstruction::ChangeItem {
                        target: position,
                        new_item: None,
                        previous_item: Some(crate::types::Items::LANSATBERRY),
                    })
                ];
                Some(BattleInstructions::new(100.0, instructions))
            } else {
                None
            }
        },
        
        Items::STARFBERRY => {
            let hp_percentage = pokemon.hp as f32 / pokemon.max_hp as f32;
            if hp_percentage <= 0.25 {
                // Starf Berry boosts a random stat by 2 stages (excluding HP)
                use rand::seq::SliceRandom;
                use rand::thread_rng;
                
                let boostable_stats = vec![
                    Stat::Attack, 
                    Stat::Defense, 
                    Stat::SpecialAttack, 
                    Stat::SpecialDefense, 
                    Stat::Speed, 
                    Stat::Accuracy, 
                    Stat::Evasion
                ];
                
                let mut rng = thread_rng();
                let random_stat = *boostable_stats.choose(&mut rng)
                    .expect("Boostable stats vec should not be empty");
                let mut stat_changes = StatBoostArray::default();
                stat_changes.insert(random_stat, 2);
                
                let instructions = vec![
                    BattleInstruction::Stats(StatsInstruction::BoostStats {
                        target: position,
                        stat_changes: stat_changes.to_hashmap(),
                        previous_boosts: std::collections::HashMap::new(),
                    }),
                    BattleInstruction::Pokemon(PokemonInstruction::ChangeItem {
                        target: position,
                        new_item: None,
                        previous_item: Some(crate::types::Items::STARFBERRY),
                    })
                ];
                Some(BattleInstructions::new(100.0, instructions))
            } else {
                None
            }
        },
        
        Items::MICLEBERRY => {
            let hp_percentage = pokemon.hp as f32 / pokemon.max_hp as f32;
            if hp_percentage <= 0.25 {
                // Micle Berry increases accuracy of next move
                let instructions = vec![
                    BattleInstruction::Status(StatusInstruction::ApplyVolatile {
                        target: position,
                        status: crate::core::instructions::VolatileStatus::MicleBoost,
                        duration: Some(1), // Lasts for one move
                        previous_had_status: false,
                        previous_duration: None,
                    }),
                    BattleInstruction::Pokemon(PokemonInstruction::ChangeItem {
                        target: position,
                        new_item: None,
                        previous_item: Some(crate::types::Items::MICLEBERRY),
                    })
                ];
                Some(BattleInstructions::new(100.0, instructions))
            } else {
                None
            }
        },
        
        Items::CUSTAPBERRY => {
            let hp_percentage = pokemon.hp as f32 / pokemon.max_hp as f32;
            if hp_percentage <= 0.25 {
                // Custap Berry increases priority of next move
                let instructions = vec![
                    BattleInstruction::Status(StatusInstruction::ApplyVolatile {
                        target: position,
                        status: crate::core::instructions::VolatileStatus::CustapBoost,
                        duration: Some(1), // Lasts for one move
                        previous_had_status: false,
                        previous_duration: None,
                    }),
                    BattleInstruction::Pokemon(PokemonInstruction::ChangeItem {
                        target: position,
                        new_item: None,
                        previous_item: Some(crate::types::Items::CUSTAPBERRY),
                    })
                ];
                Some(BattleInstructions::new(100.0, instructions))
            } else {
                None
            }
        },
        
        _ => None,
    }
}

