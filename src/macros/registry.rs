use crate::ast::WatAST;
use crate::name_map::NameMap;
use crate::span::Span;
use std::collections::{HashMap, HashSet};

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
    pub(super) macros: NameMap<MacroDef>,
    /// Clojure symbol spelling → the keyword key in `macros`. Not a second
    /// `MacroDef`: the body stays in one entry. Absent when both joins are
    /// registered, so the symbol is not forced onto one of them.
    symbol_alias: HashMap<String, String>,
    /// Spellings where both joins are registered. A call of one of these
    /// still asks `reconstruct_call_path`. Every other clojure spelling is
    /// either the alias above or not a macro.
    ambiguous_symbols: HashSet<String>,
}

impl MacroRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.macros.contains_key(name) || self.symbol_alias.contains_key(name)
    }

    pub fn get(&self, name: &str) -> Option<&MacroDef> {
        if let Some(def) = self.macros.get(name) {
            return Some(def);
        }
        self.symbol_alias.get(name).and_then(|key| self.macros.get(key))
    }

    pub fn get_name(&self, name: &crate::scope::Name) -> Option<&MacroDef> {
        self.macros.get_name(name)
    }

    pub fn len(&self) -> usize {
        self.macros.len()
    }

    pub fn is_empty(&self) -> bool {
        self.macros.is_empty()
    }

    /// Spellings `Name::enter` refused. Measurement for stone 255.92.
    #[cfg(test)]
    pub(crate) fn rendered_key_report(&self) -> Vec<String> {
        let mut out: Vec<String> = self.macros.rendered_spellings().cloned().collect();
        out.sort();
        out
    }

    /// Register a macro through the ONE gate (resolve::registration). `privilege` is
    /// threaded EXPLICITLY from the expand phase (Stdlib for the baked-stdlib pass, User
    /// for user source) — no ambient flag. Arc 054: a byte-equivalent re-registration is a
    /// no-op; a divergent one errors DuplicateMacro; a new reserved-prefix name from User
    /// errors ReservedPrefix.
    pub fn register(
        &mut self,
        def: MacroDef,
        privilege: crate::resolve::Privilege,
    ) -> Result<(), MacroError> {
        use crate::resolve::Existing;
        let existing = match self.macros.get(&def.name) {
            None => Existing::Absent,
            Some(e) if macro_structurally_equivalent(e, &def) => Existing::Equivalent,
            Some(_) => Existing::Divergent,
        };
        let name = def.name.clone();
        let span = def.span.clone();
        crate::resolve::register(
            &name,
            privilege,
            existing,
            &span,
            || -> Result<(), MacroError> {
                self.macros.insert(def.name.clone(), def);
                Ok(())
            },
        )?;
        self.install_symbol_alias(&name);
        Ok(())
    }

    /// Index the clojure symbol beside the keyword key so a converted call head
    /// is one hash lookup. Both joins already registered (`:Type/method` and
    /// `:Type::method`) drop the alias: the symbol would name two macros, and
    /// the call still resolves through `reconstruct_call_path`.
    fn install_symbol_alias(&mut self, primary: &str) {
        let Some(alias) = crate::edn::render::wat_keyword_to_clojure_symbol(primary) else {
            return;
        };
        if alias == primary {
            return;
        }
        if crate::scope::Name::enter(primary) == crate::scope::Name::enter(&alias) {
            return;
        }
        if let Some(other) = crate::types::other_join_spelling(primary) {
            if other != primary
                && crate::scope::Name::enter(&other) != crate::scope::Name::enter(primary)
                && self.macros.contains_key(&other)
            {
                self.symbol_alias.remove(&alias);
                self.ambiguous_symbols.insert(alias);
                return;
            }
        }
        if self.macros.contains_key(&alias) || self.symbol_alias.contains_key(&alias) {
            return;
        }
        if self.macros.contains_key(primary) {
            self.ambiguous_symbols.remove(&alias);
            self.symbol_alias.insert(alias, primary.to_string());
        }
    }

    /// Both joins are macros under this clojure spelling, so expand still
    /// reconstructs. A miss here, on a `ns/name` symbol, is not a macro.
    pub(crate) fn symbol_join_ambiguous(&self, spelling: &str) -> bool {
        self.ambiguous_symbols.contains(spelling)
    }

    /// 2a4c — the stdlib-mode door's private copy only. A divergent
    /// re-declaration of a snapshot macro is retracted so a subsequent
    /// `register_stdlib_defmacros` sees `Existing::Absent`. Equivalent is
    /// left in place (register is a no-op).
    pub(crate) fn retract_if_divergent(&mut self, def: &MacroDef) {
        match self.macros.get(&def.name) {
            Some(e) if !macro_structurally_equivalent(e, def) => {
                let alias = crate::edn::render::wat_keyword_to_clojure_symbol(&def.name);
                if let Some(alias) = &alias {
                    self.symbol_alias.remove(alias);
                }
                let other = crate::types::other_join_spelling(&def.name);
                self.macros.remove(&def.name);
                if let Some(other) = other {
                    if self.macros.contains_key(&other) {
                        self.install_symbol_alias(&other);
                    } else if let Some(alias) = alias {
                        self.ambiguous_symbols.remove(&alias);
                    }
                } else if let Some(alias) = alias {
                    self.ambiguous_symbols.remove(&alias);
                }
            }
            _ => {}
        }
    }
}

/// Arc 054 — structural equivalence check for two `MacroDef` values.
///
/// Compares params + rest_param + body AST for structural equivalence,
/// span-agnostic. Ignores `name` (it's the registry key, identical by
/// construction).
///
/// A reference symbol and the keyword of its identity are the same node.
/// `register_aggregate_kwargs_companions` bakes the keyword spelling before
/// expand; a converted `defrecord` then emits the symbol spelling of that
/// same companion. Byte equality called that a second macro. Identity does
/// not. The keyword/symbol cross arm compares `Name::from_keyword` with the
/// identifier's `Name`. Keyword against keyword still goes through
/// `canonical_identity`. Symbol against symbol is pair equality.
fn macro_structurally_equivalent(a: &MacroDef, b: &MacroDef) -> bool {
    a.params == b.params && a.rest_param == b.rest_param && ast_same_identity(&a.body, &b.body)
}

fn ast_same_identity(a: &crate::ast::WatAST, b: &crate::ast::WatAST) -> bool {
    use crate::ast::WatAST;
    match (a, b) {
        (WatAST::Keyword(ka, _), WatAST::Keyword(kb, _)) => {
            crate::edn::render::canonical_identity(ka) == crate::edn::render::canonical_identity(kb)
        }
        (WatAST::Symbol(sa, _), WatAST::Symbol(sb, _)) => sa == sb,
        (WatAST::Keyword(k, _), WatAST::Symbol(id, _))
        | (WatAST::Symbol(id, _), WatAST::Keyword(k, _)) => {
            id.is_reference()
                && crate::scope::Name::from_keyword(k).as_ref() == Some(id.pair())
        }
        (WatAST::List(xs, _), WatAST::List(ys, _))
        | (WatAST::Vector(xs, _), WatAST::Vector(ys, _))
        | (WatAST::Set(xs, _), WatAST::Set(ys, _)) => {
            xs.len() == ys.len() && xs.iter().zip(ys).all(|(x, y)| ast_same_identity(x, y))
        }
        (WatAST::Map(xs, _), WatAST::Map(ys, _)) => {
            xs.len() == ys.len()
                && xs.iter().zip(ys).all(|((ak, av), (bk, bv))| {
                    ast_same_identity(ak, bk) && ast_same_identity(av, bv)
                })
        }
        _ => a == b,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::WatAST;
    use crate::scope::Identifier;

    fn sym(s: &str) -> WatAST {
        WatAST::Symbol(Identifier::bare(s), crate::rust_caller_span!())
    }
    fn kw(s: &str) -> WatAST {
        WatAST::Keyword(s.to_string(), crate::rust_caller_span!())
    }

    /// Stone 255.12 — the POSITIVE control for the cross-spelling arm.
    /// A reference symbol and the keyword of its identity ARE the same node, and
    /// they already were before 255.12.
    #[test]
    fn a_reference_symbol_and_its_keyword_are_one_node() {
        assert!(ast_same_identity(&sym("wat.core/mapv"), &kw(":wat::core::mapv")));
        assert!(ast_same_identity(
            &WatAST::List(vec![sym("wat.i64/+"), sym("wat.core/x")], crate::rust_caller_span!()),
            &WatAST::List(vec![kw(":wat::i64::+"), kw(":wat::core::x")], crate::rust_caller_span!()),
        ));
    }

    /// ⭐ Stone 255.12 — the MEASURED REFUTATION of the brief's site-2 lead.
    ///
    /// The lead: `wat/holon/Ngram.wat` fails to re-register under conversion
    /// because this arm requires `id.is_reference()`, which `<-` and `->` are
    /// not. The `is_reference()` observation is TRUE and IRRELEVANT — the arm
    /// compares `canonical_identity` of both sides, and `<-`, `->` and `:-` are
    /// three DIFFERENT identities. Relaxing `is_reference()` could not have
    /// equated them; nothing in the identity family can.
    ///
    /// What the codemod does to a quasiquoted template is a GRAMMAR flip
    /// (`<-`/`->` → `:-`), not a namespace re-spelling, so site 2 was REPORTED
    /// rather than cured. See `tests/macros/probe_arc255_12_macro_*` for the
    /// behavioural half.
    #[test]
    fn the_annotation_markers_are_three_identities_not_one_name() {
        use crate::edn::render::canonical_identity;
        assert_eq!(canonical_identity("<-"), "<-");
        assert_eq!(canonical_identity("->"), "->");
        assert_eq!(canonical_identity(":-"), ":-");
        assert!(
            !ast_same_identity(&sym("<-"), &kw(":-")),
            "⛔ if the arrow ever equated to the annotation keyword, two macro \
             templates that emit DIFFERENT surface forms would count as one macro"
        );
        assert!(
            !ast_same_identity(&sym("<-"), &sym("->")),
            "⛔ the codemod folds BOTH arrows onto `:-`; equating them would make a \
             parameter annotation and a return annotation the same node"
        );
    }

    /// Stone 255.12 — and why `is_reference()` must STAY. A let-binder symbol
    /// `v` is not a reference, so it must not equate to the keyword `:v`.
    /// Relaxing the guard — the cure the brief proposed for site 2 — would
    /// collapse them.
    #[test]
    fn a_binder_is_not_the_keyword_of_its_name() {
        assert!(!ast_same_identity(&sym("v"), &kw(":v")));
    }
}
