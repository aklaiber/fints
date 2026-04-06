//! Tests for the bank registry.

use fints::banks::{all_banks, bank_by_blz};
use fints::banks_generated::BANK_COUNT;

// ─── Registry size ───────────────────────────────────────────────────────────

#[test]
fn test_all_banks_large() {
    let banks = all_banks();
    assert!(
        banks.len() >= 1000,
        "all_banks() should return at least 1000 banks, got {}",
        banks.len()
    );
    assert_eq!(
        banks.len(),
        BANK_COUNT,
        "all_banks() count must match BANK_COUNT"
    );
}

// ─── Known banks by BLZ ──────────────────────────────────────────────────────

#[test]
fn test_bank_by_blz_dkb() {
    let bank = bank_by_blz("12030000").expect("BLZ 12030000 (DKB) must be in the registry");
    assert_eq!(bank.blz.as_str(), "12030000");
    assert!(!bank.name.as_str().is_empty(), "name should be non-empty");
    assert!(!bank.bic.as_str().is_empty(), "bic should be non-empty");
    assert!(
        bank.url.as_str().starts_with("https://"),
        "URL should use HTTPS"
    );
}

#[test]
fn test_bank_by_blz_ing() {
    let bank = bank_by_blz("50010517").expect("BLZ 50010517 (ING) must be in the registry");
    assert_eq!(bank.blz.as_str(), "50010517");
    assert!(!bank.name.as_str().is_empty());
}

#[test]
fn test_bank_by_blz_postbank() {
    let bank = bank_by_blz("10010010").expect("BLZ 10010010 (Postbank) must be in the registry");
    assert_eq!(bank.blz.as_str(), "10010010");
}

// ─── Missing lookups ─────────────────────────────────────────────────────────

#[test]
fn test_bank_by_blz_not_found() {
    assert!(
        bank_by_blz("00000000").is_none(),
        "BLZ '00000000' should not exist in registry"
    );
    assert!(bank_by_blz("").is_none(), "empty BLZ should return None");
    assert!(
        bank_by_blz("not-a-blz").is_none(),
        "non-numeric BLZ should return None"
    );
}

// ─── Data integrity ──────────────────────────────────────────────────────────

#[test]
fn test_all_banks_unique_blz() {
    let banks = all_banks();
    let mut seen = std::collections::HashSet::new();
    for bank in &banks {
        let blz = bank.blz.as_str().to_string();
        assert!(
            seen.insert(blz.clone()),
            "Duplicate BLZ found: {} (bank: {})",
            blz,
            bank.name.as_str()
        );
    }
}

#[test]
fn test_all_banks_have_required_fields() {
    let banks = all_banks();
    for bank in &banks {
        assert!(!bank.blz.as_str().is_empty(), "blz should be non-empty");
        assert_eq!(
            bank.blz.as_str().len(),
            8,
            "BLZ '{}' must be 8 digits",
            bank.blz.as_str()
        );
        assert!(
            !bank.name.as_str().is_empty(),
            "name should be non-empty for BLZ {}",
            bank.blz.as_str()
        );
        assert!(
            bank.url.as_str().starts_with("http"),
            "URL '{}' for BLZ {} should start with http",
            bank.url.as_str(),
            bank.blz.as_str()
        );
    }
}

#[test]
fn test_all_banks_blz_are_numeric() {
    let banks = all_banks();
    for bank in &banks {
        assert!(
            bank.blz.as_str().chars().all(|c| c.is_ascii_digit()),
            "BLZ '{}' must be numeric",
            bank.blz.as_str()
        );
    }
}
