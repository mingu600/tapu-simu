//! # Battle Instruction System
//! 
//! This module defines the instruction system for the V2 engine.
//! All instructions are position-aware and designed for multi-format support.

use crate::battle_format::{BattlePosition, BattleFormat};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a single battle instruction that modifies the game state
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Instruction {
    /// Deal damage to a specific position
    PositionDamage(PositionDamageInstruction),
    /// Heal a specific position
    PositionHeal(PositionHealInstruction),
    /// Deal damage to multiple positions simultaneously
    MultiTargetDamage(MultiTargetDamageInstruction),
    /// Apply a status condition to a position
    ApplyStatus(ApplyStatusInstruction),
    /// Remove a status condition from a position
    RemoveStatus(RemoveStatusInstruction),
    /// Boost stats at a position
    BoostStats(BoostStatsInstruction),
    /// Switch Pokemon at a position
    SwitchPokemon(SwitchInstruction),
    /// Apply volatile status to a position
    ApplyVolatileStatus(ApplyVolatileStatusInstruction),
    /// Remove volatile status from a position
    RemoveVolatileStatus(RemoveVolatileStatusInstruction),
    /// Change weather conditions
    ChangeWeather(ChangeWeatherInstruction),
    /// Change terrain conditions
    ChangeTerrain(ChangeTerrainInstruction),
    /// Apply side condition
    ApplySideCondition(ApplySideConditionInstruction),
    /// Remove side condition
    RemoveSideCondition(RemoveSideConditionInstruction),
}

impl Instruction {
    /// Returns all positions affected by this instruction
    pub fn affected_positions(&self) -> Vec<BattlePosition> {
        match self {
            Instruction::PositionDamage(instr) => vec![instr.target_position],
            Instruction::PositionHeal(instr) => vec![instr.target_position],
            Instruction::MultiTargetDamage(instr) => {
                instr.target_damages.iter().map(|(pos, _)| *pos).collect()
            }
            Instruction::ApplyStatus(instr) => vec![instr.target_position],
            Instruction::RemoveStatus(instr) => vec![instr.target_position],
            Instruction::BoostStats(instr) => vec![instr.target_position],
            Instruction::SwitchPokemon(instr) => vec![instr.position],
            Instruction::ApplyVolatileStatus(instr) => vec![instr.target_position],
            Instruction::RemoveVolatileStatus(instr) => vec![instr.target_position],
            Instruction::ChangeWeather(_) => vec![], // Weather affects all
            Instruction::ChangeTerrain(_) => vec![], // Terrain affects all
            Instruction::ApplySideCondition(instr) => {
                // Side conditions affect all positions on a side
                // This is a simplified representation
                vec![] // TODO: Implement side-wide position tracking
            }
            Instruction::RemoveSideCondition(instr) => vec![],
        }
    }
}

/// Deal damage to a specific battle position
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PositionDamageInstruction {
    pub target_position: BattlePosition,
    pub damage_amount: i16,
}

/// Heal a specific battle position
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PositionHealInstruction {
    pub target_position: BattlePosition,
    pub heal_amount: i16,
}

/// Deal damage to multiple positions with potentially different amounts
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MultiTargetDamageInstruction {
    pub target_damages: Vec<(BattlePosition, i16)>,
}

/// Apply a status condition to a position
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApplyStatusInstruction {
    pub target_position: BattlePosition,
    pub status: PokemonStatus,
}

/// Remove a status condition from a position
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RemoveStatusInstruction {
    pub target_position: BattlePosition,
    pub status: PokemonStatus,
}

/// Boost stats at a position
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BoostStatsInstruction {
    pub target_position: BattlePosition,
    pub stat_boosts: HashMap<Stat, i8>,
}

/// Switch Pokemon at a position
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SwitchInstruction {
    pub position: BattlePosition,
    pub pokemon_index: usize,
}

/// Apply volatile status to a position
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApplyVolatileStatusInstruction {
    pub target_position: BattlePosition,
    pub volatile_status: VolatileStatus,
    pub duration: Option<u8>,
}

/// Remove volatile status from a position
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RemoveVolatileStatusInstruction {
    pub target_position: BattlePosition,
    pub volatile_status: VolatileStatus,
}

/// Change weather conditions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChangeWeatherInstruction {
    pub weather: Weather,
    pub duration: Option<u8>,
}

/// Change terrain conditions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChangeTerrainInstruction {
    pub terrain: Terrain,
    pub duration: Option<u8>,
}

/// Apply side condition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApplySideConditionInstruction {
    pub side: crate::battle_format::SideReference,
    pub condition: SideCondition,
    pub duration: Option<u8>,
}

/// Remove side condition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RemoveSideConditionInstruction {
    pub side: crate::battle_format::SideReference,
    pub condition: SideCondition,
}

/// A collection of instructions with probability and affected positions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StateInstructions {
    /// Probability of this instruction set occurring (0.0 to 100.0)
    pub percentage: f32,
    /// List of instructions to execute
    pub instruction_list: Vec<Instruction>,
    /// All positions affected by these instructions
    pub affected_positions: Vec<BattlePosition>,
}

impl StateInstructions {
    /// Create a new StateInstructions with calculated affected positions
    pub fn new(percentage: f32, instruction_list: Vec<Instruction>) -> Self {
        let mut affected_positions = Vec::new();
        for instruction in &instruction_list {
            affected_positions.extend(instruction.affected_positions());
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

    /// Create empty instructions
    pub fn empty() -> Self {
        Self {
            percentage: 100.0,
            instruction_list: Vec::new(),
            affected_positions: Vec::new(),
        }
    }
}

impl Default for StateInstructions {
    fn default() -> Self {
        Self::empty()
    }
}

/// Pokemon status conditions (V1 compatible naming)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PokemonStatus {
    NONE,
    BURN,
    FREEZE,
    PARALYZE,
    POISON,
    TOXIC,
    SLEEP,
}

/// Pokemon stats that can be boosted/lowered
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Stat {
    Hp,
    Attack,
    Defense,
    SpecialAttack,
    SpecialDefense,
    Speed,
    Accuracy,
    Evasion,
}

/// Volatile status conditions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VolatileStatus {
    Confusion,
    Flinch,
    Substitute,
    LeechSeed,
    Curse,
    Nightmare,
    Attract,
    Torment,
    Disable,
    Encore,
    Taunt,
    HelpingHand,
    MagicCoat,
    FollowMe,
    Protect,
    Endure,
}

/// Weather conditions (V1 compatible naming)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Weather {
    NONE,
    SUN,
    RAIN,
    SAND,
    HAIL,
    SNOW,
    HARSHSUN,
    HEAVYRAIN,
}

/// Terrain conditions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Terrain {
    NONE,
    ELECTRICTERRAIN,
    GRASSYTERRAIN,
    MISTYTERRAIN,
    PSYCHICTERRAIN,
}

/// Side conditions that affect an entire side
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SideCondition {
    Reflect,
    LightScreen,
    Mist,
    Safeguard,
    Spikes,
    ToxicSpikes,
    StealthRock,
    StickyWeb,
    TailWind,
    WideGuard,
    QuickGuard,
    AuroraVeil,
}

/// The main instruction generator for the V2 engine
pub struct InstructionGenerator {
    format: BattleFormat,
}

impl InstructionGenerator {
    /// Create a new instruction generator for the specified format
    pub fn new(format: BattleFormat) -> Self {
        Self { format }
    }

    /// Get the battle format this generator is configured for
    pub fn format(&self) -> &BattleFormat {
        &self.format
    }

    /// Generate instructions from move choices
    /// This is a placeholder - will be implemented with actual move mechanics
    pub fn generate_instructions(
        &self,
        _state: &crate::state::State,
        _move1: &crate::move_choice::MoveChoice,
        _move2: &crate::move_choice::MoveChoice,
    ) -> Vec<StateInstructions> {
        // Placeholder implementation
        vec![StateInstructions::empty()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::battle_format::{BattlePosition, SideReference};

    #[test]
    fn test_state_instructions_creation() {
        let damage_instr = Instruction::PositionDamage(PositionDamageInstruction {
            target_position: BattlePosition::new(SideReference::SideOne, 0),
            damage_amount: 50,
        });

        let heal_instr = Instruction::PositionHeal(PositionHealInstruction {
            target_position: BattlePosition::new(SideReference::SideTwo, 0),
            heal_amount: 25,
        });

        let instructions = StateInstructions::new(100.0, vec![damage_instr, heal_instr]);

        assert_eq!(instructions.percentage, 100.0);
        assert_eq!(instructions.instruction_list.len(), 2);
        assert_eq!(instructions.affected_positions.len(), 2);
        assert!(instructions.affected_positions.contains(&BattlePosition::new(SideReference::SideOne, 0)));
        assert!(instructions.affected_positions.contains(&BattlePosition::new(SideReference::SideTwo, 0)));
    }

    #[test]
    fn test_multi_target_damage() {
        let multi_damage = Instruction::MultiTargetDamage(MultiTargetDamageInstruction {
            target_damages: vec![
                (BattlePosition::new(SideReference::SideOne, 0), 40),
                (BattlePosition::new(SideReference::SideOne, 1), 40),
            ],
        });

        let affected = multi_damage.affected_positions();
        assert_eq!(affected.len(), 2);
        assert!(affected.contains(&BattlePosition::new(SideReference::SideOne, 0)));
        assert!(affected.contains(&BattlePosition::new(SideReference::SideOne, 1)));
    }
}