use serde::{Deserialize, Serialize};
use std::fmt;
use std::num::NonZeroU32;

/// Type-safe wrapper for team slot indices with validation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SlotIndex(u8);

/// Maximum team size constant for validation
pub const MAX_TEAM_SIZE: u8 = 6;

#[derive(Debug, thiserror::Error)]
#[error("Invalid slot index {slot}: must be less than {MAX_TEAM_SIZE}")]
pub struct InvalidSlotError {
    pub slot: u8,
}

impl SlotIndex {
    /// Create a new SlotIndex with validation
    pub fn new(slot: u8) -> Result<Self, InvalidSlotError> {
        if slot < MAX_TEAM_SIZE {
            Ok(SlotIndex(slot))
        } else {
            Err(InvalidSlotError { slot })
        }
    }
    
    /// Create a SlotIndex without validation (use carefully)
    pub fn new_unchecked(slot: u8) -> Self {
        SlotIndex(slot)
    }
    
    /// Get the raw slot value
    pub fn as_u8(self) -> u8 {
        self.0
    }
    
    /// Get the raw slot value as usize for indexing
    pub fn as_usize(self) -> usize {
        self.0 as usize
    }
}

impl TryFrom<u8> for SlotIndex {
    type Error = InvalidSlotError;
    
    fn try_from(slot: u8) -> Result<Self, Self::Error> {
        Self::new(slot)
    }
}

impl TryFrom<usize> for SlotIndex {
    type Error = InvalidSlotError;
    
    fn try_from(slot: usize) -> Result<Self, Self::Error> {
        if slot < MAX_TEAM_SIZE as usize {
            Ok(SlotIndex(slot as u8))
        } else {
            Err(InvalidSlotError { slot: slot as u8 })
        }
    }
}

impl fmt::Display for SlotIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Type-safe wrapper for turn numbers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TurnNumber(NonZeroU32);

impl TurnNumber {
    /// Create a new TurnNumber starting from 1
    pub fn new(turn: u32) -> Option<Self> {
        NonZeroU32::new(turn).map(TurnNumber)
    }
    
    /// Create the first turn
    pub fn first() -> Self {
        TurnNumber(NonZeroU32::new(1).unwrap())
    }
    
    /// Get the next turn number
    pub fn next(self) -> Self {
        TurnNumber(NonZeroU32::new(self.0.get() + 1).unwrap())
    }
    
    /// Get the raw turn value
    pub fn as_u32(self) -> u32 {
        self.0.get()
    }
}

impl From<NonZeroU32> for TurnNumber {
    fn from(turn: NonZeroU32) -> Self {
        TurnNumber(turn)
    }
}

impl fmt::Display for TurnNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Type-safe wrapper for battle position indices
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PositionIndex(u8);

impl PositionIndex {
    /// Create a new PositionIndex
    pub fn new(position: u8) -> Self {
        PositionIndex(position)
    }
    
    /// Get the raw position value
    pub fn as_u8(self) -> u8 {
        self.0
    }
    
    /// Get the raw position value as usize for indexing
    pub fn as_usize(self) -> usize {
        self.0 as usize
    }
}

impl From<u8> for PositionIndex {
    fn from(position: u8) -> Self {
        PositionIndex(position)
    }
}

impl fmt::Display for PositionIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}