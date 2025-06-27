//! # Domain-Grouped Battle Instructions
//! 
//! This module provides the modernized instruction system with instructions
//! grouped by domain for better maintainability and organization.

pub mod pokemon;
pub mod field;
pub mod status;
pub mod stats;

pub use pokemon::{PokemonInstruction, MoveCategory};
pub use field::{FieldInstruction, SideCondition};
pub use status::{StatusInstruction};
pub use stats::{StatsInstruction};

// Re-export the moved enums from types for convenience
pub use crate::types::{Weather, Terrain, PokemonStatus, VolatileStatus, Stat};

use crate::core::battle_format::BattlePosition;
use serde::{Deserialize, Serialize};

/// Modern domain-grouped battle instruction system
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BattleInstruction {
    /// Pokemon-related instructions (damage, healing, fainting, switching)
    Pokemon(PokemonInstruction),
    /// Field-related instructions (weather, terrain, global effects)
    Field(FieldInstruction),
    /// Status-related instructions (status conditions, volatile statuses)
    Status(StatusInstruction),
    /// Stats-related instructions (boosts, raw stat changes)
    Stats(StatsInstruction),
}

impl BattleInstruction {
    /// Returns all positions affected by this instruction
    pub fn affected_positions(&self, format: &crate::core::battle_format::BattleFormat) -> Vec<BattlePosition> {
        match self {
            BattleInstruction::Pokemon(instr) => instr.affected_positions(),
            BattleInstruction::Field(instr) => instr.affected_positions(format),
            BattleInstruction::Status(instr) => instr.affected_positions(),
            BattleInstruction::Stats(instr) => instr.affected_positions(),
        }
    }

    /// Whether this instruction can be undone
    pub fn is_undoable(&self) -> bool {
        match self {
            BattleInstruction::Pokemon(instr) => instr.is_undoable(),
            BattleInstruction::Field(instr) => instr.is_undoable(),
            BattleInstruction::Status(instr) => instr.is_undoable(),
            BattleInstruction::Stats(instr) => instr.is_undoable(),
        }
    }
}

/// A collection of modern battle instructions with probability and affected positions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BattleInstructions {
    /// Probability of this instruction set occurring (0.0 to 100.0)
    pub percentage: f32,
    /// List of modern battle instructions to execute
    pub instruction_list: Vec<BattleInstruction>,
    /// All positions affected by these instructions
    pub affected_positions: Vec<BattlePosition>,
}

impl BattleInstructions {
    /// Create a new BattleInstructions with calculated affected positions
    pub fn new_with_format(percentage: f32, instruction_list: Vec<BattleInstruction>, format: &crate::core::battle_format::BattleFormat) -> Self {
        let mut affected_positions = Vec::new();
        for instruction in &instruction_list {
            affected_positions.extend(instruction.affected_positions(format));
        }
        
        // Remove duplicates and sort for consistency
        affected_positions.sort();
        affected_positions.dedup();

        Self {
            percentage,
            instruction_list,
            affected_positions,
        }
    }

    /// Create a new BattleInstructions with manually specified affected positions
    /// This is useful when you know the affected positions without needing format calculation
    pub fn new_with_positions(percentage: f32, instruction_list: Vec<BattleInstruction>, affected_positions: Vec<BattlePosition>) -> Self {
        Self {
            percentage,
            instruction_list,
            affected_positions,
        }
    }

    /// Create a new BattleInstructions (backward compatibility - assumes empty affected positions)
    /// Use new_with_format or new_with_positions for better position tracking
    pub fn new(percentage: f32, instruction_list: Vec<BattleInstruction>) -> Self {
        Self {
            percentage,
            instruction_list,
            affected_positions: Vec::new(),
        }
    }
}

