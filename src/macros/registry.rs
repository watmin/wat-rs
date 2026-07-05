use crate::ast::WatAST;
use crate::span::Span;
use std::collections::HashMap;

use super::error::{MacroError, MacroErrorKind};

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
}

/// Keyword-path ↦ `MacroDef` registry.
#[derive(Debug, Default, Clone)]
pub struct MacroRegistry {
    pub(super) macros: HashMap<String, MacroDef>,
    /// When true, [`register`](Self::register) bypasses the reserved-prefix gate.
    /// Set only while expanding the BAKED STDLIB (see `freeze/env.rs`), so a
    /// `:wat::` macro-generating-macro's expansion-born companion (e.g. a
    /// `defservice`'s `…/start`) registers with the same privilege the literal
    /// top-level path ([`register_stdlib`](Self::register_stdlib)) already grants.
    /// Cleared for user expansion, so mis-namespaced user macros still halt.
    /// Defaults false (unprivileged) — existing behavior is untouched unless set.
    stdlib_privilege: bool,
}

impl MacroRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set whether [`register`](Self::register) is privileged (bypasses the
    /// reserved-prefix gate). The freeze pipeline sets this true around stdlib
    /// expansion and false for user expansion.
    pub fn set_stdlib_privilege(&mut self, on: bool) {
        self.stdlib_privilege = on;
    }

    pub fn contains(&self, name: &str) -> bool {
        self.macros.contains_key(name)
    }

    pub fn get(&self, name: &str) -> Option<&MacroDef> {
        self.macros.get(name)
    }

    /// Register a macro. Errors on duplicate or reserved prefix.
    ///
    /// Arc 054: byte-equivalent re-registration is a no-op; divergent
    /// re-registration remains an error. Two `defmacro` forms with
    /// matching params + rest_param + body AST count as equivalent.
    pub fn register(&mut self, def: MacroDef) -> Result<(), MacroError> {
        if !self.stdlib_privilege && crate::resolve::is_reserved_prefix(&def.name) {
            return Err(MacroError { span: def.span.clone(), kind: MacroErrorKind::ReservedPrefix(def.name) });
        }
        if let Some(existing) = self.macros.get(&def.name) {
            if macro_structurally_equivalent(existing, &def) {
                return Ok(());
            }
            return Err(MacroError { span: def.span.clone(), kind: MacroErrorKind::DuplicateMacro(def.name) });
        }
        self.macros.insert(def.name.clone(), def);
        Ok(())
    }

    /// Register a TRUSTED stdlib macro. Bypasses the reserved-prefix
    /// gate because stdlib forms live under `:wat::std::*` by design.
    /// Still errors on duplicates. Intended for the baked stdlib
    /// loader; user source paths through `register` where the prefix
    /// check catches mis-namespaced user defmacros.
    ///
    /// Arc 054: idempotent re-declaration applies — byte-equivalent
    /// re-registration is a no-op.
    pub(super) fn register_stdlib(&mut self, def: MacroDef) -> Result<(), MacroError> {
        if let Some(existing) = self.macros.get(&def.name) {
            if macro_structurally_equivalent(existing, &def) {
                return Ok(());
            }
            return Err(MacroError { span: def.span.clone(), kind: MacroErrorKind::DuplicateMacro(def.name) });
        }
        self.macros.insert(def.name.clone(), def);
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
