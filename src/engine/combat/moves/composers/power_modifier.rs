//! # Power Modifier Composer
//!
//! This module provides a builder pattern for composing power modifier calculations
//! used by variable power moves. It allows chaining different modifiers together
//! and eliminates duplicate code across different move implementations.

use crate::core::battle_format::BattlePosition;
use crate::core::battle_state::BattleState;
use crate::core::instructions::{Weather, PokemonStatus};
use crate::engine::combat::move_context::MoveExecutionContext;
use crate::engine::combat::moves::apply_generic_effects;
use crate::core::instructions::BattleInstructions;
use crate::data::showdown_types::MoveData;

/// Builder for composing power modifier calculations
pub struct PowerModifierBuilder {
    base_calculation: Box<dyn Fn(&BattleState, BattlePosition, &[BattlePosition]) -> f32 + Send + Sync>,
    modifiers: Vec<Box<dyn Fn(&BattleState, BattlePosition, &[BattlePosition], f32) -> f32 + Send + Sync>>,
}

impl PowerModifierBuilder {
    /// Create a new power modifier builder with a base calculation
    pub fn new<F>(calc: F) -> Self 
    where 
        F: Fn(&BattleState, BattlePosition, &[BattlePosition]) -> f32 + Send + Sync + 'static 
    {
        Self {
            base_calculation: Box::new(calc),
            modifiers: Vec::new(),
        }
    }

    /// Add a weather-based power boost
    pub fn with_weather_boost(mut self, weather: Weather, boost: f32) -> Self {
        let modifier = move |state: &BattleState, _user: BattlePosition, _targets: &[BattlePosition], current_power: f32| -> f32 {
            if state.field.weather.condition == weather {
                current_power * boost
            } else {
                current_power
            }
        };
        self.modifiers.push(Box::new(modifier));
        self
    }

    /// Add HP-based power scaling with thresholds
    /// Thresholds are (hp_ratio, power_multiplier) pairs
    pub fn with_hp_threshold(mut self, thresholds: Vec<(f32, f32)>) -> Self {
        let modifier = move |state: &BattleState, user: BattlePosition, _targets: &[BattlePosition], _current_power: f32| -> f32 {
            if let Some(user_pokemon) = state.get_pokemon_at_position(user) {
                let hp_ratio = user_pokemon.hp as f32 / user_pokemon.max_hp as f32;
                
                // Find the appropriate threshold (sorted by hp_ratio descending)
                for &(threshold, power) in &thresholds {
                    if hp_ratio >= threshold {
                        return power;
                    }
                }
                
                // Default to last threshold if none match
                thresholds.last().map(|(_, power)| *power).unwrap_or(1.0)
            } else {
                1.0
            }
        };
        self.modifiers.push(Box::new(modifier));
        self
    }

    /// Add status-based power boost
    pub fn with_status_boost(mut self, boost: f32) -> Self {
        let modifier = move |state: &BattleState, user: BattlePosition, _targets: &[BattlePosition], current_power: f32| -> f32 {
            if let Some(user_pokemon) = state.get_pokemon_at_position(user) {
                if user_pokemon.status != PokemonStatus::None {
                    current_power * boost
                } else {
                    current_power
                }
            } else {
                current_power
            }
        };
        self.modifiers.push(Box::new(modifier));
        self
    }

    /// Add weight-based power calculation
    pub fn with_weight_calculation<F>(mut self, weight_calc: F) -> Self 
    where 
        F: Fn(f32) -> f32 + Send + Sync + 'static // weight -> power
    {
        let modifier = move |state: &BattleState, _user: BattlePosition, targets: &[BattlePosition], _current_power: f32| -> f32 {
            if let Some(target_pos) = targets.first() {
                if let Some(target_pokemon) = state.get_pokemon_at_position(*target_pos) {
                    // TODO: Get weight from Pokemon data
                    // For now, use a default weight and let specific moves override
                    let weight = 100.0; // Default weight in kg
                    return weight_calc(weight);
                }
            }
            1.0
        };
        self.modifiers.push(Box::new(modifier));
        self
    }

    /// Add speed-based power calculation
    pub fn with_speed_calculation<F>(mut self, speed_calc: F) -> Self
    where
        F: Fn(i16, i16) -> f32 + Send + Sync + 'static // (user_speed, target_speed) -> power
    {
        let modifier = move |state: &BattleState, user: BattlePosition, targets: &[BattlePosition], _current_power: f32| -> f32 {
            if let Some(target_pos) = targets.first() {
                if let (Some(user_pokemon), Some(target_pokemon)) = (
                    state.get_pokemon_at_position(user),
                    state.get_pokemon_at_position(*target_pos)
                ) {
                    return speed_calc(user_pokemon.stats.speed, target_pokemon.stats.speed);
                }
            }
            1.0
        };
        self.modifiers.push(Box::new(modifier));
        self
    }

    /// Build the final move effect function
    pub fn build(self) -> Box<dyn Fn(&mut MoveExecutionContext) -> Vec<BattleInstructions> + Send + Sync> {
        Box::new(move |ctx: &mut MoveExecutionContext| -> Vec<BattleInstructions> {
            // Calculate base power
            let mut power = (self.base_calculation)(ctx.state, ctx.user_position, ctx.target_positions);
            
            // Apply all modifiers in sequence
            for modifier in &self.modifiers {
                power = modifier(ctx.state, ctx.user_position, ctx.target_positions, power);
            }
            
            // Create modified move data with the calculated power
            let modified_move_data = MoveData {
                base_power: power.max(1.0) as u16, // Minimum 1 power
                ..ctx.move_data.clone()
            };
            
            // Apply generic effects with the modified power
            apply_generic_effects(
                ctx.state,
                &modified_move_data,
                ctx.user_position,
                ctx.target_positions,
                ctx.generation,
                ctx.branch_on_damage,
            )
        })
    }
}

/// Convenience functions for common power calculations

/// Create a simple HP-based power calculation (used by Eruption, Water Spout, etc.)
pub fn hp_based_power(max_power: f32) -> PowerModifierBuilder {
    PowerModifierBuilder::new(move |state, user_pos, _| {
        if let Some(user) = state.get_pokemon_at_position(user_pos) {
            let hp_ratio = user.hp as f32 / user.max_hp as f32;
            max_power * hp_ratio
        } else {
            1.0
        }
    })
}

/// Create a weight-based power calculation (used by Low Kick, Grass Knot, etc.)
pub fn weight_based_power() -> PowerModifierBuilder {
    PowerModifierBuilder::new(|_state, _user_pos, _targets| 1.0)
        .with_weight_calculation(|weight| {
            match weight {
                w if w < 10.0 => 20.0,
                w if w < 25.0 => 40.0,
                w if w < 50.0 => 60.0,
                w if w < 100.0 => 80.0,
                w if w < 200.0 => 100.0,
                _ => 120.0,
            }
        })
}

/// Create a speed-based power calculation (used by Electro Ball)
pub fn speed_based_power() -> PowerModifierBuilder {
    PowerModifierBuilder::new(|_state, _user_pos, _targets| 1.0)
        .with_speed_calculation(|user_speed, target_speed| {
            if target_speed <= 0 {
                150.0 // Maximum power if target speed is 0 or negative
            } else {
                let ratio = user_speed as f32 / target_speed as f32;
                (25.0 * ratio).min(150.0).max(1.0)
            }
        })
}

/// Create a reverse speed-based power calculation (used by Gyro Ball)
pub fn reverse_speed_based_power() -> PowerModifierBuilder {
    PowerModifierBuilder::new(|_state, _user_pos, _targets| 1.0)
        .with_speed_calculation(|user_speed, target_speed| {
            if user_speed <= 0 {
                1.0 // Minimum power if user speed is 0 or negative
            } else {
                let ratio = target_speed as f32 / user_speed as f32;
                (25.0 * ratio).min(150.0).max(1.0)
            }
        })
}