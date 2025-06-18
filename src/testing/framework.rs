//! # Test Framework for Advanced Integration Testing
//! 
//! This module provides utilities for creating realistic battle scenarios
//! using actual Pokemon Showdown data and testing the damage calculation
//! pipeline with real Pokemon data.

use crate::core::state::{Pokemon, State, MoveCategory, Move};
use crate::core::battle_format::{BattleFormat, BattlePosition, SideReference, FormatType};
use crate::data::loader::PSDataRepository;
use crate::engine::combat::damage_calc::calculate_damage;
use crate::engine::turn::instruction_generator::GenerationXInstructionGenerator;
use crate::data::types::EngineMoveData;
use crate::core::move_choice::{MoveChoice, MoveIndex};
use crate::core::instruction::{Instruction, StateInstructions, PokemonStatus, Stat};
use crate::generation::Generation;

/// Normalize names to match PS conventions (lowercase, no spaces/hyphens)
pub fn normalize_name(name: &str) -> String {
    name.to_lowercase()
        .replace(" ", "")
        .replace("-", "")
        .replace("'", "")
        .replace(".", "")
        .replace(":", "")
}

/// Test framework for damage calculation with real PS data
pub struct TestFramework {
    pub ps_data: PSDataRepository,
}

impl TestFramework {
    /// Create a new test framework with PS data
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let ps_data = PSDataRepository::load_from_directory("data/ps-extracted")?;
        
        Ok(Self {
            ps_data,
        })
    }

    /// Create a Pokemon from PS data
    pub fn create_pokemon_from_ps_data(
        &self,
        species_id: &str,
        ability: Option<&str>,
        level: Option<u8>,
    ) -> Result<Pokemon, Box<dyn std::error::Error>> {
        let pokemon_data = self.ps_data.get_pokemon(species_id)
            .ok_or_else(|| format!("Pokemon '{}' not found in PS data", species_id))?;

        let mut pokemon = Pokemon::new(species_id.to_string());
        
        // Set types from PS data
        if let Some(types) = pokemon_data["types"].as_array() {
            pokemon.types = types.iter()
                .filter_map(|t| t.as_str())
                .map(|t| t.to_string())
                .collect();
        }

        // Set base stats from PS data
        if let Some(base_stats) = pokemon_data["baseStats"].as_object() {
            if let Some(hp) = base_stats["hp"].as_u64() {
                pokemon.max_hp = hp as i16;
                pokemon.hp = pokemon.max_hp;
            }
            if let Some(atk) = base_stats["atk"].as_u64() {
                pokemon.stats.attack = atk as i16;
            }
            if let Some(def) = base_stats["def"].as_u64() {
                pokemon.stats.defense = def as i16;
            }
            if let Some(spa) = base_stats["spa"].as_u64() {
                pokemon.stats.special_attack = spa as i16;
            }
            if let Some(spd) = base_stats["spd"].as_u64() {
                pokemon.stats.special_defense = spd as i16;
            }
            if let Some(spe) = base_stats["spe"].as_u64() {
                pokemon.stats.speed = spe as i16;
            }
        }

        // Set ability (normalized)
        if let Some(ability_name) = ability {
            pokemon.ability = normalize_name(ability_name);
        } else {
            // When no ability is specified, use a neutral ability that doesn't affect damage
            pokemon.ability = "noability".to_string();
        }

        // Set level
        pokemon.level = level.unwrap_or(50);

        Ok(pokemon)
    }

    /// Create a move from PS data
    pub fn create_move_from_ps_data(
        &self,
        move_name: &str,
    ) -> Result<EngineMoveData, Box<dyn std::error::Error>> {
        let ps_move = self.ps_data.get_move_by_name(move_name)
            .ok_or_else(|| format!("Move '{}' not found in PS data", move_name))?;

        // Convert PS category string to MoveCategory enum
        let category = match ps_move.category.as_str() {
            "Physical" => MoveCategory::Physical,
            "Special" => MoveCategory::Special,
            "Status" => MoveCategory::Status,
            _ => MoveCategory::Physical, // Default fallback
        };

        // Convert target string to PSMoveTarget enum
        use crate::data::ps_types::PSMoveTarget;
        let target = match ps_move.target.as_str() {
            "normal" => PSMoveTarget::Normal,
            "self" => PSMoveTarget::Self_,
            "adjacentAlly" => PSMoveTarget::AdjacentAlly,
            "adjacentAllyOrSelf" => PSMoveTarget::AdjacentAllyOrSelf,
            "adjacentFoe" => PSMoveTarget::AdjacentFoe,
            "allAdjacentFoes" => PSMoveTarget::AllAdjacentFoes,
            "allAdjacent" => PSMoveTarget::AllAdjacent,
            "all" => PSMoveTarget::All,
            "allyTeam" => PSMoveTarget::AllyTeam,
            "allySide" => PSMoveTarget::AllySide,
            "foeSide" => PSMoveTarget::FoeSide,
            "any" => PSMoveTarget::Any,
            "randomNormal" => PSMoveTarget::RandomNormal,
            "scripted" => PSMoveTarget::Scripted,
            "allies" => PSMoveTarget::Allies,
            _ => PSMoveTarget::Normal, // Default fallback
        };

        // Convert PS flags to vector of flag names
        let flags: Vec<String> = ps_move.flags
            .iter()
            .filter_map(|(flag_name, flag_value)| {
                if *flag_value == 1 {
                    Some(flag_name.clone())
                } else {
                    None
                }
            })
            .collect();

        Ok(EngineMoveData {
            id: ps_move.num,
            name: ps_move.name.clone(),
            base_power: if ps_move.base_power > 0 { Some(ps_move.base_power as i16) } else { None },
            accuracy: if ps_move.accuracy > 0 { Some(ps_move.accuracy as i16) } else { None },
            pp: ps_move.pp as i16,
            move_type: ps_move.move_type.clone(),
            category,
            priority: ps_move.priority,
            target,
            effect_chance: None,
            effect_description: String::new(),
            flags,
        })
    }

    /// Test damage calculation between two Pokemon with a specific move
    pub fn test_damage_calculation(
        &self,
        attacker: &Pokemon,
        defender: &Pokemon,
        move_data: &EngineMoveData,
        state: &State,
    ) -> i16 {
        calculate_damage(state, attacker, defender, move_data, false, 1.0)
    }

    /// Test ability immunity - returns true if move deals 0 damage due to ability
    pub fn test_ability_immunity(
        &self,
        attacker_species: &str,
        defender_species: &str,
        defender_ability: &str,
        move_name: &str,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let attacker = self.create_pokemon_from_ps_data(attacker_species, None, Some(50))?;
        let defender = self.create_pokemon_from_ps_data(defender_species, Some(defender_ability), Some(50))?;
        let move_data = self.create_move_from_ps_data(move_name)?;
        let state = State::new(BattleFormat::gen9_ou());

        let damage = self.test_damage_calculation(&attacker, &defender, &move_data, &state);
        Ok(damage == 0)
    }

    /// Test ability damage reduction - returns the damage multiplier (e.g., 0.5 for 50% reduction)
    pub fn test_ability_damage_reduction(
        &self,
        attacker_species: &str,
        defender_species: &str,
        defender_ability: &str,
        move_name: &str,
    ) -> Result<f32, Box<dyn std::error::Error>> {
        // Test without ability
        let defender_normal = self.create_pokemon_from_ps_data(defender_species, None, Some(50))?;
        let attacker = self.create_pokemon_from_ps_data(attacker_species, None, Some(50))?;
        let move_data = self.create_move_from_ps_data(move_name)?;
        let state = State::new(BattleFormat::gen9_ou());

        let normal_damage = self.test_damage_calculation(&attacker, &defender_normal, &move_data, &state);
        
        // Test with ability
        let defender_with_ability = self.create_pokemon_from_ps_data(defender_species, Some(defender_ability), Some(50))?;
        let ability_damage = self.test_damage_calculation(&attacker, &defender_with_ability, &move_data, &state);

        if normal_damage == 0 {
            Ok(0.0) // Avoid division by zero
        } else {
            Ok(ability_damage as f32 / normal_damage as f32)
        }
    }

    /// Test weather negation by abilities like Cloud Nine and Air Lock
    pub fn test_weather_negation(
        &self,
        attacker_species: &str,
        defender_species: &str,
        weather_negation_ability: &str,
        weather: crate::core::instruction::Weather,
        move_name: &str,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let attacker = self.create_pokemon_from_ps_data(attacker_species, None, Some(50))?;
        let defender = self.create_pokemon_from_ps_data(defender_species, None, Some(50))?;
        let move_data = self.create_move_from_ps_data(move_name)?;

        // Test with weather but no negation ability
        let mut state_with_weather = State::new(BattleFormat::gen9_ou());
        state_with_weather.weather = weather;
        let weather_damage = self.test_damage_calculation(&attacker, &defender, &move_data, &state_with_weather);

        // Test with weather negation ability on field
        let mut state_with_negation = State::new(BattleFormat::gen9_ou());
        state_with_negation.weather = weather;
        let negator = self.create_pokemon_from_ps_data(defender_species, Some(weather_negation_ability), Some(50))?;
        state_with_negation.side_two.pokemon.push(negator);

        let negation_damage = self.test_damage_calculation(&attacker, &defender, &move_data, &state_with_negation);

        // Weather should be negated if damage is different (less weather boost)
        Ok(negation_damage != weather_damage)
    }

    /// Test weather stat boosts (Sandstorm SpDef for Rock, Snow Def for Ice)
    pub fn test_weather_stat_boost(
        &self,
        attacker_species: &str,
        defender_species: &str,
        defender_types: &[&str],
        weather: crate::core::instruction::Weather,
        move_name: &str,
    ) -> Result<f32, Box<dyn std::error::Error>> {
        let attacker = self.create_pokemon_from_ps_data(attacker_species, None, Some(50))?;
        let mut defender = self.create_pokemon_from_ps_data(defender_species, None, Some(50))?;
        
        // Override defender types for testing
        defender.types = defender_types.iter().map(|&t| t.to_string()).collect();
        
        let move_data = self.create_move_from_ps_data(move_name)?;

        // Test without weather
        let state_no_weather = State::new(BattleFormat::gen9_ou());
        let normal_damage = self.test_damage_calculation(&attacker, &defender, &move_data, &state_no_weather);

        // Test with weather
        let mut state_with_weather = State::new(BattleFormat::gen9_ou());
        state_with_weather.weather = weather;
        let weather_damage = self.test_damage_calculation(&attacker, &defender, &move_data, &state_with_weather);

        if normal_damage == 0 {
            Ok(0.0)
        } else {
            Ok(weather_damage as f32 / normal_damage as f32)
        }
    }

    /// Test terrain effects on damage
    pub fn test_terrain_effects(
        &self,
        attacker_species: &str,
        defender_species: &str,
        terrain: crate::core::instruction::Terrain,
        move_name: &str,
    ) -> Result<f32, Box<dyn std::error::Error>> {
        let attacker = self.create_pokemon_from_ps_data(attacker_species, None, Some(50))?;
        let defender = self.create_pokemon_from_ps_data(defender_species, None, Some(50))?;
        let move_data = self.create_move_from_ps_data(move_name)?;

        // Test without terrain
        let state_no_terrain = State::new(BattleFormat::gen9_ou());
        let normal_damage = self.test_damage_calculation(&attacker, &defender, &move_data, &state_no_terrain);

        // Test with terrain
        let mut state_with_terrain = State::new(BattleFormat::gen9_ou());
        state_with_terrain.terrain = terrain;
        let terrain_damage = self.test_damage_calculation(&attacker, &defender, &move_data, &state_with_terrain);

        if normal_damage == 0 {
            Ok(0.0)
        } else {
            Ok(terrain_damage as f32 / normal_damage as f32)
        }
    }

    /// Test grounded status for terrain effects
    pub fn test_grounded_immunity(
        &self,
        attacker_species: &str,
        attacker_ability: &str,
        terrain: crate::core::instruction::Terrain,
        move_name: &str,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let grounded_attacker = self.create_pokemon_from_ps_data(attacker_species, None, Some(50))?;
        let ungrounded_attacker = self.create_pokemon_from_ps_data(attacker_species, Some(attacker_ability), Some(50))?;
        let defender = self.create_pokemon_from_ps_data("charizard", None, Some(50))?;
        let move_data = self.create_move_from_ps_data(move_name)?;

        let mut state = State::new(BattleFormat::gen9_ou());
        state.terrain = terrain;

        let grounded_damage = self.test_damage_calculation(&grounded_attacker, &defender, &move_data, &state);
        let ungrounded_damage = self.test_damage_calculation(&ungrounded_attacker, &defender, &move_data, &state);

        // Ungrounded Pokemon should not get terrain boost
        Ok(grounded_damage != ungrounded_damage)
    }

    /// Test screen effects on damage reduction
    pub fn test_screen_effects(
        &self,
        attacker_species: &str,
        defender_species: &str,
        screen: crate::core::instruction::SideCondition,
        move_name: &str,
    ) -> Result<f32, Box<dyn std::error::Error>> {
        let attacker = self.create_pokemon_from_ps_data(attacker_species, None, Some(50))?;
        let defender = self.create_pokemon_from_ps_data(defender_species, None, Some(50))?;
        let move_data = self.create_move_from_ps_data(move_name)?;

        // Test without screen
        let mut state_no_screen = State::new(BattleFormat::gen9_ou());
        state_no_screen.side_one.pokemon.push(attacker.clone());
        state_no_screen.side_two.pokemon.push(defender.clone());
        let normal_damage = calculate_damage(&state_no_screen, &state_no_screen.side_one.pokemon[0], &state_no_screen.side_two.pokemon[0], &move_data, false, 1.0);

        // Test with screen
        let mut state_with_screen = State::new(BattleFormat::gen9_ou());
        state_with_screen.side_one.pokemon.push(attacker);
        state_with_screen.side_two.pokemon.push(defender);
        state_with_screen.side_two.side_conditions.insert(screen, 5);
        let screen_damage = calculate_damage(&state_with_screen, &state_with_screen.side_one.pokemon[0], &state_with_screen.side_two.pokemon[0], &move_data, false, 1.0);

        if normal_damage == 0 {
            Ok(0.0)
        } else {
            Ok(screen_damage as f32 / normal_damage as f32)
        }
    }

    /// Test Infiltrator ability bypassing screens
    pub fn test_infiltrator_bypass(
        &self,
        attacker_species: &str,
        defender_species: &str,
        screen: crate::core::instruction::SideCondition,
        move_name: &str,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let normal_attacker = self.create_pokemon_from_ps_data(attacker_species, None, Some(50))?;
        let infiltrator_attacker = self.create_pokemon_from_ps_data(attacker_species, Some("Infiltrator"), Some(50))?;
        let defender = self.create_pokemon_from_ps_data(defender_species, None, Some(50))?;
        let move_data = self.create_move_from_ps_data(move_name)?;

        // Test normal attacker with screen
        let mut state_normal = State::new(BattleFormat::gen9_ou());
        state_normal.side_one.pokemon.push(normal_attacker);
        state_normal.side_two.pokemon.push(defender.clone());
        state_normal.side_two.side_conditions.insert(screen, 5);
        let normal_damage = calculate_damage(&state_normal, &state_normal.side_one.pokemon[0], &state_normal.side_two.pokemon[0], &move_data, false, 1.0);

        // Test Infiltrator attacker with screen
        let mut state_infiltrator = State::new(BattleFormat::gen9_ou());
        state_infiltrator.side_one.pokemon.push(infiltrator_attacker);
        state_infiltrator.side_two.pokemon.push(defender);
        state_infiltrator.side_two.side_conditions.insert(screen, 5);
        let infiltrator_damage = calculate_damage(&state_infiltrator, &state_infiltrator.side_one.pokemon[0], &state_infiltrator.side_two.pokemon[0], &move_data, false, 1.0);

        // Infiltrator should bypass screen (deal more damage)
        Ok(infiltrator_damage > normal_damage)
    }

    /// Create a Move object from PS data (for instruction generation testing)
    pub fn create_state_move_from_ps_data(
        &self,
        move_name: &str,
    ) -> Result<Move, Box<dyn std::error::Error>> {
        let ps_move = self.ps_data.get_move_by_name(move_name)
            .ok_or_else(|| format!("Move '{}' not found in PS data", move_name))?;

        // Convert PS category string to MoveCategory enum
        let category = match ps_move.category.as_str() {
            "Physical" => MoveCategory::Physical,
            "Special" => MoveCategory::Special,
            "Status" => MoveCategory::Status,
            _ => MoveCategory::Physical, // Default fallback
        };

        // Convert target string to PSMoveTarget enum
        use crate::data::ps_types::PSMoveTarget;
        let target = match ps_move.target.as_str() {
            "normal" => PSMoveTarget::Normal,
            "self" => PSMoveTarget::Self_,
            "adjacentAlly" => PSMoveTarget::AdjacentAlly,
            "adjacentAllyOrSelf" => PSMoveTarget::AdjacentAllyOrSelf,
            "adjacentFoe" => PSMoveTarget::AdjacentFoe,
            "allAdjacentFoes" => PSMoveTarget::AllAdjacentFoes,
            "allAdjacent" => PSMoveTarget::AllAdjacent,
            "all" => PSMoveTarget::All,
            "allyTeam" => PSMoveTarget::AllyTeam,
            "allySide" => PSMoveTarget::AllySide,
            "foeSide" => PSMoveTarget::FoeSide,
            "any" => PSMoveTarget::Any,
            "randomNormal" => PSMoveTarget::RandomNormal,
            "scripted" => PSMoveTarget::Scripted,
            "allies" => PSMoveTarget::Allies,
            _ => PSMoveTarget::Normal, // Default fallback
        };

        Ok(Move::new_with_details(
            ps_move.name.clone(),
            ps_move.base_power as u8,
            ps_move.accuracy as u8,
            ps_move.move_type.clone(),
            ps_move.pp as u8,
            target,
            category,
            ps_move.priority,
        ))
    }

    /// Create a test battle state with two Pokemon and their moves
    pub fn create_test_battle(
        &self,
        attacker_species: &str,
        attacker_moves: &[&str],
        defender_species: &str,
        format: Option<BattleFormat>,
    ) -> Result<(State, Vec<MoveIndex>), Box<dyn std::error::Error>> {
        let battle_format = format.unwrap_or_else(|| BattleFormat::new("Singles".to_string(), Generation::Gen9, FormatType::Singles));
        let mut state = State::new(battle_format);

        // Create attacker
        let mut attacker = self.create_pokemon_from_ps_data(attacker_species, None, Some(50))?;
        
        // Add moves to attacker
        let mut move_indices = Vec::new();
        for (i, move_name) in attacker_moves.iter().enumerate() {
            if i >= 4 { break; } // Max 4 moves
            let move_data = self.create_state_move_from_ps_data(move_name)?;
            let move_index = match i {
                0 => MoveIndex::M0,
                1 => MoveIndex::M1,
                2 => MoveIndex::M2,
                3 => MoveIndex::M3,
                _ => unreachable!(),
            };
            attacker.add_move(move_index, move_data);
            move_indices.push(move_index);
        }

        // Create defender
        let defender = self.create_pokemon_from_ps_data(defender_species, None, Some(50))?;

        // Add Pokemon to state
        state.side_one.add_pokemon(attacker);
        state.side_one.set_active_pokemon_at_slot(0, Some(0));
        
        state.side_two.add_pokemon(defender);
        state.side_two.set_active_pokemon_at_slot(0, Some(0));

        Ok((state, move_indices))
    }

    /// Test instruction generation for a move choice
    pub fn test_instruction_generation(
        &self,
        state: &mut State,
        move_choice: MoveChoice,
        format: Option<BattleFormat>,
    ) -> Vec<StateInstructions> {
        let battle_format = format.unwrap_or_else(|| state.format.clone());
        let generator = GenerationXInstructionGenerator::new(battle_format);
        
        generator.generate_instructions(state, &move_choice, &MoveChoice::None)
    }

    /// Test that a move generates damage instructions
    pub fn verify_damage_instructions(
        &self,
        instructions: &[StateInstructions],
    ) -> bool {
        instructions.iter().any(|instr| {
            instr.instruction_list.iter().any(|i| {
                matches!(i, Instruction::PositionDamage(_) | Instruction::MultiTargetDamage(_))
            })
        })
    }

    /// Test that a move generates status instructions
    pub fn verify_status_instructions(
        &self,
        instructions: &[StateInstructions],
        expected_status: PokemonStatus,
    ) -> bool {
        instructions.iter().any(|instr| {
            instr.instruction_list.iter().any(|i| {
                matches!(i, Instruction::ApplyStatus(status_instr) 
                    if status_instr.status == expected_status)
            })
        })
    }

    /// Test that a move generates stat boost instructions
    pub fn verify_stat_boost_instructions(
        &self,
        instructions: &[StateInstructions],
        expected_stat: Stat,
        expected_boost: i8,
    ) -> bool {
        instructions.iter().any(|instr| {
            instr.instruction_list.iter().any(|i| {
                matches!(i, Instruction::BoostStats(boost_instr) 
                    if boost_instr.stat_boosts.get(&expected_stat) == Some(&expected_boost))
            })
        })
    }

    /// Test critical hit branching - verifies we have multiple instruction sets with different damage
    pub fn verify_critical_hit_branching(
        &self,
        instructions: &[StateInstructions],
    ) -> bool {
        if instructions.len() < 2 {
            return false;
        }

        // Extract damage amounts
        let mut damage_amounts = Vec::new();
        for instr_set in instructions {
            for instr in &instr_set.instruction_list {
                if let Instruction::PositionDamage(damage_instr) = instr {
                    damage_amounts.push(damage_instr.damage_amount);
                }
            }
        }

        // Should have at least 2 different damage amounts (normal + crit)
        damage_amounts.len() >= 2 && {
            damage_amounts.sort();
            damage_amounts.dedup();
            damage_amounts.len() >= 2
        }
    }

    /// Test probability distribution - verifies all probabilities sum to 100%
    pub fn verify_probability_distribution(
        &self,
        instructions: &[StateInstructions],
    ) -> bool {
        let total_percentage: f32 = instructions.iter().map(|i| i.percentage).sum();
        (total_percentage - 100.0).abs() < 0.001
    }

    /// Test type immunity prevents move effects
    pub fn test_type_immunity_blocks_status(
        &self,
        attacker_species: &str,
        move_name: &str,
        defender_species: &str,
        defender_types: &[&str],
        expected_status: PokemonStatus,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let (mut state, move_indices) = self.create_test_battle(
            attacker_species,
            &[move_name],
            defender_species,
            None,
        )?;

        // Override defender types for immunity testing
        if let Some(defender) = state.side_two.get_active_pokemon_at_slot_mut(0) {
            defender.types = defender_types.iter().map(|&t| t.to_string()).collect();
        }

        let move_choice = MoveChoice::new_move(
            move_indices[0],
            vec![BattlePosition::new(SideReference::SideTwo, 0)],
        );

        let instructions = self.test_instruction_generation(&mut state, move_choice, None);
        
        // If immune, should NOT have the expected status instruction
        Ok(!self.verify_status_instructions(&instructions, expected_status))
    }

    /// Test ability immunity blocks damage
    pub fn test_ability_immunity_blocks_damage(
        &self,
        attacker_species: &str,
        move_name: &str,
        defender_species: &str,
        defender_ability: &str,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let (mut state, move_indices) = self.create_test_battle(
            attacker_species,
            &[move_name],
            defender_species,
            None,
        )?;

        // Set defender ability
        if let Some(defender) = state.side_two.get_active_pokemon_at_slot_mut(0) {
            defender.ability = normalize_name(defender_ability);
        }

        let move_choice = MoveChoice::new_move(
            move_indices[0],
            vec![BattlePosition::new(SideReference::SideTwo, 0)],
        );

        let instructions = self.test_instruction_generation(&mut state, move_choice, None);
        
        // If immune, should NOT have damage instructions
        Ok(!self.verify_damage_instructions(&instructions))
    }
}

