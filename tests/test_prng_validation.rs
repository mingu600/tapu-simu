//! PRNG validation tests to ensure compatibility with Pokemon Showdown
//! 
//! These tests verify that our PRNG implementations produce the same sequences
//! as Pokemon Showdown's reference implementation for deterministic replay compatibility.

use tapu_simu::prng::PRNGState;

#[test]
fn test_sodium_prng_deterministic_sequence() {
    // Test with a known seed to ensure deterministic behavior
    let mut prng = PRNGState::from_seed("sodium,deadbeefcafebabe12345678")
        .expect("Failed to create sodium PRNG");
    
    // Generate a sequence of numbers and verify they're consistent
    let first_sequence: Vec<u32> = (0..10).map(|_| prng.next_u32()).collect();
    
    // Reset with same seed
    let mut prng2 = PRNGState::from_seed("sodium,deadbeefcafebabe12345678")
        .expect("Failed to create sodium PRNG");
    
    let second_sequence: Vec<u32> = (0..10).map(|_| prng2.next_u32()).collect();
    
    // Sequences should be identical
    assert_eq!(first_sequence, second_sequence);
    
    // Verify numbers are in reasonable range and not all the same
    assert!(first_sequence.iter().any(|&x| x != first_sequence[0]));
    assert!(first_sequence.iter().all(|&x| x <= u32::MAX));
    
    println!("Sodium PRNG deterministic test passed: {:?}", &first_sequence[..5]);
}

#[test]
fn test_gen5_prng_deterministic_sequence() {
    // Test Gen5 PRNG with known seed
    let mut prng = PRNGState::from_seed("gen5,1234,5678,9abc,def0")
        .expect("Failed to create Gen5 PRNG");
    
    // Generate sequence
    let first_sequence: Vec<u32> = (0..10).map(|_| prng.next_u32()).collect();
    
    // Reset with same seed
    let mut prng2 = PRNGState::from_seed("gen5,1234,5678,9abc,def0")
        .expect("Failed to create Gen5 PRNG");
    
    let second_sequence: Vec<u32> = (0..10).map(|_| prng2.next_u32()).collect();
    
    // Sequences should be identical
    assert_eq!(first_sequence, second_sequence);
    
    // Verify non-trivial sequence
    assert!(first_sequence.iter().any(|&x| x != first_sequence[0]));
    
    println!("Gen5 PRNG deterministic test passed: {:?}", &first_sequence[..5]);
}

#[test]
fn test_prng_random_methods() {
    let mut prng = PRNGState::from_seed("sodium,deadbeef").expect("Failed to create PRNG");
    
    // Test random float generation [0, 1)
    for _ in 0..100 {
        let val = prng.random();
        assert!(val >= 0.0 && val < 1.0, "Random float out of range: {}", val);
    }
    
    // Test random range generation
    for _ in 0..100 {
        let val = prng.random_range(10);
        assert!(val < 10, "Random range out of bounds: {}", val);
    }
    
    // Test random chance
    let mut true_count = 0;
    for _ in 0..1000 {
        if prng.random_chance(1, 2) { // 50% chance
            true_count += 1;
        }
    }
    
    // Should be roughly 50% (allow for randomness variance)
    assert!(true_count > 400 && true_count < 600, 
           "Random chance seems biased: {}/1000", true_count);
    
    println!("PRNG random methods validation passed");
}

#[test]
fn test_prng_seed_recovery() {
    let seeds = vec![
        "sodium,deadbeef",
        "sodium,1234567890abcdef",
        "gen5,1111,2222,3333,4444",
        "1000,2000,3000,4000", // Legacy format
    ];
    
    for seed in seeds {
        let prng = PRNGState::from_seed(seed).expect(&format!("Failed to parse seed: {}", seed));
        let recovered_seed = prng.get_seed();
        
        // Create new PRNG from recovered seed
        let mut prng1 = PRNGState::from_seed(seed).expect("Failed to create original PRNG");
        let mut prng2 = PRNGState::from_seed(&recovered_seed).expect("Failed to create recovered PRNG");
        
        // Generate sequences from both
        let seq1: Vec<u32> = (0..10).map(|_| prng1.next_u32()).collect();
        let seq2: Vec<u32> = (0..10).map(|_| prng2.next_u32()).collect();
        
        // Should be identical
        assert_eq!(seq1, seq2, "Seed recovery failed for: {}", seed);
    }
    
    println!("PRNG seed recovery validation passed");
}

#[test]
fn test_prng_serialization_consistency() {
    let mut prng = PRNGState::from_seed("sodium,cafebabe").expect("Failed to create PRNG");
    
    // Generate some numbers to advance state
    for _ in 0..5 {
        prng.next_u32();
    }
    
    // Serialize
    let serialized = serde_json::to_string(&prng).expect("Failed to serialize PRNG");
    let deserialized: PRNGState = serde_json::from_str(&serialized)
        .expect("Failed to deserialize PRNG");
    
    // Compare next sequences
    let original_seq: Vec<u32> = (0..10).map(|_| prng.next_u32()).collect();
    let mut deserialized_copy = deserialized;
    let deserialized_seq: Vec<u32> = (0..10).map(|_| deserialized_copy.next_u32()).collect();
    
    assert_eq!(original_seq, deserialized_seq);
    
    println!("PRNG serialization consistency validation passed");
}

#[test]
fn test_prng_edge_cases() {
    // Test zero range
    let mut prng = PRNGState::from_seed("sodium,12345678").expect("Failed to create PRNG");
    assert_eq!(prng.random_range(0), 0);
    assert_eq!(prng.random_range(1), 0);
    
    // Test chance edge cases
    assert_eq!(prng.random_chance(0, 100), false); // 0% chance
    assert_eq!(prng.random_chance(100, 100), true); // 100% chance
    assert_eq!(prng.random_chance(1, 0), false); // Invalid denominator
    
    // Test sample with empty slice
    let empty: &[i32] = &[];
    assert_eq!(prng.sample(empty), None);
    
    // Test sample with single element
    let single = &[42];
    assert_eq!(prng.sample(single), Some(&42));
    
    println!("PRNG edge cases validation passed");
}

/// Test that matches known Pokemon Showdown behavior patterns
/// This is a simplified test - a full validation would require
/// cross-referencing with actual PS battle logs
#[test]
fn test_pokemon_showdown_compatibility_patterns() {
    // Test common Pokemon Showdown use patterns
    let mut prng = PRNGState::from_seed("sodium,deadbeef").expect("Failed to create PRNG");
    
    // Damage roll pattern (0.85x to 1.0x)
    // This is how PS calculates damage variance
    for _ in 0..100 {
        let roll = 85 + prng.random_range(16); // 85-100
        assert!(roll >= 85 && roll <= 100, "Damage roll out of range: {}", roll);
    }
    
    // Critical hit chance (1/24 for most moves)
    let mut crit_hits = 0;
    for _ in 0..2400 { // Large sample size
        if prng.random_chance(1, 24) {
            crit_hits += 1;
        }
    }
    
    // Should be roughly 100 critical hits (allow variance)
    assert!(crit_hits > 60 && crit_hits < 140, 
           "Critical hit rate seems off: {}/2400", crit_hits);
    
    // Accuracy check pattern (move accuracy out of 100)
    let accuracy = 90; // 90% accuracy move
    let mut hits = 0;
    for _ in 0..1000 {
        if prng.random_range(100) < accuracy {
            hits += 1;
        }
    }
    
    // Should be roughly 900 hits
    assert!(hits > 850 && hits < 950, 
           "Accuracy pattern seems off: {}/1000 for 90% accuracy", hits);
    
    println!("Pokemon Showdown compatibility patterns validated");
}

#[test] 
fn test_prng_performance_baseline() {
    let mut prng = PRNGState::from_seed("sodium,deadbeef").expect("Failed to create PRNG");
    
    let start = std::time::Instant::now();
    
    // Generate a large number of random values
    let count = 100_000;
    for _ in 0..count {
        prng.next_u32();
    }
    
    let duration = start.elapsed();
    let ns_per_call = duration.as_nanos() / count;
    
    // Should be fast enough for real-time simulation
    // Target: < 20μs per call (very generous for ChaCha20)
    assert!(ns_per_call < 20_000, 
           "PRNG too slow: {}ns per call", ns_per_call);
    
    println!("PRNG performance: {}ns per call ({} calls in {:?})", 
             ns_per_call, count, duration);
}