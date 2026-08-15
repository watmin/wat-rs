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
    name.contains("::")
}

/// Arc 296 stone H-1 — true if the NAME half (the segment after the LAST `::`) contains
/// a `.`. The namespace half is untouched: `:wat::core::Fault` has namespace `wat::core`
/// and name `Fault`; only `Fault` is checked. Works whether or not `name` carries the
/// leading `:` (parametric heads drop it — arc 170's 24t seam), because `rsplit("::")`
/// finds the last segment either way. At registration the name is always `::`-separated
/// (dots appear later, only in the wire tag built by `tag_from_type_path`), so this is a
/// pure ban, not a parse of an already-dotted form.
fn has_dotted_name(name: &str) -> bool {
    name.rsplit("::").next().unwrap_or(name).contains('.')
}

/// THE gate. The rule + ordering, once:
///
/// ```text
///   Existing::Equivalent                        -> NoOp       (benign re-declaration)
///   Existing::Divergent                         -> Duplicate
///   Absent + !namespaced                         -> Unnamespaced
///   Absent + namespaced + dotted name             -> DottedName
///   Absent + namespaced + undotted + reserved + Privilege::User -> Reserved
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
pub fn gate(name: &str, privilege: Privilege, existing: Existing) -> Registration {
    match existing {
        Existing::Equivalent => Registration::NoOp,
        Existing::Divergent => Registration::Duplicate,
        Existing::Absent => {
            if !is_namespaced(name) {
                Registration::Unnamespaced
            } else if has_dotted_name(name) {
                Registration::DottedName
            } else if privilege == Privilege::User && is_reserved_prefix(name) {
                Registration::Reserved
            } else {
                Registration::Insert
            }
        }
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
        assert_eq!(gate(":wat::query::Store", Privilege::User, Existing::Equivalent), Registration::NoOp);
        assert_eq!(gate(":wat::query::Store", Privilege::Stdlib, Existing::Equivalent), Registration::NoOp);
        assert_eq!(gate(":rust::sqlite::Db", Privilege::User, Existing::Equivalent), Registration::NoOp);
        assert_eq!(gate(":my::Thing", Privilege::User, Existing::Equivalent), Registration::NoOp);
    }

    #[test]
    fn divergent_redeclaration_is_duplicate_regardless_of_privilege() {
        assert_eq!(gate(":wat::query::Store", Privilege::Stdlib, Existing::Divergent), Registration::Duplicate);
        assert_eq!(gate(":wat::query::Store", Privilege::User, Existing::Divergent), Registration::Duplicate);
        assert_eq!(gate(":my::Thing", Privilege::User, Existing::Divergent), Registration::Duplicate);
    }

    #[test]
    fn new_reserved_name_from_user_is_rejected() {
        assert_eq!(gate(":wat::query::Store", Privilege::User, Existing::Absent), Registration::Reserved);
        assert_eq!(gate(":rust::sqlite::Db", Privilege::User, Existing::Absent), Registration::Reserved);
    }

    #[test]
    fn new_reserved_name_from_stdlib_inserts() {
        assert_eq!(gate(":wat::query::Store", Privilege::Stdlib, Existing::Absent), Registration::Insert);
        assert_eq!(gate(":rust::sqlite::Db", Privilege::Stdlib, Existing::Absent), Registration::Insert);
    }

    #[test]
    fn new_user_name_inserts_under_either_privilege() {
        assert_eq!(gate(":my::Thing", Privilege::User, Existing::Absent), Registration::Insert);
        assert_eq!(gate(":my::Thing", Privilege::Stdlib, Existing::Absent), Registration::Insert);
    }

    #[test]
    fn bare_name_from_user_is_unnamespaced() {
        assert_eq!(gate(":no-ns", Privilege::User, Existing::Absent), Registration::Unnamespaced);
    }

    #[test]
    fn namespaced_user_name_inserts() {
        assert_eq!(gate(":my::ok", Privilege::User, Existing::Absent), Registration::Insert);
    }

    #[test]
    fn parametric_head_without_leading_colon_is_namespaced() {
        assert_eq!(gate("wat::kernel::Peer", Privilege::Stdlib, Existing::Absent), Registration::Insert);
    }

    #[test]
    fn bare_name_from_stdlib_is_still_unnamespaced() {
        // No privilege escape from the namespacing wall.
        assert_eq!(gate(":no-ns", Privilege::Stdlib, Existing::Absent), Registration::Unnamespaced);
    }

    #[test]
    fn bare_name_idempotent_replay_still_noops() {
        assert_eq!(gate(":no-ns", Privilege::User, Existing::Equivalent), Registration::NoOp);
    }

    // ─── Arc 296 stone H-1 — the dot wall ──────────────────────────────

    #[test]
    fn dotted_name_from_user_is_rejected() {
        assert_eq!(gate(":my::Shape.Circle", Privilege::User, Existing::Absent), Registration::DottedName);
    }

    #[test]
    fn dotted_name_from_stdlib_is_still_rejected() {
        // No privilege escape from the dot wall, same as Unnamespaced.
        assert_eq!(gate(":wat::telemetry::Numeric.I64", Privilege::Stdlib, Existing::Absent), Registration::DottedName);
    }

    #[test]
    fn dot_in_namespace_half_is_untouched() {
        // Only the segment AFTER the last `::` is checked; a dot earlier in the path
        // (however unlikely) does not trip the wall.
        assert_eq!(gate(":my::v1.2::Thing", Privilege::User, Existing::Absent), Registration::Insert);
    }

    #[test]
    fn dotted_name_idempotent_replay_still_noops() {
        // A benign equivalent re-declaration is never blocked, even for a name that
        // would fail the dot wall on first registration — same ordering guarantee as
        // Unnamespaced/Reserved.
        assert_eq!(gate(":my::Shape.Circle", Privilege::User, Existing::Equivalent), Registration::NoOp);
    }
}
