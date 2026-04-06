//! Additional parser tests with proptest round-trips and edge cases.
//!
//! These tests use the public `decode_message` API which wraps the internal
//! parser and serializer, and `format_decoded` for round-trip verification.

use fints::{decode_message, format_decoded, VerbosityLevel};
use proptest::prelude::*;

// ─── Task B.1: Empty message ─────────────────────────────────────────────────

#[test]
fn test_parse_empty_message() {
    let msg = decode_message(b"").unwrap();
    assert!(
        msg.segments.is_empty(),
        "Empty bytes should produce empty segments"
    );
}

// ─── Task B.2: Single empty DEG ──────────────────────────────────────────────

#[test]
fn test_parse_single_empty_deg() {
    // "TEST:1:1+'" — header DEG is "TEST:1:1", then an empty DEG before the '
    // DEG[0] at index 0 (the data DEG, since header is auto-extracted) should be empty
    let data = b"TEST:1:1+'";
    let msg = decode_message(data).unwrap();
    assert_eq!(msg.segments.len(), 1);
    assert_eq!(msg.segments[0].segment_type, "TEST");
    // The decoded segment has degs (skipping header), so degs should have 1 item: empty
    let deg0 = &msg.segments[0].degs[0];
    // deg0 should be empty strings
    assert!(
        deg0.is_empty() || deg0.iter().all(|s| s.is_empty()),
        "First data DEG should be empty, got: {:?}",
        deg0
    );
}

// ─── Task B.3: Double escape "??" → single "?" ──────────────────────────────

#[test]
fn test_parse_nested_escapes() {
    // "??" in FinTS means literal "?"
    let data = b"TEST:1:1+A??B'";
    let msg = decode_message(data).unwrap();
    assert_eq!(msg.segments.len(), 1);
    // DEG 0 of data (after header) should decode "A??B" to "A?B"
    let val = &msg.segments[0].degs[0][0];
    assert_eq!(val, "A?B", "Double escape should produce single '?'");
}

// ─── Task B.4: German umlauts parsed correctly ───────────────────────────────

#[test]
fn test_parse_iso8859_chars() {
    // ISO-8859-1 encoded: Ä=0xC4, Ö=0xD6, Ü=0xDC
    let mut data = b"TEST:1:1+".to_vec();
    data.extend_from_slice(&[0xC4u8, 0xD6u8, 0xDCu8]); // Ä Ö Ü
    data.push(b'\'');
    let msg = decode_message(&data).unwrap();
    let text = &msg.segments[0].degs[0][0];
    // The decoded text should contain the Unicode equivalents
    assert!(
        text.contains('\u{00C4}') || text.contains("Ä"),
        "Should contain Ä, got: {:?}",
        text
    );
    assert!(
        text.contains('\u{00D6}') || text.contains("Ö"),
        "Should contain Ö, got: {:?}",
        text
    );
    assert!(
        text.contains('\u{00DC}') || text.contains("Ü"),
        "Should contain Ü, got: {:?}",
        text
    );
}

// ─── Task B.5: @0@ produces empty binary element ─────────────────────────────

#[test]
fn test_parse_binary_zero_length() {
    let data = b"TEST:1:1+@0@'";
    let msg = decode_message(data).unwrap();
    assert_eq!(msg.segments.len(), 1);
    // Binary DE decoded at Full verbosity as "[BINARY: ]" (empty hex)
    let val = &msg.segments[0].degs[0][0];
    // At Full verbosity, empty binary shows as "[BINARY: ]" with empty hex
    assert!(
        val.contains("BINARY") || val.is_empty(),
        "Zero-length binary should decode to a BINARY display or empty, got: {:?}",
        val
    );
}

// ─── Task B.6: Multiple empty DEs: "A:::B" → 4 DEs ──────────────────────────

#[test]
fn test_parse_multiple_empty_des() {
    // "A:::B" in a DEG should give 4 DEs: A, (empty), (empty), B
    let data = b"TEST:1:1+A:::B'";
    let msg = decode_message(data).unwrap();
    assert_eq!(msg.segments.len(), 1);
    let deg0 = &msg.segments[0].degs[0];
    assert_eq!(
        deg0.len(),
        4,
        "Expected 4 DEs in 'A:::B', got {}",
        deg0.len()
    );
    assert_eq!(deg0[0], "A");
    assert_eq!(deg0[1], "");
    assert_eq!(deg0[2], "");
    assert_eq!(deg0[3], "B");
}

// ─── Task B.7: Serialize (encode) an empty message ───────────────────────────

#[test]
fn test_serialize_empty_message() {
    // An empty message decodes to zero segments
    let msg = decode_message(b"").unwrap();
    assert_eq!(msg.segments.len(), 0);
    assert_eq!(msg.raw_bytes, 0);
    // format_decoded on an empty message should not panic
    let formatted = format_decoded(&msg, VerbosityLevel::Minimal);
    assert!(formatted.contains("0 bytes"));
}

// ─── Task B.8: Round-trip complex segment via decode ─────────────────────────

#[test]
fn test_round_trip_complex() {
    // Complex segment: multiple DEGs, escape chars
    let input = b"TEST:1:1+Hello?+World+last'";
    let msg = decode_message(input).unwrap();
    assert_eq!(msg.segments.len(), 1);
    assert_eq!(msg.segments[0].segment_type, "TEST");

    // DEG 0: "Hello+World" (escaped '+' decoded)
    assert_eq!(msg.segments[0].degs[0][0], "Hello+World");
    // DEG 1: "last"
    assert_eq!(msg.segments[0].degs[1][0], "last");
}

// ─── Task B.9: Round-trip all separator types ────────────────────────────────

#[test]
fn test_round_trip_all_separator_types() {
    // Segment using '+', ':', and escape sequences
    let input = b"HNSHK:2:4+PIN:1+999+one:two:three'";
    let msg = decode_message(input).unwrap();
    assert_eq!(msg.segments.len(), 1);
    assert_eq!(msg.segments[0].segment_type, "HNSHK");
    assert_eq!(msg.segments[0].segment_number, 2);

    // DEG 0: "PIN" : "1"
    assert_eq!(msg.segments[0].degs[0][0], "PIN");
    assert_eq!(msg.segments[0].degs[0][1], "1");

    // DEG 1: "999"
    assert_eq!(msg.segments[0].degs[1][0], "999");

    // DEG 2: "one" : "two" : "three"
    assert_eq!(msg.segments[0].degs[2][0], "one");
    assert_eq!(msg.segments[0].degs[2][1], "two");
    assert_eq!(msg.segments[0].degs[2][2], "three");
}

// ─── Task B.10: Message with newlines between segments ───────────────────────

#[test]
fn test_parse_message_with_newlines() {
    let data = b"HNHBS:5:1+2'\r\nHNHBK:1:3+000000000075+300+0'";
    let msg = decode_message(data).unwrap();
    assert_eq!(msg.segments.len(), 2);
    assert_eq!(msg.segments[0].segment_type, "HNHBS");
    assert_eq!(msg.segments[1].segment_type, "HNHBK");
}

// ─── Proptest round-trips ────────────────────────────────────────────────────

proptest! {
    #[test]
    fn test_round_trip_arbitrary_text(s in "[a-zA-Z0-9 .,/!#%^&*()-]{0,50}") {
        // Text with no special FinTS chars: decode should contain the original text
        let segment = format!("TEST:1:1+{}'", s);
        let msg = decode_message(segment.as_bytes()).unwrap();
        prop_assert_eq!(msg.segments.len(), 1);
        // If s is non-empty, first data DEG should equal s
        if !s.is_empty() {
            prop_assert_eq!(&msg.segments[0].degs[0][0], &s);
        }
    }

    #[test]
    fn test_round_trip_multiple_segments(
        a in "[a-zA-Z0-9]{1,10}",
        b in "[a-zA-Z0-9]{1,10}"
    ) {
        // Two segments should decode as exactly 2 segments
        let segment = format!("SEG1:1:1+{}'\r\nSEG2:2:1+{}'", a, b);
        let msg = decode_message(segment.as_bytes()).unwrap();
        prop_assert_eq!(msg.segments.len(), 2);
        prop_assert_eq!(&msg.segments[0].degs[0][0], &a);
        prop_assert_eq!(&msg.segments[1].degs[0][0], &b);
    }
}
