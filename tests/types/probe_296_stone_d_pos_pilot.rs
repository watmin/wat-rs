//! Arc 296 stone D — PILOT: `#[derive(Edn)]` on `Pos` round-trips via the
//! derive's self-registration (no hand-written PROBE in types.rs).
//!
//! Contracts:
//! C1 — `TypeEnv::with_builtins()` registers `:wat::core::Pos` via the
//!      inventory drain (not the removed hand-PROBE).
//! C2 — `read_edn("#wat.core/Pos {:line 1 :col 2}", Some(&types))`
//!      reconstructs a Value::Aggregate (not an UnknownTag error).
//! C3 — The write side still emits `#wat.core/Pos {:line N :col N}`.

use wat::edn_shim::read_edn;
use wat::runtime::Value;
use wat::types::TypeEnv;
use wat_reader::span::Pos;
use wat_edn::ToEdn;

/// C1: the derive drain registered :wat::core::Pos.
#[test]
fn c1_pos_is_registered_via_derive_drain() {
    let types = TypeEnv::with_builtins();
    assert!(
        types.get(":wat::core::Pos").is_some(),
        "C1 FAIL: :wat::core::Pos not found in TypeEnv — drain did not run or Edn derive did not submit"
    );
    eprintln!("C1 PASS: :wat::core::Pos is registered");
}

/// C2: edn::read reconstructs #wat.core/Pos {:line 1 :col 2} to an Aggregate record.
#[test]
fn c2_pos_edn_read_reconstructs() {
    let types = TypeEnv::with_builtins();
    // rune:lint(no-inlined-edn) — input under test: a tagged Pos record source fed to read_edn
    let result = read_edn(r##"#wat.core/Pos {:line 1 :col 2}"##, Some(&types), None);
    match result {
        Ok(Value::Aggregate(agg)) => {
            eprintln!("C2 PASS: edn::read returned Aggregate {:?}", agg.class);
            assert_eq!(agg.class.as_ref(), "wat::core::Pos",
                "C2 FAIL: class mismatch, expected wat::core::Pos");
        }
        Ok(other) => {
            panic!("C2 FAIL: expected Aggregate, got {:?}", other);
        }
        Err(e) => {
            panic!("C2 FAIL: edn::read returned error: {:?}", e);
        }
    }
}

/// C3: the write side still emits the correct EDN string.
#[test]
fn c3_pos_to_edn_write() {
    let pos = Pos { line: 5, col: 12 };
    let edn_value = pos.to_edn();
    let s = wat_edn::write(&edn_value);
    eprintln!("C3 written EDN: {}", s);
    wat::assert_edn_matches_file!(s, "probe_296_stone_d_pos_pilot__pos_write.edn", "C3 FAIL: write output mismatch");
}
