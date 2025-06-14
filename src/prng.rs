//! Pseudo-random number generation for deterministic battles
//! 
//! This module provides deterministic random number generation that exactly
//! matches Pokemon Showdown's PRNG behavior for replay compatibility.

use chacha20::ChaCha20;
use chacha20::cipher::{KeyIvInit, StreamCipher};
use serde::{Deserialize, Serialize};
use crate::errors::{BattleError, BattleResult};

/// PRNG state that can be serialized and restored
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PRNGState {
    Sodium(SodiumRNG),
    Gen5(Gen5RNG),
}

impl PRNGState {
    /// Create a new PRNG from a seed string
    pub fn from_seed(seed: &str) -> BattleResult<Self> {
        if seed.starts_with("sodium,") {
            let parts: Vec<&str> = seed.split(',').collect();
            if parts.len() != 2 {
                return Err(BattleError::InvalidSeed(format!("Invalid sodium seed format: {}", seed)));
            }
            Ok(PRNGState::Sodium(SodiumRNG::from_seed(parts[1])?))
        } else if seed.starts_with("gen5,") {
            let seed_part = &seed[5..];
            let parts: Vec<&str> = seed_part.split(',').collect();
            if parts.len() != 4 {
                return Err(BattleError::InvalidSeed(format!("Invalid gen5 seed format: {}", seed)));
            }
            
            let mut gen5_seed = [0u16; 4];
            for (i, part) in parts.iter().enumerate() {
                gen5_seed[i] = u16::from_str_radix(part, 16)
                    .map_err(|_| BattleError::InvalidSeed(format!("Invalid hex value: {}", part)))?;
            }
            Ok(PRNGState::Gen5(Gen5RNG::from_seed(gen5_seed)))
        } else {
            // Try to parse as comma-separated numbers (legacy format)
            let parts: Vec<&str> = seed.split(',').collect();
            if parts.len() == 4 {
                let mut gen5_seed = [0u16; 4];
                for (i, part) in parts.iter().enumerate() {
                    gen5_seed[i] = part.parse::<u16>()
                        .map_err(|_| BattleError::InvalidSeed(format!("Invalid number: {}", part)))?;
                }
                Ok(PRNGState::Gen5(Gen5RNG::from_seed(gen5_seed)))
            } else {
                Err(BattleError::InvalidSeed(format!("Unrecognized seed format: {}", seed)))
            }
        }
    }
    
    /// Generate a new random seed
    pub fn generate_seed() -> Self {
        PRNGState::Sodium(SodiumRNG::generate())
    }
    
    /// Get the next random u32
    pub fn next_u32(&mut self) -> u32 {
        match self {
            PRNGState::Sodium(rng) => rng.next(),
            PRNGState::Gen5(rng) => rng.next(),
        }
    }
    
    /// Generate a random float in [0, 1)
    pub fn random(&mut self) -> f64 {
        self.next_u32() as f64 / (u32::MAX as f64 + 1.0)
    }
    
    /// Generate a random integer in [0, n)
    pub fn random_range(&mut self, n: u32) -> u32 {
        if n == 0 { return 0; }
        (self.random() * n as f64) as u32
    }
    
    /// Generate a random integer in [min, max)
    pub fn random_range_inclusive(&mut self, min: u32, max: u32) -> u32 {
        if min >= max { return min; }
        min + self.random_range(max - min)
    }
    
    /// Random chance with given probability
    pub fn random_chance(&mut self, numerator: u32, denominator: u32) -> bool {
        if denominator == 0 { return false; }
        self.random_range(denominator) < numerator
    }
    
    /// Sample a random element from a slice
    pub fn sample<'a, T>(&mut self, items: &'a [T]) -> Option<&'a T> {
        if items.is_empty() {
            None
        } else {
            Some(&items[self.random_range(items.len() as u32) as usize])
        }
    }
    
    /// Fisher-Yates shuffle
    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        let len = items.len();
        for i in 0..len.saturating_sub(1) {
            let j = i + self.random_range((len - i) as u32) as usize;
            items.swap(i, j);
        }
    }
    
    /// Get seed string for this PRNG
    pub fn get_seed(&self) -> String {
        match self {
            PRNGState::Sodium(rng) => rng.get_seed(),
            PRNGState::Gen5(rng) => rng.get_seed(),
        }
    }
}

/// Sodium-based RNG compatible with Pokemon Showdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SodiumRNG {
    seed: [u8; 32],
}

impl SodiumRNG {
    /// Create from hex seed string
    pub fn from_seed(hex_seed: &str) -> BattleResult<Self> {
        let mut seed = [0u8; 32];
        let hex_clean = hex_seed.trim();
        
        if hex_clean.len() > 64 {
            return Err(BattleError::InvalidSeed("Seed too long".to_string()));
        }
        
        // Pad with zeros if needed
        let padded = format!("{:0<64}", hex_clean);
        
        for (i, chunk) in padded.as_bytes().chunks(2).enumerate() {
            if i >= 32 { break; }
            let hex_str = std::str::from_utf8(chunk)
                .map_err(|_| BattleError::InvalidSeed("Invalid UTF-8 in seed".to_string()))?;
            seed[i] = u8::from_str_radix(hex_str, 16)
                .map_err(|_| BattleError::InvalidSeed(format!("Invalid hex byte: {}", hex_str)))?;
        }
        
        Ok(SodiumRNG { seed })
    }
    
    /// Generate a new random seed
    pub fn generate() -> Self {
        let mut seed = [0u8; 32];
        // Fill with random data - in a real implementation, use a secure RNG
        for (i, byte) in seed.iter_mut().enumerate() {
            *byte = ((i * 17 + 42) % 256) as u8; // Simple deterministic "random" for now
        }
        SodiumRNG { seed }
    }
    
    /// Get the next random number
    pub fn next(&mut self) -> u32 {
        // ChaCha20 implementation
        let nonce = b"LibsodiumDRG"; // 12 bytes
        let zero_buf = [0u8; 36];
        
        let mut cipher = ChaCha20::new(&self.seed.into(), nonce.into());
        let mut output = zero_buf;
        cipher.apply_keystream(&mut output);
        
        // Update seed with first 32 bytes, return last 4 bytes as u32
        self.seed.copy_from_slice(&output[..32]);
        u32::from_be_bytes([output[32], output[33], output[34], output[35]])
    }
    
    /// Get seed as hex string
    pub fn get_seed(&self) -> String {
        format!("sodium,{}", hex::encode(&self.seed))
    }
}

/// Gen 5 Linear Congruential Generator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gen5RNG {
    seed: [u16; 4], // 64-bit seed as 4 x 16-bit values
}

impl Gen5RNG {
    /// Create from 64-bit seed
    pub fn from_seed(seed: [u16; 4]) -> Self {
        Gen5RNG { seed }
    }
    
    /// Generate a new random seed
    pub fn generate() -> Self {
        // Simple seed generation for now
        Gen5RNG {
            seed: [0x1234, 0x5678, 0x9ABC, 0xDEF0]
        }
    }
    
    /// Get the next random number
    pub fn next(&mut self) -> u32 {
        // LCG constants: a = 0x5D588B656C078965, c = 0x00269EC3
        let a = [0x5D58, 0x8B65, 0x6C07, 0x8965];
        let c = [0x0000, 0x0000, 0x0026, 0x9EC3];
        
        // Multiply-add: seed = seed * a + c (mod 2^64)
        self.seed = self.multiply_add(self.seed, a, c);
        
        // Return upper 32 bits
        ((self.seed[0] as u32) << 16) | (self.seed[1] as u32)
    }
    
    /// 64-bit multiply-add operation
    fn multiply_add(&self, a: [u16; 4], b: [u16; 4], c: [u16; 4]) -> [u16; 4] {
        let mut result = [0u16; 4];
        let mut carry = 0u32;
        
        // Long multiplication
        for i in (0..4).rev() {
            for j in i..4 {
                let ai = 3 - (j - i);
                carry += (a[ai] as u32) * (b[j] as u32);
            }
            carry += c[i] as u32;
            result[i] = (carry & 0xFFFF) as u16;
            carry >>= 16;
        }
        
        result
    }
    
    /// Get seed as string
    pub fn get_seed(&self) -> String {
        format!("gen5,{:04x},{:04x},{:04x},{:04x}", 
                self.seed[0], self.seed[1], self.seed[2], self.seed[3])
    }
}

// Add hex encoding dependency
mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_prng_deterministic() {
        let mut rng1 = PRNGState::from_seed("sodium,0123456789abcdef").unwrap();
        let mut rng2 = PRNGState::from_seed("sodium,0123456789abcdef").unwrap();
        
        for _ in 0..100 {
            assert_eq!(rng1.next_u32(), rng2.next_u32());
        }
    }
    
    #[test]
    fn test_random_chance() {
        let mut rng = PRNGState::generate();
        
        // Test extreme cases
        assert!(!rng.random_chance(0, 100));
        assert!(rng.random_chance(100, 100));
        
        // Test probability distribution (this is probabilistic)
        let mut successes = 0;
        for _ in 0..1000 {
            if rng.random_chance(1, 4) { // 25% chance
                successes += 1;
            }
        }
        
        // Should be around 250, allow some variance
        assert!(successes > 200 && successes < 300);
    }
}