//! # Stats-Related Instructions
//! 
//! Instructions that affect Pokemon stats: stat boosts/drops,
//! raw stat modifications, etc.

use crate::core::battle_format::BattlePosition;
use crate::types::{Stat, StatBoostArray};
use serde::{Deserialize, Serialize};

// Use HashMap for compatibility with tests, convert to StatBoostArray during processing
pub type StatBoosts = std::collections::HashMap<Stat, i8>;



/// Stats-related instruction types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StatsInstruction {
    /// Boost or drop stats at a position
    BoostStats {
        target: BattlePosition,
        stat_changes: StatBoosts,
        previous_boosts: StatBoosts,
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
        previous_boosts: StatBoostArray,
    },
    /// Copy stat boosts from another Pokemon
    CopyBoosts {
        target: BattlePosition,
        source: BattlePosition,
        stats_to_copy: Vec<Stat>,
        previous_boosts: StatBoostArray,
    },
    /// Swap stat boosts between two Pokemon
    SwapBoosts {
        target1: BattlePosition,
        target2: BattlePosition,
        stats_to_swap: Vec<Stat>,
        previous_boosts1: StatBoostArray,
        previous_boosts2: StatBoostArray,
    },
    /// Invert stat boosts (positive becomes negative and vice versa)
    InvertBoosts {
        target: BattlePosition,
        stats_to_invert: Vec<Stat>,
        previous_boosts: StatBoostArray,
    },
}

impl StatsInstruction {
    /// Create a BoostStats instruction
    pub fn boost_stats(
        target: BattlePosition,
        stat_changes: std::collections::HashMap<Stat, i8>,
        previous_boosts: std::collections::HashMap<Stat, i8>,
    ) -> Self {
        Self::BoostStats {
            target,
            stat_changes,
            previous_boosts,
        }
    }

    /// Get stat changes as StatBoostArray for efficient processing
    pub fn get_stat_changes_array(&self) -> Option<StatBoostArray> {
        match self {
            StatsInstruction::BoostStats { stat_changes, .. } => {
                Some(StatBoostArray::from_hashmap(stat_changes))
            }
            _ => None,
        }
    }

    /// Get previous boosts as StatBoostArray for efficient processing
    pub fn get_previous_boosts_array(&self) -> Option<StatBoostArray> {
        match self {
            StatsInstruction::BoostStats { previous_boosts, .. } => {
                Some(StatBoostArray::from_hashmap(previous_boosts))
            }
            _ => None,
        }
    }

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