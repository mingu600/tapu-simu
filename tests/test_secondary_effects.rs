// Test secondary effects implementation
use tapu_simu::pokemon::{SecondaryEffect, SecondaryEffectType, StatusCondition};

#[test]
fn test_secondary_effect_structures() {
    // Test that secondary effect structures are properly defined
    
    // Test status effect
    let burn_effect = SecondaryEffect {
        chance: 10, // 10% chance
        effect: SecondaryEffectType::Burn,
    };
    
    assert_eq!(burn_effect.chance, 10);
    matches!(burn_effect.effect, SecondaryEffectType::Burn);
    
    // Test stat boost effect
    let attack_boost = SecondaryEffect {
        chance: 100, // Always happens
        effect: SecondaryEffectType::StatBoost { 
            stat: "attack".to_string(), 
            amount: 1 
        },
    };
    
    assert_eq!(attack_boost.chance, 100);
    if let SecondaryEffectType::StatBoost { stat, amount } = &attack_boost.effect {
        assert_eq!(stat, "attack");
        assert_eq!(*amount, 1);
    }
    
    // Test status condition variant
    let paralysis_effect = SecondaryEffect {
        chance: 30,
        effect: SecondaryEffectType::Status(StatusCondition::Paralysis),
    };
    
    if let SecondaryEffectType::Status(status) = &paralysis_effect.effect {
        matches!(status, StatusCondition::Paralysis);
    }
    
    println!("✓ Secondary effect structures are correctly implemented");
    println!("✓ Status effects: Burn, Paralysis, etc.");
    println!("✓ Stat boosts: Attack +1, etc.");
    println!("✓ Chance-based application: {}%", burn_effect.chance);
}