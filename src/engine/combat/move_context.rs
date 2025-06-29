//! # Move Execution Context
//!
//! This module provides a unified context structure for move effect execution,
//! consolidating all the different function signatures into a single interface.

use crate::core::battle_format::BattlePosition;
use crate::core::battle_state::BattleState;
use crate::core::instructions::BattleInstructions;
use crate::data::{GameDataRepository, showdown_types::MoveData};
use crate::generation::GenerationMechanics;
use super::moves::MoveContext;

/// Unified context for move effect execution
/// This replaces the 5 different function signatures currently used
pub struct MoveExecutionContext<'a> {
    pub state: &'a BattleState,
    pub move_data: &'a MoveData,
    pub user_position: BattlePosition,
    pub target_positions: &'a [BattlePosition],
    pub generation: &'a GenerationMechanics,
    pub move_context: &'a MoveContext,
    pub repository: &'a GameDataRepository,
    pub branch_on_damage: bool,
}

/// Unified move effect function signature
pub type MoveEffectFn = Box<dyn Fn(&mut MoveExecutionContext) -> Vec<BattleInstructions> + Send + Sync>;

impl<'a> MoveExecutionContext<'a> {
    /// Create a new move execution context
    pub fn new(
        state: &'a BattleState,
        move_data: &'a MoveData,
        user_position: BattlePosition,
        target_positions: &'a [BattlePosition],
        generation: &'a GenerationMechanics,
        move_context: &'a MoveContext,
        repository: &'a GameDataRepository,
        branch_on_damage: bool,
    ) -> Self {
        Self {
            state,
            move_data,
            user_position,
            target_positions,
            generation,
            move_context,
            repository,
            branch_on_damage,
        }
    }

    /// Get the user Pokemon
    pub fn user(&self) -> Option<&crate::core::battle_state::Pokemon> {
        self.state.get_pokemon_at_position(self.user_position)
    }

    /// Get a target Pokemon by position
    pub fn target(&self, position: BattlePosition) -> Option<&crate::core::battle_state::Pokemon> {
        self.state.get_pokemon_at_position(position)
    }

    /// Get all target Pokemon
    pub fn targets(&self) -> Vec<Option<&crate::core::battle_state::Pokemon>> {
        self.target_positions
            .iter()
            .map(|&pos| self.state.get_pokemon_at_position(pos))
            .collect()
    }
}

/// Adapter functions for converting existing move signatures to the unified signature

/// Adapt a simple move effect function
pub fn adapt_simple_move(
    original: fn(&BattleState, BattlePosition, &[BattlePosition], &GenerationMechanics) -> Vec<BattleInstructions>
) -> MoveEffectFn {
    Box::new(move |ctx: &mut MoveExecutionContext| {
        original(ctx.state, ctx.user_position, ctx.target_positions, ctx.generation)
    })
}

/// Adapt an extended move effect function (with move data)
pub fn adapt_extended_move(
    original: fn(&BattleState, &MoveData, BattlePosition, &[BattlePosition], &GenerationMechanics) -> Vec<BattleInstructions>
) -> MoveEffectFn {
    Box::new(move |ctx: &mut MoveExecutionContext| {
        original(ctx.state, ctx.move_data, ctx.user_position, ctx.target_positions, ctx.generation)
    })
}

/// Adapt a variable power move effect function (with branching)
pub fn adapt_variable_power_move(
    original: fn(&BattleState, &MoveData, BattlePosition, &[BattlePosition], &GenerationMechanics, bool) -> Vec<BattleInstructions>
) -> MoveEffectFn {
    Box::new(move |ctx: &mut MoveExecutionContext| {
        original(ctx.state, ctx.move_data, ctx.user_position, ctx.target_positions, ctx.generation, ctx.branch_on_damage)
    })
}

/// Adapt a context-aware move effect function
pub fn adapt_context_aware_move(
    original: fn(&BattleState, &MoveData, BattlePosition, &[BattlePosition], &GenerationMechanics, &MoveContext, bool) -> Vec<BattleInstructions>
) -> MoveEffectFn {
    Box::new(move |ctx: &mut MoveExecutionContext| {
        original(ctx.state, ctx.move_data, ctx.user_position, ctx.target_positions, ctx.generation, ctx.move_context, ctx.branch_on_damage)
    })
}

/// Adapt a repository-aware move effect function
pub fn adapt_repository_aware_move(
    original: fn(&BattleState, &MoveData, BattlePosition, &[BattlePosition], &GenerationMechanics, &MoveContext, &GameDataRepository, bool) -> Vec<BattleInstructions>
) -> MoveEffectFn {
    Box::new(move |ctx: &mut MoveExecutionContext| {
        original(ctx.state, ctx.move_data, ctx.user_position, ctx.target_positions, ctx.generation, ctx.move_context, ctx.repository, ctx.branch_on_damage)
    })
}