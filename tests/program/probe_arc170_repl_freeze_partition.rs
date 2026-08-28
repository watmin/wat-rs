//! Arc 170 — the REPL's classifier: does the FREEZE partition a session's forms into
//! "declarations" (registered) and "expressions" (residue)?
//!
//! # Why this probe exists
//!
//! A REPL's `E` phase must answer one question per line: *is this a DECLARATION that joins
//! the session's definition set, or an EXPRESSION to evaluate?* The tempting answer is to
//! classify by the error `eval-ast!` returns. `wat-scripts/scratch-pad/probe-repl-declaration-refusal.wat`
//! MEASURED that and it does not work — the substrate refuses declarations through THREE
//! different mechanisms, and two of them are byte-identical to a typo:
//!
//! | form        | eval-ast! error kind                                   |
//! |-------------|--------------------------------------------------------|
//! | `def`       | `runtime-error` / `DeclarationInExpressionPosition`     |
//! | `defenum`   | `mutation-form-refused`                                 |
//! | `defn`      | `unknown-function: :wat::core::defn`                    |
//! | `defrecord` | `unknown-function: :wat::core::defrecord`               |
//! | *(a typo)*  | `unknown-function: :wat::core::deffn`                   |
//!
//! `defn` and `defrecord` are indistinguishable from a misspelling, so an error-classifying
//! REPL would answer *"unknown function :wat::core::defn"* to the single most common thing a
//! user types — and silently discard the definition. That is a hidden failure in the arc
//! whose law is that nothing hides one.
//!
//! The next tempting answer was the freeze: `FrozenWorld.program` is documented as *"the
//! residue of forms left after all definitions were registered"*, so surely a declaration
//! vanishes and an expression survives. **This probe measured that and it is ALSO wrong** —
//! and the shape of the wrongness is the useful part, so it is pinned here.
//!
//! The freeze partitions on TYPE-ness, not declaration-ness:
//!
//! | form                            | fate                                          |
//! |---------------------------------|-----------------------------------------------|
//! | `defrecord` / `defenum`         | CONSUMED by the freeze → `TypeEnv`            |
//! | `defn`                          | macro-EXPANDS to `def` (`wat/core.wat:1175`)  |
//! | `def` (incl. the expanded one)  | SURVIVES as residue                           |
//! | a bare expression               | SURVIVES as residue                           |
//!
//! So residue is not "the expressions" — it is "value-declarations AND expressions", and
//! it is exactly what [`wat::runtime::register_runtime_defs`] walks at runtime to register
//! `def`/`defclause`/`extend-type`. Residue-vs-registered is therefore NOT a REPL classifier
//! either.
//!
//! What this leaves for the REPL is a UNION of two facts, both read off the substrate rather
//! than mirrored from it: a form that VANISHED was a type declaration, and a form that
//! REMAINS is a value declaration exactly when `register_runtime_defs_form`'s own match
//! consumes its head. One authority per half, neither of them a hand-kept list.

use std::sync::Arc;
use wat::freeze::StartupError;

/// Freeze one session's forms and return the residue the freeze left behind,
/// rendered head-first so a failure message names the actual forms.
fn residue_of(src: &str, file: &str) -> Result<Vec<String>, StartupError> {
    let forms = wat::parse_all_with_file(src, file).map_err(StartupError::Parse)?;
    let loader: Arc<dyn wat::load::loader::SourceLoader> = Arc::new(wat::load::loader::InMemoryLoader::new());
    let world = wat::freeze::startup_from_forms(forms, None, loader)?;
    Ok(world.program.iter().map(|f| format!("{f:?}")).collect())
}

/// The freeze partitions on TYPE-ness, not declaration-ness — so a REPL cannot read
/// "is this a declaration?" off residue membership.
#[test]
fn the_freeze_partitions_on_typeness_not_declarationness() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/program/probe_arc170_repl_freeze_partition__session.wat"
    );
    let src = std::fs::read_to_string(path).expect("the session fixture must be readable");
    let residue = residue_of(&src, path).expect("the session must freeze");

    // TYPE declarations are consumed into the TypeEnv and leave nothing behind.
    for head in [":wat::core::defrecord", ":wat::core::defenum"] {
        assert!(
            !residue.iter().any(|f| f.contains(head)),
            "`{head}` survived into FrozenWorld.program — a type declaration is expected to be \
             CONSUMED by the freeze. If this fires, the freeze's partition changed and the REPL \
             classifier that reads it must be re-derived. Residue was:\n  {}",
            residue.join("\n  ")
        );
    }

    // VALUE declarations survive as residue — this is what register_runtime_defs walks.
    // `defn` is not a primitive: it macro-expands to `def` + `fn`, so it appears HERE, as a
    // `def`, and never as a `defn`. That expansion is why an eval-time classifier sees
    // `unknown function :wat::core::defn` — there is no such runtime verb to find.
    let defs_in_residue = residue
        .iter()
        .filter(|f| f.contains(":wat::core::def\""))
        .count();
    assert_eq!(
        defs_in_residue, 2,
        "expected TWO `def` forms in residue — the literal `(def :usr::x 1)` and the one \
         `(defn :usr::f …)` expands into. Residue was:\n  {}",
        residue.join("\n  ")
    );

    // And the expression survives alongside them: residue mixes both kinds, which is exactly
    // why residue-membership alone cannot answer "was this line a declaration?".
    assert_eq!(
        residue.len(),
        3,
        "expected THREE residue forms (two `def`s + the `(+ 1 2)` expression) — residue holds \
         value-declarations AND expressions together. Residue was:\n  {}",
        residue.join("\n  ")
    );
}
