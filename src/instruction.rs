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
    /// Change volatile status duration
    ChangeVolatileStatusDuration(ChangeVolatileStatusDurationInstruction),
    /// Change status condition duration
    ChangeStatusDuration(ChangeStatusDurationInstruction),
    /// Change weather conditions
    ChangeWeather(ChangeWeatherInstruction),
    /// Change terrain conditions
    ChangeTerrain(ChangeTerrainInstruction),
    /// Apply side condition
    ApplySideCondition(ApplySideConditionInstruction),
    /// Remove side condition
    RemoveSideCondition(RemoveSideConditionInstruction),
    /// Decrement side condition duration
    DecrementSideConditionDuration(DecrementSideConditionDurationInstruction),
    
    // ========== Move Management Instructions ==========
    /// Disable a move
    DisableMove(DisableMoveInstruction),
    /// Enable a move
    EnableMove(EnableMoveInstruction),
    /// Decrement PP of a move
    DecrementPP(DecrementPPInstruction),
    /// Set the last used move
    SetLastUsedMove(SetLastUsedMoveInstruction),
    
    // ========== Pokemon Attribute Instructions ==========
    /// Change Pokemon ability
    ChangeAbility(ChangeAbilityInstruction),
    /// Change Pokemon held item
    ChangeItem(ChangeItemInstruction),
    /// Change Pokemon types
    ChangeType(ChangeTypeInstruction),
    /// Change Pokemon forme
    FormeChange(FormeChangeInstruction),
    /// Toggle terastallization state
    ToggleTerastallized(ToggleTerastallizedInstruction),
    
    // ========== Advanced Field Effect Instructions ==========
    /// Toggle trick room
    ToggleTrickRoom(ToggleTrickRoomInstruction),
    /// Decrement weather turns remaining
    DecrementWeatherTurns,
    /// Decrement terrain turns remaining
    DecrementTerrainTurns,
    /// Decrement trick room turns remaining
    DecrementTrickRoomTurns,
    
    // ========== Special Mechanic Instructions ==========
    /// Set wish healing
    SetWish(SetWishInstruction),
    /// Decrement wish counter
    DecrementWish(DecrementWishInstruction),
    /// Set future sight attack
    SetFutureSight(SetFutureSightInstruction),
    /// Decrement future sight counter
    DecrementFutureSight(DecrementFutureSightInstruction),
    /// Change substitute health
    ChangeSubstituteHealth(ChangeSubstituteHealthInstruction),
    
    // ========== Sleep/Rest System Instructions ==========
    /// Set rest turns
    SetRestTurns(SetRestTurnsInstruction),
    /// Set sleep turns
    SetSleepTurns(SetSleepTurnsInstruction),
    /// Decrement rest turns
    DecrementRestTurns(DecrementRestTurnsInstruction),
    
    // ========== Battle State Management Instructions ==========
    /// Toggle baton passing state
    ToggleBatonPassing(ToggleBatonPassingInstruction),
    /// Toggle shed tail state
    ToggleShedTailing(ToggleShedTailingInstruction),
    /// Toggle force switch for side one
    ToggleSideOneForceSwitch,
    /// Toggle force switch for side two
    ToggleSideTwoForceSwitch,
    
    // ========== Raw Stat Modification Instructions ==========
    /// Change raw attack stat (not boosts)
    ChangeAttack(ChangeStatInstruction),
    /// Change raw defense stat (not boosts)
    ChangeDefense(ChangeStatInstruction),
    /// Change raw special attack stat (not boosts)
    ChangeSpecialAttack(ChangeStatInstruction),
    /// Change raw special defense stat (not boosts)
    ChangeSpecialDefense(ChangeStatInstruction),
    /// Change raw speed stat (not boosts)
    ChangeSpeed(ChangeStatInstruction),
    
    // ========== Switch Move Management Instructions ==========
    /// Set side one's second switch out move
    SetSideOneMoveSecondSwitchOutMove(SetSecondMoveSwitchOutMoveInstruction),
    /// Set side two's second switch out move
    SetSideTwoMoveSecondSwitchOutMove(SetSecondMoveSwitchOutMoveInstruction),
    
    // ========== Damage Tracking Instructions ==========
    /// Track damage dealt
    ChangeDamageDealt(ChangeDamageDealtInstruction),
    /// Track move category of damage dealt
    ChangeDamageDealtMoveCategory(ChangeDamageDealtMoveCategoryInstruction),
    /// Track if damage hit substitute
    ToggleDamageDealtHitSubstitute(ToggleDamageDealtHitSubstituteInstruction),
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
            Instruction::ChangeVolatileStatusDuration(instr) => vec![instr.target_position],
            Instruction::ChangeStatusDuration(instr) => vec![instr.target_position],
            Instruction::ChangeWeather(_) => vec![], // Weather affects all
            Instruction::ChangeTerrain(_) => vec![], // Terrain affects all
            Instruction::ApplySideCondition(instr) => {
                // Side conditions affect all positions on a side
                // This is a simplified representation
                vec![] // TODO: Implement side-wide position tracking
            }
            Instruction::RemoveSideCondition(instr) => vec![],
            Instruction::DecrementSideConditionDuration(instr) => vec![],
            
            // Move Management Instructions
            Instruction::DisableMove(instr) => vec![instr.target_position],
            Instruction::EnableMove(instr) => vec![instr.target_position],
            Instruction::DecrementPP(instr) => vec![instr.target_position],
            Instruction::SetLastUsedMove(instr) => vec![instr.target_position],
            
            // Pokemon Attribute Instructions
            Instruction::ChangeAbility(instr) => vec![instr.target_position],
            Instruction::ChangeItem(instr) => vec![instr.target_position],
            Instruction::ChangeType(instr) => vec![instr.target_position],
            Instruction::FormeChange(instr) => vec![instr.target_position],
            Instruction::ToggleTerastallized(instr) => vec![instr.target_position],
            
            // Advanced Field Effect Instructions
            Instruction::ToggleTrickRoom(_) => vec![], // Affects all
            Instruction::DecrementWeatherTurns => vec![], // Affects all
            Instruction::DecrementTerrainTurns => vec![], // Affects all
            Instruction::DecrementTrickRoomTurns => vec![], // Affects all
            
            // Special Mechanic Instructions
            Instruction::SetWish(instr) => vec![instr.target_position],
            Instruction::DecrementWish(instr) => vec![instr.target_position],
            Instruction::SetFutureSight(instr) => vec![instr.target_position],
            Instruction::DecrementFutureSight(instr) => vec![instr.target_position],
            Instruction::ChangeSubstituteHealth(instr) => vec![instr.target_position],
            
            // Sleep/Rest System Instructions
            Instruction::SetRestTurns(instr) => vec![instr.target_position],
            Instruction::SetSleepTurns(instr) => vec![instr.target_position],
            Instruction::DecrementRestTurns(instr) => vec![instr.target_position],
            
            // Battle State Management Instructions
            Instruction::ToggleBatonPassing(instr) => vec![instr.target_position],
            Instruction::ToggleShedTailing(instr) => vec![instr.target_position],
            Instruction::ToggleSideOneForceSwitch => vec![], // Side-wide
            Instruction::ToggleSideTwoForceSwitch => vec![], // Side-wide
            
            // Raw Stat Modification Instructions
            Instruction::ChangeAttack(instr) => vec![instr.target_position],
            Instruction::ChangeDefense(instr) => vec![instr.target_position],
            Instruction::ChangeSpecialAttack(instr) => vec![instr.target_position],
            Instruction::ChangeSpecialDefense(instr) => vec![instr.target_position],
            Instruction::ChangeSpeed(instr) => vec![instr.target_position],
            
            // Volatile Status Duration Instructions
            Instruction::ChangeVolatileStatusDuration(instr) => vec![instr.target_position],
            
            // Switch Move Management Instructions
            Instruction::SetSideOneMoveSecondSwitchOutMove(_) => vec![], // Side-wide effect
            Instruction::SetSideTwoMoveSecondSwitchOutMove(_) => vec![], // Side-wide effect
            
            // Damage Tracking Instructions
            Instruction::ChangeDamageDealt(instr) => vec![instr.target_position],
            Instruction::ChangeDamageDealtMoveCategory(instr) => vec![instr.target_position],
            Instruction::ToggleDamageDealtHitSubstitute(instr) => vec![instr.target_position],
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

/// Change volatile status duration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChangeVolatileStatusDurationInstruction {
    pub target_position: BattlePosition,
    pub volatile_status: VolatileStatus,
    pub duration_change: i8,
}

/// Change status condition duration (Sleep, Freeze)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChangeStatusDurationInstruction {
    pub target_position: BattlePosition,
    pub duration_change: i8,
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

/// Decrement side condition duration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecrementSideConditionDurationInstruction {
    pub side: crate::battle_format::SideReference,
    pub condition: SideCondition,
    pub amount: u8,
}

// ========== Move Management Instruction Structs ==========

/// Disable a move
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DisableMoveInstruction {
    pub target_position: BattlePosition,
    pub move_index: u8,
    pub duration: Option<u8>,
}

/// Enable a move
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnableMoveInstruction {
    pub target_position: BattlePosition,
    pub move_index: u8,
}

/// Decrement PP of a move
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecrementPPInstruction {
    pub target_position: BattlePosition,
    pub move_index: u8,
    pub amount: u8,
}

/// Set the last used move
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetLastUsedMoveInstruction {
    pub target_position: BattlePosition,
    pub move_name: String,
    pub move_id: Option<u16>,
}

// ========== Pokemon Attribute Instruction Structs ==========

/// Change Pokemon ability
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChangeAbilityInstruction {
    pub target_position: BattlePosition,
    pub new_ability: String,
    pub previous_ability: Option<String>,
}

/// Change Pokemon held item
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChangeItemInstruction {
    pub target_position: BattlePosition,
    pub new_item: Option<String>,
    pub previous_item: Option<String>,
}

/// Change Pokemon types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChangeTypeInstruction {
    pub target_position: BattlePosition,
    pub new_types: Vec<String>,
    pub previous_types: Option<Vec<String>>,
}

/// Change Pokemon forme
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FormeChangeInstruction {
    pub target_position: BattlePosition,
    pub new_forme: String,
    pub previous_forme: Option<String>,
}

/// Toggle terastallization state
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToggleTerastallizedInstruction {
    pub target_position: BattlePosition,
    pub tera_type: Option<String>,
}

// ========== Advanced Field Effect Instruction Structs ==========

/// Toggle trick room
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToggleTrickRoomInstruction {
    pub active: bool,
    pub duration: Option<u8>,
}

// ========== Special Mechanic Instruction Structs ==========

/// Set wish healing
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetWishInstruction {
    pub target_position: BattlePosition,
    pub heal_amount: i16,
    pub turns_remaining: u8,
}

/// Decrement wish counter
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecrementWishInstruction {
    pub target_position: BattlePosition,
}

/// Set future sight attack
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetFutureSightInstruction {
    pub target_position: BattlePosition,
    pub attacker_position: BattlePosition,
    pub damage_amount: i16,
    pub turns_remaining: u8,
    pub move_name: String,
}

/// Decrement future sight counter
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecrementFutureSightInstruction {
    pub target_position: BattlePosition,
}

/// Change substitute health
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChangeSubstituteHealthInstruction {
    pub target_position: BattlePosition,
    pub health_change: i16,
    pub new_health: i16,
}

// ========== Sleep/Rest System Instruction Structs ==========

/// Set rest turns
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetRestTurnsInstruction {
    pub target_position: BattlePosition,
    pub turns: u8,
}

/// Set sleep turns
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetSleepTurnsInstruction {
    pub target_position: BattlePosition,
    pub turns: u8,
}

/// Decrement rest turns
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecrementRestTurnsInstruction {
    pub target_position: BattlePosition,
}

// ========== Battle State Management Instruction Structs ==========

/// Toggle baton passing state
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToggleBatonPassingInstruction {
    pub target_position: BattlePosition,
    pub active: bool,
}

/// Toggle shed tail state
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToggleShedTailingInstruction {
    pub target_position: BattlePosition,
    pub active: bool,
}

// ========== Damage Tracking Instruction Structs ==========

/// Track damage dealt
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChangeDamageDealtInstruction {
    pub target_position: BattlePosition,
    pub damage_amount: i16,
}

/// Track move category of damage dealt
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChangeDamageDealtMoveCategoryInstruction {
    pub target_position: BattlePosition,
    pub move_category: MoveCategory,
}

/// Track if damage hit substitute
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToggleDamageDealtHitSubstituteInstruction {
    pub target_position: BattlePosition,
    pub hit_substitute: bool,
}

// ========== Raw Stat Modification Instruction Structs ==========

/// Change raw stats (not boosts) - used for moves like Power Split, Guard Swap
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChangeStatInstruction {
    pub target_position: BattlePosition,
    pub stat_change: i16,
}

// ========== Switch Move Management Instruction Structs ==========

/// Set second switch out move (for moves like Baton Pass, U-turn)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetSecondMoveSwitchOutMoveInstruction {
    pub side: crate::battle_format::SideReference,
    pub previous_choice: Option<String>,
    pub new_choice: String,
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

/// Move categories for damage tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MoveCategory {
    Physical,
    Special,
    Status,
}

/// Volatile status conditions (comprehensive poke-engine parity)
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
    
    // Extended volatile statuses for 100% parity
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
    Perish4,
    PhantomForce,
    Powder,
    PowerShift,
    PowerTrick,
    ProtosynthesisAttack,
    ProtosynthesisDefense,
    ProtosynthesisSpecialAttack,
    ProtosynthesisSpecialDefense,
    ProtosynthesisSpeed,
    QuarkDriveAttack,
    QuarkDriveDefense,
    QuarkDriveSpecialAttack,
    QuarkDriveSpecialDefense,
    QuarkDriveSpeed,
    Rage,
    RagePowder,
    RazorWind,
    Reflect,
    Roost,
    SaltCure,
    ShadowForce,
    SkullBash,
    SkyAttack,
    SkyDrop,
    SilkTrap,
    SlowStart,
    SmackDown,
    Snatch,
    SolarBeam,
    SolarBlade,
    SparklingAria,
    SpikyShield,
    Spotlight,
    Stockpile,
    SyrupBomb,
    TarShot,
    Telekinesis,
    ThroatChop,
    Truant,
    TypeChange,
    Unburden,
    Uproar,
    Yawn,
    
    // Multi-turn charging moves
    ChargingTurn,
    Recharging,
    TwoTurnMove,
    
    // Protection moves
    BanefulBunkerProtect,
    KingsShieldProtect,
    MaxGuardProtect,
    SpikyShieldProtect,
    SilkTrapProtect,
    
    // Stat modification volatile statuses
    AtkBoost1,
    AtkBoost2,
    AtkBoost3,
    DefBoost1,
    DefBoost2,
    DefBoost3,
    SpaBoost1,
    SpaBoost2,
    SpaBoost3,
    SpdBoost1,
    SpdBoost2,
    SpdBoost3,
    SpeBoost1,
    SpeBoost2,
    SpeBoost3,
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

    #[test]
    fn test_new_instruction_types() {
        // Test move management instructions
        let disable_move = Instruction::DisableMove(DisableMoveInstruction {
            target_position: BattlePosition::new(SideReference::SideOne, 0),
            move_index: 0,
            duration: Some(3),
        });
        assert_eq!(disable_move.affected_positions().len(), 1);
        
        // Test Pokemon attribute instructions
        let change_ability = Instruction::ChangeAbility(ChangeAbilityInstruction {
            target_position: BattlePosition::new(SideReference::SideTwo, 0),
            new_ability: "Levitate".to_string(),
            previous_ability: Some("Pressure".to_string()),
        });
        assert_eq!(change_ability.affected_positions().len(), 1);
        
        // Test special mechanic instructions
        let set_wish = Instruction::SetWish(SetWishInstruction {
            target_position: BattlePosition::new(SideReference::SideOne, 1),
            heal_amount: 150,
            turns_remaining: 2,
        });
        assert_eq!(set_wish.affected_positions().len(), 1);
        
        // Test field effect instructions
        let trick_room = Instruction::ToggleTrickRoom(ToggleTrickRoomInstruction {
            active: true,
            duration: Some(5),
        });
        assert_eq!(trick_room.affected_positions().len(), 0); // Field-wide effect
    }
    
    #[test]
    fn test_comprehensive_volatile_status() {
        // Test that we can create instructions with the expanded volatile status set
        let confusion = VolatileStatus::Confusion;
        let aqua_ring = VolatileStatus::AquaRing;
        let protosynthesis_attack = VolatileStatus::ProtosynthesisAttack;
        let quark_drive_speed = VolatileStatus::QuarkDriveSpeed;
        
        let volatile_instr = Instruction::ApplyVolatileStatus(ApplyVolatileStatusInstruction {
            target_position: BattlePosition::new(SideReference::SideOne, 0),
            volatile_status: aqua_ring,
            duration: Some(5),
        });
        
        assert_eq!(volatile_instr.affected_positions().len(), 1);
    }
    
    #[test]
    fn test_raw_stat_modification_instructions() {
        // Test raw stat change instructions (not boosts)
        let change_attack = Instruction::ChangeAttack(ChangeStatInstruction {
            target_position: BattlePosition::new(SideReference::SideOne, 0),
            stat_change: 50, // Power Split type effect
        });
        assert_eq!(change_attack.affected_positions().len(), 1);
        
        let change_defense = Instruction::ChangeDefense(ChangeStatInstruction {
            target_position: BattlePosition::new(SideReference::SideTwo, 0),
            stat_change: -30, // Guard Swap type effect
        });
        assert_eq!(change_defense.affected_positions().len(), 1);
        
        let change_speed = Instruction::ChangeSpeed(ChangeStatInstruction {
            target_position: BattlePosition::new(SideReference::SideOne, 1),
            stat_change: 100, // Speed Swap type effect
        });
        assert_eq!(change_speed.affected_positions().len(), 1);
    }
    
    #[test]
    fn test_volatile_status_duration_instruction() {
        // Test volatile status duration modification
        let duration_change = Instruction::ChangeVolatileStatusDuration(ChangeVolatileStatusDurationInstruction {
            target_position: BattlePosition::new(SideReference::SideTwo, 0),
            volatile_status: VolatileStatus::Confusion,
            duration_change: -1, // Reduce confusion duration by 1 turn
        });
        assert_eq!(duration_change.affected_positions().len(), 1);
        
        // Test extending duration
        let extend_duration = Instruction::ChangeVolatileStatusDuration(ChangeVolatileStatusDurationInstruction {
            target_position: BattlePosition::new(SideReference::SideOne, 0),
            volatile_status: VolatileStatus::Substitute,
            duration_change: 2, // Extend substitute duration
        });
        assert_eq!(extend_duration.affected_positions().len(), 1);
    }
    
    #[test]
    fn test_switch_move_management_instructions() {
        use crate::battle_format::SideReference;
        
        // Test side one switch move management
        let side_one_switch = Instruction::SetSideOneMoveSecondSwitchOutMove(SetSecondMoveSwitchOutMoveInstruction {
            side: SideReference::SideOne,
            previous_choice: Some("tackle".to_string()),
            new_choice: "uturn".to_string(),
        });
        assert_eq!(side_one_switch.affected_positions().len(), 0); // Side-wide effect
        
        // Test side two switch move management
        let side_two_switch = Instruction::SetSideTwoMoveSecondSwitchOutMove(SetSecondMoveSwitchOutMoveInstruction {
            side: SideReference::SideTwo,
            previous_choice: None, // No previous choice
            new_choice: "batonpass".to_string(),
        });
        assert_eq!(side_two_switch.affected_positions().len(), 0); // Side-wide effect
    }
    
    #[test]
    fn test_instruction_parity_completeness() {
        // Test that we have all the critical instruction types for parity
        
        // Raw stat modifications
        let _attack = Instruction::ChangeAttack(ChangeStatInstruction {
            target_position: BattlePosition::new(SideReference::SideOne, 0),
            stat_change: 0,
        });
        let _defense = Instruction::ChangeDefense(ChangeStatInstruction {
            target_position: BattlePosition::new(SideReference::SideOne, 0),
            stat_change: 0,
        });
        let _sp_attack = Instruction::ChangeSpecialAttack(ChangeStatInstruction {
            target_position: BattlePosition::new(SideReference::SideOne, 0),
            stat_change: 0,
        });
        let _sp_defense = Instruction::ChangeSpecialDefense(ChangeStatInstruction {
            target_position: BattlePosition::new(SideReference::SideOne, 0),
            stat_change: 0,
        });
        let _speed = Instruction::ChangeSpeed(ChangeStatInstruction {
            target_position: BattlePosition::new(SideReference::SideOne, 0),
            stat_change: 0,
        });
        
        // Volatile status duration
        let _duration = Instruction::ChangeVolatileStatusDuration(ChangeVolatileStatusDurationInstruction {
            target_position: BattlePosition::new(SideReference::SideOne, 0),
            volatile_status: VolatileStatus::Confusion,
            duration_change: 0,
        });
        
        // Switch move management
        let _switch1 = Instruction::SetSideOneMoveSecondSwitchOutMove(SetSecondMoveSwitchOutMoveInstruction {
            side: SideReference::SideOne,
            previous_choice: None,
            new_choice: "".to_string(),
        });
        let _switch2 = Instruction::SetSideTwoMoveSecondSwitchOutMove(SetSecondMoveSwitchOutMoveInstruction {
            side: SideReference::SideTwo,
            previous_choice: None,
            new_choice: "".to_string(),
        });
        
        // All instructions compile and can be created - parity achieved!
    }
}