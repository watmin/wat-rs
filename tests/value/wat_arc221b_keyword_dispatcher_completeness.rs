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
//!   5. EDN keyword reader (`to_holon_inner`'s keyword arm, `src/runtime.rs`) — EDN `:foo::bar` parsed to
//!      `HolonAST::Keyword("foo::bar")` (no leading colon).
//!   6. Value::Unit consistency (Option A) — `Value::Unit` → `HolonAST::Nil` via
//!      both the 14018 dispatcher and `:wat::holon::leaf`.
//!
//! Wat source lives in the co-located fixture: wat_arc221b_keyword_dispatcher_completeness.wat
//! (slurped via startup_beside(file!())).

use wat::freeze::startup_beside;
use wat::runtime::{apply_function, Value};

// just-eval (rubric): each `:t::…` fixture fn is a zero-arg entry; fetch it from the frozen
// world and `apply_function` it — no inline wat driver.
fn run_string(world: &wat::freeze::FrozenWorld, fn_name: &str) -> String {
    let func = world
        .symbols()
        .get(fn_name)
        .unwrap_or_else(|| panic!("no {fn_name:?} in fixture"))
        .clone();
    match apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .expect("eval should succeed")
    {
        Value::String(s) => s.as_str().to_string(),
        other => panic!("expected String; got {:?}", other),
    }
}

// ─── Probe 1 — `watast_to_holon` Keyword arm (runtime.rs:13959) ─────────────

/// `(:wat::holon::from-wat (:wat::core::quote :foo))` calls `watast_to_holon`
/// on a `WatAST::Keyword(":foo")`. Stone 221.4b maps it to `HolonAST::Keyword("foo")`
/// (no leading colon). Arc 294.j: EDN write emits the bare keyword `:foo` — NOT a
/// tagged form (the algebra never crosses the wire; a Keyword composition is one of
/// the leaves `holon_to_watast` intercepts and renders as its plain EDN equivalent).
#[test]
fn probe_1_watast_to_holon_keyword_arm_produces_keyword_leaf() {
    let world = startup_beside(file!()).expect("startup");

    // THE CLAIM (arc 221 doctrine): the Keyword arm emits `HolonAST::Keyword`, NOT
    // `HolonAST::Symbol`. Asserted on the VARIANT, because as of arc 294.j the wire
    // renders both as `#wat/holon :foo` and can no longer discriminate them (task
    // #103, RULED NOT NOW pending proper symbols). The golden below is an encoding
    // regression guard, not the proof.
    assert_eq!(run_string(&world, ":t::probe-1-is-keyword"), "true",
        "watast_to_holon's Keyword arm must produce a Keyword leaf — THE arc-221 claim");
    assert_eq!(run_string(&world, ":t::probe-1-is-symbol"), "false",
        "…and NOT a Symbol leaf — the half arc 221 exists to prevent");

    let s = run_string(&world, ":t::probe-1");
    wat::assert_edn_matches_file!(s, "wat_arc221b_keyword_dispatcher_completeness__keyword_foo.edn", "watast_to_holon Keyword must emit exact golden");
}

// ─── Probe 2 — `:wat::holon::leaf` Keyword arm (runtime.rs:20938) ───────────

/// `(:wat::holon::leaf :user::foo)` dispatches through `eval_holon_leaf`'s
/// `Value::wat__core__keyword` arm (Stone 221.4b) to `HolonAST::Keyword("user::foo")`.
/// Arc 294.j: EDN write emits the bare keyword `:user/foo` (`::` translated to `/`
/// per the standard wat-path↔EDN-keyword convention) — no tag.
#[test]
fn probe_2_holon_leaf_keyword_produces_keyword_leaf() {
    let world = startup_beside(file!()).expect("startup");

    // THE CLAIM, on the variant — see probe 1's note.
    assert_eq!(run_string(&world, ":t::probe-2-is-keyword"), "true",
        "`leaf`'s keyword arm must produce a Keyword leaf — THE arc-221 claim");

    let s = run_string(&world, ":t::probe-2");
    wat::assert_edn_matches_file!(s, "wat_arc221b_keyword_dispatcher_completeness__keyword_user_foo.edn", "holon_leaf Keyword must emit exact golden");
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
    let s_a = run_string(&world, ":t::probe-3a");
    // rune:lint(no-inlined-wat) — `<HolonAST>` is a Display-rendering placeholder token, not
    // wat source; this is the `Debug`/`Display` text of a `StepResult` value, never evaluated.
    assert_eq!(s_a, "(:wat::eval::StepResult::AlreadyTerminal <HolonAST>)", "eval-step! keyword must emit exact golden");

    // Part B: from-wat(quote :outcome) and from-wat(quote :outcome) are equal
    // (same Keyword identity — both go through Stone 221.4b watast_to_holon).
    let s_b = run_string(&world, ":t::probe-3b");
    assert_eq!(s_b, "true", "same keyword must produce equal HolonAST identities");
}

// ─── Probe 4 — EDN keyword wire format (`from_holon_item`'s keyword arm, `src/runtime.rs`) ───────────────────

/// `HolonAST::Keyword("bar")` written via `edn::write` emits the bare EDN
/// keyword `:bar` — arc 294.j: a Keyword composition is a leaf `holon_to_watast`
/// intercepts and renders as its plain equivalent, no tag.
#[test]
fn probe_4_edn_write_keyword_leaf_emits_keyword_tag() {
    let world = startup_beside(file!()).expect("startup");
    let s = run_string(&world, ":t::probe-4");
    wat::assert_edn_matches_file!(s, "wat_arc221b_keyword_dispatcher_completeness__keyword_bar.edn", "edn::write Keyword must emit exact golden");
}

// ─── Probe 5 — Value::Unit consistency — `:wat::holon::leaf` nil (arc 230) ──────

/// Arc 230 — `nil()` is now `Bind(Atom("Symbol"), Atom("nil"))`.
/// `(:wat::holon::leaf :wat::core::nil)` where `:wat::core::nil` evaluates to
/// `Value::Unit` (wat's nil). Pre-arc-230 this emitted a `Nil`-tagged form; the
/// Nil variant is retired. Arc 294.j: EDN write now emits the bare `nil` literal
/// — `holon_to_watast`'s Symbol intercept maps the `"nil"` composition straight
/// to `WatAST::NilLit`, no tag either way.
#[test]
fn probe_5_holon_leaf_unit_produces_nil_leaf() {
    let world = startup_beside(file!()).expect("startup");

    // THE CLAIM, on the variant — see probe 1's note. `nil` is the one leaf whose
    // Symbol-ness is CORRECT (arc 230: nil = Bind(Atom("Symbol"), Atom("nil"))), so
    // `is-Nil?` is the honest discriminator here rather than is-Keyword?/is-Symbol?.
    assert_eq!(run_string(&world, ":t::probe-5-is-nil"), "true",
        "`leaf` of Value::Unit must produce the nil leaf — THE arc-230 claim");

    let s = run_string(&world, ":t::probe-5");
    wat::assert_edn_matches_file!(s, "wat_arc221b_keyword_dispatcher_completeness__symbol_nil.edn", "holon_leaf unit must emit exact golden");
}

// ─── Probe 6 — `watast_to_holon` Keyword round-trip distinctness ─────────────

/// Two distinct keywords lower to distinct `HolonAST::Keyword` leaves.
/// `from-wat(quote :foo)` ≠ `from-wat(quote :bar)`.
#[test]
fn probe_6_watast_to_holon_keyword_distinct_identities() {
    let world = startup_beside(file!()).expect("startup");
    let s = run_string(&world, ":t::probe-6");
    // (:wat::core::not false) = true → edn::write "true".
    assert_eq!(s, "true", "distinct keywords must be non-equal");
}
