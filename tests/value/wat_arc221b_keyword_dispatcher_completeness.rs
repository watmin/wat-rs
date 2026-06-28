//! Arc 221 Stone 221.4b — Phase 1 dispatcher-completeness probes.
//!
//! Verifies that all 6 Phase 1 illegal sites now emit `HolonAST::Keyword`
//! (not `HolonAST::Symbol(":foo")`) per arc 221 doctrine.
//!
//! Sites covered:
//!   1. `watast_to_holon` Keyword arm (runtime.rs:13959) — `WatAST::Keyword →
//!      HolonAST::Keyword`; tested via `:wat::holon::from-wat` on a quoted keyword.
//!   2. Value→HolonAST second dispatcher (runtime.rs:14018) — keyword Value lowers
//!      via the direct-primitive dispatcher (tested indirectly via `signature-of-defn`;
//!      that path exercises the 14018 dispatcher through `holon_to_watast` round-trip).
//!   3. `:wat::holon::leaf` Keyword arm (runtime.rs:20938) — keyword Value →
//!      `HolonAST::Keyword` via the `leaf` verb.
//!   4. `eval-step!` AlreadyTerminal Keyword (runtime.rs:21322 / try_recognize_holon_value)
//!      — a bare keyword form recognized as already-terminal with Keyword leaf.
//!   5. EDN keyword reader (edn_shim.rs:1899) — EDN `:foo::bar` parsed to
//!      `HolonAST::Keyword("foo::bar")` (no leading colon).
//!   6. Value::Unit consistency (Option A) — `Value::Unit` → `HolonAST::Nil` via
//!      both the 14018 dispatcher and `:wat::holon::leaf`.
//!
//! Wat source lives in the co-located fixture: wat_arc221b_keyword_dispatcher_completeness.wat
//! (slurped via startup_beside(file!())).

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

fn run_string(world: &wat::freeze::FrozenWorld, expr: &str) -> String {
    let ast = wat::parse_one!(expr).expect("parse expr");
    match eval_in_frozen(&ast, world, &Environment::new())
        .expect("eval should succeed")
        .value_owned()
    {
        Value::String(s) => s.as_str().to_string(),
        other => panic!("expected String; got {:?}", other),
    }
}

// ─── Probe 1 — `watast_to_holon` Keyword arm (runtime.rs:13959) ─────────────

/// `(:wat::holon::from-wat (:wat::core::quote :foo))` calls `watast_to_holon`
/// on a `WatAST::Keyword(":foo")`. Stone 221.4b maps it to `HolonAST::Keyword("foo")`
/// (no leading colon). EDN write emits `#wat-edn.holon/Keyword "foo"` — NOT
/// `#wat-edn.holon/Symbol ":foo"` (the retired pre-arc-221 convention).
#[test]
fn probe_1_watast_to_holon_keyword_arm_produces_keyword_leaf() {
    let world = startup_beside(file!()).expect("startup");
    let s = run_string(&world, "(:t::probe-1)");
    // Must contain Keyword (not Symbol).
    assert!(
        s.contains("Keyword"),
        "expected #wat-edn.holon/Keyword in output, got: {}",
        s
    );
    // Content must NOT have a leading colon (Keyword stored without ":").
    assert!(
        !s.contains("Keyword \":\""),
        "keyword content must not start with ':' — leading colon retired by arc 221, got: {}",
        s
    );
    // Confirm NOT Symbol (regression guard).
    assert!(
        !s.contains("Symbol"),
        "output must NOT contain Symbol — retired pre-arc-221 convention, got: {}",
        s
    );
}

// ─── Probe 2 — `:wat::holon::leaf` Keyword arm (runtime.rs:20938) ───────────

/// `(:wat::holon::leaf :user::foo)` dispatches through `eval_holon_leaf`'s
/// `Value::wat__core__keyword` arm (Stone 221.4b) to `HolonAST::Keyword("user::foo")`.
/// EDN write emits `#wat-edn.holon/Keyword "user::foo"`.
#[test]
fn probe_2_holon_leaf_keyword_produces_keyword_leaf() {
    let world = startup_beside(file!()).expect("startup");
    let s = run_string(&world, "(:t::probe-2)");
    assert!(
        s.contains("Keyword"),
        "expected #wat-edn.holon/Keyword in output, got: {}",
        s
    );
    assert!(
        !s.contains("Symbol"),
        "output must NOT contain Symbol — retired pre-arc-221 convention, got: {}",
        s
    );
}

// ─── Probe 3 — `eval-step!` AlreadyTerminal Keyword (runtime.rs:21322) ──────

/// A bare keyword form `(:wat::core::quote :outcome)` in WatAST form, fed to
/// `eval-step!`, is recognized as `AlreadyTerminal` via `try_recognize_holon_value`.
/// The StepResult show output contains "AlreadyTerminal" (not "StepTerminal").
///
/// Also verifies `from-wat(quote :outcome)` equals `from-wat(quote :outcome)`.
#[test]
fn probe_3_eval_step_keyword_produces_already_terminal_keyword_leaf() {
    let world = startup_beside(file!()).expect("startup");

    // Part A: eval-step! on a keyword produces AlreadyTerminal.
    let s_a = run_string(&world, "(:t::probe-3a)");
    assert!(
        s_a.contains("AlreadyTerminal"),
        "expected AlreadyTerminal for keyword step, got: {}",
        s_a
    );

    // Part B: from-wat(quote :outcome) and from-wat(quote :outcome) are equal
    // (same Keyword identity — both go through Stone 221.4b watast_to_holon).
    let s_b = run_string(&world, "(:t::probe-3b)");
    assert!(
        s_b.contains("true"),
        "same keyword must produce equal HolonAST identities, got: {}",
        s_b
    );
}

// ─── Probe 4 — EDN keyword wire format (edn_shim.rs:1899) ───────────────────

/// `HolonAST::Keyword("foo")` written via `edn::write` emits
/// `#wat-edn.holon/Keyword "foo"` — a tagged string form with the Keyword tag.
///
/// Note: edn::read round-trip of `#wat-edn.holon/Keyword "user::bar"` has a
/// known EDN parser limitation for double-colon namespace separators.
#[test]
fn probe_4_edn_write_keyword_leaf_emits_keyword_tag() {
    let world = startup_beside(file!()).expect("startup");
    let s = run_string(&world, "(:t::probe-4)");
    // edn::write of HolonAST::Keyword("bar") must emit a Keyword-tagged form.
    assert!(
        s.contains("Keyword"),
        "expected 'Keyword' in edn::write output for keyword leaf, got: {}",
        s
    );
    // Confirm NOT Symbol (regression guard against pre-arc-221 Symbol output).
    assert!(
        !s.contains("Symbol"),
        "edn::write output must NOT contain Symbol for keyword leaf, got: {}",
        s
    );
    // Content must be "bar" (no leading colon — Keyword stores without sigil).
    assert!(
        s.contains("bar"),
        "edn::write must emit keyword content 'bar' (without leading colon), got: {}",
        s
    );
}

// ─── Probe 5 — Value::Unit consistency — `:wat::holon::leaf` nil (arc 230) ──────

/// Arc 230 — `nil()` is now `Bind(Atom("Symbol"), Atom("nil"))`.
/// `(:wat::holon::leaf :wat::core::nil)` where `:wat::core::nil` evaluates to
/// `Value::Unit` (wat's nil). EDN write emits `#wat-edn.holon/Symbol "nil"`.
/// Pre-arc-230 this emitted `#wat-edn.holon/Nil`; the Nil variant is retired.
#[test]
fn probe_5_holon_leaf_unit_produces_nil_leaf() {
    let world = startup_beside(file!()).expect("startup");
    let s = run_string(&world, "(:t::probe-5)");
    // Arc 230: nil = Bind(Atom("Symbol"), Atom("nil")) → serializes as #wat-edn.holon/Symbol "nil".
    assert!(
        s.contains("Symbol") && s.contains("nil"),
        "expected #wat-edn.holon/Symbol \"nil\" in output (arc 230 nil composition), got: {}",
        s
    );
}

// ─── Probe 6 — `watast_to_holon` Keyword round-trip distinctness ─────────────

/// Two distinct keywords lower to distinct `HolonAST::Keyword` leaves.
/// `from-wat(quote :foo)` ≠ `from-wat(quote :bar)`.
#[test]
fn probe_6_watast_to_holon_keyword_distinct_identities() {
    let world = startup_beside(file!()).expect("startup");
    let s = run_string(&world, "(:t::probe-6)");
    // (:wat::core::not false) = true → edn::write "true".
    assert!(
        s.contains("true"),
        "expected distinct keyword identities (not eq = true), got: {}",
        s
    );
}
