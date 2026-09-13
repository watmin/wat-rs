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
}

impl MacroRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.macros.contains_key(name)
    }

    pub fn get(&self, name: &str) -> Option<&MacroDef> {
        self.macros.get(name)
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
