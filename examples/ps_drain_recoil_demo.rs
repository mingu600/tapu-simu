#!/usr/bin/env cargo run --example ps_drain_recoil_demo --

//! # Pokemon Showdown Drain/Recoil Data Demo
//! 
//! This example demonstrates how drain and recoil ratios are read directly
//! from Pokemon Showdown data, eliminating the need for manual enhancements.

use tapu_simu::data::ps_move_factory::PSMoveFactory;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧬 Pokemon Showdown Drain/Recoil Data Demo");
    println!("==========================================\n");

    let factory = PSMoveFactory::new()?;

    println!("💧 Testing Drain Moves:");
    let drain_moves = vec![
        "absorb",
        "megadrain", 
        "gigadrain",
        "dreameater",
        "drainingkiss",
        "roost", // Should have no drain
    ];

    for move_name in &drain_moves {
        if let Some(ratio) = factory.get_drain_ratio(move_name) {
            println!("   {}: {:.1} drain ({:.1}%)", move_name, ratio, ratio * 100.0);
        } else {
            println!("   {}: No drain", move_name);
        }
    }

    println!("\n💥 Testing Recoil Moves:");
    let recoil_moves = vec![
        "doubleedge",
        "submission",
        "takedown", 
        "volttackle",
        "flareblitz",
        "wildcharge",
        "thunderbolt", // Should have no recoil
    ];

    for move_name in &recoil_moves {
        if let Some(ratio) = factory.get_recoil_ratio(move_name) {
            println!("   {}: {:.1} recoil ({:.1}%)", move_name, ratio, ratio * 100.0);
        } else {
            println!("   {}: No recoil", move_name);
        }
    }

    println!("\n🔍 Detailed Analysis:");
    
    // Show exact PS data for a few moves
    if let Some(absorb_drain) = factory.get_drain_ratio("absorb") {
        println!("   Absorb drain ratio: {} (from PS data [1, 2])", absorb_drain);
    }
    
    if let Some(doubleedge_recoil) = factory.get_recoil_ratio("doubleedge") {
        println!("   Double-Edge recoil ratio: {} (from PS data [33, 100])", doubleedge_recoil);
    }

    println!("\n✅ Demo completed!");
    println!("\n💡 Key Benefits:");
    println!("   • Drain/recoil data read directly from Pokemon Showdown");
    println!("   • No need for manual enhancement registration");
    println!("   • Always up-to-date with PS balance changes");
    println!("   • Fraction arrays [numerator, denominator] provide exact ratios");

    Ok(())
}