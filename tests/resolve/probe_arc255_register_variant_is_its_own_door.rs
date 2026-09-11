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
//! orders, `duplicate define` at `--check`, exit 1) — BEFORE the dot-flip (arc 255
//! ③b-ii). At `7ccce48ba` the composed variant name was `Foo::Bar` (`::`), a perfectly
//! ordinary caller-typeable namespaced name, so a `defn` spelled identically collided
//! with it and both orders raised `DuplicateDefine` from the later `register` pass
//! (`src/declare/register.rs::register_enum_methods`) — a guard that had never failed
//! once and was not yet a guard.
//!
//! ⚠ **arc 255 ③b-ii residue (the dot-flip) — NOT A REPAIR.** The composed variant name
//! is now `Foo.Bar` (`.`), and H-1 makes a dot in ANY caller-typed declared name
//! unconstructible (`DottedName`, absolute, checked BEFORE a definition ever reaches the
//! duplicate-define pass). So a `defn` can no longer be spelled `:my::app::Foo.Bar` at
//! all — the two rows below now assert the STRONGER truth: the collision the builder
//! asked about is refused as `DottedName` before collision-detection is ever reached, in
//! EITHER order. This is the rung ABOVE catching a `DuplicateDefine`; restoring the old
//! expectation would be reverting to a weaker guarantee, not a fix.
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

/// arc 255 ③b-ii (dot-flip) residue — NOT the original collision this fixture was named
/// for. Order A: a `defn` textually spelled `:my::app::Foo.Bar` — the composed form of
/// enum `:my::app::Foo`'s `Bar` variant, post-flip — appears FIRST, ahead of the
/// `defenum`. Pre-flip this was `Foo::Bar`, an ordinary caller-typeable name, and this
/// row proved `DuplicateDefine` once the enum registered its variant second. Post-flip,
/// `:my::app::Foo.Bar` is a DOTTED caller-typed name, and H-1 refuses it OUTRIGHT — the
/// `defn`'s own declaration never survives long enough to collide with anything, so the
/// enum on the next line is never even reached. The stronger truth: the collision is
/// structurally unconstructible, not merely caught.
#[test]
fn defn_then_variant_ctor_collide_at_check() {
    let (code, out) = run_check("defn_then_variant");
    assert_eq!(code, 1, "a dotted caller-typed name must be refused before any collision check runs:\n{out}");
    wat::assert_edn_eq!(
        out,
        include_str!("probe_arc255_register_variant_is_its_own_door__defn_then_variant.edn")
    );
}

/// The other order, same arc 255 ③b-ii finding: the `defenum` (legally spelled, no dot —
/// `Bar` is a bare variant leaf) registers FIRST; the `defn` typed literally as
/// `:my::app::Foo.Bar` SECOND is refused as `DottedName`, identically to the other
/// order — H-1 does not care whether a same-named declaration already exists, only that
/// the caller typed a dot. Both orders now raise the identical `DottedName`, not
/// `DuplicateDefine`; asserting the old expectation would revert to a weaker guarantee.
#[test]
fn variant_ctor_then_defn_collide_at_check() {
    let (code, out) = run_check("variant_then_defn");
    assert_eq!(code, 1, "a dotted caller-typed name must be refused before any collision check runs:\n{out}");
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
