//! PROBE — arc 296 stone ③b-i, `register_variant` is its own door.
//!
//! `docs/arc/2026/06/255-builtin-registry/DESIGN-the-variant-name-is-COMPOSED-never-TYPED.md`
//! and its sibling BRIEF. H-1 (the `DottedName` wall in `src/resolve/registration.rs`)
//! is origin-gated: it fires for a caller-TYPED name (`register`) and is exempt for the
//! grammar's own composed variant name (`register_variant`), which takes `(parent, leaf)`
//! separately and composes internally.
//!
//! Nothing in the tree pinned the builder's own question before this stone: *"if they
//! make a defn and an enum with the same name, one loses — this is pathological and we
//! catch it, right?"* The collision was measured on this tree at `7ccce48ba` (both
//! orders, `duplicate define` at `--check`, exit 1) but UNPINNED — the loud
//! `DuplicateDefine` fires from a later `register` pass
//! (`src/declare/register.rs::register_enum_methods`), a phase entirely separate from
//! `preregister.rs`'s `Existing::Equivalent` mapping (which the gate turns into a benign
//! `NoOp` — on its own that reads "already there, fine"). A guard that has never failed
//! once is not yet a guard.
//!
//! These rows pin it, plus the end-to-end surface check that H-1 stays absolute for a
//! `defn`'s own (caller-typed) name.

use std::path::PathBuf;
use std::process::{Command, Stdio};

fn rel(case: &str) -> String {
    format!("tests/resolve/probe_arc255_register_variant_is_its_own_door__{case}.wat")
}

fn run_check(case: &str) -> (i32, String) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    assert!(root.join(rel(case)).exists(), "fixture missing: {}", rel(case));
    let out = Command::new(env!("CARGO_BIN_EXE_wat"))
        .current_dir(&root)
        .arg("--check")
        .arg(rel(case))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("spawn wat --check");
    (
        out.status.code().unwrap_or(-1),
        format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ),
    )
}

/// The builder's question, order A: the `defn` (a caller-typed name) registers FIRST;
/// the enum's variant ctor path (`register_variant`, composed from `(:my::app::Foo,
/// Bar)`) collides against it SECOND. First-definer-wins never silently applies —
/// `preregister.rs`'s `Existing::Equivalent`-as-`NoOp` mapping does NOT swallow this: the
/// later `register_enum_methods` pass raises its own explicit `DuplicateDefine`.
#[test]
fn defn_then_variant_ctor_collide_at_check() {
    let (code, out) = run_check("defn_then_variant");
    assert_eq!(code, 1, "a defn and a same-named variant ctor must collide:\n{out}");
    wat::assert_edn_eq!(
        out,
        include_str!("probe_arc255_register_variant_is_its_own_door__defn_then_variant.edn")
    );
}

/// The other order: the enum's variant ctor path registers FIRST (via
/// `register_variant`), and the `defn` typed literally under that same name collides
/// against it SECOND. The composed door gets no first-mover advantage over a
/// caller-typed name, and vice versa — both orders raise the identical
/// `DuplicateDefine`, symmetrically.
#[test]
fn variant_ctor_then_defn_collide_at_check() {
    let (code, out) = run_check("variant_then_defn");
    assert_eq!(code, 1, "a variant ctor and a same-named defn must collide:\n{out}");
    wat::assert_edn_eq!(
        out,
        include_str!("probe_arc255_register_variant_is_its_own_door__variant_then_defn.edn")
    );
}

/// H-1 is pinned from the SURFACE, not only at the gate: a plain `defn` whose own name
/// carries a dot in its name segment is still refused at `--check`, end to end. The
/// origin split exempts only the composed-variant door (`register_variant`); a name a
/// caller typed goes through `register` (`NameOrigin::Declared`) and the wall is
/// unchanged.
#[test]
fn a_defn_with_a_dotted_name_is_still_refused_end_to_end() {
    let (code, out) = run_check("dotted_defn");
    assert_eq!(code, 1, "a caller-typed dotted name must still be refused:\n{out}");
    wat::assert_edn_eq!(
        out,
        include_str!("probe_arc255_register_variant_is_its_own_door__dotted_defn.edn")
    );
}

/// Room ⑤a/⑤b, pinned from the SURFACE: a `defenum` variant whose OWN name carries a
/// dot is refused at `--check`, end to end. This is nothing `dotted_defn` above already
/// covers — a `defn`'s name goes through `register`/`NameOrigin::Declared`, but a
/// variant's name goes through the COMPOSED door (`register_variant`, and
/// `TypeEnv::register_variant_type` for the variant's own singleton type) — and it is
/// the door's precondition on its `variant_leaf` argument, not `gate`'s origin-gated
/// arm, that catches this: `gate` never sees the raw leaf, only the already-composed
/// name.
#[test]
fn a_defenum_with_a_dotted_variant_name_is_still_refused_end_to_end() {
    let (code, out) = run_check("dotted_variant");
    assert_eq!(code, 1, "a variant name typed with a dot must still be refused:\n{out}");
    wat::assert_edn_eq!(
        out,
        include_str!("probe_arc255_register_variant_is_its_own_door__dotted_variant.edn")
    );
}
