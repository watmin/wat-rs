use crate::ast::WatAST;
use crate::span::Span;
use std::collections::HashMap;

use super::error::MacroError;

/// A registered macro.
#[derive(Debug, Clone)]
pub struct MacroDef {
    /// Full keyword-path of the macro (e.g. `:wat::holon::Subtract`).
    pub name: String,
    /// Fixed-arity parameter names in order. Positional binding.
    pub params: Vec<String>,
    /// Optional rest-parameter name. When present, the macro accepts
    /// `args.len() >= params.len()` at expansion; the first N args
    /// bind to `params` as usual, and the REMAINING args are bundled
    /// into a `WatAST::List` and bound to this name. A template's
    /// `,@rest-name` unquote-splicing then drops the list's elements
    /// into the surrounding form at expansion. Syntax at declaration
    /// (canonical Vector-of-triples form, per parse.rs:97-108):
    /// `(:wat::core::defmacro :name [p1 <- :T1 ... & rest <- :AST<...>] -> :AST<Ret> body)`.
    /// The `&` marker separates fixed params from the rest-binder.
    pub rest_param: Option<String>,
    /// The template — typically `(:wat::core::quasiquote ...)`.
    pub body: WatAST,
    /// Source span of the `(:wat::core::defmacro ...)` form that registered
    /// this macro. Used by register/register_stdlib to attribute MacroError
    /// emissions back to the user's source position.
    pub span: Span,
    /// Arc 278 — the RETAINED `(:wat::core::defmacro …)` form, verbatim.
    ///
    /// A `MacroDef` CANNOT reconstruct its own declaration: `params` holds
    /// names only, and the return type is not kept, so the canonical form
    /// `(defmacro :name [p <- :T … ] -> :AST<Ret> body)` is unrecoverable
    /// from the parts. Closure extraction must SHIP macros — the forms it
    /// sends a forked child still contain macro CALLS (every kwargs
    /// constructor), which expand on the far side — so it needs the form
    /// itself, not a rebuild.
    ///
    /// Same discipline `TypeEnv::source_form` already follows: ship the
    /// retained original, never a reconstruction. A rebuild from a
    /// description drops whatever the description does not model, and that
    /// is precisely how a synthesized record shipped without its
    /// constructor (`DESIGN-STONE-registry-kind-one-door.md`).
    ///
    /// ⚠ NOT part of `macro_structurally_equivalent` — two structurally
    /// identical macros registered from different sources must stay
    /// equivalent for the redef gate.
    pub source_form: WatAST,
}

/// Keyword-path ↦ `MacroDef` registry.
#[derive(Debug, Default, Clone)]
pub struct MacroRegistry {
    pub(super) macros: HashMap<String, MacroDef>,
    /// Expanded `defsurface` forms, keyed by the surface's colon-name
    /// (`:wat::queue::Queue`). Filled as each surface expands; `defservice` reads
    /// the matching form so wrap-or-not for `:max-entries` can walk `:features`
    /// at expand time. Per-registry (per freeze), never process-lifetime.
    pub(crate) surface_forms: HashMap<String, WatAST>,
}

impl MacroRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// ⛔ THE ONE DOOR every macro-head probe goes through — and therefore the one place the
    /// boot cache can WITNESS which names the stdlib expansion actually consulted. See
    /// `crate::freeze::boot_cache`'s header: a cached expansion is only reusable if no USER
    /// macro could have been consulted while it ran, and this is where that is measured rather
    /// than argued. The witness is `None` on every boot but the one that builds the snapshot,
    /// so the cost here is one thread-local read.
    pub fn contains(&self, name: &str) -> bool {
        crate::freeze::boot_cache::note_probe(name);
        self.macros.contains_key(name)
    }

    pub fn get(&self, name: &str) -> Option<&MacroDef> {
        crate::freeze::boot_cache::note_probe(name);
        self.macros.get(name)
    }

    /// Boot-cache door — the raw tables, for serialisation. `pub(crate)` and named for its one
    /// consumer so it cannot quietly become a general-purpose back door around `register`.
    pub(crate) fn cache_parts(&self) -> (&HashMap<String, MacroDef>, &HashMap<String, WatAST>) {
        (&self.macros, &self.surface_forms)
    }

    /// Boot-cache door — every registered name. Used to take the exact BEFORE/AFTER difference
    /// across user `defmacro` registration, so the names subtracted from the cached payload are
    /// the ones the user actually added and not an over-approximation of them.
    pub(crate) fn name_set(&self) -> std::collections::HashSet<String> {
        self.macros.keys().cloned().collect()
    }

    /// Boot-cache door — this registry minus `names` (and minus any `surface_forms` they keyed).
    /// The payload must hold the STDLIB world alone; the user's step-4 registrations are peeled
    /// back off here.
    pub(crate) fn without_names(&self, keep_out: &std::collections::HashSet<String>) -> Self {
        MacroRegistry {
            macros: self
                .macros
                .iter()
                .filter(|(k, _)| !keep_out.contains(*k))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            surface_forms: self
                .surface_forms
                .iter()
                .filter(|(k, _)| !keep_out.contains(*k))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
        }
    }

    /// Boot-cache door — rebuild from a decoded payload. Bypasses `register`'s gates on purpose:
    /// every entry here ALREADY passed them when the snapshot was derived, and the payload is
    /// keyed on the exact build that derived it.
    pub(crate) fn from_cache_parts(
        macros: HashMap<String, MacroDef>,
        surface_forms: HashMap<String, WatAST>,
    ) -> Self {
        MacroRegistry { macros, surface_forms }
    }

    /// Register a macro through the ONE gate (resolve::registration). `privilege` is
    /// threaded EXPLICITLY from the expand phase (Stdlib for the baked-stdlib pass, User
    /// for user source) — no ambient flag. Arc 054: a byte-equivalent re-registration is a
    /// no-op; a divergent one errors DuplicateMacro; a new reserved-prefix name from User
    /// errors ReservedPrefix.
    pub fn register(&mut self, def: MacroDef, privilege: crate::resolve::Privilege) -> Result<(), MacroError> {
        use crate::resolve::Existing;
        let existing = match self.macros.get(&def.name) {
            None => Existing::Absent,
            Some(e) if macro_structurally_equivalent(e, &def) => Existing::Equivalent,
            Some(_) => Existing::Divergent,
        };
        let name = def.name.clone();
        let span = def.span.clone();
        crate::resolve::register(&name, privilege, existing, &span, || -> Result<(), MacroError> {
            self.macros.insert(def.name.clone(), def);
            Ok(())
        })?;
        Ok(())
    }

}

/// Arc 054 — structural equivalence check for two `MacroDef` values.
///
/// Compares params + rest_param + body AST for structural equivalence,
/// span-agnostic. Ignores `name` (it's the registry key, identical by
/// construction).
fn macro_structurally_equivalent(a: &MacroDef, b: &MacroDef) -> bool {
    a.params == b.params && a.rest_param == b.rest_param && a.body == b.body
}
