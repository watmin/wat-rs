//! HYGIENE-CLASS INTEGRITY GATE — Stone 249.5g (the self-enforcing `.scopes()`-reader gate).
//!
//! # What this gate makes impossible
//!
//! The macro-hygiene class — variable capture, scoped resolution, scoped identity —
//! was annihilated across four stones (249.5b/d/e/f) and *proven last* by a manual
//! enumeration: an `Identifier`'s scope-set is consumed for keying/identity/transport
//! at exactly THREE sanctioned chokepoints —
//!   1. `crate::scope::resolution::env_key` — in-process resolution (eval + check),
//!   2. the canonical hasher (`src/hash.rs`) — cross-process identity (renumbered),
//!   3. the plain-EDN bridge (`src/edn/bridge.rs`) — cross-process TRANSPORT
//!      (remapped; arc 170) —
//!      and every resolution bind of a possibly-scoped ident routes through `env_key`.
//!
//! (3) joined in arc 170 and is the sibling of (2), not a new keying surface: both
//! cross a process boundary, and both refuse to carry the sender's raw ids — the
//! hasher RENUMBERS to first-appearance indices, the bridge REMAPS each distinct wire
//! id to a `fresh_scope()`. Neither ever keys by a scope; they preserve the structure
//! and hand resolution back to `env_key` on the far side.
//!
//! A manual enumeration is a snapshot; this gate makes it a **build-time invariant.**
//! A future `src/` file that reads `Identifier::scopes()` directly is a candidate
//! name-only-keying leak — a fourth keying surface that could silently reopen the
//! class three stones deep. This gate FAILS LOUD the moment such a reader is added,
//! forcing the author to either route the keying through `env_key` (the fix) or
//! justify-and-allowlist it (reviewed). This is the construction-time-check rung of
//! the failure-engineering ladder for the whole class — the move that turns
//! "proven last" into "can't-be-reopened".
//!
//! # Why the gate is COMPLETE
//!
//! The `Identifier.scopes` field is PRIVATE (Stone 249.5c-fix2 encapsulated it), so
//! the ONLY way any code can read a scope-set is via the `.scopes()` getter. Gating
//! every `.scopes()` caller therefore covers every possible scope-reader — there is
//! no other access path. (The companion defense for the *other* surface — a scoped
//! ident keyed by `.as_str()` instead of `env_key` — is the hygiene PROBE family
//! [`probe_macro_hygiene_capture`, `probe_argspec_rest_param_hygiene`,
//! `probe_check_scoped_param_resolution`, `probe_hash_scope_renumber`]: a mis-keying
//! shows up as a captured/unbound/imprecise/non-deterministic result those probes
//! catch. Gate + probes = the two-part structural defense.)
//!
//! # The allowlist
//!
//! Each entry is a `src/`-relative file path permitted to read `.scopes()`, with the
//! reason it is a sanctioned reader. Adding an entry is a deliberate, reviewed act.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// The ONLY `src/` files permitted to read `Identifier::scopes()`. Each is a
/// sanctioned scope-reader with its justification.
const SANCTIONED_SCOPES_READERS: &[(&str, &str)] = &[
    (
        "scope/resolution.rs",
        "env_key — THE scope-aware resolution policy; the in-process keying chokepoint \
         (runtime + check both route through it).",
    ),
    (
        "hash.rs",
        "the canonical hasher — scope-aware identity (cross-process); scopes are \
         renumbered to first-appearance DFS indices (Stone 249.5f).",
    ),
    (
        "macros/tests.rs",
        "TEST — asserts the expander (walk_template) APPLIES scope tags; reads scopes \
         to verify the mint side, never keys/identifies by them.",
    ),
    (
        "edn/bridge.rs",
        "the plain-EDN program bridge — cross-process TRANSPORT of hygiene (arc 170). \
         Reads scopes ONLY to serialize them onto the wire as \
         `#wat.ast/ScopedSymbol {:name … :scopes […]}`, and NEVER keys or identifies \
         by them. Sibling of hash.rs (also cross-process, also refuses the sender's \
         raw ids): the hasher renumbers, the bridge REMAPS each distinct wire id to a \
         `fresh_scope()`, so an imported scope can never collide with one this process \
         mints — resolution on the far side is still env_key's alone. \
         Necessary because `spawn-process` ships an evaluated `Vector<WatAST>` a macro \
         built (kernel/spawn.rs:485); erasing its scopes in transit manufactures \
         HygieneScopeDivergence (check.rs:2035) — disconfirmed by a run. \
         BACKED BY: tests/program/probe_arc170_edn_bridge_hygiene.rs (C02 scopes \
         survive, C03 shared-stays-shared + distinct-stays-distinct + the two \
         structures do not collapse, C04 imported ids are fresh, C05 the wire form).",
    ),
];

fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("src/ must be readable") {
        let p = entry.expect("dir entry").path();
        if p.is_dir() {
            collect_rs(&p, out);
        } else if p.extension().is_some_and(|e| e == "rs") {
            out.push(p);
        }
    }
}

#[test]
fn only_sanctioned_files_read_identifier_scopes() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    collect_rs(&src, &mut files);

    let allowed: HashSet<&str> = SANCTIONED_SCOPES_READERS.iter().map(|(f, _)| *f).collect();

    let mut violations = Vec::new();
    for f in &files {
        let rel = f
            .strip_prefix(&src)
            .expect("under src/")
            .to_string_lossy()
            .replace('\\', "/");
        let body = fs::read_to_string(f).expect("readable .rs");
        if body.contains(".scopes()") && !allowed.contains(rel.as_str()) {
            violations.push(rel);
        }
    }

    assert!(
        violations.is_empty(),
        "HYGIENE-CLASS INTEGRITY VIOLATION — these src/ files read \
         Identifier::scopes() but are NOT sanctioned scope-readers:\n  {:?}\n\n\
         The macro-hygiene class (Stones 249.5b/d/e/f) funnels ALL scope consumption \
         through env_key (resolution) + the canonical hasher (identity). A new \
         .scopes() reader is a candidate name-only-keying leak / a fourth keying \
         surface that could silently reopen the class.\n\n\
         FIX: route the keying through `crate::scope::env_key` (the sanctioned \
         chokepoint). If this is a GENUINELY new sanctioned consumer, add it to \
         SANCTIONED_SCOPES_READERS in this file WITH its reason — and back it with a \
         hygiene probe.",
        violations
    );
}

/// Guards the gate itself against rot: every allowlisted path must still exist and
/// still actually read `.scopes()`. A stale allowlist entry (file moved, or no longer
/// reads scopes) is a silent weakening of the gate — fail loud so it stays honest.
#[test]
fn allowlist_has_no_stale_entries() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut stale = Vec::new();
    for (rel, _reason) in SANCTIONED_SCOPES_READERS {
        let p = src.join(rel);
        let reads = fs::read_to_string(&p)
            .map(|b| b.contains(".scopes()"))
            .unwrap_or(false);
        if !reads {
            stale.push(*rel);
        }
    }
    assert!(
        stale.is_empty(),
        "stale SANCTIONED_SCOPES_READERS entries (missing, or no longer read .scopes()): \
         {:?}. Remove them — an allowlist that over-permits is a weakened gate.",
        stale
    );
}
