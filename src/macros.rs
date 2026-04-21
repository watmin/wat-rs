//! `defmacro` — parse-time macro expansion with Racket sets-of-scopes
//! hygiene (Flatt 2016).
//!
//! Per 058-031: macros transform source forms BEFORE hashing, signing,
//! type-checking, or evaluation. Two source files that differ only in
//! macro aliases (e.g. `Subtract` vs `Blend _ _ 1 -1`) expand to the
//! same canonical AST and the same hash — the substrate commit of
//! hash-IS-identity holds.
//!
//! # Hygiene by construction
//!
//! A macro that introduces a name (`(let ((tmp ,x)) ...)`) cannot
//! collide with a user's `tmp` in the caller's scope. The mechanism,
//! per FOUNDATION's specified algorithm:
//!
//! 1. At each macro invocation, allocate a fresh [`ScopeId`].
//! 2. Walk the macro's template. Every identifier whose origin is the
//!    template source has the fresh scope added to its scope set.
//! 3. Identifiers that came in via macro arguments (substituted at
//!    `,x` unquote sites) KEEP their original scope sets.
//! 4. Lexical-scope lookup compares `(name, scope_set)` pairs — so
//!    `tmp[{macro-scope}]` and `tmp[{user-scope}]` resolve to distinct
//!    bindings.
//!
//! Variable capture is structurally impossible, not discipline-enforced.
//!
//! # What this slice supports
//!
//! - `defmacro` forms with quasiquote-template bodies: `` ` `` for the
//!   template, `,expr` for parameter substitution, `,@expr` for list
//!   splicing.
//! - Fixpoint expansion (macros expand to more macros until no more
//!   remain). Depth limit prevents pathological infinite expansion.
//! - Full hygiene for the classic capture pattern.
//!
//! # What's deferred
//!
//! - Arbitrary-Lisp macro bodies (computed conditional templates,
//!   macro-authoring helpers beyond quasiquote). The spec admits them
//!   but the common case — and every 058 stdlib macro — uses
//!   quasiquote alone.
//! - Typed-macro checking (058-032). Macro parameters here are
//!   positional AST arguments; the type checker validates `:AST<T>`
//!   annotations against body positions in its own phase.

use crate::ast::WatAST;
use crate::identifier::{fresh_scope, ScopeId};
use std::collections::HashMap;
use std::fmt;

/// A registered macro.
#[derive(Debug, Clone)]
pub struct MacroDef {
    /// Full keyword-path of the macro (e.g. `:wat::std::Subtract`).
    pub name: String,
    /// Fixed-arity parameter names in order. Positional binding.
    pub params: Vec<String>,
    /// Optional rest-parameter name. When present, the macro accepts
    /// `args.len() >= params.len()` at expansion; the first N args
    /// bind to `params` as usual, and the REMAINING args are bundled
    /// into a `WatAST::List` and bound to this name. A template's
    /// `,@rest-name` unquote-splicing then drops the list's elements
    /// into the surrounding form at expansion. Syntax at declaration:
    /// `(:wat::core::defmacro (:name (p1 :AST<T1>) ... & (rest :AST<Vec<R>>) -> :AST<Ret>) body)`.
    /// The `&` marker separates fixed params from the rest-binder.
    pub rest_param: Option<String>,
    /// The template — typically `(:wat::core::quasiquote ...)`.
    pub body: WatAST,
}

/// Keyword-path ↦ `MacroDef` registry.
#[derive(Debug, Default, Clone)]
pub struct MacroRegistry {
    macros: HashMap<String, MacroDef>,
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

    /// Register a macro. Errors on duplicate or reserved prefix.
    pub fn register(&mut self, def: MacroDef) -> Result<(), MacroError> {
        if crate::resolve::is_reserved_prefix(&def.name) {
            return Err(MacroError::ReservedPrefix(def.name));
        }
        if self.macros.contains_key(&def.name) {
            return Err(MacroError::DuplicateMacro(def.name));
        }
        self.macros.insert(def.name.clone(), def);
        Ok(())
    }

    /// Register a TRUSTED stdlib macro. Bypasses the reserved-prefix
    /// gate because stdlib forms live under `:wat::std::*` by design.
    /// Still errors on duplicates. Intended for the baked stdlib
    /// loader; user source paths through `register` where the prefix
    /// check catches mis-namespaced user defmacros.
    pub fn register_stdlib(&mut self, def: MacroDef) -> Result<(), MacroError> {
        if self.macros.contains_key(&def.name) {
            return Err(MacroError::DuplicateMacro(def.name));
        }
        self.macros.insert(def.name.clone(), def);
        Ok(())
    }
}

/// Errors during macro registration / expansion.
#[derive(Debug)]
pub enum MacroError {
    /// Two `(:wat::core::defmacro ...)` forms registered the same name.
    DuplicateMacro(String),
    /// A user macro declared under a reserved `:wat::...` prefix.
    ReservedPrefix(String),
    /// A `defmacro` form was malformed.
    MalformedDefmacro { reason: String },
    /// The macro's body wasn't a quasiquote template — this slice only
    /// supports quasiquote bodies.
    UnsupportedBody { name: String, reason: String },
    /// A macro call passed the wrong number of arguments.
    ArityMismatch {
        name: String,
        expected: usize,
        got: usize,
    },
    /// An `unquote` reference named a parameter the macro didn't declare.
    UnboundMacroParam { name: String },
    /// `unquote-splicing` was applied to a non-list argument.
    SpliceNotList { name: String, got: &'static str },
    /// Expansion depth exceeded a sanity limit — probably an infinite
    /// recursive macro.
    ExpansionDepthExceeded { limit: usize },
    /// Other malformation in a macro invocation or template.
    MalformedTemplate { reason: String },
}

impl fmt::Display for MacroError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MacroError::DuplicateMacro(n) => {
                write!(f, "duplicate macro registration: {}", n)
            }
            MacroError::ReservedPrefix(n) => write!(
                f,
                "cannot declare macro {} — reserved prefix ({}); user macros must use their own prefix",
                n,
                crate::resolve::reserved_prefix_list()
            ),
            MacroError::MalformedDefmacro { reason } => {
                write!(f, "malformed defmacro: {}", reason)
            }
            MacroError::UnsupportedBody { name, reason } => write!(
                f,
                "macro {} body not supported: {} (this slice handles quasiquote-template bodies only)",
                name, reason
            ),
            MacroError::ArityMismatch { name, expected, got } => {
                write!(
                    f,
                    "macro {} expects {} arguments; got {}",
                    name, expected, got
                )
            }
            MacroError::UnboundMacroParam { name } => {
                write!(f, "unquote references unbound macro parameter: {}", name)
            }
            MacroError::SpliceNotList { name, got } => write!(
                f,
                "unquote-splicing (,@{}) requires a List argument; got {}",
                name, got
            ),
            MacroError::ExpansionDepthExceeded { limit } => write!(
                f,
                "macro expansion exceeded depth limit {} — likely infinite recursion",
                limit
            ),
            MacroError::MalformedTemplate { reason } => {
                write!(f, "malformed template: {}", reason)
            }
        }
    }
}

impl std::error::Error for MacroError {}

const EXPANSION_DEPTH_LIMIT: usize = 512;

/// Walk `forms`, register every `(:wat::core::defmacro ...)` into
/// `registry`, and return the remaining forms in order.
pub fn register_defmacros(
    forms: Vec<WatAST>,
    registry: &mut MacroRegistry,
) -> Result<Vec<WatAST>, MacroError> {
    let mut rest = Vec::new();
    for form in forms {
        if is_defmacro_form(&form) {
            let def = parse_defmacro_form(form)?;
            registry.register(def)?;
        } else {
            rest.push(form);
        }
    }
    Ok(rest)
}

/// Stdlib-registration variant of [`register_defmacros`] that
/// bypasses the `:wat::std::*` reserved-prefix gate. Called by the
/// startup pipeline on the baked stdlib sources; user source still
/// goes through [`register_defmacros`] so mis-namespaced user
/// defmacros halt at startup.
pub fn register_stdlib_defmacros(
    forms: Vec<WatAST>,
    registry: &mut MacroRegistry,
) -> Result<Vec<WatAST>, MacroError> {
    let mut rest = Vec::new();
    for form in forms {
        if is_defmacro_form(&form) {
            let def = parse_defmacro_form(form)?;
            registry.register_stdlib(def)?;
        } else {
            rest.push(form);
        }
    }
    Ok(rest)
}

fn is_defmacro_form(form: &WatAST) -> bool {
    matches!(
        form,
        WatAST::List(items)
            if matches!(items.first(), Some(WatAST::Keyword(k)) if k == ":wat::core::defmacro")
    )
}

/// Parse `(:wat::core::defmacro (:name::path (p :AST<T>) ... -> :AST<R>) body)`.
fn parse_defmacro_form(form: WatAST) -> Result<MacroDef, MacroError> {
    let items = match form {
        WatAST::List(items) => items,
        _ => {
            return Err(MacroError::MalformedDefmacro {
                reason: "expected list form".into(),
            })
        }
    };
    if items.len() != 3 {
        return Err(MacroError::MalformedDefmacro {
            reason: format!(
                "expected (:wat::core::defmacro signature body); got {} elements",
                items.len()
            ),
        });
    }
    let mut iter = items.into_iter();
    let _defmacro_kw = iter.next();
    let signature = iter.next().expect("length checked");
    let body = iter.next().expect("length checked");

    let (name, params, rest_param) = parse_defmacro_signature(signature)?;
    Ok(MacroDef {
        name,
        params,
        rest_param,
        body,
    })
}

fn parse_defmacro_signature(
    sig: WatAST,
) -> Result<(String, Vec<String>, Option<String>), MacroError> {
    let items = match sig {
        WatAST::List(items) => items,
        _ => {
            return Err(MacroError::MalformedDefmacro {
                reason: "signature must be a list".into(),
            })
        }
    };
    let mut iter = items.into_iter();
    let name = match iter.next() {
        Some(WatAST::Keyword(k)) => k,
        Some(_other) => {
            return Err(MacroError::MalformedDefmacro {
                reason: "macro name must be a keyword-path".into(),
            })
        }
        None => {
            return Err(MacroError::MalformedDefmacro {
                reason: "signature is empty".into(),
            })
        }
    };
    let mut params = Vec::new();
    let mut rest_param: Option<String> = None;
    let mut saw_rest_marker = false;
    for item in iter {
        match item {
            WatAST::Symbol(ref s) if s.as_str() == "->" => break,
            // `&` marker — the next binder is the rest-param. Only one
            // rest-binder is allowed; additional params after it are
            // rejected (same as Common Lisp's `&rest` discipline).
            WatAST::Symbol(ref s) if s.as_str() == "&" => {
                if saw_rest_marker {
                    return Err(MacroError::MalformedDefmacro {
                        reason: "duplicate `&` rest-marker in macro signature".into(),
                    });
                }
                if rest_param.is_some() {
                    return Err(MacroError::MalformedDefmacro {
                        reason: "`&` must precede its rest-binder".into(),
                    });
                }
                saw_rest_marker = true;
            }
            WatAST::List(pair) => {
                let paramname = match pair.into_iter().next() {
                    Some(WatAST::Symbol(ident)) => ident.name,
                    _ => {
                        return Err(MacroError::MalformedDefmacro {
                            reason: "parameter name must be a bare symbol".into(),
                        })
                    }
                };
                if saw_rest_marker {
                    if rest_param.is_some() {
                        return Err(MacroError::MalformedDefmacro {
                            reason: "only one rest-binder is allowed after `&`".into(),
                        });
                    }
                    rest_param = Some(paramname);
                } else {
                    params.push(paramname);
                }
            }
            _ => {
                return Err(MacroError::MalformedDefmacro {
                    reason: "unexpected signature element".into(),
                })
            }
        }
    }
    if saw_rest_marker && rest_param.is_none() {
        return Err(MacroError::MalformedDefmacro {
            reason: "`&` rest-marker with no binder".into(),
        });
    }
    Ok((name, params, rest_param))
}

// ─── Expansion ──────────────────────────────────────────────────────────

/// Expand every macro call in `forms` to fixpoint. Returns the expanded
/// AST list.
pub fn expand_all(
    forms: Vec<WatAST>,
    registry: &MacroRegistry,
) -> Result<Vec<WatAST>, MacroError> {
    let mut out = Vec::with_capacity(forms.len());
    for form in forms {
        out.push(expand_form(form, registry, 0)?);
    }
    Ok(out)
}

/// Expand a single form. Recursively expands children, then checks
/// whether the resulting node is itself a macro call; if so, expand it,
/// and continue to fixpoint.
fn expand_form(
    form: WatAST,
    registry: &MacroRegistry,
    depth: usize,
) -> Result<WatAST, MacroError> {
    if depth > EXPANSION_DEPTH_LIMIT {
        return Err(MacroError::ExpansionDepthExceeded {
            limit: EXPANSION_DEPTH_LIMIT,
        });
    }

    match form {
        WatAST::List(items) => {
            // Recurse into children first. This gives us the shape
            // (expanded-head expanded-args...) — any inner macro calls
            // resolved before we check the outer for a macro call.
            let expanded_children: Result<Vec<_>, _> = items
                .into_iter()
                .map(|c| expand_form(c, registry, depth + 1))
                .collect();
            let expanded_children = expanded_children?;

            // Is the (now-expanded) head a registered macro?
            if let Some(WatAST::Keyword(head)) = expanded_children.first() {
                if let Some(def) = registry.get(head) {
                    // Macro call — expand this call site.
                    let args = expanded_children[1..].to_vec();
                    let expanded = expand_macro_call(def, args)?;
                    // Re-expand the result to fixpoint.
                    return expand_form(expanded, registry, depth + 1);
                }
            }

            Ok(WatAST::List(expanded_children))
        }
        other => Ok(other),
    }
}

/// Expand a single macro call. Allocates a fresh [`ScopeId`], walks the
/// template substituting parameters with argument ASTs, adds the macro
/// scope to every template-origin symbol, returns the expansion.
///
/// Variadic macros (MacroDef with `rest_param: Some(_)`) accept
/// `args.len() >= params.len()`. The first N args bind positionally to
/// the fixed params; the rest are wrapped in a `WatAST::List` and
/// bound to the rest-name. The template's `,@rest-name` splice drops
/// those elements into the surrounding list context at expansion.
fn expand_macro_call(
    def: &MacroDef,
    args: Vec<WatAST>,
) -> Result<WatAST, MacroError> {
    let fixed_arity = def.params.len();
    match &def.rest_param {
        None => {
            if args.len() != fixed_arity {
                return Err(MacroError::ArityMismatch {
                    name: def.name.clone(),
                    expected: fixed_arity,
                    got: args.len(),
                });
            }
        }
        Some(_) => {
            if args.len() < fixed_arity {
                return Err(MacroError::ArityMismatch {
                    name: def.name.clone(),
                    expected: fixed_arity,
                    got: args.len(),
                });
            }
        }
    }

    let mut bindings: HashMap<String, WatAST> = HashMap::new();
    let mut iter = args.into_iter();
    for param in &def.params {
        bindings.insert(
            param.clone(),
            iter.next().expect("arity checked above"),
        );
    }
    if let Some(rest_name) = &def.rest_param {
        let rest: Vec<WatAST> = iter.collect();
        bindings.insert(rest_name.clone(), WatAST::List(rest));
    }

    let macro_scope = fresh_scope();
    expand_template(&def.body, &bindings, macro_scope, &def.name)
}

/// Walk a macro template, substituting `,param` and `,@param` at
/// unquote sites and adding the macro scope to template-origin symbols.
///
/// The template's top-level form is usually `(:wat::core::quasiquote X)`.
/// If it's not a quasiquote, we error (this slice doesn't do arbitrary
/// macro bodies).
fn expand_template(
    template: &WatAST,
    bindings: &HashMap<String, WatAST>,
    macro_scope: ScopeId,
    macro_name: &str,
) -> Result<WatAST, MacroError> {
    let quasi_body = match template {
        WatAST::List(items) if items.len() == 2 => match items.first() {
            Some(WatAST::Keyword(k)) if k == ":wat::core::quasiquote" => &items[1],
            _ => {
                return Err(MacroError::UnsupportedBody {
                    name: macro_name.into(),
                    reason: "body must be a quasiquote template (`X form)".into(),
                })
            }
        },
        _ => {
            return Err(MacroError::UnsupportedBody {
                name: macro_name.into(),
                reason: "body must be a quasiquote template (`X form)".into(),
            })
        }
    };

    walk_template(quasi_body, bindings, macro_scope, macro_name)
}

/// Walk a quasiquoted form, expanding `,x` unquotes to their argument
/// ASTs, `,@x` unquote-splicing to their list elements, and tagging
/// every template-origin symbol with the macro scope.
fn walk_template(
    form: &WatAST,
    bindings: &HashMap<String, WatAST>,
    macro_scope: ScopeId,
    macro_name: &str,
) -> Result<WatAST, MacroError> {
    match form {
        WatAST::List(items) => {
            // Detect `(:wat::core::unquote X)` — substitute the argument.
            if let Some(arg) = match_unquote(items, ":wat::core::unquote") {
                return unquote_argument(arg, bindings, macro_name);
            }

            // Walk each child, handling unquote-splicing inline.
            let mut out = Vec::with_capacity(items.len());
            for child in items {
                if let WatAST::List(child_items) = child {
                    if let Some(splice_arg) =
                        match_unquote(child_items, ":wat::core::unquote-splicing")
                    {
                        let spliced = splice_argument(splice_arg, bindings, macro_name)?;
                        out.extend(spliced);
                        continue;
                    }
                }
                out.push(walk_template(child, bindings, macro_scope, macro_name)?);
            }
            Ok(WatAST::List(out))
        }
        WatAST::Symbol(ident) => {
            // Template-origin symbol — add the macro scope to its scope set.
            Ok(WatAST::Symbol(ident.add_scope(macro_scope)))
        }
        // Literals and keywords pass through unchanged; keywords carry
        // no scope tracking.
        other => Ok(other.clone()),
    }
}

/// If `items` is `(head arg)` for the given head keyword, return `arg`.
fn match_unquote<'a>(items: &'a [WatAST], head_kw: &str) -> Option<&'a WatAST> {
    if items.len() != 2 {
        return None;
    }
    match items.first() {
        Some(WatAST::Keyword(k)) if k == head_kw => items.get(1),
        _ => None,
    }
}

/// `,X` — the argument is either a macro parameter (substitute its
/// bound AST) or some other template form (walk normally and expand).
fn unquote_argument(
    arg: &WatAST,
    bindings: &HashMap<String, WatAST>,
    macro_name: &str,
) -> Result<WatAST, MacroError> {
    match arg {
        WatAST::Symbol(ident) => match bindings.get(&ident.name) {
            Some(bound) => Ok(bound.clone()),
            None => Err(MacroError::UnboundMacroParam {
                name: ident.name.clone(),
            }),
        },
        _ => Err(MacroError::MalformedTemplate {
            reason: format!(
                "macro {} — unquote ',X' requires a parameter name; got non-symbol",
                macro_name
            ),
        }),
    }
}

/// `,@X` — argument must be a parameter bound to a List AST; splice
/// its elements into the surrounding list context.
fn splice_argument(
    arg: &WatAST,
    bindings: &HashMap<String, WatAST>,
    macro_name: &str,
) -> Result<Vec<WatAST>, MacroError> {
    let paramname = match arg {
        WatAST::Symbol(ident) => &ident.name,
        _ => {
            return Err(MacroError::MalformedTemplate {
                reason: format!(
                    "macro {} — unquote-splicing ',@X' requires a parameter name",
                    macro_name
                ),
            })
        }
    };
    let bound = bindings
        .get(paramname)
        .ok_or_else(|| MacroError::UnboundMacroParam {
            name: paramname.clone(),
        })?;
    match bound {
        WatAST::List(items) => Ok(items.clone()),
        other => Err(MacroError::SpliceNotList {
            name: paramname.clone(),
            got: ast_variant_name(other),
        }),
    }
}

fn ast_variant_name(ast: &WatAST) -> &'static str {
    match ast {
        WatAST::IntLit(_) => "int literal",
        WatAST::FloatLit(_) => "float literal",
        WatAST::BoolLit(_) => "bool literal",
        WatAST::StringLit(_) => "string literal",
        WatAST::Keyword(_) => "keyword",
        WatAST::Symbol(_) => "symbol",
        WatAST::List(_) => "list",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identifier::Identifier;
    use crate::parser::parse_all;

    fn expand(src: &str) -> Result<Vec<WatAST>, MacroError> {
        let forms = parse_all(src).expect("parse ok");
        let mut reg = MacroRegistry::new();
        let rest = register_defmacros(forms, &mut reg)?;
        expand_all(rest, &reg)
    }

    // ─── Pure alias macro ───────────────────────────────────────────────

    #[test]
    fn alias_macro_expands_to_primitive() {
        let forms = expand(
            r#"
            (:wat::core::defmacro (:my::vocab::Concurrent (xs :AST<List<holon::HolonAST>>) -> :AST<holon::HolonAST>)
              `(:wat::algebra::Bundle ,xs))
            (:my::vocab::Concurrent (:wat::core::vec :holon::HolonAST a b c))
            "#,
        )
        .unwrap();
        assert_eq!(forms.len(), 1);
        // Expansion: (:wat::algebra::Bundle (:wat::core::vec :holon::HolonAST a b c))
        match &forms[0] {
            WatAST::List(items) => {
                assert_eq!(items.len(), 2);
                assert!(matches!(&items[0], WatAST::Keyword(k) if k == ":wat::algebra::Bundle"));
            }
            _ => panic!("expected List after expansion"),
        }
    }

    // ─── Transforming macro with multiple params ────────────────────────

    #[test]
    fn subtract_macro_expansion() {
        let forms = expand(
            r#"
            (:wat::core::defmacro (:my::vocab::Subtract (x :AST<holon::HolonAST>) (y :AST<holon::HolonAST>) -> :AST<holon::HolonAST>)
              `(:wat::algebra::Blend ,x ,y 1 -1))
            (:my::vocab::Subtract foo bar)
            "#,
        )
        .unwrap();
        // (:wat::algebra::Blend foo bar 1 -1)
        match &forms[0] {
            WatAST::List(items) => {
                assert_eq!(items.len(), 5);
                assert!(matches!(&items[0], WatAST::Keyword(k) if k == ":wat::algebra::Blend"));
                assert!(matches!(&items[1], WatAST::Symbol(i) if i.as_str() == "foo"));
                assert!(matches!(&items[2], WatAST::Symbol(i) if i.as_str() == "bar"));
                assert!(matches!(items[3], WatAST::IntLit(1)));
                assert!(matches!(items[4], WatAST::IntLit(-1)));
            }
            _ => panic!("expected List"),
        }
    }

    // ─── Unquote-splicing ───────────────────────────────────────────────

    #[test]
    fn splice_list_arg_into_template() {
        let forms = expand(
            r#"
            (:wat::core::defmacro (:my::vocab::SumAll (xs :AST<List<holon::HolonAST>>) -> :AST<holon::HolonAST>)
              `(:wat::algebra::Bundle ,@xs))
            (:my::vocab::SumAll (a b c))
            "#,
        )
        .unwrap();
        // (:wat::algebra::Bundle a b c) — the list elements are spliced in.
        match &forms[0] {
            WatAST::List(items) => {
                assert_eq!(items.len(), 4);
                assert!(matches!(&items[0], WatAST::Keyword(k) if k == ":wat::algebra::Bundle"));
                assert!(matches!(&items[1], WatAST::Symbol(i) if i.as_str() == "a"));
                assert!(matches!(&items[2], WatAST::Symbol(i) if i.as_str() == "b"));
                assert!(matches!(&items[3], WatAST::Symbol(i) if i.as_str() == "c"));
            }
            _ => panic!("expected List"),
        }
    }

    // ─── Nested macros (fixpoint) ───────────────────────────────────────

    #[test]
    fn nested_macro_expands_to_fixpoint() {
        let forms = expand(
            r#"
            (:wat::core::defmacro (:my::outer (x :AST) -> :AST)
              `(:my::inner ,x))
            (:wat::core::defmacro (:my::inner (x :AST) -> :AST)
              `(:wat::algebra::Atom ,x))
            (:my::outer 42)
            "#,
        )
        .unwrap();
        // (:wat::algebra::Atom 42) after fixpoint.
        match &forms[0] {
            WatAST::List(items) => {
                assert_eq!(items.len(), 2);
                assert!(matches!(&items[0], WatAST::Keyword(k) if k == ":wat::algebra::Atom"));
                assert!(matches!(items[1], WatAST::IntLit(42)));
            }
            _ => panic!("expected List"),
        }
    }

    // ─── Hygiene — template-origin identifiers get the macro scope ─────

    #[test]
    fn template_identifier_carries_macro_scope() {
        let forms = expand(
            r#"
            (:wat::core::defmacro (:my::vocab::WithTmp (body :AST) -> :AST)
              `(:wat::core::let (((tmp :i64) 1)) ,body))
            (:my::vocab::WithTmp tmp)
            "#,
        )
        .unwrap();
        // Expansion: (:wat::core::let (((tmp[macro-scope] :i64) 1)) tmp[user-empty])
        // The two `tmp`s must have DIFFERENT Identifiers.
        let list = match &forms[0] {
            WatAST::List(items) => items,
            _ => panic!("expected list"),
        };
        // (((tmp :i64) 1)) — drill through the bindings list, the
        // binding, and the typed-name pair to reach tmp.
        let bindings = match &list[1] {
            WatAST::List(bs) => bs,
            _ => panic!("expected bindings list"),
        };
        let first_binding = match &bindings[0] {
            WatAST::List(b) => b,
            _ => panic!("expected binding pair"),
        };
        let typed_name = match &first_binding[0] {
            WatAST::List(tn) => tn,
            _ => panic!("expected (name :Type) pair"),
        };
        let template_tmp = match &typed_name[0] {
            WatAST::Symbol(i) => i,
            _ => panic!("expected Symbol"),
        };
        // The body position's `tmp` — user-supplied argument, not macro-origin.
        let user_tmp = match &list[2] {
            WatAST::Symbol(i) => i,
            _ => panic!("expected Symbol in body"),
        };
        assert_eq!(template_tmp.name, "tmp");
        assert_eq!(user_tmp.name, "tmp");
        assert!(
            !template_tmp.scopes.is_empty(),
            "template tmp must have macro scope attached"
        );
        assert!(
            user_tmp.scopes.is_empty(),
            "user-argument tmp must NOT have the macro scope"
        );
        assert_ne!(
            template_tmp, user_tmp,
            "template and user tmp must be DIFFERENT Identifiers"
        );
    }

    // ─── Argument identifiers are preserved unchanged ──────────────────

    #[test]
    fn argument_identifiers_pass_through_unchanged() {
        // User passes a symbol; the macro should splice it verbatim.
        let forms = expand(
            r#"
            (:wat::core::defmacro (:my::wrap (v :AST) -> :AST)
              `(:wat::algebra::Atom ,v))
            (:my::wrap some-var)
            "#,
        )
        .unwrap();
        let list = match &forms[0] {
            WatAST::List(items) => items,
            _ => panic!("expected list"),
        };
        let v_arg = match &list[1] {
            WatAST::Symbol(i) => i,
            _ => panic!("expected Symbol at arg position"),
        };
        // Argument identifier — no macro scope added.
        assert_eq!(v_arg.name, "some-var");
        assert!(
            v_arg.scopes.is_empty(),
            "argument identifier should have no macro scope"
        );
    }

    // ─── Classic capture: two macros introduce the same template name ─

    #[test]
    fn two_macro_invocations_get_distinct_scopes() {
        let forms = expand(
            r#"
            (:wat::core::defmacro (:my::twice (x :AST) -> :AST)
              `(:wat::core::let (((t :i64) ,x)) t))
            (:my::twice 1)
            (:my::twice 2)
            "#,
        )
        .unwrap();
        // Both expansions bind `t` in the template; each invocation should
        // tag its `t` with a FRESH scope. The two `t`s differ.
        let extract_binding_sym = |f: &WatAST| -> Identifier {
            let outer = if let WatAST::List(items) = f {
                items.clone()
            } else {
                panic!("expected list")
            };
            let bindings = if let WatAST::List(b) = &outer[1] {
                b.clone()
            } else {
                panic!()
            };
            let pair = if let WatAST::List(b) = &bindings[0] {
                b.clone()
            } else {
                panic!()
            };
            let typed_name = if let WatAST::List(tn) = &pair[0] {
                tn.clone()
            } else {
                panic!()
            };
            if let WatAST::Symbol(i) = &typed_name[0] {
                i.clone()
            } else {
                panic!()
            }
        };
        let t1 = extract_binding_sym(&forms[0]);
        let t2 = extract_binding_sym(&forms[1]);
        assert_eq!(t1.name, "t");
        assert_eq!(t2.name, "t");
        assert_ne!(t1, t2, "each invocation should mint a fresh macro scope");
    }

    // ─── Error paths ────────────────────────────────────────────────────

    #[test]
    fn reserved_prefix_macro_rejected() {
        let err = expand(
            r#"(:wat::core::defmacro (:wat::std::MyMacro (x :AST) -> :AST) `,x)"#,
        )
        .unwrap_err();
        assert!(matches!(err, MacroError::ReservedPrefix(_)));
    }

    #[test]
    fn duplicate_defmacro_rejected() {
        let err = expand(
            r#"
            (:wat::core::defmacro (:my::m (x :AST) -> :AST) `,x)
            (:wat::core::defmacro (:my::m (x :AST) -> :AST) `,x)
            "#,
        )
        .unwrap_err();
        assert!(matches!(err, MacroError::DuplicateMacro(_)));
    }

    #[test]
    fn macro_arity_mismatch() {
        let err = expand(
            r#"
            (:wat::core::defmacro (:my::two (x :AST) (y :AST) -> :AST)
              `(:wat::core::vec ,x ,y))
            (:my::two 1)
            "#,
        )
        .unwrap_err();
        assert!(matches!(err, MacroError::ArityMismatch { .. }));
    }

    #[test]
    fn non_quasiquote_body_rejected() {
        // Body is a plain list, not a quasiquote — this slice doesn't
        // evaluate arbitrary macro bodies.
        let err = expand(
            r#"
            (:wat::core::defmacro (:my::m (x :AST) -> :AST)
              (:wat::core::vec :bogus x))
            (:my::m 1)
            "#,
        )
        .unwrap_err();
        assert!(matches!(err, MacroError::UnsupportedBody { .. }));
    }

    #[test]
    fn splice_non_list_arg_rejected() {
        let err = expand(
            r#"
            (:wat::core::defmacro (:my::s (xs :AST) -> :AST)
              `(:wat::core::vec ,@xs))
            (:my::s 42)
            "#,
        )
        .unwrap_err();
        assert!(matches!(err, MacroError::SpliceNotList { .. }));
    }

    // ─── Non-macro forms pass through unchanged ─────────────────────────

    #[test]
    fn non_macro_forms_unchanged() {
        let forms = expand(r#"(:wat::algebra::Atom "hello") 42 "world""#).unwrap();
        assert_eq!(forms.len(), 3);
        assert!(matches!(forms[1], WatAST::IntLit(42)));
        assert!(matches!(&forms[2], WatAST::StringLit(s) if s == "world"));
    }
}
