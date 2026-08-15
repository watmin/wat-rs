//! Function and Environment cluster — Function, Environment, EnvBuilder, BoundEntry, EnvCell.
//!
//! Moved from `src/runtime.rs` (block 1409–1581) in Stone 251.2c.
//! Co-located because Function carries `closed_env: Option<Environment>`.

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use crate::ast::WatAST;
use crate::span::Span;
use crate::types::TypeExpr;
use crate::value::{TrackedValue, Provenance};

/// Stone 255.1a — body representation for a `Function`.
///
/// Every user-defined function today is `Wat(body)`. `Native` is a unit
/// marker reserved for Rust-implemented builtins registered into `sym` in
/// arc 255.1b+. Nothing constructs `Native` in this slice; execution of a
/// `Native` body via fn-apply is unreachable because the runtime dispatch
/// `match` intercepts builtin names before reaching fn-apply.
#[derive(Clone, Debug)]
pub enum FunctionBody {
    /// A wat-defined function body. All functions today.
    Wat(Arc<WatAST>),
    /// A Rust-implemented builtin. Unit marker — no handler stored here;
    /// execution is via the runtime dispatch `match`, not fn-apply.
    /// Used starting in arc 255.1b; nothing constructs this in slice 255.1a.
    Native,
}

/// Arc 278 #88 — the rete contract a `(:wat::rete::core::defn …)` declaration attests, checked
/// AT THE DEFINITION SITE against the four axes `crate::rete::purity` already measures (Pure ∧
/// Deterministic ∧ Total ∧ Law A — reusing those walks, never a second implementation; STOP-1).
///
/// `#87` hangs the bound (`depth` / `nodes` / `fold_nesting`) here — adding a field to THIS
/// struct costs nothing, where a second field on [`Function`] would re-cascade across every
/// construction site a second time. Empty today; `Some(ReteContract {})` is itself the marker.
#[derive(Clone, Debug, Default)]
pub struct ReteContract {}

/// A callable. `define`-registered functions have `name = Some(path)`
/// and `closed_env = None` (they resolve symbols via the global
/// [`SymbolTable`] at call time). `fn` values have `name = None`
/// and carry their `closed_env` from the creation site.
#[derive(Clone)]
pub struct Function {
    pub name: Option<String>,
    /// Parameter binders, as IDENTIFIERS — name plus hygiene scope set.
    ///
    /// This field held `Vec<String>` (the flattened `env_key`) until arc 170.
    /// That flattening baked a scope id into a NAME — `"kwargs\u{1}952"` — and
    /// anything rebuilding a binder from it (`closure_extract`'s
    /// `Identifier::bare(param)`) produced exactly what
    /// `HygieneScopeDivergence`'s own remedy warns against: "a macro rebuilt
    /// this binder from its name instead of reusing the node."
    ///
    /// It was illegal on its face — `Identifier::bare` debug-asserts that a name
    /// contains no U+0001 — and a debug run of `probe_arc170_gapj_each_kwargs`
    /// panicked on it. Release hid it, because the flattened key happens to
    /// EQUAL the scoped identifier's `env_key`, so resolution matched by
    /// accident.
    ///
    /// It stopped being an accident the moment a program had to cross a process
    /// boundary: an exec'd child restarts `fresh_scope()` at 1, so imported
    /// scopes must be REMAPPED — and a scope inside a string cannot be. Carrying
    /// the `Identifier` means the binder is REUSED, which is what the diagnostic
    /// asked for all along. Call `env_key` at the point of use for a key.
    pub params: Vec<crate::scope::Identifier>,
    /// Declared type-parameter list from the function name keyword
    /// (e.g., `<T,U>` on `:my::ns::foo<T,U>`). Empty for monomorphic
    /// functions. Names appearing in `param_types` / `ret_type` that
    /// match an entry here are treated as type variables at check
    /// time.
    /// **TRANSFORMS (clojure-ination):** scheme/type fields
    pub type_params: Vec<String>,
    /// Declared parameter types, parallel to `params`. Populated from
    /// the `(:wat::core::defn :name [p1 <- :T1 ...] -> :Ret body)` signature.
    /// Stone 241.16 — `parse_define_form` DELETED; defn uses `parse_defn_signature`.
    /// Used by the type checker for call-site unification and body-vs-signature
    /// checks. Empty only for fn values (type-untracked).
    pub param_types: Vec<TypeExpr>,
    /// Declared return type. `:()` (unit) if the signature omitted a
    /// return type. For fns, `:()` — the checker treats fn
    /// values as opaque function values in slice 7b.
    pub ret_type: TypeExpr,
    /// Arc 150 — optional rest-parameter name. When present, the
    /// function accepts `args.len() >= params.len()` at apply time;
    /// the first N args bind positionally to `params`, and the
    /// REMAINING args are wrapped in a `Value::Vec(Arc::new(rest))`
    /// and bound to this name. Mirrors `MacroDef.rest_param`. Syntax
    /// at declaration (Stone 241.16 — defn form):
    /// `(:wat::core::defn :name [p1 <- :T1 ... & xs <- :Vector<R>] -> :Ret body)`.
    /// `None` for strict-arity defns and for all fns.
    pub rest_param: Option<String>,
    /// Arc 150 — declared type of the rest-parameter. Always
    /// `Some(TypeExpr::Parametric { head: "Vec", args: [T] })` when
    /// `rest_param.is_some()`; `None` otherwise. The element type T
    /// is what each rest-arg must unify against at call sites.
    pub rest_param_type: Option<TypeExpr>,
    /// Stone 255.1a — see [`FunctionBody`].
    pub body: FunctionBody,
    pub closed_env: Option<Environment>,
    /// `Some` iff declared by `:wat::rete::core::defn` (arc 278 #88) and its body PROVED, once,
    /// at that declaration, against all four rete axes. `head_ok` (`src/rete/purity.rs`)
    /// consults this INSTEAD of walking the body — that substitution is the membrane. `None` for
    /// every ordinary `:wat::core::defn` / `fn` value; a `None` fn is refused inside a `where`
    /// regardless of how clean its body happens to be — the declaration IS the contract now, not
    /// an accident of what the body contains.
    pub rete: Option<ReteContract>,
    /// Arc 198 strike 2 (BRIEF-198-companion-propagation-A1-B2) — `Some(T)` iff this fn is a
    /// RUNTIME-SYNTHESIZED companion minted for aggregate type `T` (the positional prime ctor
    /// `T'` at `register_struct_methods`'s ctor mint, or the membership predicate `is-T?` at its
    /// mint site), where `T` is the type's FQDN exactly as it appears in `binding_metadata`
    /// (e.g. `":my::Token"`). `None` for every ordinary user/macro-authored fn.
    ///
    /// Consulted ONLY by `walk_for_restricted_call` (`src/check.rs`) — B2's exemption: a
    /// synthesized companion may MENTION the type it was generated for (its body necessarily
    /// does — `T'` passes `T` to `aggregate-new`; `is-T?` passes `T` to `conforms?`) without
    /// tripping `T`'s own `:restricted-to` whitelist. This is an OWNER-SCOPED exemption, not a
    /// blanket one: a mention of any OTHER restricted binding inside a synthesized body is still
    /// walked and still refused — `walk_for_restricted_call` only skips the emission when the
    /// mentioned keyword equals `synthesized_for`. Ruled out: B1 (append the companion's own FQDN
    /// into T's `:restricted-to` — fails Honest, the diagnostic would quote entries the author
    /// never wrote); B3 (exempt by name pattern — a FORGERY, a user fn literally named `:my::Token'`
    /// would inherit it); B4 (skip synthesized bodies entirely — exempts generated code from EVERY
    /// restriction, not just its own type's).
    pub synthesized_for: Option<String>,
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Function")
            .field("name", &self.name)
            .field("params", &self.params)
            .field("rest_param", &self.rest_param)
            .field(
                "closed_env",
                &if self.closed_env.is_some() { "<env>" } else { "<none>" },
            )
            .finish()
    }
}

/// Lexical-scope chain.
#[derive(Clone)]
pub struct Environment {
    inner: Arc<EnvCell>,
}

/// Arc 233 Stone 233.2.e: named struct carrying TrackedValue + binding_span.
/// binding_span is the source position of the LHS name in the let binder
/// (e.g., position of `x` in `(let [x 42] ...)`). Used by env.lookup to
/// construct SymbolBound provenance at the lookup boundary.
pub struct BoundEntry {
    pub value: TrackedValue,
    pub binding_span: Span,
}

struct EnvCell {
    bindings: HashMap<String, BoundEntry>,
    parent: Option<Environment>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            inner: Arc::new(EnvCell {
                bindings: HashMap::new(),
                parent: None,
            }),
        }
    }

    pub fn child(&self) -> EnvBuilder {
        EnvBuilder {
            bindings: HashMap::new(),
            parent: Some(self.clone()),
        }
    }

    /// Look up a name in the environment, constructing SymbolBound provenance
    /// at the lookup boundary for Unknown/Literal provenance values
    /// (Arc 233 Stone 233.2.e).
    ///
    /// `head_span` is the source position where the symbol appears in the call
    /// (e.g., position of `x` in `(some-fn x)`). The returned TrackedValue
    /// carries SymbolBound { binding_span: <where x was bound>, head_span: <where x is used> }
    /// when the stored value has Unknown or Literal provenance.
    ///
    /// Provenance replacement rule (honest reconciliation of Decision 2 with
    /// Stone 233.2.k's regression guard):
    /// - Unknown → SymbolBound (no prior context; binding IS the context)
    /// - Literal → SymbolBound (literal's source position is embedded in the
    ///   binding span; the symbol reference is the diagnostic context)
    /// - RuntimeBuilt → keep RuntimeBuilt (producer context is more informative
    ///   than the binding coordinates for errors on producer-built values)
    /// - SymbolBound → replace with new SymbolBound (update binding/head context)
    pub fn lookup(&self, name: &str, head_span: &Span) -> Option<TrackedValue> {
        if let Some(entry) = self.inner.bindings.get(name) {
            let value = entry.value.value().clone();
            let provenance = match entry.value.provenance().clone() {
                Provenance::RuntimeBuilt { producer, call_span } => {
                    // RuntimeBuilt: keep producer provenance. The producer context is
                    // more informative than binding coordinates for diagnostic errors.
                    Provenance::RuntimeBuilt { producer, call_span }
                }
                _ => {
                    // Unknown / Literal / SymbolBound: replace with SymbolBound.
                    // The binding coordinates (where the name was defined +
                    // where it is used) are the useful diagnostic context.
                    Provenance::SymbolBound {
                        binding_span: entry.binding_span.clone(),
                        head_span: head_span.clone(),
                    }
                }
            };
            return Some(TrackedValue::new(value, provenance));
        }
        self.inner.parent.as_ref().and_then(|p| p.lookup(name, head_span))
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder that accumulates bindings, then freezes into an [`Environment`].
pub struct EnvBuilder {
    bindings: HashMap<String, BoundEntry>,
    parent: Option<Environment>,
}

impl EnvBuilder {
    /// Bind a name to a TrackedValue with its binding_span (Arc 233 Stone 233.2.e).
    /// The binding_span is the source position of the LHS name in the let binder;
    /// env.lookup uses it to construct SymbolBound provenance at lookup time.
    pub fn bind(mut self, name: impl Into<String>, binding_span: Span, tv: TrackedValue) -> Self {
        self.bindings.insert(name.into(), BoundEntry { value: tv, binding_span });
        self
    }

    /// Bind a name without a meaningful source span (Unknown binding_span).
    /// Used by sites that bind values without let-binder source coordinates
    /// (e.g., function argument binding, matches? pattern binding).
    /// These sites get Provenance::Unknown when the value is looked up.
    pub fn bind_unknown_span(mut self, name: impl Into<String>, tv: TrackedValue) -> Self {
        self.bindings.insert(name.into(), BoundEntry { value: tv, binding_span: crate::rust_caller_span!() });
        self
    }

    pub fn build(self) -> Environment {
        Environment {
            inner: Arc::new(EnvCell {
                bindings: self.bindings,
                parent: self.parent,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::WatAST;
    use crate::types::TypeExpr;

    fn nil_body() -> FunctionBody {
        FunctionBody::Wat(Arc::new(WatAST::nil()))
    }

    /// Lines 57-67: `Debug` impl for `Function` — named fields with selective
    /// display (closed_env shown as literal string, not full env dump).
    #[test]
    fn debug_function_no_closed_env() {
        let f = Function {
            name: Some(":my::fn".into()),
            params: vec![crate::scope::Identifier::bare("x")],
            type_params: vec![],
            param_types: vec![TypeExpr::Path(":wat::core::i64".into())],
            ret_type: TypeExpr::Path(":wat::core::i64".into()),
            rest_param: None,
            rest_param_type: None,
            body: nil_body(),
            closed_env: None,
            rete: None,
            synthesized_for: None,
        };
        let dbg = format!("{:?}", f);
        assert_eq!(
            dbg,
            // Arc 170 — `params` carries `Identifier`, not `String`, so Debug now
            // shows the hygiene scope set. That is the point: a binder's scopes
            // are part of its identity, and this rendering says so.
            r#"Function { name: Some(":my::fn"), params: [Identifier { name: "x", scopes: {} }], rest_param: None, closed_env: "<none>" }"#,
            "Debug output mismatch"
        );
    }

    #[test]
    fn debug_function_with_closed_env() {
        let env = Environment::new();
        let f = Function {
            name: None,
            params: vec![],
            type_params: vec![],
            param_types: vec![],
            ret_type: TypeExpr::Path(":wat::core::nil".into()),
            rest_param: None,
            rest_param_type: None,
            body: nil_body(),
            closed_env: Some(env),
            rete: None,
            synthesized_for: None,
        };
        let dbg = format!("{:?}", f);
        assert_eq!(
            dbg,
            r#"Function { name: None, params: [], rest_param: None, closed_env: "<env>" }"#,
            "Debug output mismatch"
        );
    }
}
