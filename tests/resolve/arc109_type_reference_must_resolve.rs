//! Arc 109 (DESIGN-STONE-a-type-reference-must-resolve) — rows 1-5 of the acceptance table.
//!
//! Every fixture here names an unresolvable type in a DECLARED position (D2-A: params,
//! returns, fields, variant payloads, alias RHS, surface methods). Before this stone, none of
//! them raised an error unless a CALLER happened to exercise the phantom — and even then the
//! diagnostic blamed the caller (`TypeMismatch`/`ReturnTypeMismatch`), not the declaration.
//!
//! Run: `cargo test --release --test resolve -- arc109`
//!
//! ## Why these assert CONTAINS, not `assert_edn_matches_file!`'s exact match
//!
//! `probe_arc279_format.rs` (the shape this brief pointed at) uses an exact EDN golden because
//! its fixture's error is the ONLY finding in the program. That assumption does not hold here:
//! `startup_from_file` always loads the FULL stdlib (`build_env`'s `stdlib_forms()`), and this
//! sweep found a PRE-EXISTING, out-of-boundary defect there — `wat/spawn.wat`'s `Locus` surface
//! synthesizes two op aliases (`spawn-runner/Request`, `spawn-runner/Response`, `launch/Request`,
//! `launch/Response`) via `src/types.rs`'s `AliasDef { type_params: surf.type_params.clone(), ..
//! }`, which copies the SURFACE's own (empty) type params instead of the METHOD's OWN
//! (`spawn-runner<D,I,O,W>` / `launch<S,R,St,Sh,Lu>`) — so `D`/`I`/`O`/`W`/`S`/`R`/`Sh`/`Lu`
//! appear free in those aliases' bodies with no binder. Every `startup_from_file` call in this
//! process therefore ALSO reports those 8 (pre-existing, unrelated) findings alongside whatever
//! this fixture itself contributes. `src/types.rs` is out of this stone's boundary (`src/
//! resolve/`, `src/freeze.rs`'s precedence, `symbol_table.rs`'s iterator, tests — not `types.rs`)
//! and STOP-3 explicitly forbids "fixing a stdlib type name to make the pass green" — so these
//! tests assert MEMBERSHIP (mirroring the sanctioned `assert_check_error_present!` pattern for
//! exactly this shape: a non-deterministically-ordered SET of findings, of which only one is
//! this fixture's own) rather than exact-match a golden that would be permanently polluted by
//! an unrelated defect until someone fixes `src/types.rs` outside this stone.

use wat::freeze::startup_from_file;
use wat::resolve::ReferenceKind;
use wat::{ResolveError, StartupError};

/// Assert `path` fires this program's phantom-type diagnostic — `check_program` names 0 for it —
/// through the resolver as an `UnresolvedReference{ kind: Type }`. This also proves the STOP-2
/// precedence fix: had `check_program`'s `TypeMismatch`/`ReturnTypeMismatch` won instead, the
/// error variant here would be `StartupError::Check`, not `StartupError::Resolve` — the `other`
/// arm below fires loudly rather than silently passing a weaker check.
fn assert_names_phantom_type(rel_path: &str, path: &str) {
    let err = match startup_from_file(rel_path) {
        Ok(_) => panic!("{rel_path}: startup should fail — it declares an unresolvable type"),
        Err(e) => e,
    };
    match err {
        StartupError::Resolve(ResolveError::UnresolvedReferences(refs)) => {
            assert!(
                refs.iter().any(|r| r.path == path && r.kind == ReferenceKind::Type),
                "{rel_path}: expected an UnresolvedReference{{path: {path:?}, kind: Type}} \
                 among: {refs:?}"
            );
        }
        other => panic!(
            "{rel_path}: expected #wat.resolve/UnresolvedReferences naming {path:?} (a \
             declaration defect), not a check-layer error blaming a caller/body; got {other:?}"
        ),
    }
}

/// Row 1★ — a phantom in an UNCALLED declaration. Nothing evaluates `:user::NoSuchType` unless
/// the declaration sweep does; there is no caller to trip a symptom.
#[test]
fn row1_phantom_in_uncalled_declaration_is_rejected() {
    assert_names_phantom_type(
        "tests/resolve/arc109_type_reference_must_resolve_row1_uncalled.wat",
        ":user::NoSuchType",
    );
}

/// Row 2★★ — THE STONE. A phantom WITH a caller. Before this fix, row 1 alone could pass while
/// this case kept its old `TypeMismatch` blaming parameter #1 (STOP-2). This is the row that
/// actually proves the precedence change landed.
#[test]
fn row2_phantom_with_a_caller_names_the_type_not_the_caller() {
    assert_names_phantom_type(
        "tests/resolve/arc109_type_reference_must_resolve_row2_called.wat",
        ":user::NoSuchType",
    );
}

/// Row 3 — a phantom in a RETURN slot.
#[test]
fn row3_phantom_in_return_slot_is_rejected() {
    assert_names_phantom_type(
        "tests/resolve/arc109_type_reference_must_resolve_row3_return.wat",
        ":user::NoSuchType",
    );
}

/// Row 4 — a phantom as a PARAMETRIC form's HEAD, with a legitimate arg. Catches a walk that
/// checks only `Parametric.args` and never `Parametric.head` itself.
#[test]
fn row4_phantom_parametric_head_is_rejected() {
    assert_names_phantom_type(
        "tests/resolve/arc109_type_reference_must_resolve_row4_parametric.wat",
        ":wat::cache::NoSuchType",
    );
}

/// Row 5 — a phantom in a `defrecord` FIELD. `register_types_impl` consumes the `defrecord`
/// form entirely at freeze step 5 — proof this must be a registry sweep, not a form walk.
#[test]
fn row5_phantom_in_record_field_is_rejected() {
    assert_names_phantom_type(
        "tests/resolve/arc109_type_reference_must_resolve_row5_field.wat",
        ":user::NoSuchType",
    );
}
