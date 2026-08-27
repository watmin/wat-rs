//! Arc 278 — STONE-Value: `:wat::core::Value`, the universal subtype-top. THE DISCONFIRMING PROBE.
//! RED at HEAD (`:wat::core::Value` is unregistered); GREEN once the stone lands.
//! Contract: DESIGN-STONE-Value-universal-top.md.
//!
//! `:wat::core::Value` is the universal subtype-TOP — "basically Ruby's Object, the value unit for all
//! types" — NOT a defenum/union (we are ADT). The variance is the whole point and it is ONE-directional:
//!
//!   - UP is FREE.  Every type `<: Value`. Any value is assignable where `Value` is wanted. This rides
//!     the EXISTING directional acceptance `assignable` (check.rs:13962 — `is_subtype` FIRST, then a
//!     fall-through to `unify`) + ONE root rule in `is_subtype` (types.rs:3142). No wrapping: the `i64`
//!     stays an `i64`, it just *is-a* `Value` at a `Value`-typed slot.
//!   - DOWN is CHECKED.  A `Value` is NOT assignable where a specific type is wanted.
//!     `assignable(Value, i64)` → `is_subtype(Value, i64) = false` → falls to `unify(Value, i64)` →
//!     distinct concrete paths → FAIL. **This rejection is the whole discipline** — it is what keeps
//!     `Value` from being a loose "any". Narrowing is an explicit, checked downcast (the revive door's
//!     `from-edn :T`), never implicit.
//!
//! Two surfaces, both pinned by the stone's NEXT-ACTION note:
//!   A. `is_subtype` — the type-engine predicate, in isolation (no parser, no checker).
//!   B. `check_program` (via `startup_from_file`, the full type-check pipeline) — the AUTHOR surface:
//!      widen accepted, narrow rejected.
//!
//! THE DISCONFIRM lives in the UP / WIDEN asserts: at HEAD they FAIL because `:wat::core::Value` does not
//! exist, so `is_subtype(_, Value) = false` and any program naming the type is a check error. The
//! DOWN / NARROW asserts encode the NON-NEGOTIABLE discipline — they already hold at HEAD (vacuously, since
//! Value is unregistered) and MUST STILL hold after the stone (for the right reason: narrow rejected). If
//! a down/narrow assert ever flips to accept, `Value` has become a loose any and the stone has FAILED.
//!
//! Out of this probe's scope (delivered-by-consequence, verified elsewhere — exigere):
//!   - DESIGN #2 heterogeneous `PersistentMap<String, Value>`: proven by re-typing Token/Element.bindings
//!     (rete.wat:30,37) → the stone's EXPECTATIONS scorecard (rete.wat still compiles + the engine net).
//!   - DESIGN #4 display ∀T: `println` is already ∀T→EDN; exercised by the P12 EXPLAIN demo (`bound` render).
//!
//! Run: cargo test --release -p wat --test probe_arc278_value_universal_top

use wat::check::error::CheckErrorKind;
use wat::freeze::startup_from_file;
use wat::types::{is_subtype, TypeEnv};

const VALUE: &str = ":wat::core::Value";
const I64: &str = ":wat::core::i64";
const STRING: &str = ":wat::core::String";

/// Type-check a program through the full freeze pipeline (parse → `check_program` → freeze).
/// `Ok(())` iff the program type-checks.
fn typechecks_file(path: &str) -> Result<(), String> {
    startup_from_file(path)
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))
}

// ─────────────────────────────────────────────────────────────────────────────
// Surface A — is_subtype, the predicate in isolation.
// ─────────────────────────────────────────────────────────────────────────────

/// UP is free: every type <: Value. RED at HEAD (Value unregistered → false).
#[test]
fn up_i64_is_subtype_of_value() {
    let env = TypeEnv::with_builtins();
    assert!(
        is_subtype(I64, VALUE, &env),
        "UP must be free: i64 <: Value (RED at HEAD — :wat::core::Value not yet registered)"
    );
}

/// UP is free for String too — Value is the top of ALL types, not just numerics. RED at HEAD.
#[test]
fn up_string_is_subtype_of_value() {
    let env = TypeEnv::with_builtins();
    assert!(
        is_subtype(STRING, VALUE, &env),
        "UP must be free: String <: Value (RED at HEAD)"
    );
}

/// DOWN is rejected: Value is NOT a subtype of i64. THE DISCIPLINE. Holds at HEAD (vacuous);
/// MUST still hold after the stone — if this flips, Value is a loose any and the stone failed.
#[test]
fn down_value_is_not_subtype_of_i64() {
    let env = TypeEnv::with_builtins();
    assert!(
        !is_subtype(VALUE, I64, &env),
        "DOWN must be rejected: Value is NOT <: i64 (the non-negotiable discipline)"
    );
}

/// DOWN rejected for String too — narrowing to any specific type needs an explicit checked downcast.
#[test]
fn down_value_is_not_subtype_of_string() {
    let env = TypeEnv::with_builtins();
    assert!(
        !is_subtype(VALUE, STRING, &env),
        "DOWN must be rejected: Value is NOT <: String (the non-negotiable discipline)"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Surface B — check_program (full pipeline): widen accepted, narrow rejected.
// ─────────────────────────────────────────────────────────────────────────────

/// WIDEN accepted: a record field typed `:wat::core::Value` accepts BOTH an i64 and a String at the
/// constructor boundary (which routes through `assignable` → `is_subtype(_, Value) = true`). RED at HEAD
/// (the type `:wat::core::Value` is unknown → check error).
#[test]
fn widen_record_value_field_accepts_i64_and_string() {
    assert!(
        typechecks_file("tests/types/probe_arc278_value_universal_top_widen.wat").is_ok(),
        "WIDEN must be accepted: i64 AND String are assignable to a :wat::core::Value field \
         (RED at HEAD — :wat::core::Value is an unknown type)"
    );
}

/// NARROW rejected: a `:wat::core::Value`-typed value passed where `:wat::core::i64` is expected is a
/// TYPE ERROR. THE DISCIPLINE at the author surface. `is_err()` holds at HEAD (Value unknown) AND must
/// still hold after the stone (for the right reason: `assignable(Value, i64)` falls to a failing unify).
#[test]
fn narrow_value_into_i64_param_is_type_error() {
    // Bypasses `typechecks_file` (formats to a bare String) — the discriminant needs the
    // structured `StartupError` (arc 296 Stone L).
    let r = startup_from_file("tests/types/probe_arc278_value_universal_top_narrow.wat.bad");
    wat::assert_startup_error!(r, check
        CheckErrorKind::TypeMismatch { callee, param, expected, got, .. }
            if callee == ":my::needs-int"
            && param == "#1"
            && expected == ":wat::core::i64"
            && got == ":wat::core::Value"
    );
}
