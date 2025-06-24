//! # Status-Related Instructions
//! 
//! Instructions that affect status conditions: applying/removing status,
//! volatile statuses, sleep/rest mechanics, etc.

use crate::core::battle_format::BattlePosition;
use crate::core::move_choice::MoveIndex;
use serde::{Deserialize, Serialize};

/// Pokemon status conditions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PokemonStatus {
    None,
    Burn,
    Freeze,
    Paralysis,
    Poison,
    BadlyPoisoned, // Toxic/Badly Poisoned
    Sleep,
}

impl From<u8> for PokemonStatus {
    fn from(value: u8) -> Self {
        match value {
            0 => PokemonStatus::None,
            1 => PokemonStatus::Burn,
            2 => PokemonStatus::Freeze,
            3 => PokemonStatus::Paralysis,
            4 => PokemonStatus::Poison,
            5 => PokemonStatus::BadlyPoisoned,
            6 => PokemonStatus::Sleep,
            _ => PokemonStatus::None, // Default fallback
        }
    }
}

impl Default for PokemonStatus {
    fn default() -> Self {
        PokemonStatus::None
    }
}

/// Volatile status conditions (comprehensive coverage)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VolatileStatus {
    // Basic volatile statuses
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
    
    // Extended volatile statuses
    AquaRing,
    Autotomize,
    BanefulBunker,
    Bide,
    Bounce,
    BurningBulwark,
    Charge,
    DefenseCurl,
    DestinyBond,
    Dig,
    Dive,
    Electrify,
    Electroshot,
    Embargo,
    FlashFire,
    Fly,
    FocusEnergy,
    Foresight,
    FreezeeShock,
    GastroAcid,
    Geomancy,
    GlaiveRush,
    Grudge,
    HealBlock,
    IceBurn,
    Imprison,
    Ingrain,
    KingsShield,
    LaserFocus,
    LightScreen,
    LockedMove,
    MagnetRise,
    MaxGuard,
    MeteorBeam,
    Minimize,
    MiracleEye,
    MustRecharge,
    NoRetreat,
    Octolock,
    PartiallyTrapped,
    Perish1,
    Perish2,
    Perish3,
    PowerTrick,
    Protect2,
    Rage,
    Reflect,
    Roost,
    Safeguard,
    SkyDrop,
    SmackDown,
    SpikesL1,
    SpikesL2,
    SpikesL3,
    StealthRock,
    Stockpile1,
    Stockpile2,
    Stockpile3,
    TelekineticMove,
    ToxicSpikes,
    Transform,
    TwoTurnMove,
    Uproar,
    WishHealing,
    Yawn,
    Telekinesis,
    MustSwitch,
    MicleBoost,
    CustapBoost,
}

impl From<u8> for VolatileStatus {
    fn from(value: u8) -> Self {
        match value {
            0 => VolatileStatus::Confusion,
            1 => VolatileStatus::Flinch,
            2 => VolatileStatus::Substitute,
            3 => VolatileStatus::LeechSeed,
            4 => VolatileStatus::Curse,
            5 => VolatileStatus::Nightmare,
            6 => VolatileStatus::Attract,
            7 => VolatileStatus::Torment,
            8 => VolatileStatus::Disable,
            9 => VolatileStatus::Encore,
            10 => VolatileStatus::Taunt,
            11 => VolatileStatus::HelpingHand,
            12 => VolatileStatus::MagicCoat,
            13 => VolatileStatus::FollowMe,
            14 => VolatileStatus::Protect,
            15 => VolatileStatus::Endure,
            _ => VolatileStatus::Confusion, // Default fallback
        }
    }
}

/// Status-related instruction types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StatusInstruction {
    /// Apply a status condition to a Pokemon
    Apply {
        target: BattlePosition,
        status: PokemonStatus,
        duration: Option<u8>,
        previous_status: Option<PokemonStatus>,
        previous_duration: Option<u8>,
    },
    /// Remove a status condition from a Pokemon
    Remove {
        target: BattlePosition,
        status: PokemonStatus,
        previous_duration: Option<u8>,
    },
    /// Change status condition duration
    ChangeDuration {
        target: BattlePosition,
        status: PokemonStatus,
        new_duration: Option<u8>,
        previous_duration: Option<u8>,
    },
    /// Apply volatile status to a Pokemon
    ApplyVolatile {
        target: BattlePosition,
        status: VolatileStatus,
        duration: Option<u8>,
        previous_had_status: bool,
        previous_duration: Option<u8>,
    },
    /// Remove volatile status from a Pokemon
    RemoveVolatile {
        target: BattlePosition,
        status: VolatileStatus,
        previous_duration: Option<u8>,
    },
    /// Change volatile status duration
    ChangeVolatileDuration {
        target: BattlePosition,
        status: VolatileStatus,
        new_duration: Option<u8>,
        previous_duration: Option<u8>,
    },
    /// Set sleep turns (for natural sleep)
    SetSleepTurns {
        target: BattlePosition,
        turns: u8,
        previous_turns: Option<u8>,
    },
    /// Set rest turns (for Rest move)
    SetRestTurns {
        target: BattlePosition,
        turns: u8,
        previous_turns: Option<u8>,
    },
    /// Decrement rest turns
    DecrementRestTurns {
        target: BattlePosition,
        previous_turns: u8,
    },
    /// Disable a move
    DisableMove {
        target: BattlePosition,
        move_index: MoveIndex,
        duration: u8,
        previous_disabled: bool,
    },
    /// Enable a move
    EnableMove {
        target: BattlePosition,
        move_index: MoveIndex,
        was_disabled: bool,
    },
    /// Decrement PP of a move
    DecrementPP {
        target: BattlePosition,
        move_index: MoveIndex,
        amount: u8,
        previous_pp: u8,
    },
    /// Set the last used move
    SetLastUsedMove {
        target: BattlePosition,
        move_name: String,
        previous_move: Option<String>,
    },
    /// Restore last used move (for Disable ending)
    RestoreLastUsedMove {
        target: BattlePosition,
        move_name: String,
    },
}

impl StatusInstruction {
    /// Returns all positions affected by this instruction
    pub fn affected_positions(&self) -> Vec<BattlePosition> {
        match self {
            StatusInstruction::Apply { target, .. } => vec![*target],
            StatusInstruction::Remove { target, .. } => vec![*target],
            StatusInstruction::ChangeDuration { target, .. } => vec![*target],
            StatusInstruction::ApplyVolatile { target, .. } => vec![*target],
            StatusInstruction::RemoveVolatile { target, .. } => vec![*target],
            StatusInstruction::ChangeVolatileDuration { target, .. } => vec![*target],
            StatusInstruction::SetSleepTurns { target, .. } => vec![*target],
            StatusInstruction::SetRestTurns { target, .. } => vec![*target],
            StatusInstruction::DecrementRestTurns { target, .. } => vec![*target],
            StatusInstruction::DisableMove { target, .. } => vec![*target],
            StatusInstruction::EnableMove { target, .. } => vec![*target],
            StatusInstruction::DecrementPP { target, .. } => vec![*target],
            StatusInstruction::SetLastUsedMove { target, .. } => vec![*target],
            StatusInstruction::RestoreLastUsedMove { target, .. } => vec![*target],
        }
    }

    /// Whether this instruction can be undone
    pub fn is_undoable(&self) -> bool {
        match self {
            // Most status instructions store previous state for undo
            StatusInstruction::Apply { previous_status, .. } => previous_status.is_some(),
            StatusInstruction::Remove { .. } => true,
            StatusInstruction::ChangeDuration { previous_duration, .. } => previous_duration.is_some(),
            StatusInstruction::ApplyVolatile { .. } => true,
            StatusInstruction::RemoveVolatile { previous_duration, .. } => previous_duration.is_some(),
            StatusInstruction::ChangeVolatileDuration { previous_duration, .. } => previous_duration.is_some(),
            StatusInstruction::SetSleepTurns { previous_turns, .. } => previous_turns.is_some(),
            StatusInstruction::SetRestTurns { previous_turns, .. } => previous_turns.is_some(),
            StatusInstruction::DecrementRestTurns { .. } => true,
            StatusInstruction::DisableMove { .. } => true,
            StatusInstruction::EnableMove { .. } => true,
            StatusInstruction::DecrementPP { .. } => true,
            StatusInstruction::SetLastUsedMove { previous_move, .. } => previous_move.is_some(),
            StatusInstruction::RestoreLastUsedMove { .. } => true,
        }
    }
}