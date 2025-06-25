//! # Damage Calculation Utilities
//!
//! Utility functions used throughout the damage calculation system.

/// Pokemon-specific rounding function used for damage calculations.
///
/// This implements the rounding behavior used in Pokemon games starting from Generation 5.
/// The function rounds 0.5 up to 1, which is different from standard rounding in most
/// programming languages.
pub fn poke_round(value: f64) -> i16 {
    (value + 0.5).floor() as i16
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_poke_round() {
        assert_eq!(poke_round(10.5), 11);
        assert_eq!(poke_round(10.4), 10);
        assert_eq!(poke_round(10.6), 11);
        assert_eq!(poke_round(0.5), 1);
        assert_eq!(poke_round(0.4), 0);
        assert_eq!(poke_round(100.5), 101);
    }
}