//! Arc 278 — WHICH USER TYPES CARRY A RETAINED SOURCE FORM, AND WHICH DO NOT.
//!
//! THE OWED GROUNDING. The arc's live defect is one root cause wearing three faces:
//! `type_def_to_ast` RECONSTRUCTS a type where a retained form should be SHIPPED, and
//! every reconstruction drops whatever its description does not model (the constructor,
//! then the accessors). The prescribed fix is the mirror of what `MacroDef` just got —
//! *synthesized types retain a source form* — and the seam attaches one condition to it:
//!
//!   > does the expansion that generates `recordtype` still hold its form at
//!   > registration, the way `parse_defmacro_form` did? It came back YES for macros.
//!   > **Do not assume it transfers.**
//!
//! `TypeEnv`'s own doc comment ANSWERS this in prose — "synthesized `derived` defs
//! (backing records / `::Op` / `::Reply`) have no user form and fall back to
//! reconstruction" (`src/types.rs:449`). A doc comment is a claim about the code, not a
//! measurement of this program. `closure_extract.rs:398` branches on exactly this
//! predicate, so the branch each type takes decides whether the child receives the
//! declaration verbatim or a rebuild — and that is a FACT ABOUT A WORLD, obtainable only
//! by freezing one and asking.
//!
//! So this probe asks, per user type in the live gate's world: `source_form(name)` —
//! Some (shipped verbatim) or None (reconstructed)?
//!
//! ★ WHAT IT ANSWERED (2026-08-11), and it REFUTED the act it was written to ground.
//! `:probe::ffx::Record` and `:probe::ffx::State` — the two types whose ACCESSORS the live
//! gate reported unresolved — are in the RETAINED column. Their declarations were already
//! shipping verbatim; `type_def_to_ast` never fired for them, so no amount of teaching
//! synthesized types to retain a form could have touched that failure. The seam's stated
//! root cause did not transfer, exactly as the seam itself warned it might not.
//!
//! The RECONSTRUCTED set is six names, and every one is surface-DERIVED
//! (`$core-record`, `$holon-record`, `::Op`, `::Reply`, and the two op aliases) — defs the
//! parent synthesizes from a `defsurface`, which have no user form by construction. Whether
//! THOSE should retain one is a separate, still-unproven question; nothing here shows a
//! consumer harmed by their reconstruction.
//!
//! The accessor failure's real mechanism was found downstream of this measurement and was
//! not in `src/` at all — see the gate's own `decl-key` comment
//! (`wat-scripts/scratch-pad/probe-arc278-union-closure-boots-a-process-child.wat`): a
//! name-keyed dedup in the INSTRUMENT collapsed a `recordtype` and its same-named kwargs
//! `defmacro` — two facets of one concept, exactly the `[Macro, Type]` pairing this arc's
//! own registry census had already counted at 182 names — and kept only the macro.
//!
//! ⚠ NON-VACUITY. Two ways this instrument could measure nothing, both asserted below:
//! if the world holds no user types at all, the partition is empty and says nothing; and
//! if EVERY user type retains a form, the reconstruction path is unreachable in this
//! world and the gate's accessor failure cannot be explained by it — that is a real
//! result, but it must be read as a refutation, not as a pass.

use wat::types::TypeDef;

/// The live gate — the same world `probe_arc278_registry_census` measures, and the
/// same one `probe-arc278-union-closure-boots-a-process-child.wat` runs. Sharing the
/// subject is deliberate: the census counted registry FACETS per name, this counts
/// RETENTION per name, and a finding in either has to survive the other.
const SUBJECT: &str = "wat-scripts/scratch-pad/probe-arc278-union-closure-boots-a-process-child.wat";

/// A name the child must be able to receive. `:wat::*` is re-registered in the child by
/// `with_builtins` and is never shipped, so it is out of the question's scope entirely.
fn is_user_name(name: &str) -> bool {
    !wat::resolve::is_reserved_prefix(name)
}

/// Which decl head a `TypeDef` presents as — enough to see whether the RETAINED and
/// RECONSTRUCTED sets split along a kind boundary or cut across one. Exhaustive by law;
/// a new `TypeDef` variant turns this red rather than being silently absorbed.
fn kind_of(def: &TypeDef) -> String {
    match def {
        TypeDef::Aggregate(a) => format!("aggregate/{:?}", a.nature),
        TypeDef::Enum(_) => "enum".to_string(),
        TypeDef::Newtype(_) => "newtype".to_string(),
        TypeDef::Alias(_) => "alias".to_string(),
        TypeDef::Union(_) => "union".to_string(),
        TypeDef::Surface(_) => "surface".to_string(),
    }
}

#[test]
fn retained_source_forms_partition_the_user_types() {
    let world = wat::freeze::startup_from_file(SUBJECT).expect("freeze should succeed");
    let sym = world.symbols();
    let types = sym.types_deref().expect("the world registers types");

    let mut retained: Vec<(String, String)> = Vec::new();
    let mut reconstructed: Vec<(String, String)> = Vec::new();

    for (name, def) in types.iter() {
        if !is_user_name(name) {
            continue;
        }
        let row = (name.clone(), kind_of(def));
        if types.source_form(name).is_some() {
            retained.push(row);
        } else {
            reconstructed.push(row);
        }
    }
    retained.sort();
    reconstructed.sort();

    println!("\n=== RETAINED SOURCE FORMS (user types only) ===");
    println!("subject                : {SUBJECT}");
    println!("retained (shipped)     : {}", retained.len());
    for (n, k) in &retained {
        println!("      {n}   [{k}]");
    }
    println!("\nRECONSTRUCTED (rebuilt): {}", reconstructed.len());
    for (n, k) in &reconstructed {
        println!("      {n}   [{k}]");
    }
    println!();

    // NON-VACUITY 1 — a world with no user types measures nothing.
    assert!(
        !retained.is_empty() || !reconstructed.is_empty(),
        "no user types in the subject world — the instrument is vacuous, not the answer"
    );

    // NON-VACUITY 2 — if nothing falls to reconstruction, the reconstruction path is
    // unreachable HERE, and the gate's accessor failure cannot be laid at its door.
    // That is a refutation of the arc's stated root cause, and it must be read as one
    // rather than as a quiet pass.
    assert!(
        !reconstructed.is_empty(),
        "EVERY user type retained a source form — `type_def_to_ast` never fires in this \
         world, so the reconstruction cannot be the accessor defect's cause. This is a \
         REFUTATION of the seam's root-cause claim, not a green result: re-ground before \
         building the retention fix."
    );
}
