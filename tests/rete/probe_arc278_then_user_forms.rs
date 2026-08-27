//! Arc 278 — Stone B (`BRIEF-then-user-forms.md`): `:then` admits user forms.
//!
//! Four worlds, mirroring `probe_arc278_6b_ii_a_where_oracle.rs` / `probe_fence_names_the_head.rs`'s
//! own co-located-fixture + `startup_from_file` pattern exactly, one layer down (RHS instead of
//! LHS):
//!
//!   - `probe_arc278_then_user_forms_expr.wat`     — GREEN, widening (b): a constructor item
//!     whose value-position operand is a fenced expression (`(:wat::rete::i64::+ ?n 1 :undefined 0)`,
//!     the brief's own headline example). Fired through BOTH the oracle (`fire-rules$oracle`, the
//!     interpreted `build_insert_fact` reference) and the native kernel (`fire-rules`, the
//!     compiled `RhsOp::Expr` path) — same expected value proves compiled == interpreted
//!     end-to-end, not only in `compiled_rhs.rs`'s own unit differential.
//!   - `probe_arc278_then_user_forms_userfn.wat`   — GREEN, widening (a): the item's HEAD is a
//!     user fn (not a fact-type constructor), reading an accumulate-bound
//!     `PersistentVector<Record>` — new capability, unreachable by `:then` before this stone.
//!   - `probe_arc278_then_user_forms_impure.wat`   — RED: the item's head fn is composed of a
//!     core-namespaced, genuinely impure op. The compile fence must PANIC (`Option/expect` →
//!     `panic_any`, same mechanism `where`'s fence uses), naming the offending head and axis.
//!   - `probe_arc278_then_user_forms_notfact.wat`  — RED: the item's head fn is pure ∧
//!     deterministic ∧ rete-composed but does NOT return a fact type. This is `:then`'s OWN
//!     second check (`where` never claims to produce anything) — it raises normally (a
//!     `field-names-of` diagnostic), not via panic; there is no axis to name.
//!
//! Run: cargo test --release -p wat --test probe_arc278_then_user_forms

use wat::assertion::AssertionPayload;
use wat::freeze::{startup_from_file, FrozenWorld};
use wat::runtime::{apply_function, RuntimeError, RuntimeErrorKind, Value};

const WORLD_EXPR: &str = "tests/rete/probe_arc278_then_user_forms_expr.wat";
const WORLD_USERFN: &str = "tests/rete/probe_arc278_then_user_forms_userfn.wat";
const WORLD_IMPURE: &str = "tests/rete/probe_arc278_then_user_forms_impure.wat";
const WORLD_NOTFACT: &str = "tests/rete/probe_arc278_then_user_forms_notfact.wat";

/// Call the named zero-arg entry fn and return its result, or an `Err` string for either an
/// ordinary raise OR the fence's `Option/expect` panic (caught via `catch_unwind`, exactly as
/// `probe_fence_names_the_head.rs`'s `compile_message` does) — the caller decides which shape it
/// expected.
fn run(world_path: &str, fn_name: &str) -> Result<Value, String> {
    let world: FrozenWorld = startup_from_file(world_path).map_err(|e| format!("startup: {e:?}"))?;
    let func = world.symbols().get(fn_name).unwrap_or_else(|| panic!("no entry fn {fn_name:?}")).clone();
    let sym = world.symbols();
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        apply_function(func, vec![], sym, wat::rust_caller_span!())
    })) {
        Ok(res) => res.map_err(|e| format!("eval: {e:?}")),
        Err(panic_payload) => {
            if let Some(p) = panic_payload.downcast_ref::<AssertionPayload>() {
                Err(p.message.clone())
            } else if let Some(s) = panic_payload.downcast_ref::<String>() {
                Err(s.clone())
            } else if let Some(s) = panic_payload.downcast_ref::<&str>() {
                Err((*s).to_string())
            } else {
                Err("panic-opaque".to_string())
            }
        }
    }
}

/// Sibling of `run` that keeps the typed `RuntimeError` instead of `format!`-collapsing it to a
/// `String` — for a call site known NOT to hit the compile-fence panic path (arc 296 Stone L:
/// `run`'s String-collapse erases the discriminant `non_fact_return_type_is_refused` needs).
fn run_typed(world_path: &str, fn_name: &str) -> Result<Value, RuntimeError> {
    let world: FrozenWorld = startup_from_file(world_path).expect("startup");
    let func = world.symbols().get(fn_name).unwrap_or_else(|| panic!("no entry fn {fn_name:?}")).clone();
    apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
}

/// GREEN, widening (b), oracle path: n=5 -> count=6. Unconfounded — no fact of count=6 could
/// pre-exist; only the derivation produces it.
#[test]
fn expr_operand_derives_via_oracle() {
    let r = run(WORLD_EXPR, ":user::run-count-oracle");
    assert!(matches!(r, Ok(Value::i64(6))), "expected count=6 via the oracle; got {r:?}");
}

/// GREEN, widening (b), NATIVE path: same rule, same expected value, through `compile_rhs`'s
/// `RhsOp::Expr` instead of the interpreted reference.
#[test]
fn expr_operand_derives_via_native_kernel() {
    let r = run(WORLD_EXPR, ":user::run-count-native");
    assert!(matches!(r, Ok(Value::i64(6))), "expected count=6 via the native kernel; got {r:?}");
}

/// GREEN, widening (a): the fn-headed item compiles and fires without raising, and the derived
/// (accumulated + extracted) fact's field reads back the value it was seeded with. See the
/// fixture's own doc for why this is not — and cannot be, short of fixing a separate, pre-existing
/// substrate gap — an UNCONFOUNDED "a new fact was derived" witness.
#[test]
fn userfn_head_item_compiles_and_fires() {
    let r = run(WORLD_USERFN, ":user::run-first-count");
    assert!(matches!(r, Ok(Value::i64(5))), "expected count=5 (the seeded Rate); got {r:?}");
}

/// GREEN, widening (a), NATIVE: same rule through compiled `CompiledRhs::Call`.
#[test]
fn userfn_head_item_fires_via_native_kernel() {
    let r = run(WORLD_USERFN, ":user::run-first-count-native");
    assert!(
        matches!(r, Ok(Value::i64(5))),
        "expected count=5 via compiled fn-headed :then; got {r:?}"
    );
}

/// RED: an impure fn composed of a core-namespaced op is refused at compile, naming the exact
/// offending head and axis — `assert_eq!` (not a substring), same discipline as
/// `probe_fence_names_the_head.rs`'s mutation-proven pins.
#[test]
fn impure_fn_head_names_the_offending_head_and_axis() {
    let r = run(WORLD_IMPURE, ":user::run-compile");
    let msg = r.expect_err("an impure :then item head must fail to compile");
    assert_eq!(
        msg,
        "compile-condition: then expr is not pure — ':wat::io::IOReader/open-file' is not pure"
    );
}

/// RED: a pure ∧ deterministic ∧ rete-composed fn that does not return a fact type is STILL
/// refused — `:then`'s own second check, independent of the axis fence `where` already has.
#[test]
fn non_fact_return_type_is_refused() {
    // `run_typed`, not `run` — this path raises normally (no compile-fence panic, per this
    // test's own doc comment above), so the typed `RuntimeError` survives; grounded via
    // `./target/release/wat` on a scratch `:user::main` invoking `:user::run-compile`.
    let typed = run_typed(WORLD_NOTFACT, ":user::run-compile");
    assert!(
        matches!(
            typed.as_ref().map_err(RuntimeError::kind),
            Err(RuntimeErrorKind::MalformedForm { head, reason })
                if head == ":wat::runtime::field-names-of"
                && reason == "unknown type ':wat::core::i64'"
        ),
        "expected RuntimeErrorKind::MalformedForm(field-names-of, unknown type i64); got {:?}",
        typed
    );
    let r = run(WORLD_NOTFACT, ":user::run-compile");
    let msg = r.unwrap_err();
    // rune:lint(loose-assert) — the diagnostic embeds an absolute file path (Span), which is
    // non-deterministic across machines/CI; assert the load-bearing SUBSTANCE (which type was
    // rejected), not the whole rendered blob.
    assert!(msg.contains("wat::core::i64"), "must name the offending (non-fact) return type:\n{msg}");
    // rune:lint(loose-assert) — same span/path reason. `:wat::core::i64` is a built-in primitive,
    // not a registered user TypeEnv entry at all, so `field-names-of` reports it "unknown" rather
    // than "not a struct/record type (no fields)" (that second phrasing is reserved for a
    // KNOWN-but-non-aggregate type, e.g. a Newtype/Enum) — either phrasing proves the same thing:
    // this head does not return a fact.
    assert!(msg.contains("unknown type"), "must say WHY it is not a fact:\n{msg}");
}
