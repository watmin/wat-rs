//! The ONE reserved-prefix + idempotent registration gate.
//!
//! A single authoritative implementation of the rule enforced at every point where a
//! name is registered into a registry — types, macros, runtime defs / aliases /
//! accessors / constructors / defclause. It replaces the ELEVEN hand-rolled gates and
//! FOUR privilege mechanisms catalogued in
//! `docs/arc/2026/06/278-rules-engine/DESIGN-reserved-prefix-one-gate.md`
//! (`MacroRegistry::stdlib_privilege`, `RegistrationPrivilege`, the `check_reserved*`
//! bool params, and the `register_stdlib` / duplicated-call-chain methods).
//!
//! The load-bearing invariant lives here, once and correct by construction: **the
//! idempotent no-op is checked BEFORE the reserved-prefix gate**, so a byte/structurally
//! equivalent re-declaration of an already-registered form is ALWAYS a no-op — regardless
//! of privilege or namespace (e.g. a forked child that re-bakes the stdlib and then
//! re-declares a baked `:wat::` form it already holds). The reserved gate rejects only
//! GENUINELY NEW names from unprivileged source. This is why "you cannot declare an
//! existing form" cannot recur: there is one gate, and it checks equivalence first.

use super::reserved::is_reserved_prefix;
use crate::span::Span;

/// The ONE privilege bit — "is this registration processing STDLIB source or USER
/// source?" — the single distinction the four old mechanisms all encoded. Threaded
/// EXPLICITLY from the phase split (`freeze/env.rs`'s privileged/unprivileged expand
/// passes), never carried as ambient mutable state. Stdlib may declare reserved-prefix
/// (`:wat::` / `:rust::`) names; User may not.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Privilege {
    Stdlib,
    User,
}

/// Arc 296 stone ③b-i — HOW the name reaching the gate was made. `Privilege` says WHO
/// made the name; `NameOrigin` says HOW THE NAME WAS MADE. Two axes, one table, told
/// apart by an explicit parameter rather than by a call-site bypass.
///
/// `DottedName` (H-1) is the one arm this gates: a name a caller TYPED is refused if its
/// leaf carries a dot; a name the grammar's own `compose_variant` COMPOSED from separate
/// `(parent, leaf)` halves is exempt — not because the wall weakened, but because the
/// only entry point that accepts a `ComposedVariant` origin never receives a dotted
/// string in the first place, it receives the two halves and builds the name itself
/// (see [`register_variant`]). See
/// `docs/arc/2026/06/255-builtin-registry/DESIGN-the-variant-name-is-COMPOSED-never-TYPED.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NameOrigin {
    /// A name a caller TYPED — a def form's own name. A name-half dot is refused.
    Declared,
    /// A name the grammar's own `compose_variant` built from (parent, leaf). Its dot,
    /// when the variant separator becomes one, is the composer's, not a caller's.
    ComposedVariant,
}

/// What the caller found in its OWN registry for this name, before registering. The
/// caller computes this (each registry keys differently, and "equivalent" means
/// structural-equivalence for macros, `==` for types, etc.); the gate reasons over the
/// classification, not the registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Existing {
    /// No entry under this name yet.
    Absent,
    /// An entry exists and is equivalent to the incoming definition (idempotent).
    Equivalent,
    /// An entry exists and DIVERGES from the incoming definition.
    Divergent,
}

/// The gate's verdict. The caller maps it to its own action + error type (the gate stays
/// error-taxonomy-neutral, so `MacroError` / `TypeError` / `RuntimeError` stay put).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Registration {
    /// New and allowed — proceed with insertion.
    Insert,
    /// Already registered and equivalent — a benign no-op (idempotent re-declaration).
    NoOp,
    /// Already registered and divergent — caller emits its own Duplicate error.
    Duplicate,
    /// A new reserved-prefix name from unprivileged source — caller emits its own
    /// ReservedPrefix error.
    Reserved,
    /// A new top-level name with no namespace — caller emits its own UnnamespacedName
    /// error. Held even against `Privilege::Stdlib` — there is no privilege escape.
    Unnamespaced,
    /// Arc 296 stone H-1 — the type NAME (the segment after the last `::`) contains a
    /// `.`. Caller emits its own DottedName-shaped error. Held even against
    /// `Privilege::Stdlib` — there is no privilege escape, same as `Unnamespaced`. A dot
    /// in a name is reserved: stone H's tagged-variant wire form is `#ns/Enum.Variant`,
    /// where the dot in the NAME half is the discriminator that says "this tag is a
    /// variant, not a record." If a record's own name could contain a dot, it could
    /// forge that tag — so the dot is banned at the one door every name passes through,
    /// not merely absent from the corpus by chance.
    DottedName,
}

/// A top-level name must carry a namespace. Only fn args and `let` bindings may be bare,
/// and those are lexical — they never reach this gate.
///
/// NOT "starts with ':' and contains '::'": parametric heads drop the leading colon
/// (`wat::kernel::Peer`), recorded in arc 170's 24t seam. The test is containment.
pub fn is_namespaced(name: &str) -> bool {
    // rune:lint(one-variant-separator, namespace) — the containment test IS the definition of
    // "namespaced" for a top-level declared name; no enum or variant is involved.
    name.contains("::")
}

/// Arc 296 stone H-1 — true if the NAME half (the segment after the LAST `::`) contains
/// a `.`. The namespace half is untouched: `:wat::core::Fault` has namespace `wat::core`
/// and name `Fault`; only `Fault` is checked. Works whether or not `name` carries the
/// leading `:` (parametric heads drop it — arc 170's 24t seam), because `leaf()`
/// finds the last segment either way. At registration the name is always `::`-separated
/// (dots appear later, only in the wire tag built by `tag_from_type_path`), so this is a
/// pure ban, not a parse of an already-dotted form.
fn has_dotted_name(name: &str) -> bool {
    // rune:lint(one-variant-separator, namespace) — isolates the name half (after the last
    // `::`) of a namespaced declaration to check for an illegal embedded dot; a namespace/leaf
    // split on the declared name itself, not a decompose of an enum's variant.
    wat_reader::identifier::leaf(name).contains('.')
}

/// THE gate. The rule + ordering, once:
///
/// ```text
///   Existing::Equivalent                                            -> NoOp       (benign re-declaration)
///   Existing::Divergent                                             -> Duplicate
///   Absent + !namespaced                                             -> Unnamespaced
///   Absent + namespaced + Declared + dotted name                     -> DottedName   (origin-gated)
///   Absent + namespaced + undotted + reserved + Privilege::User      -> Reserved
///   Absent + namespaced + undotted + (Privilege::Stdlib | !reserved) -> Insert
/// ```
///
/// Idempotent-BEFORE-reserved is the ordering that fixes "you cannot declare an existing
/// form": an equivalent re-declaration is a `NoOp` even for a reserved name from
/// unprivileged source, because it grants nothing (the name already resolves to the same
/// definition). A DIVERGENT re-declaration still errors (`Duplicate`), and a genuinely
/// NEW reserved name from `User` source is still rejected (`Reserved`) — the gate's
/// purpose is fully preserved.
///
/// `Unnamespaced` is tested before `Reserved` because a bare name cannot be reserved
/// (every reserved prefix contains `::`), and "not namespaced" is the more specific
/// truth about it.
///
/// `DottedName` (arc 296 stone H-1) is tested right after `Unnamespaced` and before
/// `Reserved`, on the same footing as `Unnamespaced`: it is a WALL, not a
/// privilege-gated permission, so it is held even against `Privilege::Stdlib` — there is
/// no privilege escape from it, exactly as there is none from the namespacing wall.
///
/// Arc 296 stone ③b-i adds one condition to this arm, and only this arm: the wall fires
/// only when `origin == NameOrigin::Declared`, i.e. the name is a caller's own typed
/// name. A `NameOrigin::ComposedVariant` name is exempt — not a weakening of H-1, because
/// the only way to reach the gate with that origin is through [`register_variant`], which
/// never receives a dotted string from a caller; it receives `(parent, leaf)` separately
/// and composes the name itself via `compose_variant`. The EDN wire discriminator stays
/// sound by construction: a dot in the name half still means "variant", because the only
/// path that can produce one is the grammar's own composer, never a caller-typed string.
/// Every other wall — `Duplicate`, `Unnamespaced`, `Reserved` — applies to a composed
/// variant exactly as it does to a declared name; a user still cannot compose a variant
/// name under a reserved prefix they don't own.
fn gate(name: &str, origin: NameOrigin, privilege: Privilege, existing: Existing) -> Registration {
    match existing {
        Existing::Equivalent => Registration::NoOp,
        Existing::Divergent => Registration::Duplicate,
        Existing::Absent => {
            if !is_namespaced(name) {
                Registration::Unnamespaced
            } else if origin == NameOrigin::Declared && has_dotted_name(name) {
                Registration::DottedName
            } else if privilege == Privilege::User && is_reserved_prefix(name) {
                Registration::Reserved
            } else {
                Registration::Insert
            }
        }
    }
}

/// Arc 296 stone I — what a caller gets back when [`register`] refuses. Carries enough
/// for the caller's OWN error taxonomy to build its located error: the verdict (which
/// wall fired), the name (what was being registered), and the span (where). `From<Rejection>`
/// is implemented for each of the four error taxonomies (`RuntimeError`, `TypeError`,
/// `MacroError`, `CheckError`) next to their own definitions, so `?` performs the
/// taxonomy conversion at every call site — no caller matches on `Registration` itself.
///
/// `verdict` is never `Registration::Insert` or `Registration::NoOp` — [`register`] only
/// ever builds a `Rejection` for a genuine refusal (`Duplicate` / `Reserved` /
/// `Unnamespaced` / `DottedName`); those two are handled internally by performing (or
/// skipping) the insert instead.
#[derive(Debug, Clone)]
pub struct Rejection {
    pub verdict: Registration,
    pub name: String,
    pub span: Span,
}

/// THE one public entry to the gate. `gate` is private to this module — asking the door
/// and separately performing the insert has no form, because inserting IS passing
/// through: `insert` runs exactly when (and only when) the gate answers `Insert`, inside
/// this call, so there is no window in which a caller can hold an `Insert` verdict and
/// fail to act on it (or act on it wrong).
///
/// `insert` is fallible (`FnOnce() -> Result<T, E>`) so a caller whose own insert step
/// has an independent failure mode (e.g. a cycle check that must run before the actual
/// write) can surface it through the SAME `E` the rejection converts into — `E:
/// From<Rejection>` is the only requirement, so `?` at the call site handles both the
/// gate's rejection and the insert's own failure uniformly, with no arm written on
/// `Registration` anywhere outside this module.
///
/// Returns `Ok(Some(t))` when the insert ran (gate said `Insert`) and produced `t`;
/// `Ok(None)` when the name is already registered identically (gate said `NoOp` — a
/// benign idempotent re-declaration, insert does NOT run); `Err(_)` when the gate
/// refused (`Duplicate` / `Reserved` / `Unnamespaced` / `DottedName`).
pub fn register<T, E>(
    name: &str,
    privilege: Privilege,
    existing: Existing,
    span: &Span,
    insert: impl FnOnce() -> Result<T, E>,
) -> Result<Option<T>, E>
where
    E: From<Rejection>,
{
    match gate(name, NameOrigin::Declared, privilege, existing) {
        Registration::Insert => insert().map(Some),
        Registration::NoOp => Ok(None),
        verdict => Err(E::from(Rejection { verdict, name: name.to_string(), span: span.clone() })),
    }
}

/// Arc 296 stone ③b-i — the DOOR door. A sibling of [`register`], same generic shape,
/// for the one caller that composes a name rather than typing one: an enum's variant
/// constructor path.
///
/// Takes `parent` and `variant_leaf` SEPARATELY — on purpose, and this is the whole
/// ruling, not an implementation detail. A caller must not be able to hand this door a
/// string it built itself; if it holds only the already-composed string, the ruling's
/// premise is wrong (see the brief's STOP-4), not a reason to widen this signature to
/// accept one. Composing internally via `wat_reader::identifier::compose_variant` is what
/// lets the gate trust `NameOrigin::ComposedVariant`: the dot (once the variant separator
/// becomes one) is the composer's, never a caller's.
///
/// `register` keeps refusing a dot in a name a caller typed; `register_variant` accepts
/// the dot because it wrote it. See
/// `docs/arc/2026/06/255-builtin-registry/DESIGN-the-variant-name-is-COMPOSED-never-TYPED.md`.
///
/// ⛔ Precondition on the door's OWN input, not a dot-count on the composed output:
/// `variant_leaf` itself must carry no `.`. `gate`'s `ComposedVariant` arm is untouched —
/// still exactly the one origin-gated condition room② describes — because the wall this
/// door must not let slip is upstream of the gate, in what `variant_leaf` is ALLOWED to
/// be. Without this, `register_variant(":my::Shape", "Circle.Baz", …)` would compose
/// `:my::Shape::Circle.Baz`, and — once the variant separator itself becomes `.` — that
/// string decodes via `edn/render.rs`'s `split_variant_tag_name` (`rfind('.')`) as enum
/// `Shape.Circle`, variant `Baz`: a silent misdecode, the same shape H-1 exists to
/// prevent, now reachable through the composer's OWN leaf argument instead of a caller's
/// typed name. The rejection's `name` is still the composed string, so the error names
/// what was refused, not merely the raw leaf.
///
/// Checked only when `existing != Existing::Equivalent` — same idempotent-before-every-
/// wall invariant this module documents at the top (`register`'s doc comment): a benign
/// re-declaration is never blocked by ANY wall, this one included.
pub fn register_variant<T, E>(
    parent: &str,
    variant_leaf: &str,
    privilege: Privilege,
    existing: Existing,
    span: &Span,
    insert: impl FnOnce() -> Result<T, E>,
) -> Result<Option<T>, E>
where
    E: From<Rejection>,
{
    let name = wat_reader::identifier::compose_variant(parent, variant_leaf);
    if existing != Existing::Equivalent && variant_leaf.contains('.') {
        return Err(E::from(Rejection { verdict: Registration::DottedName, name, span: span.clone() }));
    }
    match gate(&name, NameOrigin::ComposedVariant, privilege, existing) {
        Registration::Insert => insert().map(Some),
        Registration::NoOp => Ok(None),
        verdict => Err(E::from(Rejection { verdict, name, span: span.clone() })),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equivalent_redeclaration_is_always_a_noop() {
        // The load-bearing fix: a benign re-declaration is NEVER blocked by the gate,
        // regardless of privilege or reservedness. This is the fork case (the child
        // re-declaring a baked `:wat::` form it already holds).
        assert_eq!(gate(":wat::query::Store", NameOrigin::Declared, Privilege::User, Existing::Equivalent), Registration::NoOp);
        assert_eq!(gate(":wat::query::Store", NameOrigin::Declared, Privilege::Stdlib, Existing::Equivalent), Registration::NoOp);
        assert_eq!(gate(":rust::sqlite::Db", NameOrigin::Declared, Privilege::User, Existing::Equivalent), Registration::NoOp);
        assert_eq!(gate(":my::Thing", NameOrigin::Declared, Privilege::User, Existing::Equivalent), Registration::NoOp);
    }

    #[test]
    fn divergent_redeclaration_is_duplicate_regardless_of_privilege() {
        assert_eq!(gate(":wat::query::Store", NameOrigin::Declared, Privilege::Stdlib, Existing::Divergent), Registration::Duplicate);
        assert_eq!(gate(":wat::query::Store", NameOrigin::Declared, Privilege::User, Existing::Divergent), Registration::Duplicate);
        assert_eq!(gate(":my::Thing", NameOrigin::Declared, Privilege::User, Existing::Divergent), Registration::Duplicate);
    }

    #[test]
    fn new_reserved_name_from_user_is_rejected() {
        assert_eq!(gate(":wat::query::Store", NameOrigin::Declared, Privilege::User, Existing::Absent), Registration::Reserved);
        assert_eq!(gate(":rust::sqlite::Db", NameOrigin::Declared, Privilege::User, Existing::Absent), Registration::Reserved);
    }

    #[test]
    fn new_reserved_name_from_stdlib_inserts() {
        assert_eq!(gate(":wat::query::Store", NameOrigin::Declared, Privilege::Stdlib, Existing::Absent), Registration::Insert);
        assert_eq!(gate(":rust::sqlite::Db", NameOrigin::Declared, Privilege::Stdlib, Existing::Absent), Registration::Insert);
    }

    #[test]
    fn new_user_name_inserts_under_either_privilege() {
        assert_eq!(gate(":my::Thing", NameOrigin::Declared, Privilege::User, Existing::Absent), Registration::Insert);
        assert_eq!(gate(":my::Thing", NameOrigin::Declared, Privilege::Stdlib, Existing::Absent), Registration::Insert);
    }

    #[test]
    fn bare_name_from_user_is_unnamespaced() {
        assert_eq!(gate(":no-ns", NameOrigin::Declared, Privilege::User, Existing::Absent), Registration::Unnamespaced);
    }

    #[test]
    fn namespaced_user_name_inserts() {
        assert_eq!(gate(":my::ok", NameOrigin::Declared, Privilege::User, Existing::Absent), Registration::Insert);
    }

    #[test]
    fn parametric_head_without_leading_colon_is_namespaced() {
        assert_eq!(gate("wat::kernel::Peer", NameOrigin::Declared, Privilege::Stdlib, Existing::Absent), Registration::Insert);
    }

    #[test]
    fn bare_name_from_stdlib_is_still_unnamespaced() {
        // No privilege escape from the namespacing wall.
        assert_eq!(gate(":no-ns", NameOrigin::Declared, Privilege::Stdlib, Existing::Absent), Registration::Unnamespaced);
    }

    #[test]
    fn bare_name_idempotent_replay_still_noops() {
        assert_eq!(gate(":no-ns", NameOrigin::Declared, Privilege::User, Existing::Equivalent), Registration::NoOp);
    }

    // ─── Arc 296 stone H-1 — the dot wall ──────────────────────────────

    #[test]
    fn dotted_name_from_user_is_rejected() {
        assert_eq!(gate(":my::Shape.Circle", NameOrigin::Declared, Privilege::User, Existing::Absent), Registration::DottedName);
    }

    #[test]
    fn dotted_name_from_stdlib_is_still_rejected() {
        // No privilege escape from the dot wall, same as Unnamespaced.
        assert_eq!(gate(":wat::telemetry::Numeric.I64", NameOrigin::Declared, Privilege::Stdlib, Existing::Absent), Registration::DottedName);
    }

    #[test]
    fn dot_in_namespace_half_is_untouched() {
        // Only the segment AFTER the last `::` is checked; a dot earlier in the path
        // (however unlikely) does not trip the wall.
        assert_eq!(gate(":my::v1.2::Thing", NameOrigin::Declared, Privilege::User, Existing::Absent), Registration::Insert);
    }

    #[test]
    fn dotted_name_idempotent_replay_still_noops() {
        // A benign equivalent re-declaration is never blocked, even for a name that
        // would fail the dot wall on first registration — same ordering guarantee as
        // Unnamespaced/Reserved.
        assert_eq!(gate(":my::Shape.Circle", NameOrigin::Declared, Privilege::User, Existing::Equivalent), Registration::NoOp);
    }

    // ─── Arc 296 stone ③b-i — the composed-variant door ────────────────

    #[test]
    fn composed_variant_dotted_leaf_inserts() {
        // The new arm, non-vacuous: a name with the SAME shape that DottedName refuses
        // under Declared is accepted under ComposedVariant — because the only way to
        // reach the gate with that origin is through `register_variant`, which composed
        // the dot itself rather than receiving it from a caller.
        assert_eq!(
            gate(":my::Shape.Circle", NameOrigin::ComposedVariant, Privilege::User, Existing::Absent),
            Registration::Insert
        );
    }

    #[test]
    fn declared_name_with_two_dots_is_still_refused() {
        // D is not "any dot is fine" the way retracted option C was ("a leaf may carry
        // AT MOST ONE dot" — a syntactic count threshold, applied regardless of origin).
        // D's `Declared` arm is a pure presence check, not a count: a leaf with TWO dots
        // is refused exactly as a leaf with one dot is (see
        // `dotted_name_from_user_is_rejected`) — there is no threshold to cross.
        assert_eq!(
            gate(":my::Shape.Circle.Extra", NameOrigin::Declared, Privilege::User, Existing::Absent),
            Registration::DottedName
        );
    }

    #[test]
    fn composed_variant_under_reserved_prefix_from_user_is_still_reserved() {
        // A user still cannot define a variant under `:wat::*` — the composed origin
        // exempts only the DottedName wall; Reserved fires exactly as it would for a
        // declared name.
        assert_eq!(
            gate(":wat::telemetry::Numeric.I64", NameOrigin::ComposedVariant, Privilege::User, Existing::Absent),
            Registration::Reserved
        );
    }

    // ─── Arc 296 stone ③b-i, room ⑤a — the precondition on the door's own input ────

    #[test]
    fn register_variant_refuses_a_dotted_variant_leaf() {
        // `variant_leaf` itself carrying a `.` is refused BEFORE `gate` is ever
        // consulted — this is not a dot-count on the composed name (`gate`'s
        // `ComposedVariant` arm is untouched, see `composed_variant_dotted_leaf_inserts`
        // above), it is a precondition on the one thing `register_variant` receives from
        // its caller rather than composing itself. Without it, `compose_variant(":my::
        // Shape", "Circle.Baz")` → `:my::Shape::Circle.Baz` would insert — and once the
        // variant separator becomes `.`, that string decodes as enum `Shape.Circle`,
        // variant `Baz`, exactly the misdecode H-1 exists to prevent.
        let err = register_variant::<(), Rejection>(
            ":my::Shape",
            "Circle.Baz",
            Privilege::User,
            Existing::Absent,
            &crate::rust_caller_span!(),
            || Ok(()),
        )
        .unwrap_err();
        assert_eq!(err.verdict, Registration::DottedName);
        assert_eq!(err.name, ":my::Shape::Circle.Baz");
    }

    #[test]
    fn register_variant_dotted_leaf_precondition_does_not_block_an_equivalent_replay() {
        // Idempotent-before-every-wall: an equivalent re-declaration is never blocked,
        // even one shaped so it would fail the new precondition on first registration —
        // same ordering guarantee `dotted_name_idempotent_replay_still_noops` pins for
        // the `Declared` door.
        let result = register_variant::<(), Rejection>(
            ":my::Shape",
            "Circle.Baz",
            Privilege::User,
            Existing::Equivalent,
            &crate::rust_caller_span!(),
            || Ok(()),
        );
        assert!(result.is_ok(), "an equivalent replay must NoOp, not refuse: {result:?}");
    }
}
