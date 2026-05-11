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
//! | Primitive    | `wat_arc144_hardcoded_primitives.rs::lookup_define_length_renders_primitive_sentinel` + `wat_arc143_lookup.rs::lookup_define_substrate_primitive_returns_some` + `wat_arc144_lookup_form.rs::signature_of_substrate_primitive_*` | Smoke (regression-guard) |
//! | SpecialForm  | `wat_arc144_special_forms.rs` (9 tests, full trio with sentinel + slot verification) | Smoke (regression-guard) |
//! | Type         | `wat_arc144_lookup_form.rs` (3 tests, full trio)                       | Smoke (regression-guard) |
//! | Dispatch     | `wat_arc146_dispatch_mechanism.rs` (synthetic `:test::describe`)        | Real-builtin: `:wat::core::length` |
//!
//! Plus a length canary regression test on a HashMap (brief explicitly
//! requests this shape — complementary to the Vector variant pinned in
//! `wat_arc143_define_alias.rs::define_alias_length_to_user_size_*`).

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

fn with_nil_main(src: &str) -> String {
    format!(
        "{}\n(:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)",
        src
    )
}

fn run_bool(src: &str) -> bool {
    let src = with_nil_main(src);
    let world = startup_from_source(
        &src,
        Some(concat!(file!(), ":", line!())),
        Arc::new(InMemoryLoader::new()),
    )
    .expect("startup");
    let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
    let env = Environment::new();
    match eval_in_frozen(&ast, &world, &env).expect("compute") {
        Value::bool(b) => b,
        other => panic!("expected bool; got {:?}", other),
    }
}

fn run_string(src: &str) -> String {
    let src = with_nil_main(src);
    let world = startup_from_source(
        &src,
        Some(concat!(file!(), ":", line!())),
        Arc::new(InMemoryLoader::new()),
    )
    .expect("startup");
    let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
    let env = Environment::new();
    match eval_in_frozen(&ast, &world, &env).expect("compute") {
        Value::String(s) => s.as_str().to_owned(),
        other => panic!("expected String; got {:?}", other),
    }
}

fn run_i64(src: &str) -> i64 {
    let src = with_nil_main(src);
    let world = startup_from_source(
        &src,
        Some(concat!(file!(), ":", line!())),
        Arc::new(InMemoryLoader::new()),
    )
    .expect("startup");
    let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
    let env = Environment::new();
    match eval_in_frozen(&ast, &world, &env).expect("compute") {
        Value::i64(n) => n,
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Kind 1: UserFunction — full trio + head verification ──────────────────
//
// Existing `wat_arc144_lookup_form.rs::lookup_define_user_function_still_returns_some_post_refactor`
// only checks Some/None; this test additionally verifies the rendered AST
// carries the `:wat::core::define` head keyword (the load-bearing claim
// for "uniform" reflection: the head keyword distinguishes the kind).

#[test]
fn user_function_lookup_define_emits_define_head() {
    let src = r##"
        (:wat::core::define
          (:user::greet (n :wat::core::String) -> :wat::core::String)
          n)

        (:wat::core::define (:user::compute -> :wat::core::String)
          (:wat::core::let
            [def-opt
              (:wat::runtime::lookup-define :user::greet)
             rendered
              (:wat::edn::write def-opt)]
            rendered))
    "##;
    let line = run_string(src);
    assert!(
        line.contains(":wat::core::define"),
        "expected ':wat::core::define' head in user-function lookup-define AST, got: {}",
        line
    );
    assert!(
        line.contains("user::greet"),
        "expected user-function name in rendered AST, got: {}",
        line
    );
}

#[test]
fn user_function_signature_and_body_return_some() {
    // Reflection trio for UserFunction: signature-of returns Some,
    // body-of returns Some (functions have wat bodies — distinct from
    // Type/SpecialForm/Dispatch which return :None for body-of).
    let src = r##"
        (:wat::core::define
          (:user::add (x :wat::core::i64) (y :wat::core::i64) -> :wat::core::i64)
          (:wat::core::+ x y))

        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::core::let
            [sig-opt
              (:wat::runtime::signature-of :user::add)
             body-opt
              (:wat::runtime::body-of :user::add)]
            (:wat::core::match sig-opt
              -> :wat::core::bool
              ((:wat::core::Some _)
                (:wat::core::match body-opt
                  -> :wat::core::bool
                  ((:wat::core::Some _) true)
                  (:wat::core::None    false)))
              (:wat::core::None false))))
    "##;
    assert!(run_bool(src), "signature-of and body-of :user::add should both return Some");
}

// ─── Kind 2: Macro — smoke (full coverage at wat_arc144_lookup_form.rs) ────

#[test]
fn macro_lookup_define_smoke() {
    // REGRESSION GUARD only — exhaustive coverage at
    // `wat_arc144_lookup_form.rs::lookup_define_macro_returns_some_and_emits_defmacro_head`
    // (full trio incl. body template + signature-of). This thin assert
    // pins the cross-test invariant: lookup-define on a registered macro
    // returns Some.
    let src = r##"
        (:wat::core::defmacro (:my::id (x :AST) -> :AST) `~x)

        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::core::match
            (:wat::runtime::lookup-define :my::id)
            -> :wat::core::bool
            ((:wat::core::Some _) true)
            (:wat::core::None    false)))
    "##;
    assert!(run_bool(src), "lookup-define :my::id should return Some");
}

// ─── Kind 3: Primitive — smoke (full coverage at slices 1+3) ───────────────

#[test]
fn primitive_lookup_define_and_signature_smoke() {
    // REGRESSION GUARD only — exhaustive coverage at
    // `wat_arc144_hardcoded_primitives.rs::lookup_define_length_renders_primitive_sentinel`
    // (head verification on Vector/length) and
    // `wat_arc144_lookup_form.rs::signature_of_substrate_primitive_*`
    // (signature-of on foldl). This pins the slice 4 framing: a
    // TypeScheme primitive answers BOTH lookup-define + signature-of.
    let src = r##"
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::core::let
            [def-opt
              (:wat::runtime::lookup-define :wat::core::foldl)
             sig-opt
              (:wat::runtime::signature-of :wat::core::foldl)]
            (:wat::core::match def-opt
              -> :wat::core::bool
              ((:wat::core::Some _)
                (:wat::core::match sig-opt
                  -> :wat::core::bool
                  ((:wat::core::Some _) true)
                  (:wat::core::None    false)))
              (:wat::core::None false))))
    "##;
    assert!(run_bool(src), "lookup-define and signature-of :wat::core::foldl should both return Some");
}

// ─── Kind 4: SpecialForm — smoke (full coverage at slice 2) ────────────────

#[test]
fn special_form_lookup_define_smoke() {
    // REGRESSION GUARD only — exhaustive coverage at
    // `wat_arc144_special_forms.rs` (9 tests with sentinel head +
    // per-form slot verification). This pins :wat::core::if as the
    // representative special form and asserts the slice-1 sentinel
    // marker is preserved in the rendered AST.
    let src = r##"
        (:wat::core::define (:user::compute -> :wat::core::String)
          (:wat::core::let
            [def-opt
              (:wat::runtime::lookup-define :wat::core::if)
             rendered
              (:wat::edn::write def-opt)]
            rendered))
    "##;
    let line = run_string(src);
    assert!(
        line.contains(":wat::core::__internal/special-form"),
        "expected special-form sentinel head, got: {}",
        line
    );
    assert!(
        line.contains(":wat::core::if"),
        "expected ':wat::core::if' name in rendered AST, got: {}",
        line
    );
}

// ─── Kind 5: Type — smoke (full coverage at wat_arc144_lookup_form.rs) ─────

#[test]
fn type_lookup_define_smoke() {
    // REGRESSION GUARD only — exhaustive coverage at
    // `wat_arc144_lookup_form.rs::lookup_define_struct_returns_some_and_emits_struct_head`
    // (full trio with head + body-of returns :None). This pins the
    // cross-test invariant on a different struct shape.
    let src = r##"
        (:wat::core::struct :my::Pair
          (a :wat::core::i64)
          (b :wat::core::i64))

        (:wat::core::define (:user::compute -> :wat::core::String)
          (:wat::core::let
            [def-opt
              (:wat::runtime::lookup-define :my::Pair)
             rendered
              (:wat::edn::write def-opt)]
            rendered))
    "##;
    let line = run_string(src);
    assert!(
        line.contains(":wat::core::struct"),
        "expected ':wat::core::struct' head in struct lookup-define AST, got: {}",
        line
    );
    assert!(
        line.contains("my::Pair"),
        "expected struct name in rendered AST, got: {}",
        line
    );
}

// ─── Kind 6: Dispatch — real-builtin coverage on `:wat::core::length` ──────
//
// `wat_arc146_dispatch_mechanism.rs` covers Dispatch on the SYNTHETIC
// `:test::describe` (i64/f64 arms). Slice 4's contribution: pin the
// reflection trio on the REAL `:wat::core::length` migrated builtin
// (arc 146 slice 2). The Dispatch arms include `Vector<T>` and
// `HashMap<K,V>` — see `wat/core.wat:12-14`.

#[test]
fn dispatch_length_lookup_define_emits_define_dispatch_head() {
    let src = r##"
        (:wat::core::define (:user::compute -> :wat::core::String)
          (:wat::core::let
            [def-opt
              (:wat::runtime::lookup-define :wat::core::length)
             rendered
              (:wat::edn::write def-opt)]
            rendered))
    "##;
    let line = run_string(src);
    assert!(
        line.contains("define-dispatch"),
        "expected 'define-dispatch' head in :wat::core::length lookup-define, got: {}",
        line
    );
    assert!(
        line.contains(":wat::core::length"),
        "expected ':wat::core::length' name in rendered AST, got: {}",
        line
    );
    // The arms list is the load-bearing evidence for Dispatch reflection:
    // a Vector arm and a HashMap arm (per wat/core.wat:13-14). Verify
    // both arm targets are present in the rendered emission.
    assert!(
        line.contains("Vector/length"),
        "expected Vector arm target in dispatch arms, got: {}",
        line
    );
    assert!(
        line.contains("HashMap/length"),
        "expected HashMap arm target in dispatch arms, got: {}",
        line
    );
}

#[test]
fn dispatch_length_signature_and_body_shape() {
    // signature-of returns Some (the dispatch declaration form);
    // body-of returns :None (dispatchs have no wat-side body — the arms
    // table IS the contract; per arc 146 slice 1 BRIEF).
    let src = r##"
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::core::let
            [sig-opt
              (:wat::runtime::signature-of :wat::core::length)
             body-opt
              (:wat::runtime::body-of :wat::core::length)]
            (:wat::core::match sig-opt
              -> :wat::core::bool
              ((:wat::core::Some _)
                (:wat::core::match body-opt
                  -> :wat::core::bool
                  ((:wat::core::Some _) false)
                  (:wat::core::None    true)))
              (:wat::core::None false))))
    "##;
    assert!(run_bool(src), "signature-of should return Some and body-of should return None for dispatch");
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
    let src = r##"
        (:wat::runtime::define-alias :user::size :wat::core::length)

        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:user::size
            (:wat::core::HashMap :(wat::core::String,wat::core::i64)
              "a" 1 "b" 2 "c" 3)))
    "##;
    let n = run_i64(src);
    assert_eq!(
        n, 3,
        "expected alias of length to return 3 for HashMap of 3 entries, got: {}",
        n
    );
}
