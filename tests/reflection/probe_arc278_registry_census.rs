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

/// The registries a name can be registered in. This mirrors what a real
/// `RegistryKind` enum would carry — the census exists to check whether that
/// enum is a set-of-facets or a set-of-rivals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Kind {
    Macro,
    Type,
    Function,
    UnitVariant,
    DefValue,
}

fn census(path: &str) -> BTreeMap<String, BTreeSet<Kind>> {
    let world = wat::freeze::startup_from_file(path).expect("freeze should succeed");
    let sym = world.symbols();
    let mut out: BTreeMap<String, BTreeSet<Kind>> = BTreeMap::new();

    for name in sym.functions.keys() {
        out.entry(name.clone()).or_default().insert(Kind::Function);
    }
    for name in sym.unit_variants.keys() {
        out.entry(name.clone()).or_default().insert(Kind::UnitVariant);
    }
    for name in sym.runtime_def_values.keys() {
        out.entry(name.clone()).or_default().insert(Kind::DefValue);
    }
    if let Some(types) = sym.types() {
        for (name, _def) in types.iter() {
            out.entry(name.clone()).or_default().insert(Kind::Type);
        }
    }
    // Macro membership is PROBED (the registry's map is pub(super)); every name any other
    // registry knows is asked. A macro registered under a name NO other registry knows is
    // therefore invisible to this census — stated, not hidden, because it bounds the result.
    if let Some(macros) = &sym.macro_registry {
        let names: Vec<String> = out.keys().cloned().collect();
        for name in names {
            if macros.contains(&name) {
                out.entry(name).or_default().insert(Kind::Macro);
            }
        }
    }
    out
}

#[test]
fn registry_census_names_the_multi_registry_shape() {
    // A defservice world: surfaces, a service, synthesized records/enums, generated
    // constructors — the exact shape whose closure shipped uncallable.
    let all = census("wat-scripts/scratch-pad/probe-arc278-union-closure-boots-a-process-child.wat");

    let multi: BTreeMap<&String, &BTreeSet<Kind>> =
        all.iter().filter(|(_, k)| k.len() > 1).collect();

    // Group by the KIND-SET, so the shape is legible rather than a wall of names.
    let mut by_shape: BTreeMap<Vec<Kind>, Vec<&str>> = BTreeMap::new();
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
