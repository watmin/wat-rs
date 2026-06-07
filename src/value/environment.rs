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

/// A callable. `define`-registered functions have `name = Some(path)`
/// and `closed_env = None` (they resolve symbols via the global
/// [`SymbolTable`] at call time). `fn` values have `name = None`
/// and carry their `closed_env` from the creation site.
pub struct Function {
    pub name: Option<String>,
    pub params: Vec<String>,
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
        self.bindings.insert(name.into(), BoundEntry { value: tv, binding_span: Span::unknown() });
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
    use crate::span::Span;
    use crate::types::TypeExpr;

    fn nil_body() -> FunctionBody {
        FunctionBody::Wat(Arc::new(WatAST::Keyword(":wat::core::nil".into(), Span::unknown())))
    }

    /// Lines 57-67: `Debug` impl for `Function` — named fields with selective
    /// display (closed_env shown as literal string, not full env dump).
    #[test]
    fn debug_function_no_closed_env() {
        let f = Function {
            name: Some(":my::fn".into()),
            params: vec!["x".into()],
            type_params: vec![],
            param_types: vec![TypeExpr::Path(":wat::core::i64".into())],
            ret_type: TypeExpr::Path(":wat::core::i64".into()),
            rest_param: None,
            rest_param_type: None,
            body: nil_body(),
            closed_env: None,
        };
        let dbg = format!("{:?}", f);
        assert!(dbg.contains("Function"), "expected struct name; got: {dbg}");
        assert!(dbg.contains(r#"name: Some(":my::fn")"#), "expected name field; got: {dbg}");
        // closed_env: None → rendered as "<none>" per the Debug impl.
        assert!(dbg.contains("<none>"), "expected <none> for absent closed_env; got: {dbg}");
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
        };
        let dbg = format!("{:?}", f);
        // closed_env: Some(_) → rendered as "<env>" per the Debug impl.
        assert!(dbg.contains("<env>"), "expected <env> for present closed_env; got: {dbg}");
    }
}
