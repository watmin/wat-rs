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
}

/// THE gate. The rule + ordering, once:
///
/// ```text
///   Existing::Equivalent                        -> NoOp       (benign re-declaration)
///   Existing::Divergent                         -> Duplicate
///   Absent + reserved + Privilege::User          -> Reserved
///   Absent + (Privilege::Stdlib | !reserved)     -> Insert
/// ```
///
/// Idempotent-BEFORE-reserved is the ordering that fixes "you cannot declare an existing
/// form": an equivalent re-declaration is a `NoOp` even for a reserved name from
/// unprivileged source, because it grants nothing (the name already resolves to the same
/// definition). A DIVERGENT re-declaration still errors (`Duplicate`), and a genuinely
/// NEW reserved name from `User` source is still rejected (`Reserved`) — the gate's
/// purpose is fully preserved.
pub fn gate(name: &str, privilege: Privilege, existing: Existing) -> Registration {
    match existing {
        Existing::Equivalent => Registration::NoOp,
        Existing::Divergent => Registration::Duplicate,
        Existing::Absent => {
            if privilege == Privilege::User && is_reserved_prefix(name) {
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
}
