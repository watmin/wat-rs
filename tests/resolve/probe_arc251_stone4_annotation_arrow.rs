//! FM 2-bis probe — arc 251 Stone 251.4a: `:-` annotation arrow in binder + return.
//!
//! core.typed annotates with `:-` in both param and return position. wat uses `<-`
//! (binder, argspec/parse.rs:166) and `->` (return, function/parse.rs:160). 251.4a
//! adds `:-` as a DUAL-READ alias for both arrows. `:-` is a keyword token; the
//! arrows are bare symbols.
//!
//! HEAD-disconfirmation:
//! - C01: a defn with `:-` in binder AND return ⇒ FAILS at HEAD (the binder slot 1
//!   accepts only the bare symbol `<-`; the return slot accepts only `->`).
//!   Load-bearing: the body does i64 arithmetic on the param.
//! - C02: the `<-`/`->` spelling STILL checks (PRESERVATION — arrows HARD-CUT at 251.5).
//!
//! Post-251.4a: both contracts PASS.
//!
//! Run: `cargo test --release --test probe_arc251_stone4_annotation_arrow`

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

fn checks(decls: &str) -> Result<(), String> {
    let src = format!("{decls}\n(:wat::core::defn :user::main [] -> :wat::core::nil nil)");
    startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))
}

// ─── C01: THE GAP — `:-` in binder + return ─────────────────────────────────────

#[test]
fn contract_01_colon_dash_annotation_checks() {
    let r = checks(
        "(:wat::core::defn :user::inc [x :- :wat::core::i64] :- :wat::core::i64 \
           (:wat::core::i64::+ x 1))",
    );
    assert!(
        r.is_ok(),
        ":- must annotate in both binder and return position (like <- / ->); got {r:?}"
    );
}

// ─── C02: PRESERVATION — the `<-` / `->` arrows still check ──────────────────────

#[test]
fn contract_02_legacy_arrows_still_check() {
    assert!(
        checks(
            "(:wat::core::defn :user::inc [x <- :wat::core::i64] -> :wat::core::i64 \
               (:wat::core::i64::+ x 1))"
        )
        .is_ok(),
        "<- / -> arrows must keep working during the transition"
    );
}
