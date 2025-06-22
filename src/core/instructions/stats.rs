//! # Stats-Related Instructions
//! 
//! Instructions that affect Pokemon stats: stat boosts/drops,
//! raw stat modifications, etc.

use crate::core::battle_format::BattlePosition;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

impl From<u8> for Stat {
    fn from(value: u8) -> Self {
        match value {
            0 => Stat::Hp,
            1 => Stat::Attack,
            2 => Stat::Defense,
            3 => Stat::SpecialAttack,
            4 => Stat::SpecialDefense,
            5 => Stat::Speed,
            6 => Stat::Accuracy,
            7 => Stat::Evasion,
            _ => Stat::Hp, // Default fallback
        }
    }
}

/// Stats-related instruction types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StatsInstruction {
    /// Boost or drop stats at a position
    BoostStats {
        target: BattlePosition,
        stat_changes: HashMap<Stat, i8>,
        previous_boosts: HashMap<Stat, i8>,
    },
    /// Change raw attack stat (not boosts)
    ChangeAttack {
        target: BattlePosition,
        new_value: i16,
        previous_value: i16,
    },
    /// Change raw defense stat (not boosts)
    ChangeDefense {
        target: BattlePosition,
        new_value: i16,
        previous_value: i16,
    },
    /// Change raw special attack stat (not boosts)
    ChangeSpecialAttack {
        target: BattlePosition,
        new_value: i16,
        previous_value: i16,
    },
    /// Change raw special defense stat (not boosts)
    ChangeSpecialDefense {
        target: BattlePosition,
        new_value: i16,
        previous_value: i16,
    },
    /// Change raw speed stat (not boosts)
    ChangeSpeed {
        target: BattlePosition,
        new_value: i16,
        previous_value: i16,
    },
    /// Clear all stat boosts
    ClearBoosts {
        target: BattlePosition,
        previous_boosts: HashMap<Stat, i8>,
    },
    /// Copy stat boosts from another Pokemon
    CopyBoosts {
        target: BattlePosition,
        source: BattlePosition,
        stats_to_copy: Vec<Stat>,
        previous_boosts: HashMap<Stat, i8>,
    },
    /// Swap stat boosts between two Pokemon
    SwapBoosts {
        target1: BattlePosition,
        target2: BattlePosition,
        stats_to_swap: Vec<Stat>,
        previous_boosts1: HashMap<Stat, i8>,
        previous_boosts2: HashMap<Stat, i8>,
    },
    /// Invert stat boosts (positive becomes negative and vice versa)
    InvertBoosts {
        target: BattlePosition,
        stats_to_invert: Vec<Stat>,
        previous_boosts: HashMap<Stat, i8>,
    },
}

impl StatsInstruction {
    /// Returns all positions affected by this instruction
    pub fn affected_positions(&self) -> Vec<BattlePosition> {
        match self {
            StatsInstruction::BoostStats { target, .. } => vec![*target],
            StatsInstruction::ChangeAttack { target, .. } => vec![*target],
            StatsInstruction::ChangeDefense { target, .. } => vec![*target],
            StatsInstruction::ChangeSpecialAttack { target, .. } => vec![*target],
            StatsInstruction::ChangeSpecialDefense { target, .. } => vec![*target],
            StatsInstruction::ChangeSpeed { target, .. } => vec![*target],
            StatsInstruction::ClearBoosts { target, .. } => vec![*target],
            StatsInstruction::CopyBoosts { target, source, .. } => vec![*target, *source],
            StatsInstruction::SwapBoosts { target1, target2, .. } => vec![*target1, *target2],
            StatsInstruction::InvertBoosts { target, .. } => vec![*target],
        }
    }

    /// Whether this instruction can be undone
    pub fn is_undoable(&self) -> bool {
        match self {
            // All stats instructions store previous state for undo
            StatsInstruction::BoostStats { .. } => true,
            StatsInstruction::ChangeAttack { .. } => true,
            StatsInstruction::ChangeDefense { .. } => true,
            StatsInstruction::ChangeSpecialAttack { .. } => true,
            StatsInstruction::ChangeSpecialDefense { .. } => true,
            StatsInstruction::ChangeSpeed { .. } => true,
            StatsInstruction::ClearBoosts { .. } => true,
            StatsInstruction::CopyBoosts { .. } => true,
            StatsInstruction::SwapBoosts { .. } => true,
            StatsInstruction::InvertBoosts { .. } => true,
        }
    }
}