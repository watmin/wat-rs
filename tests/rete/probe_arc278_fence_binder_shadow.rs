//! Arc 278 strike **a-fence-local-binder-may-not-shadow-a-rete-var**.
//!
//! `check_fence_interior` (`src/rete/validate/typing.rs`) used to skip a fence-local `let`'s body
//! and a `match` arm's body entirely, because either could bind a `?`-prefixed name that SHADOWS
//! a rule-wide `?var` of the same name at a DIFFERENT type — and the walk carries no scope stack,
//! so reading a shadowed name through the rule-wide map would check it at the wrong type and
//! refuse legal, firing code. Driven at `f608c2a5b` (DESIGN.md): `(let [?k "str"] …)` compiles and
//! fires beside a condition binding `?k` to `i64`.
//!
//! This stone retires the JUSTIFICATION for that bound rather than building the scope stack: a
//! `?`-prefixed name may not be a BINDER in a fence-local `let`'s binding vector or a `match`
//! arm's pattern at all, refused at rule-compile time by its own variant
//! (`ReteCheckErrorKind::FenceBinderShadowsReteVar`). Making the shape unrepresentable is what
//! removes the hazard the old bound was defending against — see `check_fence_interior`'s rewritten
//! doc comment for the full argument.
//!
//! ⛔ **This does NOT make `let`/`match` BODIES type-checked.** `(i64::> x 100)` still needs `x`'s
//! type to check, and `x` is a fence-local binder — not in the rule-wide `binds` map, not a field,
//! not a literal — so `resolve_operand_type` still answers `ComputedNotDerivableHere` and skips
//! it. See [`plain_binders_still_compile`] below: `row 1` reproduces the one real corpus site
//! (`where-inline-computed.wat:166`) verbatim, and it must keep compiling.
//!
//! ## The `match` question — driven, not inferred
//!
//! The `let` hazard was driven at HEAD before this stone. The `match` hazard was NOT — the brief
//! named it an open question, my inference from `lower_pat`'s doc comment rather than a run. It
//! was driven with a scratch probe, BEFORE this stone's cure existed: `?k` bound outer-scope to
//! `i64` beside a fence `(match "shadow" (?k (string::= ?k "shadow")))`, run through
//! `./target/release/wat` (pre-cure binary), printed `"match-arm 1, control 1"` — the match-arm
//! rule FIRES, so the bare `?k` pattern really does bind (shadow), not merely read the outer
//! value.
//!
//! ⛔ **The probe itself is NOT in the tree.** It lived at
//! `wat-scripts/scratch-pad/arc278-fence-binder-shadow/probe-match-pattern-binds.wat` only long
//! enough to be driven, then was DELETED: once this stone's cure lands, the shape it demonstrates
//! is refused at rule-compile time, and `wat-scripts/` gates every declared rete rule to actually
//! compile with no exemption category (`rete_compile_gate`, `every_wat_scripts_file_loads`) — a
//! file that can no longer compile cannot live there. Its permanent home, as a fixture that is
//! SUPPOSED to fail, is `probe_arc278_fence_binder_shadow_match.wat.bad` below —
//! `match_shadow_is_refused` is the compile-time refusal of the identical shape the probe drove.

use wat::freeze::{startup_beside, startup_from_file};

/// THE `let` HAZARD, REFUSED. `(let [?k "shadow"] (string::= ?k "shadow"))` inside a fence, beside
/// a condition binding `?k` to `i64` — before this stone, accepted silently and fires; after,
/// refused at rule-compile time.
#[test]
fn let_shadow_is_refused() {
    let result = startup_from_file("tests/rete/probe_arc278_fence_binder_shadow_let.wat.bad");
    assert!(
        result.is_err(),
        "a `?`-prefixed name used as a fence-local `let` BINDER must be refused at rule-compile \
         time — it shadows the rule-wide rete variable of the same name."
    );
}

/// THE `match` HAZARD, REFUSED — and driven first (see module doc). `(match "shadow" (?k
/// (string::= ?k "shadow")))` inside a fence, beside a condition binding `?k` to `i64`.
#[test]
fn match_shadow_is_refused() {
    let result = startup_from_file("tests/rete/probe_arc278_fence_binder_shadow_match.wat.bad");
    assert!(
        result.is_err(),
        "a `?`-prefixed name used as a `match` arm PATTERN inside a fence must be refused at \
         rule-compile time — driven (not inferred): the identical shape FIRES at HEAD before this \
         stone, so the arm pattern really does bind and shadow the rule-wide rete variable."
    );
}

/// THE OVER-REJECTION GUARD, and the one EXPECTATIONS.md calls load-bearing by name: the ONE real
/// `let`-in-a-fence corpus site (`where-inline-computed.wat:166`, row 1 below) must keep
/// compiling, alongside the other plain-binder shapes the re-derived census found live in
/// `where-control.wat` (row 2: a `let` binder feeding a boolean composition; row 3: a `match` arm's
/// variant PAYLOAD binder). A cure that refuses ANY binder, not only a `?`-prefixed one, reds this.
#[test]
fn plain_binders_still_compile() {
    let result = startup_beside(file!());
    assert!(
        result.is_ok(),
        "a PLAIN (non-`?`-prefixed) binder in a fence-local `let` or `match` pattern must keep \
         compiling — the corpus's only real `let`-in-a-fence site is exactly this shape; got: {:?}",
        result.err()
    );
}
