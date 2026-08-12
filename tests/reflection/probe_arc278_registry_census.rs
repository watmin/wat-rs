//! Arc 278 — the REGISTRY CENSUS.
//!
//! THE QUESTION. A name in this substrate can be registered in five places, each answering
//! at a different phase:
//!
//!   macro_registry      EXPAND-time
//!   types (TypeEnv)     CHECK-time
//!   functions           EVAL-time
//!   unit_variants       EVAL-time
//!   runtime_def_values  EVAL-time
//!
//! There is no single resolver across them (grep for one returns nothing). Any consumer that
//! must answer "what does this name depend on" has to know to ask all five, from memory,
//! forever. `closure_extract` is exactly that consumer and asks FOUR — it never reads
//! `macro_registry` — which is why a `defservice`-synthesized record shipped to a forked child
//! as a type you cannot construct (its kwargs constructor is a `defmacro`).
//!
//! The ruling is a REGISTRY ENUM the resolver walks. This census measures the one fact that
//! decides the enum's shape:
//!
//!   ★ When a name appears in MORE THAN ONE registry, is it the SAME concept seen at two
//!     phases (`:my::Rec` = the record type + its kwargs constructor), or can one name mean
//!     two UNRELATED things?
//!
//! Same-concept ⇒ a name maps to a SET of facets and the resolver returns all of them.
//! Unrelated ⇒ the resolver must return independent registrations and PRECEDENCE becomes a
//! ruling, not an implementation detail. That is the difference the enum has to encode, and
//! it is not guessable from reading.
//!
//! `MacroRegistry`'s map is `pub(super)`, so macro membership is probed by name via its
//! public `contains` rather than enumerated. The four other registries are enumerated.

use std::collections::{BTreeMap, BTreeSet};
use wat::value::symbol_table::RegistryKind;

/// The census now goes THROUGH THE DOOR (`SymbolTable::registrations`) rather
/// than reaching into the registries by hand — the wall this stone raised made
/// the hand-reach uncompilable, and this test was its last offender. So it is
/// now a test OF the door as well as a census: if `registrations` ever stops
/// reporting a facet, the shape assertions below go red.
///
/// Name ENUMERATION still needs the per-registry iterators (the door answers
/// per-name, it does not list names). Macro-only names are therefore invisible
/// here — stated, not hidden, because it bounds the result.
fn census(path: &str) -> BTreeMap<String, BTreeSet<RegistryKind>> {
    let world = wat::freeze::startup_from_file(path).expect("freeze should succeed");
    let sym = world.symbols();

    let mut names: BTreeSet<String> = BTreeSet::new();
    names.extend(sym.functions_iter().map(|(n, _)| n.clone()));
    names.extend(sym.unit_variants_iter().map(|(n, _)| n.clone()));
    names.extend(sym.def_values_iter().map(|(n, _)| n.clone()));
    if let Some(types) = sym.types_deref() {
        names.extend(types.iter().map(|(n, _)| n.clone()));
    }

    names
        .into_iter()
        .map(|n| {
            let kinds: BTreeSet<RegistryKind> = sym.registrations(&n).iter().collect();
            (n, kinds)
        })
        .collect()
}

#[test]
fn registry_census_names_the_multi_registry_shape() {
    // A defservice world: surfaces, a service, synthesized records/enums, generated
    // constructors — the exact shape whose closure shipped uncallable.
    let all = census("wat-scripts/scratch-pad/probe-arc278-union-closure-boots-a-process-child.wat");

    let multi: BTreeMap<&String, &BTreeSet<RegistryKind>> =
        all.iter().filter(|(_, k)| k.len() > 1).collect();

    // Group by the KIND-SET, so the shape is legible rather than a wall of names.
    let mut by_shape: BTreeMap<Vec<RegistryKind>, Vec<&str>> = BTreeMap::new();
    for (name, kinds) in &multi {
        by_shape
            .entry(kinds.iter().copied().collect())
            .or_default()
            .push(name.as_str());
    }

    println!("\n=== REGISTRY CENSUS ===");
    println!("names registered anywhere : {}", all.len());
    println!("names in >1 registry      : {}", multi.len());
    for (shape, names) in &by_shape {
        println!("\n  {:?}  ({} names)", shape, names.len());
        for n in names.iter().take(12) {
            println!("      {n}");
        }
        if names.len() > 12 {
            println!("      … and {} more", names.len() - 12);
        }
    }
    println!();

    // NON-VACUITY. If nothing lands in more than one registry, this census measured
    // nothing and the enum's shape is undecided — that must fail loudly, not read as
    // a clean result.
    assert!(
        !multi.is_empty(),
        "census found NO name in more than one registry — the instrument is vacuous, \
         not the answer"
    );
}
