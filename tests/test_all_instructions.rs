use tapu_simu::battle_format::{BattleFormat, BattlePosition, SideReference};
use tapu_simu::generation::Generation;
use tapu_simu::instruction::*;
use tapu_simu::move_choice::MoveIndex;
use tapu_simu::state::{State, Pokemon, Move, MoveCategory, PokemonStats};
use std::collections::HashMap;

/// Helper function to create a basic battle state for testing
fn create_test_state() -> State {
    let mut state = State::new(BattleFormat::gen9_ou());
    
    // Add Pokemon to both sides
    let mut pokemon1 = Pokemon::new("Pikachu".to_string());
    pokemon1.hp = 100;
    pokemon1.max_hp = 100;
    pokemon1.stats = PokemonStats {
        attack: 120,
        defense: 80,
        special_attack: 100,
        special_defense: 70,
        speed: 110,
    };
    pokemon1.ability = "Static".to_string();
    pokemon1.item = Some("Light Ball".to_string());
    pokemon1.types = vec!["Electric".to_string()];
    
    // Add a move
    let move_data = Move::new_with_details(
        "Thunderbolt".to_string(),
        90,
        100,
        "Electric".to_string(),
        15,
        tapu_simu::data::ps_types::PSMoveTarget::Normal,
        MoveCategory::Special,
        0,
    );
    pokemon1.add_move(MoveIndex::M0, move_data);
    
    let mut pokemon2 = Pokemon::new("Charizard".to_string());
    pokemon2.hp = 120;
    pokemon2.max_hp = 120;
    pokemon2.stats = PokemonStats {
        attack: 104,
        defense: 98,
        special_attack: 129,
        special_defense: 105,
        speed: 120,
    };
    pokemon2.ability = "Blaze".to_string();
    pokemon2.item = Some("Leftovers".to_string());
    pokemon2.types = vec!["Fire".to_string(), "Flying".to_string()];
    
    state.side_one.add_pokemon(pokemon1);
    state.side_one.set_active_pokemon_at_slot(0, Some(0));
    
    state.side_two.add_pokemon(pokemon2);
    state.side_two.set_active_pokemon_at_slot(0, Some(0));
    
    state
}

#[cfg(test)]
mod damage_and_heal_tests {
    use super::*;

    #[test]
    fn test_position_damage_instruction() {
        let mut state = create_test_state();
        let position = BattlePosition::new(SideReference::SideOne, 0);
        let original_hp = state.get_pokemon_at_position(position).unwrap().hp;
        
        let instruction = Instruction::PositionDamage(PositionDamageInstruction {
            target_position: position,
            damage_amount: 30,
        });
        
        state.apply_instruction(&instruction);
        
        let new_hp = state.get_pokemon_at_position(position).unwrap().hp;
        assert_eq!(new_hp, original_hp - 30);
        assert_eq!(new_hp, 70);
    }

    #[test]
    fn test_position_heal_instruction() {
        let mut state = create_test_state();
        let position = BattlePosition::new(SideReference::SideOne, 0);
        
        // First damage the Pokemon
        let damage_instruction = Instruction::PositionDamage(PositionDamageInstruction {
            target_position: position,
            damage_amount: 40,
        });
        state.apply_instruction(&damage_instruction);
        
        let damaged_hp = state.get_pokemon_at_position(position).unwrap().hp;
        assert_eq!(damaged_hp, 60);
        
        // Now heal it
        let heal_instruction = Instruction::PositionHeal(PositionHealInstruction {
            target_position: position,
            heal_amount: 20,
        });
        state.apply_instruction(&heal_instruction);
        
        let healed_hp = state.get_pokemon_at_position(position).unwrap().hp;
        assert_eq!(healed_hp, 80);
    }

    #[test]
    fn test_multi_target_damage_instruction() {
        let mut state = create_test_state();
        let pos1 = BattlePosition::new(SideReference::SideOne, 0);
        let pos2 = BattlePosition::new(SideReference::SideTwo, 0);
        
        let original_hp1 = state.get_pokemon_at_position(pos1).unwrap().hp;
        let original_hp2 = state.get_pokemon_at_position(pos2).unwrap().hp;
        
        let instruction = Instruction::MultiTargetDamage(MultiTargetDamageInstruction {
            target_damages: vec![(pos1, 25), (pos2, 35)],
        });
        
        state.apply_instruction(&instruction);
        
        let new_hp1 = state.get_pokemon_at_position(pos1).unwrap().hp;
        let new_hp2 = state.get_pokemon_at_position(pos2).unwrap().hp;
        
        assert_eq!(new_hp1, original_hp1 - 25);
        assert_eq!(new_hp2, original_hp2 - 35);
        assert_eq!(new_hp1, 75);
        assert_eq!(new_hp2, 85);
    }
}

#[cfg(test)]
mod status_tests {
    use super::*;

    #[test]
    fn test_apply_status_instruction() {
        let mut state = create_test_state();
        let position = BattlePosition::new(SideReference::SideOne, 0);
        
        let instruction = Instruction::ApplyStatus(ApplyStatusInstruction {
            target_position: position,
            status: PokemonStatus::BURN,
        });
        
        state.apply_instruction(&instruction);
        
        let pokemon = state.get_pokemon_at_position(position).unwrap();
        assert_eq!(pokemon.status, PokemonStatus::BURN);
    }

    #[test]
    fn test_remove_status_instruction() {
        let mut state = create_test_state();
        let position = BattlePosition::new(SideReference::SideOne, 0);
        
        // First apply a status
        let apply_instruction = Instruction::ApplyStatus(ApplyStatusInstruction {
            target_position: position,
            status: PokemonStatus::PARALYZE,
        });
        state.apply_instruction(&apply_instruction);
        
        // Verify status was applied
        let pokemon = state.get_pokemon_at_position(position).unwrap();
        assert_eq!(pokemon.status, PokemonStatus::PARALYZE);
        
        // Now remove it
        let remove_instruction = Instruction::RemoveStatus(RemoveStatusInstruction {
            target_position: position,
        });
        state.apply_instruction(&remove_instruction);
        
        // Verify status was removed
        let pokemon = state.get_pokemon_at_position(position).unwrap();
        assert_eq!(pokemon.status, PokemonStatus::NONE);
        assert_eq!(pokemon.status_duration, None);
    }

    #[test]
    fn test_change_status_duration_instruction() {
        let mut state = create_test_state();
        let position = BattlePosition::new(SideReference::SideOne, 0);
        
        // First apply sleep with duration
        let apply_instruction = Instruction::ApplyStatus(ApplyStatusInstruction {
            target_position: position,
            status: PokemonStatus::SLEEP,
        });
        state.apply_instruction(&apply_instruction);
        
        // Check initial duration
        let pokemon = state.get_pokemon_at_position(position).unwrap();
        assert_eq!(pokemon.status, PokemonStatus::SLEEP);
        assert_eq!(pokemon.status_duration, Some(1));
        
        // Change duration
        let change_instruction = Instruction::ChangeStatusDuration(ChangeStatusDurationInstruction {
            target_position: position,
            duration_change: 2,
        });
        state.apply_instruction(&change_instruction);
        
        // Verify duration changed
        let pokemon = state.get_pokemon_at_position(position).unwrap();
        assert_eq!(pokemon.status_duration, Some(3));
        
        // Test duration reduction to zero (should remove status)
        let reduce_instruction = Instruction::ChangeStatusDuration(ChangeStatusDurationInstruction {
            target_position: position,
            duration_change: -3,
        });
        state.apply_instruction(&reduce_instruction);
        
        // Verify status was removed when duration hit zero
        let pokemon = state.get_pokemon_at_position(position).unwrap();
        assert_eq!(pokemon.status, PokemonStatus::NONE);
        assert_eq!(pokemon.status_duration, None);
    }
}

#[cfg(test)]
mod stat_boost_tests {
    use super::*;

    #[test]
    fn test_boost_stats_instruction() {
        let mut state = create_test_state();
        let position = BattlePosition::new(SideReference::SideOne, 0);
        
        let mut stat_boosts = HashMap::new();
        stat_boosts.insert(Stat::Attack, 2);
        stat_boosts.insert(Stat::Speed, -1);
        
        let instruction = Instruction::BoostStats(BoostStatsInstruction {
            target_position: position,
            stat_boosts,
        });
        
        state.apply_instruction(&instruction);
        
        let pokemon = state.get_pokemon_at_position(position).unwrap();
        assert_eq!(pokemon.stat_boosts.get(&Stat::Attack), Some(&2));
        assert_eq!(pokemon.stat_boosts.get(&Stat::Speed), Some(&-1));
        
        // Test clamping at +6/-6
        let mut extreme_boosts = HashMap::new();
        extreme_boosts.insert(Stat::Attack, 6); // Already at +2, this should clamp to +6
        
        let extreme_instruction = Instruction::BoostStats(BoostStatsInstruction {
            target_position: position,
            stat_boosts: extreme_boosts,
        });
        
        state.apply_instruction(&extreme_instruction);
        
        let pokemon = state.get_pokemon_at_position(position).unwrap();
        assert_eq!(pokemon.stat_boosts.get(&Stat::Attack), Some(&6)); // Clamped to +6
    }
}

#[cfg(test)]
mod volatile_status_tests {
    use super::*;

    #[test]
    fn test_apply_volatile_status_instruction() {
        let mut state = create_test_state();
        let position = BattlePosition::new(SideReference::SideOne, 0);
        
        let instruction = Instruction::ApplyVolatileStatus(ApplyVolatileStatusInstruction {
            target_position: position,
            volatile_status: VolatileStatus::Confusion,
            duration: Some(3),
        });
        
        state.apply_instruction(&instruction);
        
        let pokemon = state.get_pokemon_at_position(position).unwrap();
        assert!(pokemon.volatile_statuses.contains(&VolatileStatus::Confusion));
        assert_eq!(pokemon.volatile_status_durations.get(&VolatileStatus::Confusion), Some(&3));
    }

    #[test]
    fn test_remove_volatile_status_instruction() {
        let mut state = create_test_state();
        let position = BattlePosition::new(SideReference::SideOne, 0);
        
        // First apply volatile status
        let apply_instruction = Instruction::ApplyVolatileStatus(ApplyVolatileStatusInstruction {
            target_position: position,
            volatile_status: VolatileStatus::Substitute,
            duration: None,
        });
        state.apply_instruction(&apply_instruction);
        
        // Verify it was applied
        let pokemon = state.get_pokemon_at_position(position).unwrap();
        assert!(pokemon.volatile_statuses.contains(&VolatileStatus::Substitute));
        
        // Now remove it
        let remove_instruction = Instruction::RemoveVolatileStatus(RemoveVolatileStatusInstruction {
            target_position: position,
            volatile_status: VolatileStatus::Substitute,
        });
        state.apply_instruction(&remove_instruction);
        
        // Verify it was removed
        let pokemon = state.get_pokemon_at_position(position).unwrap();
        assert!(!pokemon.volatile_statuses.contains(&VolatileStatus::Substitute));
        assert_eq!(pokemon.volatile_status_durations.get(&VolatileStatus::Substitute), None);
    }

    #[test]
    fn test_change_volatile_status_duration_instruction() {
        let mut state = create_test_state();
        let position = BattlePosition::new(SideReference::SideOne, 0);
        
        // First apply volatile status with duration
        let apply_instruction = Instruction::ApplyVolatileStatus(ApplyVolatileStatusInstruction {
            target_position: position,
            volatile_status: VolatileStatus::Taunt,
            duration: Some(4),
        });
        state.apply_instruction(&apply_instruction);
        
        // Change duration
        let change_instruction = Instruction::ChangeVolatileStatusDuration(ChangeVolatileStatusDurationInstruction {
            target_position: position,
            volatile_status: VolatileStatus::Taunt,
            duration_change: -2,
        });
        state.apply_instruction(&change_instruction);
        
        // Verify duration changed
        let pokemon = state.get_pokemon_at_position(position).unwrap();
        assert_eq!(pokemon.volatile_status_durations.get(&VolatileStatus::Taunt), Some(&2));
        
        // Test duration reduction to zero (should remove status)
        let reduce_instruction = Instruction::ChangeVolatileStatusDuration(ChangeVolatileStatusDurationInstruction {
            target_position: position,
            volatile_status: VolatileStatus::Taunt,
            duration_change: -2,
        });
        state.apply_instruction(&reduce_instruction);
        
        // Verify status was removed when duration hit zero
        let pokemon = state.get_pokemon_at_position(position).unwrap();
        assert!(!pokemon.volatile_statuses.contains(&VolatileStatus::Taunt));
        assert_eq!(pokemon.volatile_status_durations.get(&VolatileStatus::Taunt), None);
    }
}

#[cfg(test)]
mod switch_tests {
    use super::*;

    #[test]
    fn test_switch_pokemon_instruction() {
        let mut state = create_test_state();
        let position = BattlePosition::new(SideReference::SideOne, 0);
        
        // Add a second Pokemon to switch to
        let pokemon2 = Pokemon::new("Raichu".to_string());
        state.side_one.add_pokemon(pokemon2);
        
        // Verify initial active Pokemon
        assert_eq!(state.side_one.active_pokemon_indices[0], Some(0));
        
        let instruction = Instruction::SwitchPokemon(SwitchInstruction {
            position,
            previous_index: 0,
            next_index: 1,
        });
        
        state.apply_instruction(&instruction);
        
        // Verify Pokemon was switched
        assert_eq!(state.side_one.active_pokemon_indices[0], Some(1));
        let active_pokemon = state.get_pokemon_at_position(position).unwrap();
        assert_eq!(active_pokemon.species, "Raichu");
    }
}

#[cfg(test)]
mod weather_and_terrain_tests {
    use super::*;

    #[test]
    fn test_change_weather_instruction() {
        let mut state = create_test_state();
        assert_eq!(state.weather, Weather::NONE);
        assert_eq!(state.weather_turns_remaining, None);
        
        let instruction = Instruction::ChangeWeather(ChangeWeatherInstruction { weather: Weather::RAIN, duration: Some(5), previous_weather: Some(Weather::NONE), previous_duration: Some(None), });
        
        state.apply_instruction(&instruction);
        
        assert_eq!(state.weather, Weather::RAIN);
        assert_eq!(state.weather_turns_remaining, Some(5));
    }

    #[test]
    fn test_change_terrain_instruction() {
        let mut state = create_test_state();
        assert_eq!(state.terrain, Terrain::NONE);
        assert_eq!(state.terrain_turns_remaining, None);
        
        let instruction = Instruction::ChangeTerrain(ChangeTerrainInstruction { terrain: Terrain::ElectricTerrain, duration: Some(5), previous_terrain: Some(Terrain::NONE), previous_duration: Some(None), });
        
        state.apply_instruction(&instruction);
        
        assert_eq!(state.terrain, Terrain::ElectricTerrain);
        assert_eq!(state.terrain_turns_remaining, Some(5));
    }

    #[test]
    fn test_decrement_weather_turns() {
        let mut state = create_test_state();
        
        // Set weather with duration
        let instruction = Instruction::ChangeWeather(ChangeWeatherInstruction { weather: Weather::Sun, duration: Some(3), previous_weather: Some(Weather::NONE), previous_duration: Some(None), });
        state.apply_instruction(&instruction);
        
        // Decrement turns
        let decrement_instruction = Instruction::DecrementWeatherTurns;
        state.apply_instruction(&decrement_instruction);
        
        assert_eq!(state.weather, Weather::Sun);
        assert_eq!(state.weather_turns_remaining, Some(2));
        
        // Decrement to zero
        state.apply_instruction(&decrement_instruction);
        state.apply_instruction(&decrement_instruction);
        
        // Weather should be cleared when turns reach zero
        assert_eq!(state.weather, Weather::NONE);
        assert_eq!(state.weather_turns_remaining, None);
    }

    #[test]
    fn test_decrement_terrain_turns() {
        let mut state = create_test_state();
        
        // Set terrain with duration
        let instruction = Instruction::ChangeTerrain(ChangeTerrainInstruction { terrain: Terrain::GrassyTerrain, duration: Some(2), previous_terrain: Some(Terrain::NONE), previous_duration: Some(None), });
        state.apply_instruction(&instruction);
        
        // Decrement turns
        let decrement_instruction = Instruction::DecrementTerrainTurns;
        state.apply_instruction(&decrement_instruction);
        
        assert_eq!(state.terrain, Terrain::GrassyTerrain);
        assert_eq!(state.terrain_turns_remaining, Some(1));
        
        // Decrement to zero
        state.apply_instruction(&decrement_instruction);
        
        // Terrain should be cleared when turns reach zero
        assert_eq!(state.terrain, Terrain::NONE);
        assert_eq!(state.terrain_turns_remaining, None);
    }
}

#[cfg(test)]
mod side_condition_tests {
    use super::*;

    #[test]
    fn test_apply_side_condition_instruction() {
        let mut state = create_test_state();
        
        let instruction = Instruction::ApplySideCondition(ApplySideConditionInstruction {
            side: SideReference::SideOne,
            condition: SideCondition::Reflect,
            duration: Some(5),
        });
        
        state.apply_instruction(&instruction);
        
        assert_eq!(state.side_one.side_conditions.get(&SideCondition::Reflect), Some(&5));
    }

    #[test]
    fn test_remove_side_condition_instruction() {
        let mut state = create_test_state();
        
        // First apply condition
        let apply_instruction = Instruction::ApplySideCondition(ApplySideConditionInstruction {
            side: SideReference::SideOne,
            condition: SideCondition::LightScreen,
            duration: Some(3),
        });
        state.apply_instruction(&apply_instruction);
        
        // Verify it was applied
        assert_eq!(state.side_one.side_conditions.get(&SideCondition::LightScreen), Some(&3));
        
        // Now remove it
        let remove_instruction = Instruction::RemoveSideCondition(RemoveSideConditionInstruction {
            side: SideReference::SideOne,
            condition: SideCondition::LightScreen,
        });
        state.apply_instruction(&remove_instruction);
        
        // Verify it was removed
        assert_eq!(state.side_one.side_conditions.get(&SideCondition::LightScreen), None);
    }

    #[test]
    fn test_decrement_side_condition_duration_instruction() {
        let mut state = create_test_state();
        
        // First apply condition
        let apply_instruction = Instruction::ApplySideCondition(ApplySideConditionInstruction {
            side: SideReference::SideOne,
            condition: SideCondition::Mist,
            duration: Some(5),
        });
        state.apply_instruction(&apply_instruction);
        
        // Decrement duration
        let decrement_instruction = Instruction::DecrementSideConditionDuration(DecrementSideConditionDurationInstruction {
            side: SideReference::SideOne,
            condition: SideCondition::Mist,
            amount: 2,
        });
        state.apply_instruction(&decrement_instruction);
        
        // Verify duration decreased
        assert_eq!(state.side_one.side_conditions.get(&SideCondition::Mist), Some(&3));
        
        // Decrement to zero (should remove condition)
        let remove_instruction = Instruction::DecrementSideConditionDuration(DecrementSideConditionDurationInstruction {
            side: SideReference::SideOne,
            condition: SideCondition::Mist,
            amount: 3,
        });
        state.apply_instruction(&remove_instruction);
        
        // Verify condition was removed
        assert_eq!(state.side_one.side_conditions.get(&SideCondition::Mist), None);
    }

    #[test]
    fn test_stackable_side_conditions() {
        let mut state = create_test_state();
        
        // Apply Spikes multiple times (should stack)
        let spikes1 = Instruction::ApplySideCondition(ApplySideConditionInstruction {
            side: SideReference::SideTwo,
            condition: SideCondition::Spikes,
            duration: None, // No duration means increment
        });
        state.apply_instruction(&spikes1);
        assert_eq!(state.side_two.side_conditions.get(&SideCondition::Spikes), Some(&1));
        
        state.apply_instruction(&spikes1);
        assert_eq!(state.side_two.side_conditions.get(&SideCondition::Spikes), Some(&2));
        
        state.apply_instruction(&spikes1);
        assert_eq!(state.side_two.side_conditions.get(&SideCondition::Spikes), Some(&3));
        
        // Fourth application should still be clamped at 3
        state.apply_instruction(&spikes1);
        assert_eq!(state.side_two.side_conditions.get(&SideCondition::Spikes), Some(&3));
    }
}

// Continue with part 2...