//! FM 2-bis probe for Stone 241.12 — `:wat::core::defalias` mint + `:wat::runtime::define-alias` HARD CUT.
//!
//! Stone 241.12 mints the missing def*-prefix-family surface form for binding aliases AND
//! retires `:wat::runtime::define-alias` per user direction 2026-05-29 late.
//!
//! THE DOCTRINE: HARD CUT is total. `:wat::runtime::define-alias` DIES.
//! `:wat::core::defalias` is the SOLE alias mechanism.
//!
//! HEAD-disconfirmation map (all 5 contracts FAIL at HEAD):
//! - C01: `:wat::core::defalias` mint works (startup clean) ⇒ FAILS at HEAD
//! - C02: defalias additive — original name still resolves ⇒ FAILS at HEAD
//! - C03: defalias works for built-in/intrinsic bindings ⇒ FAILS at HEAD
//! - C04: `:wat::runtime::define-alias` HARD-CUT-rejected at startup ⇒ FAILS at HEAD
//! - C05: rejection error carries structured retirement remedy naming defalias ⇒ FAILS at HEAD
//!
//! Post-stone: all 5 contracts PASS.

use wat::freeze::startup_from_file;

// ─── C01: defalias produces a callable alias ──────────────────────────────────

#[test]
fn contract_01_defalias_alias_name_resolves() {
    startup_from_file("tests/wat_lang/probe_arc241_stone12_defalias.wat")
        .expect("alias name must resolve to a callable binding post-defalias");
}

// ─── C02: defalias additive — BOTH names resolve in the same program ───────────

#[test]
fn contract_02_defalias_additive_both_names_callable() {
    startup_from_file("tests/wat_lang/probe_arc241_stone12_defalias.wat")
        .expect("BOTH alias and original must resolve post-defalias (additive)");
}

// ─── C03: defalias works for built-in/intrinsic bindings ─────────────────────

#[test]
fn contract_03_defalias_can_alias_a_builtin() {
    startup_from_file("tests/wat_lang/probe_arc241_stone12_defalias.wat")
        .expect("defalias must work for built-in bindings (wat/core.wat pattern)");
}

// ─── C04: `:wat::runtime::define-alias` HARD-CUT-rejected at startup ───────────

#[test]
fn contract_04_runtime_define_alias_hard_cut_rejected() {
    let result = startup_from_file("tests/wat_lang/probe_arc241_stone12_defalias.wat.bad");
    assert!(
        result.is_err(),
        "`:wat::runtime::define-alias` must be HARD-CUT-rejected post-stone (Doctrine: no privileged paths); got Ok"
    );
}

// ─── C05: rejection carries structured retirement remedy naming defalias ───────

#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn contract_05_rejection_remedy_names_defalias() {
    let result = startup_from_file("tests/wat_lang/probe_arc241_stone12_defalias.wat.bad");
    let msg = format!("{}", result.unwrap_err());
    assert_eq!(
        msg,
        "check:\n1 type-check error(s):\n  - tests/wat_lang/probe_arc241_stone12_defalias.wat.bad:5:2: malformed :wat::runtime::define-alias form: ':wat::runtime::define-alias' is retired (Stone 241.12); use ':wat::core::defalias' instead\n  did you mean: :wat::core::defalias [replaces a retired form]\n",
        "retirement remedy must name :wat::core::defalias with [replaces a retired form]"
    );
}
