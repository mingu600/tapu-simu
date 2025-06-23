//! # Test Assertions and Validation
//! 
//! This module provides specialized assertion functions for Pokemon battle testing,
//! enabling precise validation of battle outcomes and mechanics.

use tapu_simu::core::battle_format::{BattlePosition, SideReference};
use tapu_simu::core::battle_state::BattleState;
use tapu_simu::core::instructions::{BattleInstruction, BattleInstructions, PokemonStatus, Stat, Weather, Terrain, SideCondition};
use std::collections::HashMap;

/// Specialized assertions for Pokemon battle testing
pub struct BattleAssertions;

impl BattleAssertions {
    /// Assert that a Pokemon took the expected amount of damage
    pub fn assert_damage(
        state: &BattleState,
        position: BattlePosition,
        expected_damage: u16,
    ) -> Result<(), String> {
        let pokemon = state.get_pokemon_at_position(position)
            .ok_or_else(|| format!("No Pokemon at position {:?}", position))?;
        
        let actual_damage = pokemon.max_hp - pokemon.hp;
        
        if actual_damage != expected_damage as i16 {
            return Err(format!(
                "Damage assertion failed at {:?}: expected {}, got {} (Pokemon has {}/{} HP)",
                position, expected_damage, actual_damage, pokemon.hp, pokemon.max_hp
            ));
        }
        
        Ok(())
    }
    
    /// Assert that a Pokemon has the expected status condition
    pub fn assert_status(
        state: &BattleState,
        position: BattlePosition,
        expected_status: PokemonStatus,
    ) -> Result<(), String> {
        let pokemon = state.get_pokemon_at_position(position)
            .ok_or_else(|| format!("No Pokemon at position {:?}", position))?;
        
        if pokemon.status != expected_status {
            return Err(format!(
                "Status assertion failed at {:?}: expected {:?}, got {:?}",
                position, expected_status, pokemon.status
            ));
        }
        
        Ok(())
    }
    
    /// Assert that a Pokemon has no status condition
    pub fn assert_no_status(
        state: &BattleState,
        position: BattlePosition,
    ) -> Result<(), String> {
        Self::assert_status(state, position, PokemonStatus::None)
    }
    
    /// Assert that a Pokemon has the expected stat changes
    pub fn assert_stat_changes(
        state: &BattleState,
        position: BattlePosition,
        expected_changes: &HashMap<Stat, i8>,
    ) -> Result<(), String> {
        let pokemon = state.get_pokemon_at_position(position)
            .ok_or_else(|| format!("No Pokemon at position {:?}", position))?;
        
        for (stat, expected_change) in expected_changes {
            let actual_change = pokemon.stat_boosts.get(stat).unwrap_or(&0);
            if actual_change != expected_change {
                return Err(format!(
                    "Stat change assertion failed at {:?} for {:?}: expected {}, got {}",
                    position, stat, expected_change, actual_change
                ));
            }
        }
        
        Ok(())
    }
    
    /// Assert that a Pokemon has a specific stat change
    pub fn assert_stat_change(
        state: &BattleState,
        position: BattlePosition,
        stat: Stat,
        expected_change: i8,
    ) -> Result<(), String> {
        let mut map = HashMap::new();
        map.insert(stat, expected_change);
        Self::assert_stat_changes(state, position, &map)
    }
    
    /// Assert that a Pokemon has no stat changes
    pub fn assert_no_stat_changes(
        state: &BattleState,
        position: BattlePosition,
    ) -> Result<(), String> {
        let pokemon = state.get_pokemon_at_position(position)
            .ok_or_else(|| format!("No Pokemon at position {:?}", position))?;
        
        for (stat, change) in &pokemon.stat_boosts {
            if *change != 0 {
                return Err(format!(
                    "Unexpected stat change at {:?}: {:?} has change of {}",
                    position, stat, change
                ));
            }
        }
        
        Ok(())
    }
    
    /// Assert that a Pokemon has fainted
    pub fn assert_fainted(
        state: &BattleState,
        position: BattlePosition,
    ) -> Result<(), String> {
        let pokemon = state.get_pokemon_at_position(position)
            .ok_or_else(|| format!("No Pokemon at position {:?}", position))?;
        
        if pokemon.hp > 0 {
            return Err(format!(
                "Faint assertion failed at {:?}: Pokemon has {} HP (expected 0)",
                position, pokemon.hp
            ));
        }
        
        Ok(())
    }
    
    /// Assert that a Pokemon has not fainted
    pub fn assert_not_fainted(
        state: &BattleState,
        position: BattlePosition,
    ) -> Result<(), String> {
        let pokemon = state.get_pokemon_at_position(position)
            .ok_or_else(|| format!("No Pokemon at position {:?}", position))?;
        
        if pokemon.hp <= 0 {
            return Err(format!(
                "Not fainted assertion failed at {:?}: Pokemon has {} HP (expected > 0)",
                position, pokemon.hp
            ));
        }
        
        Ok(())
    }
    
    /// Assert that a Pokemon has the expected HP percentage
    pub fn assert_hp_percentage(
        state: &BattleState,
        position: BattlePosition,
        expected_percentage: f32,
        tolerance: f32,
    ) -> Result<(), String> {
        let pokemon = state.get_pokemon_at_position(position)
            .ok_or_else(|| format!("No Pokemon at position {:?}", position))?;
        
        let actual_percentage = (pokemon.hp as f32 / pokemon.max_hp as f32) * 100.0;
        
        if (actual_percentage - expected_percentage).abs() > tolerance {
            return Err(format!(
                "HP percentage assertion failed at {:?}: expected {}% (±{}%), got {}%",
                position, expected_percentage, tolerance, actual_percentage
            ));
        }
        
        Ok(())
    }
    
    /// Assert that the weather is as expected
    pub fn assert_weather(
        state: &BattleState,
        expected_weather: Weather,
    ) -> Result<(), String> {
        if state.weather() != expected_weather {
            return Err(format!(
                "Weather assertion failed: expected {:?}, got {:?}",
                expected_weather, state.weather()
            ));
        }
        
        Ok(())
    }
    
    /// Assert that the terrain is as expected
    pub fn assert_terrain(
        state: &BattleState,
        expected_terrain: Terrain,
    ) -> Result<(), String> {
        if state.terrain() != expected_terrain {
            return Err(format!(
                "Terrain assertion failed: expected {:?}, got {:?}",
                expected_terrain, state.terrain()
            ));
        }
        
        Ok(())
    }
    
    /// Assert that a side condition is present
    pub fn assert_side_condition(
        state: &BattleState,
        side: SideReference,
        condition: SideCondition,
    ) -> Result<(), String> {
        let side_state = state.get_side_by_ref(side);
        
        if !side_state.side_conditions.contains_key(&condition) {
            return Err(format!(
                "Side condition assertion failed: expected {:?} on {:?}, but not present",
                condition, side
            ));
        }
        
        Ok(())
    }
    
    /// Assert that a side condition has a specific value
    pub fn assert_side_condition_value(
        state: &BattleState,
        side: SideReference,
        condition: SideCondition,
        expected_value: i8,
    ) -> Result<(), String> {
        let side_state = state.get_side_by_ref(side);
        
        let actual_value = side_state.side_conditions.get(&condition)
            .ok_or_else(|| format!("Side condition {:?} not present on {:?}", condition, side))?;
        
        if *actual_value != expected_value as u8 {
            return Err(format!(
                "Side condition value assertion failed: expected {:?} = {} on {:?}, got {}",
                condition, expected_value, side, actual_value
            ));
        }
        
        Ok(())
    }
    
    /// Assert that instructions match exactly
    pub fn assert_instructions_exact(
        actual: &[BattleInstructions],
        expected: &[BattleInstructions],
    ) -> Result<(), String> {
        if actual.len() != expected.len() {
            return Err(format!(
                "Instruction count mismatch: expected {}, got {}",
                expected.len(), actual.len()
            ));
        }
        
        for (i, (actual_set, expected_set)) in actual.iter().zip(expected.iter()).enumerate() {
            if (actual_set.percentage - expected_set.percentage).abs() > 0.01 {
                return Err(format!(
                    "Instruction set {} percentage mismatch: expected {}, got {}",
                    i, expected_set.percentage, actual_set.percentage
                ));
            }
            
            if actual_set.instruction_list != expected_set.instruction_list {
                return Err(format!(
                    "Instruction set {} content mismatch:\nExpected: {:?}\nActual: {:?}",
                    i, expected_set.instruction_list, actual_set.instruction_list
                ));
            }
        }
        
        Ok(())
    }
    
    /// Assert that instructions contain a specific instruction
    pub fn assert_contains_instruction(
        instructions: &[BattleInstructions],
        expected_instruction: &BattleInstruction,
    ) -> Result<(), String> {
        for instruction_set in instructions {
            for instruction in &instruction_set.instruction_list {
                if instruction == expected_instruction {
                    return Ok(());
                }
            }
        }
        
        Err(format!(
            "Expected instruction not found: {:?}",
            expected_instruction
        ))
    }
    
    /// Assert that instructions do not contain a specific instruction
    pub fn assert_not_contains_instruction(
        instructions: &[BattleInstructions],
        forbidden_instruction: &BattleInstruction,
    ) -> Result<(), String> {
        for instruction_set in instructions {
            for instruction in &instruction_set.instruction_list {
                if instruction == forbidden_instruction {
                    return Err(format!(
                        "Forbidden instruction found: {:?}",
                        forbidden_instruction
                    ));
                }
            }
        }
        
        Ok(())
    }
    
    /// Assert that the total probability of all instruction sets equals 100%
    pub fn assert_total_probability(
        instructions: &[BattleInstructions],
    ) -> Result<(), String> {
        let total: f32 = instructions.iter().map(|i| i.percentage).sum();
        
        if (total - 100.0).abs() > 0.01 {
            return Err(format!(
                "Total probability assertion failed: expected 100%, got {}%",
                total
            ));
        }
        
        Ok(())
    }
}

/// Convenience functions for common assertion patterns
pub mod convenience {
    use super::*;
    
    /// Assert that an attack deals the expected damage
    pub fn assert_attack_damage(
        state: &BattleState,
        target: BattlePosition,
        expected_damage: u16,
    ) -> Result<(), String> {
        BattleAssertions::assert_damage(state, target, expected_damage)
    }
    
    /// Assert that an attack caused the expected status
    pub fn assert_attack_status(
        state: &BattleState,
        target: BattlePosition,
        expected_status: PokemonStatus,
    ) -> Result<(), String> {
        BattleAssertions::assert_status(state, target, expected_status)
    }
    
    /// Assert that a stat-boosting move worked correctly
    pub fn assert_stat_boost(
        state: &BattleState,
        target: BattlePosition,
        stat: Stat,
        boost_amount: i8,
    ) -> Result<(), String> {
        BattleAssertions::assert_stat_change(state, target, stat, boost_amount)
    }
    
    /// Assert that a move had no effect (no damage, no status, no stat changes)
    pub fn assert_no_effect(
        state: &BattleState,
        target: BattlePosition,
        original_hp: i16,
    ) -> Result<(), String> {
        let pokemon = state.get_pokemon_at_position(target)
            .ok_or_else(|| format!("No Pokemon at position {:?}", target))?;
        
        // Check HP didn't change
        if pokemon.hp != original_hp {
            return Err(format!(
                "Expected no effect but HP changed from {} to {}",
                original_hp, pokemon.hp
            ));
        }
        
        // Check no status
        BattleAssertions::assert_no_status(state, target)?;
        
        // Check no stat changes
        BattleAssertions::assert_no_stat_changes(state, target)?;
        
        Ok(())
    }
    
    /// Assert that immunity prevented any effect
    pub fn assert_immunity(
        state: &BattleState,
        target: BattlePosition,
        original_hp: i16,
    ) -> Result<(), String> {
        assert_no_effect(state, target, original_hp)
    }
    
    /// Assert that a critical hit occurred (used with damage ranges)
    pub fn assert_critical_hit(
        actual_damage: u16,
        normal_damage_range: (u16, u16),
        crit_damage_range: (u16, u16),
    ) -> Result<(), String> {
        // If damage is within normal range, it's not a crit
        if actual_damage >= normal_damage_range.0 && actual_damage <= normal_damage_range.1 {
            return Err(format!(
                "Expected critical hit but damage {} is in normal range ({}-{})",
                actual_damage, normal_damage_range.0, normal_damage_range.1
            ));
        }
        
        // If damage is within crit range, it's a crit
        if actual_damage >= crit_damage_range.0 && actual_damage <= crit_damage_range.1 {
            return Ok(());
        }
        
        Err(format!(
            "Damage {} is neither in normal range ({}-{}) nor crit range ({}-{})",
            actual_damage, normal_damage_range.0, normal_damage_range.1,
            crit_damage_range.0, crit_damage_range.1
        ))
    }
}

/// Macro for creating assertion chains
#[macro_export]
macro_rules! assert_battle_state {
    ($state:expr => {
        $($assertion:expr),* $(,)?
    }) => {
        $(
            $assertion?;
        )*
    };
}

/// Macro for asserting damage ranges
#[macro_export]
macro_rules! assert_damage_range {
    ($state:expr, $position:expr, $min:expr, $max:expr) => {
        {
            let pokemon = $state.get_pokemon_at_position($position)
                .ok_or_else(|| format!("No Pokemon at position {:?}", $position))?;
            let actual_damage = pokemon.max_hp - pokemon.hp;
            if actual_damage < $min as i16 || actual_damage > $max as i16 {
                return Err(format!(
                    "Damage {} at {:?} is outside expected range {}-{}",
                    actual_damage, $position, $min, $max
                ));
            }
        }
    };
}

/// Macro for asserting multiple stat changes at once
#[macro_export]
macro_rules! assert_stat_changes {
    ($state:expr, $position:expr, { $($stat:expr => $change:expr),* $(,)? }) => {
        {
            let mut expected = std::collections::HashMap::new();
            $(
                expected.insert($stat, $change);
            )*
            BattleAssertions::assert_stat_changes($state, $position, &expected)?;
        }
    };
}