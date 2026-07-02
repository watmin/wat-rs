//! Integration coverage for arc 144 slice 4 — UNIFORM REFLECTION
//! verification across all 6 `Binding` variants.
//!
//! The substrate's uniform-reflection foundation is now structurally
//! complete after arc 144 slices 1-3 + arc 146 + arc 148:
//!   - slice 1: Binding enum (5 variants) + lookup_form (4 walked, 1 stub)
//!   - slice 2: SpecialForm registry populated (5th variant live)
//!   - slice 3: TypeScheme inscribed for hardcoded primitives
//!   - arc 146: Dispatch entity (6th variant) + length canary turned GREEN
//!   - arc 148: polymorphic-handler anti-pattern retired for arith/compare
//!
//! Slice 4 is PURE VERIFICATION — no substrate edits. It pins the
//! end-to-end claim: `(:wat::runtime::lookup-define <name>)` returns
//! Some across every Binding variant, and the rendered AST carries the
//! kind-distinguishing head keyword.
//!
//! ─── Coverage rollup vs existing tests ─────────────────────────────────────
//!
//! Where existing tests already cover a kind exhaustively, this file
//! REFERENCES the existing test in a comment + ships a thin smoke
//! regression-guard so a regression in this slice's framing surfaces
//! here too. Where there's a real gap (UserFunction head verification;
//! Dispatch on the real `:wat::core::length` migrated builtin; the
//! HashMap-shape length canary), this file ships the new test.
//!
//! | Kind         | Existing exhaustive coverage              | Slice 4 ships             |
//! |--------------|-------------------------------------------|---------------------------|
//! | UserFunction | `wat_arc144_lookup_form.rs::lookup_define_user_function_*` (Some-only) | Full trio + head verify  |
//! | Macro        | `wat_arc144_lookup_form.rs` (3 tests, full trio)                       | Smoke (regression-guard) |
//! | Primitive    | `wat_arc144_hardcoded_primitives.rs::lookup_define_length_renders_primitive_sentinel` + `wat_arc143_lookup.rs::lookup_define_substrate_primitive_returns_some` + `wat_arc144_lookup_form.rs::signature_of_defn_substrate_primitive_*` | Smoke (regression-guard) |
//! | SpecialForm  | `wat_arc144_special_forms.rs` (9 tests, full trio with sentinel + slot verification) | Smoke (regression-guard) |
//! | Type         | `wat_arc144_lookup_form.rs` (3 tests, full trio)                       | Smoke (regression-guard) |
//! | Dispatch     | `wat_arc146_dispatch_mechanism.rs` (synthetic `:test::describe`)        | Real-builtin: `:wat::core::length` |
//!
//! Plus a length canary regression test on a HashMap (brief explicitly
//! requests this shape — complementary to the Vector variant pinned in
//! `wat_arc143_define_alias.rs::define_alias_length_to_user_size_*`).
//!
//! Fixtures co-located beside each test name — slurped via startup_from_file.

use wat::freeze::{eval_in_frozen, startup_from_file};
use wat::runtime::{Environment, Value};

fn run_bool(fixture_path: &str) -> bool {
    let world = startup_from_file(fixture_path).expect("startup");
    let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
    let env = Environment::new();
    match eval_in_frozen(&ast, &world, &env).expect("compute").value_owned() {
        Value::bool(b) => b,
        other => panic!("expected bool; got {:?}", other),
    }
}

fn run_string(fixture_path: &str) -> String {
    let world = startup_from_file(fixture_path).expect("startup");
    let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
    let env = Environment::new();
    match eval_in_frozen(&ast, &world, &env).expect("compute").value_owned() {
        Value::String(s) => s.as_str().to_owned(),
        other => panic!("expected String; got {:?}", other),
    }
}

fn run_i64(fixture_path: &str) -> i64 {
    let world = startup_from_file(fixture_path).expect("startup");
    let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
    let env = Environment::new();
    match eval_in_frozen(&ast, &world, &env).expect("compute").value_owned() {
        Value::i64(n) => n,
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Kind 1: UserFunction — full trio + head verification ──────────────────
//
// Stone 241.16 — reflection now emits `:wat::core::defn` (not `:wat::core::define`).
// `function_to_define_ast` updated to use `:wat::core::defn` head keyword.
// The "uniform" reflection claim preserves: the head keyword still distinguishes kind.

#[test]
fn user_function_lookup_define_emits_defn_head() {
    let line = run_string("tests/reflection/wat_arc144_uniform_reflection_defn_head.wat");
    // Stone 241.16 — reflection emits :wat::core::defn (not :wat::core::define)
    assert_eq!(
        line,
        "#wat.core.Option/Some #wat-edn.holon/Bundle [#wat-edn.holon/Keyword :wat::core::defn #wat-edn.holon/Bundle [#wat-edn.holon/Keyword :user::greet #wat-edn.holon/Bundle [#wat-edn.holon/Symbol \"n\" #wat-edn.holon/Keyword :wat::core::String] #wat-edn.holon/Symbol \"->\" #wat-edn.holon/Keyword :wat::core::String] #wat-edn.holon/Symbol \"n\"]",
        "user-function lookup-define must emit :wat::core::defn head with greet body"
    );
}

#[test]
fn user_function_signature_and_body_return_some() {
    // Reflection trio for UserFunction: signature-of-defn returns Some,
    // body-of returns Some (functions have wat bodies — distinct from
    // Type/SpecialForm/Dispatch which return :None for body-of).
    assert!(
        run_bool("tests/reflection/wat_arc144_uniform_reflection_sig_body.wat"),
        "signature-of-defn and body-of :user::add should both return Some"
    );
}

// ─── Kind 2: Macro — smoke (full coverage at wat_arc144_lookup_form.rs) ────

#[test]
fn macro_lookup_define_smoke() {
    // REGRESSION GUARD only — exhaustive coverage at
    // `wat_arc144_lookup_form.rs::lookup_define_macro_returns_some_and_emits_defmacro_head`
    // (full trio incl. body template + signature-of-defn). This thin assert
    // pins the cross-test invariant: lookup-define on a registered macro
    // returns Some.
    assert!(
        run_bool("tests/reflection/wat_arc144_uniform_reflection_macro.wat"),
        "lookup-define :my::id should return Some"
    );
}

// ─── Kind 3: Primitive — smoke (full coverage at slices 1+3) ───────────────

#[test]
fn primitive_lookup_define_and_signature_smoke() {
    // REGRESSION GUARD only — exhaustive coverage at
    // `wat_arc144_hardcoded_primitives.rs::lookup_define_length_renders_primitive_sentinel`
    // (head verification on Vector/length) and
    // `wat_arc144_lookup_form.rs::signature_of_defn_substrate_primitive_*`
    // (signature-of-defn on foldl). This pins the slice 4 framing: a
    // TypeScheme primitive answers BOTH lookup-define + signature-of-defn.
    assert!(
        run_bool("tests/reflection/wat_arc144_uniform_reflection_primitive.wat"),
        "lookup-define and signature-of-defn :wat::core::foldl should both return Some"
    );
}

// ─── Kind 4: SpecialForm — smoke (full coverage at slice 2) ────────────────

#[test]
fn special_form_lookup_define_smoke() {
    // REGRESSION GUARD only — exhaustive coverage at
    // `wat_arc144_special_forms.rs` (9 tests with sentinel head +
    // per-form slot verification). This pins :wat::core::if as the
    // representative special form and asserts the slice-1 sentinel
    // marker is preserved in the rendered AST.
    let line = run_string("tests/reflection/wat_arc144_uniform_reflection_special_form.wat");
    assert_eq!(
        line,
        "#wat.core.Option/Some #wat-edn.holon/Bundle [#wat-edn.holon/Keyword :wat::core::__internal/special-form #wat-edn.holon/Keyword :wat::core::if]",
        "special-form lookup-define must emit sentinel head and :if name"
    );
}

// ─── Kind 5: Type — smoke (full coverage at wat_arc144_lookup_form.rs) ─────

#[test]
fn type_lookup_define_smoke() {
    // REGRESSION GUARD only — exhaustive coverage at
    // `wat_arc144_lookup_form.rs::lookup_define_struct_returns_some_and_emits_struct_head`
    // (full trio with head + body-of returns :None). This pins the
    // cross-test invariant on a different struct shape.
    let line = run_string("tests/reflection/wat_arc144_uniform_reflection_type.wat");
    assert_eq!(
        line,
        "#wat.core.Option/Some #wat-edn.holon/Bundle [#wat-edn.holon/Keyword :wat::core::defstruct #wat-edn.holon/Keyword :my::Pair #wat-edn.holon/Bundle [#wat-edn.holon/Keyword :wat::core::__internal/type-decl #wat-edn.holon/Keyword :my::Pair]]",
        "type lookup-define must emit :defstruct head with my::Pair name"
    );
}

// ─── Kind 6: Primitive ∀T intrinsic — real-builtin coverage on `:wat::core::empty?` ──────
//
// Stone 241.13 — define-dispatch is retired. All former dispatch entities
// (length, empty?, contains?, get, conj, assoc) are ∀T intrinsic Primitives.
// `:wat::core::empty?` is the canonical exemplar: lookup-define returns a
// synthetic `:wat::core::define` form (Primitive reflection), not `:wat::core::define-dispatch`.

#[test]
fn primitive_empty_lookup_define_emits_define_head() {
    let line = run_string("tests/reflection/wat_arc144_uniform_reflection_empty.wat");
    // Stone 241.16 — primitive reflection emits :wat::core::defn (not :wat::core::define),
    // and Stone 241.13 — no define-dispatch; empty? is a ∀T intrinsic Primitive.
    assert_eq!(
        line,
        "#wat.core.Option/Some #wat-edn.holon/Bundle [#wat-edn.holon/Keyword :wat::core::defn #wat-edn.holon/Bundle [#wat-edn.holon/Keyword :wat::core::empty?<T> #wat-edn.holon/Bundle [#wat-edn.holon/Symbol \"_a0\" #wat-edn.holon/Keyword :T] #wat-edn.holon/Symbol \"->\" #wat-edn.holon/Keyword :wat::core::bool] #wat-edn.holon/Bundle [#wat-edn.holon/Keyword :wat::core::__internal/primitive #wat-edn.holon/Keyword :wat::core::empty?]]",
        "empty? lookup-define must emit :defn head with internal-primitive sentinel body"
    );
}

#[test]
fn dispatch_length_signature_and_body_shape() {
    // signature-of-defn returns Some (the ∀T intrinsic Primitive scheme);
    // body-of returns :None (Primitives have no wat-side body — the Rust
    // impl IS the contract; per arc 237.7a Stone + Stone 241.13 doctrine).
    assert!(
        run_bool("tests/reflection/wat_arc144_uniform_reflection_length_shape.wat"),
        "signature-of-defn should return Some and body-of should return None for dispatch"
    );
}

// ─── Length canary regression — HashMap shape (brief request) ──────────────
//
// `wat_arc143_define_alias.rs::define_alias_length_to_user_size_delegates_correctly`
// pins the Vector shape (3-element vec → 3). Slice 4 brief explicitly
// requests the HashMap shape — the Dispatch's HashMap<K,V> arm routing
// through `define-alias` end-to-end. RED here would mean either:
//   - arc 146 slice 2 dispatch-of-length regressed for HashMap, OR
//   - arc 143 slice 6 define-alias regressed for dispatch entities.
// Either is a substrate-foundation regression worth STOP-signalling.

#[test]
fn length_canary_hashmap_via_define_alias() {
    let n = run_i64("tests/reflection/wat_arc144_uniform_reflection_canary.wat");
    assert_eq!(
        n, 3,
        "expected alias of length to return 3 for HashMap of 3 entries, got: {}",
        n
    );
}
