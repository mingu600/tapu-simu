#[cfg(test)]
mod damage_roll_tests {
    use tapu_simu::engine::combat::damage_calc::{DamageRolls, compare_health_with_damage_multiples};

    #[test]
    fn test_damage_rolls_multipliers() {
        assert_eq!(DamageRolls::Min.as_multiplier(), 0.85);
        assert_eq!(DamageRolls::Average.as_multiplier(), 0.925);
        assert_eq!(DamageRolls::Max.as_multiplier(), 1.0);
    }

    #[test]
    fn test_16_roll_damage_calculation() {
        // Test a scenario where max damage can kill but min damage cannot
        let max_damage = 100;
        let health = 90;
        
        let (average_non_kill_damage, num_kill_rolls) = 
            compare_health_with_damage_multiples(max_damage, health);
        
        // Damage range is 85-100, so 85-89 don't kill (5 rolls), 90-100 do kill (11 rolls)
        assert_eq!(num_kill_rolls, 11); // Rolls that kill
        
        // Average of non-killing rolls should be around (85+86+87+88+89)/5 = 87
        assert!(average_non_kill_damage >= 85 && average_non_kill_damage <= 89);
    }

    #[test] 
    fn test_16_roll_all_kill() {
        // Test a scenario where all damage rolls kill
        let max_damage = 100;
        let health = 80; // Min damage is 85, so all rolls kill
        
        let (average_non_kill_damage, num_kill_rolls) = 
            compare_health_with_damage_multiples(max_damage, health);
        
        assert_eq!(num_kill_rolls, 16); // All rolls kill
        assert_eq!(average_non_kill_damage, 0); // No non-kill damage
    }

    #[test]
    fn test_16_roll_none_kill() {
        // Test a scenario where no damage rolls kill
        let max_damage = 50;
        let health = 200; // Max damage is 50, so no rolls kill
        
        let (average_non_kill_damage, num_kill_rolls) = 
            compare_health_with_damage_multiples(max_damage, health);
        
        assert_eq!(num_kill_rolls, 0); // No rolls kill
        // Average should be around (42.5 + 43.5 + ... + 50)/16 ≈ 46.25
        assert!(average_non_kill_damage >= 42 && average_non_kill_damage <= 50);
    }
}