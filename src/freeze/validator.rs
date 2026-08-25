//! Arc 294 item 9a follow-on — lift the `defrule` freeze wall's calling convention into a
//! **pluggable extension point**: `FreezeValidator`, an `inventory`-collected registration
//! any crate depending on `wat` can `inventory::submit!` into, mirroring the EXISTING pattern
//! (`RestrictionEntry` in `src/restriction_entry.rs`, `IntrinsicSubmission`/
//! `SpecialFormSubmission` in `src/intrinsic/mod.rs`, the `EdnSchema` drain in
//! `src/types.rs`).
//!
//! `src/rete/validate.rs`'s `validate_rete_rules` becomes the FIRST registered consumer — its
//! validation LOGIC is unchanged; only its caller (`src/freeze/env.rs`'s step 7.8) moves from
//! a hardcoded call to an `inventory::iter` drain, exactly like step 6.8's `RestrictionEntry`
//! drain in the SAME fn.
//!
//! See module docs on [`crate::restriction_entry::RestrictionEntry`] for the shape of the
//! wiring this mirrors (submit at module scope; gathered at link time; drained once, in
//! `build_env`, at a fixed pipeline step).

use crate::ast::WatAST;
use crate::runtime::SymbolTable;
use crate::types::TypeEnv;

/// A validator error: any type that is [`crate::edn::contract::ToEdn`] (so it can cross the wire
/// tagged with its OWN namespace — a rete error still tags `#wat.rete/…` through the box,
/// by dynamic dispatch) + `Debug` + `Display` (so `StartupError`'s own `Debug`/`Display`
/// keep working) + `Send + Sync` (an `inventory`-submitted `fn` pointer must be usable from
/// any thread that runs the freeze pipeline).
///
/// Blanket-implemented for every type that satisfies the bound — no validator crate needs to
/// write `impl FreezeValidatorError for MyError {}` by hand.
pub trait FreezeValidatorError:
    crate::edn::contract::ToEdn + std::fmt::Debug + std::fmt::Display + Send + Sync
{
}

impl<T: crate::edn::contract::ToEdn + std::fmt::Debug + std::fmt::Display + Send + Sync>
    FreezeValidatorError for T
{
}

/// One freeze-time validator registration. Mirrors [`crate::restriction_entry::RestrictionEntry`]:
/// a plain struct + `inventory::collect!` channel any dependent crate can `inventory::submit!`
/// into at module scope.
///
/// `validate` runs against the SAME `residue` + `types` + `symbols` the rete wall already
/// used (see `src/freeze/env.rs` step 7.8) — post-register, post-resolve, so a validator sees
/// fully-registered types and un-mangled quoted forms. It takes `residue` as `&mut` because
/// the rete wall's `:then` kwargs reorder REWRITES the quoted form in place; a validator that
/// only reads never needs the mutability, but the drain must offer it uniformly.
pub struct FreezeValidator {
    /// A short, human-legible name for the registration (e.g. `"wat.rete/defrule-wall"`) —
    /// diagnostic only, not dispatched on; the drain (`src/freeze/env.rs` step 7.8) never
    /// reads it back, only calls `validate`.
    #[allow(dead_code)] // diagnostic/introspection field, mirrors IntrinsicSubmission's `name`
    pub name: &'static str,
    /// The validation entry point. Returns `Ok(())` on a clean pass; `Err` carries a boxed
    /// [`FreezeValidatorError`] whose `to_edn()` preserves the concrete error's own tagged
    /// namespace (dynamic dispatch through the box — the box does NOT re-tag or generic-wrap
    /// the inner error).
    pub validate: FreezeValidateFn,
}

/// What a freeze-time validator returns: `Ok(())` on a clean pass, or a boxed
/// [`FreezeValidatorError`] whose `to_edn()` keeps the concrete error's own tagged
/// namespace (dynamic dispatch through the box — no re-tagging, no generic wrapper).
pub type FreezeValidateOutcome = Result<(), Box<dyn FreezeValidatorError>>;

/// The signature every freeze-time validator implements — the noun the nested type
/// was hiding, named so `FreezeValidator`'s field reads as a contract rather than a
/// four-deep type expression.
///
/// The three inputs are the SAME `residue` / `types` / `symbols` the rete wall
/// already ran against (`src/freeze/env.rs` step 7.8) — post-register, post-resolve,
/// so a validator sees fully-registered types and un-mangled quoted forms. `residue`
/// is `&mut` because the rete wall's `:then` kwargs reorder rewrites the quoted form
/// in place; a read-only validator never needs it, but the drain offers it uniformly.
pub type FreezeValidateFn = fn(&mut Vec<WatAST>, &TypeEnv, &SymbolTable) -> FreezeValidateOutcome;

inventory::collect!(FreezeValidator);
