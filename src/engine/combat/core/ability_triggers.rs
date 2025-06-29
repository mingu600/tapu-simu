//! Ability Trigger System
//!
//! This module handles ability triggers at different points in the battle,
//! including end-of-turn, switch-in, damage taken, etc.

use crate::core::battle_format::BattlePosition;
use crate::core::battle_state::{BattleState, Pokemon};
use crate::core::instructions::{
    BattleInstruction, BattleInstructions, PokemonInstruction, StatusInstruction,
    StatsInstruction, PokemonStatus, VolatileStatus, Stat
};
use crate::types::StatBoostArray;
use std::collections::HashMap;

/// Types of ability triggers
#[derive(Debug, Clone, PartialEq)]
pub enum AbilityTriggerType {
    EndOfTurn,
    SwitchIn,
    DamageTaken,
    StatusInflicted,
    WeatherChange,
    BeforeMove,
    AfterMove,
}

/// Result of an ability trigger
#[derive(Debug, Clone)]
pub struct AbilityTriggerResult {
    pub instructions: Vec<BattleInstruction>,
    pub prevents_other_abilities: bool,
    pub blocks_effect: bool,
}

impl Default for AbilityTriggerResult {
    fn default() -> Self {
        Self {
            instructions: Vec::new(),
            prevents_other_abilities: false,
            blocks_effect: false,
        }
    }
}

/// Trigger end-of-turn abilities for all Pokemon
pub fn trigger_end_of_turn_abilities(
    battle_state: &BattleState,
) -> Vec<BattleInstruction> {
    let mut instructions = Vec::new();
    
    // Process abilities in speed order (fastest first)
    let mut positions_with_speeds = Vec::new();
    for position in battle_state.get_all_active_positions() {
        if let Some(pokemon) = battle_state.get_pokemon_at_position(position) {
            let speed = pokemon.get_effective_speed(battle_state, position);
            positions_with_speeds.push((position, speed));
        }
    }
    
    // Sort by speed (fastest first)
    positions_with_speeds.sort_by(|a, b| b.1.cmp(&a.1));
    
    for (position, _) in positions_with_speeds {
        if let Some(pokemon) = battle_state.get_pokemon_at_position(position) {
            if let Some(ability_result) = trigger_end_of_turn_ability(pokemon, position, battle_state) {
                instructions.extend(ability_result.instructions);
            }
        }
    }
    
    instructions
}

/// Trigger a specific Pokemon's end-of-turn ability
fn trigger_end_of_turn_ability(
    pokemon: &Pokemon,
    position: BattlePosition,
    battle_state: &BattleState,
) -> Option<AbilityTriggerResult> {
    if pokemon.ability_suppressed || pokemon.ability_triggered_this_turn {
        return None;
    }
    
    match pokemon.ability.as_str() {
        "speedboost" => Some(trigger_speed_boost(position)),
        "moody" => Some(trigger_moody(position, battle_state)),
        "shedskin" => Some(trigger_shed_skin(pokemon, position)),
        "dryskin" => Some(trigger_dry_skin(pokemon, position, battle_state)),
        "raindish" => Some(trigger_rain_dish(pokemon, position, battle_state)),
        "icebody" => Some(trigger_ice_body(pokemon, position, battle_state)),
        "solarpower" => Some(trigger_solar_power(pokemon, position, battle_state)),
        "poisonheal" => Some(trigger_poison_heal(pokemon, position)),
        "magicguard" => None, // Magic Guard is passive, handled in damage prevention
        "naturalcure" => None, // Natural Cure triggers on switch-out, not end-of-turn
        "regenerator" => None, // Regenerator triggers on switch-out, not end-of-turn
        _ => None,
    }
}

/// Speed Boost - Increases Speed by 1 stage at the end of each turn
fn trigger_speed_boost(position: BattlePosition) -> AbilityTriggerResult {
    AbilityTriggerResult {
        instructions: vec![
            BattleInstruction::Stats(StatsInstruction::BoostStats {
                target: position,
                stat_changes: {
                    let mut changes = StatBoostArray::default();
                    changes.insert(Stat::Speed, 1);
                    changes.to_hashmap()
                },
                previous_boosts: std::collections::HashMap::new(),
            })
        ],
        prevents_other_abilities: false,
        blocks_effect: false,
    }
}

/// Moody - Randomly increases one stat by 2 stages and decreases another by 1 stage
fn trigger_moody(position: BattlePosition, battle_state: &BattleState) -> AbilityTriggerResult {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    
    // All stats that can be modified by Moody
    let all_stats = [
        Stat::Attack,
        Stat::Defense,
        Stat::SpecialAttack,
        Stat::SpecialDefense,
        Stat::Speed,
    ];
    
    // Pick a random stat to boost by +2
    let boost_stat = all_stats[rng.gen_range(0..all_stats.len())];
    
    // Pick a different random stat to lower by -1
    let mut lower_stat;
    loop {
        lower_stat = all_stats[rng.gen_range(0..all_stats.len())];
        if lower_stat != boost_stat {
            break;
        }
    }
    
    let mut changes = StatBoostArray::default();
    changes.insert(boost_stat, 2);
    changes.insert(lower_stat, -1);
    
    let previous_boosts = if let Some(pokemon) = battle_state.get_pokemon_at_position(position) {
        pokemon.stat_boosts.to_hashmap()
    } else {
        std::collections::HashMap::new()
    };
    
    AbilityTriggerResult {
        instructions: vec![
            BattleInstruction::Stats(StatsInstruction::BoostStats {
                target: position,
                stat_changes: changes.to_hashmap(),
                previous_boosts,
            })
        ],
        prevents_other_abilities: false,
        blocks_effect: false,
    }
}

/// Shed Skin - 30% chance to cure status condition at the end of each turn
fn trigger_shed_skin(pokemon: &Pokemon, position: BattlePosition) -> AbilityTriggerResult {
    if pokemon.status == PokemonStatus::None {
        return AbilityTriggerResult::default();
    }
    
    // 30% chance to cure status condition
    use rand::Rng;
    let mut rng = rand::thread_rng();
    if rng.gen_range(0.0..100.0) < 30.0 {
        AbilityTriggerResult {
            instructions: vec![
                BattleInstruction::Status(StatusInstruction::Remove {
                    target: position,
                    status: pokemon.status,
                    previous_duration: pokemon.status_duration,
                })
            ],
            prevents_other_abilities: false,
            blocks_effect: false,
        }
    } else {
        AbilityTriggerResult::default()
    }
}

/// Dry Skin - Heals 1/8 HP in rain, loses 1/8 HP in sun
fn trigger_dry_skin(
    pokemon: &Pokemon,
    position: BattlePosition,
    battle_state: &BattleState,
) -> AbilityTriggerResult {
    use crate::core::instructions::Weather;
    
    match battle_state.weather() {
        Weather::Rain => {
            if pokemon.hp < pokemon.max_hp {
                AbilityTriggerResult {
                    instructions: vec![
                        BattleInstruction::Pokemon(PokemonInstruction::Heal {
                            target: position,
                            amount: (pokemon.max_hp / 8).max(1),
                            previous_hp: Some(pokemon.hp),
                        })
                    ],
                    prevents_other_abilities: false,
                    blocks_effect: false,
                }
            } else {
                AbilityTriggerResult::default()
            }
        }
        Weather::Sun => {
            AbilityTriggerResult {
                instructions: vec![
                    BattleInstruction::Pokemon(PokemonInstruction::Damage {
                        target: position,
                        amount: (pokemon.max_hp / 8).max(1),
                        previous_hp: Some(pokemon.hp),
                    })
                ],
                prevents_other_abilities: false,
                blocks_effect: false,
            }
        }
        _ => AbilityTriggerResult::default(),
    }
}

/// Rain Dish - Heals 1/16 HP in rain
fn trigger_rain_dish(
    pokemon: &Pokemon,
    position: BattlePosition,
    battle_state: &BattleState,
) -> AbilityTriggerResult {
    use crate::core::instructions::Weather;
    
    if battle_state.weather() == Weather::Rain && pokemon.hp < pokemon.max_hp {
        AbilityTriggerResult {
            instructions: vec![
                BattleInstruction::Pokemon(PokemonInstruction::Heal {
                    target: position,
                    amount: (pokemon.max_hp / 16).max(1),
                    previous_hp: Some(pokemon.hp),
                })
            ],
            prevents_other_abilities: false,
            blocks_effect: false,
        }
    } else {
        AbilityTriggerResult::default()
    }
}

/// Ice Body - Heals 1/16 HP in hail
fn trigger_ice_body(
    pokemon: &Pokemon,
    position: BattlePosition,
    battle_state: &BattleState,
) -> AbilityTriggerResult {
    use crate::core::instructions::Weather;
    
    if battle_state.weather() == Weather::Hail && pokemon.hp < pokemon.max_hp {
        AbilityTriggerResult {
            instructions: vec![
                BattleInstruction::Pokemon(PokemonInstruction::Heal {
                    target: position,
                    amount: (pokemon.max_hp / 16).max(1),
                    previous_hp: Some(pokemon.hp),
                })
            ],
            prevents_other_abilities: false,
            blocks_effect: false,
        }
    } else {
        AbilityTriggerResult::default()
    }
}

/// Solar Power - Loses 1/8 HP in sun (gain Special Attack boost is handled in damage calculation)
fn trigger_solar_power(
    pokemon: &Pokemon,
    position: BattlePosition,
    battle_state: &BattleState,
) -> AbilityTriggerResult {
    use crate::core::instructions::Weather;
    
    if battle_state.weather() == Weather::Sun {
        AbilityTriggerResult {
            instructions: vec![
                BattleInstruction::Pokemon(PokemonInstruction::Damage {
                    target: position,
                    amount: (pokemon.max_hp / 8).max(1),
                    previous_hp: Some(pokemon.hp),
                })
            ],
            prevents_other_abilities: false,
            blocks_effect: false,
        }
    } else {
        AbilityTriggerResult::default()
    }
}

/// Poison Heal - Heals 1/8 HP when poisoned instead of taking damage
fn trigger_poison_heal(pokemon: &Pokemon, position: BattlePosition) -> AbilityTriggerResult {
    match pokemon.status {
        PokemonStatus::Poison | PokemonStatus::BadlyPoisoned => {
            if pokemon.hp < pokemon.max_hp {
                AbilityTriggerResult {
                    instructions: vec![
                        BattleInstruction::Pokemon(PokemonInstruction::Heal {
                            target: position,
                            amount: (pokemon.max_hp / 8).max(1),
                            previous_hp: Some(pokemon.hp),
                        })
                    ],
                    prevents_other_abilities: false,
                    blocks_effect: true, // Prevents poison damage
                }
            } else {
                AbilityTriggerResult {
                    instructions: Vec::new(),
                    prevents_other_abilities: false,
                    blocks_effect: true, // Still blocks poison damage even if at full HP
                }
            }
        }
        _ => AbilityTriggerResult::default(),
    }
}

/// Trigger switch-in abilities (for when Pokemon enter the battle)
pub fn trigger_switch_in_abilities(
    pokemon: &Pokemon,
    position: BattlePosition,
    battle_state: &BattleState,
) -> Vec<BattleInstruction> {
    if pokemon.ability_suppressed {
        return Vec::new();
    }
    
    match pokemon.ability.as_str() {
        "intimidate" => trigger_intimidate(position, battle_state),
        "drought" => trigger_drought(),
        "drizzle" => trigger_drizzle(),
        "sandstream" => trigger_sand_stream(),
        "snowwarning" => trigger_snow_warning(),
        "trace" => trigger_trace(position, battle_state),
        "download" => trigger_download(position, battle_state),
        _ => Vec::new(),
    }
}

/// Intimidate - Lowers opponent's Attack by 1 stage
fn trigger_intimidate(
    _user_position: BattlePosition,
    battle_state: &BattleState,
) -> Vec<BattleInstruction> {
    let mut instructions = Vec::new();
    
    // TODO: Implement proper opponent targeting for all battle formats
    // For now, simplified for singles
    for position in battle_state.get_all_active_positions() {
        if let Some(pokemon) = battle_state.get_pokemon_at_position(position) {
            // Check if immune to Intimidate
            if !is_immune_to_intimidate(pokemon) {
                instructions.push(BattleInstruction::Stats(StatsInstruction::BoostStats {
                    target: position,
                    stat_changes: {
                        let mut changes = StatBoostArray::default();
                        changes.insert(Stat::Attack, -1);
                        changes.to_hashmap()
                    },
                    previous_boosts: std::collections::HashMap::new(),
                }));
            }
        }
    }
    
    instructions
}

/// Check if Pokemon is immune to Intimidate
fn is_immune_to_intimidate(pokemon: &Pokemon) -> bool {
    match pokemon.ability.as_str() {
        "clearbody" | "whitesmoke" | "hypercutter" | "fullmetalbody" => true,
        _ => false,
    }
}

/// Weather-setting abilities
fn trigger_drought() -> Vec<BattleInstruction> {
    vec![
        BattleInstruction::Field(crate::core::instructions::FieldInstruction::Weather {
            new_weather: crate::core::instructions::Weather::Sun,
            turns: Some(5), // 5 turns in newer generations
            source: None,
            previous_weather: crate::core::instructions::Weather::None,
            previous_turns: None,
        })
    ]
}

fn trigger_drizzle() -> Vec<BattleInstruction> {
    vec![
        BattleInstruction::Field(crate::core::instructions::FieldInstruction::Weather {
            new_weather: crate::core::instructions::Weather::Rain,
            turns: Some(5),
            source: None,
            previous_weather: crate::core::instructions::Weather::None,
            previous_turns: None,
        })
    ]
}

fn trigger_sand_stream() -> Vec<BattleInstruction> {
    vec![
        BattleInstruction::Field(crate::core::instructions::FieldInstruction::Weather {
            new_weather: crate::core::instructions::Weather::Sandstorm,
            turns: Some(5),
            source: None,
            previous_weather: crate::core::instructions::Weather::None,
            previous_turns: None,
        })
    ]
}

fn trigger_snow_warning() -> Vec<BattleInstruction> {
    vec![
        BattleInstruction::Field(crate::core::instructions::FieldInstruction::Weather {
            new_weather: crate::core::instructions::Weather::Hail,
            turns: Some(5),
            source: None,
            previous_weather: crate::core::instructions::Weather::None,
            previous_turns: None,
        })
    ]
}

/// Trace - Copies opponent's ability
fn trigger_trace(position: BattlePosition, battle_state: &BattleState) -> Vec<BattleInstruction> {
    // Find the first opponent Pokemon with a copyable ability
    for opponent_position in battle_state.get_all_active_positions() {
        // Skip if it's the same side
        if opponent_position.side == position.side {
            continue;
        }
        
        if let Some(opponent) = battle_state.get_pokemon_at_position(opponent_position) {
            // Don't copy certain abilities that can't be traced
            match opponent.ability.as_str() {
                "trace" | "multitype" | "flowergift" | "forecast" | "illusion" | 
                "imposter" | "neutralizinggas" | "wonderguard" | "powerofalchemy" |
                "receiver" | "disguise" | "rkssystem" | "comatose" | "schooling" |
                "shieldsdown" | "battlebond" | "powerconstruct" | "corrosion" |
                "stancechange" => {
                    continue; // Skip uncopyable abilities
                }
                _ => {
                    // Copy this ability
                    return vec![BattleInstruction::Pokemon(
                        crate::core::instructions::PokemonInstruction::ChangeAbility {
                            target: position,
                            new_ability: opponent.ability,
                            previous_ability: battle_state.get_pokemon_at_position(position)
                                .map(|p| p.ability),
                        }
                    )];
                }
            }
        }
    }
    
    // No copyable ability found
    Vec::new()
}

/// Download - Boosts Attack or Special Attack based on opponent's defenses
fn trigger_download(position: BattlePosition, battle_state: &BattleState) -> Vec<BattleInstruction> {
    let mut total_defense = 0i32;
    let mut total_special_defense = 0i32;
    let mut opponent_count = 0;
    
    // Calculate total defenses of all opponents
    for opponent_position in battle_state.get_all_active_positions() {
        // Skip if it's the same side
        if opponent_position.side == position.side {
            continue;
        }
        
        if let Some(opponent) = battle_state.get_pokemon_at_position(opponent_position) {
            let defense = opponent.get_effective_stat(Stat::Defense);
            let special_defense = opponent.get_effective_stat(Stat::SpecialDefense);
            
            total_defense += defense as i32;
            total_special_defense += special_defense as i32;
            opponent_count += 1;
        }
    }
    
    if opponent_count == 0 {
        return Vec::new();
    }
    
    // Compare average defenses
    let avg_defense = total_defense / opponent_count;
    let avg_special_defense = total_special_defense / opponent_count;
    
    let previous_boosts = if let Some(pokemon) = battle_state.get_pokemon_at_position(position) {
        pokemon.stat_boosts.to_hashmap()
    } else {
        std::collections::HashMap::new()
    };
    
    if avg_defense < avg_special_defense {
        // Boost Attack if Defense is lower
        let mut changes = StatBoostArray::default();
        changes.insert(Stat::Attack, 1);
        
        vec![BattleInstruction::Stats(StatsInstruction::BoostStats {
            target: position,
            stat_changes: changes.to_hashmap(),
            previous_boosts,
        })]
    } else {
        // Boost Special Attack if Special Defense is lower or equal
        let mut changes = StatBoostArray::default();
        changes.insert(Stat::SpecialAttack, 1);
        
        vec![BattleInstruction::Stats(StatsInstruction::BoostStats {
            target: position,
            stat_changes: changes.to_hashmap(),
            previous_boosts,
        })]
    }
}