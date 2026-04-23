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
use crate::span::Span;
use crate::identifier::{fresh_scope, ScopeId};
use std::collections::HashMap;
use std::fmt;

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
        WatAST::List(items, _)
            if matches!(items.first(), Some(WatAST::Keyword(k, _)) if k == ":wat::core::defmacro")
    )
}

/// Parse `(:wat::core::defmacro (:name::path (p :AST<T>) ... -> :AST<R>) body)`.
fn parse_defmacro_form(form: WatAST) -> Result<MacroDef, MacroError> {
    let items = match form {
        WatAST::List(items, _) => items,
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
        WatAST::List(items, _) => items,
        _ => {
            return Err(MacroError::MalformedDefmacro {
                reason: "signature must be a list".into(),
            })
        }
    };
    let mut iter = items.into_iter();
    let name = match iter.next() {
        Some(WatAST::Keyword(k, _)) => k,
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
            WatAST::Symbol(ref s, _) if s.as_str() == "->" => break,
            // `&` marker — the next binder is the rest-param. Only one
            // rest-binder is allowed; additional params after it are
            // rejected (same as Common Lisp's `&rest` discipline).
            WatAST::Symbol(ref s, _) if s.as_str() == "&" => {
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
            WatAST::List(pair, _) => {
                let paramname = match pair.into_iter().next() {
                    Some(WatAST::Symbol(ident, _)) => ident.name,
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
    // Arc 029 slice 1: handle macro-generating-macros. A macro call
    // may expand to a `(:wat::core::defmacro ...)` registration for
    // a new macro — e.g., `:wat::test::make-deftest` expanding to a
    // fully-configured deftest variant. Register each such form as
    // it appears so subsequent forms in the stream can invoke the
    // new macro. Clone the caller's registry so our in-flight
    // additions stay scoped to this expansion.
    let mut reg = registry.clone();
    let mut out = Vec::with_capacity(forms.len());
    for form in forms {
        let expanded = expand_form(form, &reg, 0)?;
        if is_defmacro_form(&expanded) {
            let def = parse_defmacro_form(expanded)?;
            reg.register(def)?;
        } else {
            out.push(expanded);
        }
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
        WatAST::List(items, list_span) => {
            // Recurse into children first. This gives us the shape
            // (expanded-head expanded-args...) — any inner macro calls
            // resolved before we check the outer for a macro call.
            let expanded_children: Result<Vec<_>, _> = items
                .into_iter()
                .map(|c| expand_form(c, registry, depth + 1))
                .collect();
            let expanded_children = expanded_children?;

            // Is the (now-expanded) head a registered macro?
            if let Some(WatAST::Keyword(head, _)) = expanded_children.first() {
                if let Some(def) = registry.get(head) {
                    // Macro call — expand this call site. Pass the
                    // outer list's span so the expansion can inherit
                    // it (call-site span, per arc 016 slice 1
                    // DESIGN: generated forms inherit the caller's
                    // span).
                    let args = expanded_children[1..].to_vec();
                    let expanded = expand_macro_call(def, args, list_span.clone())?;
                    // Re-expand the result to fixpoint.
                    return expand_form(expanded, registry, depth + 1);
                }
            }

            // Not a macro call — preserve the outer list's span.
            Ok(WatAST::List(expanded_children, list_span))
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
    call_site_span: Span,
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
        // Rest-list wrapper inherits the call-site span — the
        // `,@rest` splice drops these into the template's
        // surrounding context.
        bindings.insert(rest_name.clone(), WatAST::List(rest, call_site_span.clone()));
    }

    let macro_scope = fresh_scope();
    expand_template(&def.body, &bindings, macro_scope, &def.name, &call_site_span)
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
    call_site_span: &Span,
) -> Result<WatAST, MacroError> {
    let quasi_body = match template {
        WatAST::List(items, _) if items.len() == 2 => match items.first() {
            Some(WatAST::Keyword(k, _)) if k == ":wat::core::quasiquote" => &items[1],
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

    walk_template(quasi_body, bindings, macro_scope, macro_name, call_site_span, 1)
}

/// Walk a quasiquoted form, expanding `,x` unquotes to their argument
/// ASTs, `,@x` unquote-splicing to their list elements, and tagging
/// every template-origin symbol with the macro scope.
///
/// Arc 016 slice 1: template-origin nodes (those built from the
/// defmacro's template, not from unquoted user args) inherit the
/// `call_site_span` — the span of the macro INVOCATION in user
/// source, not the template's span in the defmacro file. Matches
/// Racket's sets-of-scopes approach: when a user reads a failure
/// message, they want a pointer to their own code, not the
/// library's template.
///
/// Arc 029 slice 1: `depth` tracks how many layers of quasiquote
/// we're inside. Entry from `expand_template` is at depth 1 (the
/// outer `(:wat::core::quasiquote ...)` has just been stripped).
/// Encountering another `(:wat::core::quasiquote X)` in the template
/// bumps depth and preserves the wrapper. `(:wat::core::unquote X)`
/// at depth 1 substitutes; at depth > 1 it preserves the wrapper
/// and walks X at depth-1. Same discipline for
/// `(:wat::core::unquote-splicing X)`. This enables macro-
/// generating-macro patterns like `:wat::test::make-deftest` where
/// some unquotes fire at the outer expansion and others survive
/// for the inner macro's eventual expansion.
fn walk_template(
    form: &WatAST,
    bindings: &HashMap<String, WatAST>,
    macro_scope: ScopeId,
    macro_name: &str,
    call_site_span: &Span,
    depth: u32,
) -> Result<WatAST, MacroError> {
    match form {
        WatAST::List(items, _) => {
            // Nested quasiquote — bump depth, preserve the wrapper.
            // Arc 029 slice 1.
            if let Some(arg) = match_unquote(items, ":wat::core::quasiquote") {
                let inner = walk_template(
                    arg,
                    bindings,
                    macro_scope,
                    macro_name,
                    call_site_span,
                    depth + 1,
                )?;
                return Ok(WatAST::List(
                    vec![
                        WatAST::Keyword(
                            ":wat::core::quasiquote".into(),
                            call_site_span.clone(),
                        ),
                        inner,
                    ],
                    call_site_span.clone(),
                ));
            }

            // Unquote — fires at depth 1, preserves + peels at depth > 1.
            if let Some(arg) = match_unquote(items, ":wat::core::unquote") {
                if depth == 1 {
                    return unquote_argument(arg, bindings);
                } else {
                    let inner = walk_template(
                        arg,
                        bindings,
                        macro_scope,
                        macro_name,
                        call_site_span,
                        depth - 1,
                    )?;
                    return Ok(WatAST::List(
                        vec![
                            WatAST::Keyword(
                                ":wat::core::unquote".into(),
                                call_site_span.clone(),
                            ),
                            inner,
                        ],
                        call_site_span.clone(),
                    ));
                }
            }

            // Walk each child, handling unquote-splicing inline.
            let mut out = Vec::with_capacity(items.len());
            for child in items {
                if let WatAST::List(child_items, _) = child {
                    if let Some(splice_arg) =
                        match_unquote(child_items, ":wat::core::unquote-splicing")
                    {
                        if depth == 1 {
                            // Fire: splice the argument's elements.
                            let spliced = splice_argument(splice_arg, bindings, macro_name)?;
                            out.extend(spliced);
                            continue;
                        } else {
                            // Preserve + peel: walk arg at depth-1,
                            // rebuild `(:wat::core::unquote-splicing ...)`.
                            let inner = walk_template(
                                splice_arg,
                                bindings,
                                macro_scope,
                                macro_name,
                                call_site_span,
                                depth - 1,
                            )?;
                            out.push(WatAST::List(
                                vec![
                                    WatAST::Keyword(
                                        ":wat::core::unquote-splicing".into(),
                                        call_site_span.clone(),
                                    ),
                                    inner,
                                ],
                                call_site_span.clone(),
                            ));
                            continue;
                        }
                    }
                }
                out.push(walk_template(
                    child,
                    bindings,
                    macro_scope,
                    macro_name,
                    call_site_span,
                    depth,
                )?);
            }
            Ok(WatAST::List(out, call_site_span.clone()))
        }
        WatAST::Symbol(ident, _) => {
            // Template-origin symbol — add the macro scope to its scope set.
            Ok(WatAST::Symbol(
                ident.add_scope(macro_scope),
                call_site_span.clone(),
            ))
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
        Some(WatAST::Keyword(k, _)) if k == head_kw => items.get(1),
        _ => None,
    }
}

/// `,X` — the argument is either a macro parameter (substitute its
/// bound AST) or an already-substituted literal value from a prior
/// expansion pass (arc 029 slice 1: the tail-end of the `,,X`
/// resolution path). Symbols look up in `bindings`; everything else
/// returns as-is.
fn unquote_argument(
    arg: &WatAST,
    bindings: &HashMap<String, WatAST>,
) -> Result<WatAST, MacroError> {
    match arg {
        WatAST::Symbol(ident, _) => match bindings.get(&ident.name) {
            Some(bound) => Ok(bound.clone()),
            None => Err(MacroError::UnboundMacroParam {
                name: ident.name.clone(),
            }),
        },
        // Already-substituted literal (from a `,,X` outer pass or any
        // other macro that built `(:wat::core::unquote <value>)`
        // directly). Return as-is; the parent list absorbs it.
        _ => Ok(arg.clone()),
    }
}

/// `,@X` — argument must be a parameter bound to a List AST OR an
/// already-substituted List value (arc 029 slice 1: the `,,@X`
/// resolution tail); splice its elements into the surrounding list
/// context.
fn splice_argument(
    arg: &WatAST,
    bindings: &HashMap<String, WatAST>,
    macro_name: &str,
) -> Result<Vec<WatAST>, MacroError> {
    match arg {
        WatAST::Symbol(ident, _) => {
            let bound = bindings
                .get(&ident.name)
                .ok_or_else(|| MacroError::UnboundMacroParam {
                    name: ident.name.clone(),
                })?;
            match bound {
                WatAST::List(items, _) => Ok(items.clone()),
                other => Err(MacroError::SpliceNotList {
                    name: ident.name.clone(),
                    got: ast_variant_name(other),
                }),
            }
        }
        // Already-substituted list value.
        WatAST::List(items, _) => Ok(items.clone()),
        other => Err(MacroError::MalformedTemplate {
            reason: format!(
                "macro {} — unquote-splicing ',@X' requires a list (parameter \
                 or already-substituted value); got {}",
                macro_name,
                ast_variant_name(other)
            ),
        }),
    }
}

fn ast_variant_name(ast: &WatAST) -> &'static str {
    match ast {
        WatAST::IntLit(_, _) => "int literal",
        WatAST::FloatLit(_, _) => "float literal",
        WatAST::BoolLit(_, _) => "bool literal",
        WatAST::StringLit(_, _) => "string literal",
        WatAST::Keyword(_, _) => "keyword",
        WatAST::Symbol(_, _) => "symbol",
        WatAST::List(_, _) => "list",
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

    /// Like `expand`, but DOES NOT strip generated defmacros from the
    /// output. Arc 029 slice 1 tests use this to inspect the body of
    /// a defmacro produced by an outer macro-generating-macro call.
    fn expand_keeping_defmacros(src: &str) -> Result<Vec<WatAST>, MacroError> {
        let forms = parse_all(src).expect("parse ok");
        let mut reg = MacroRegistry::new();
        let rest = register_defmacros(forms, &mut reg)?;
        let mut out = Vec::with_capacity(rest.len());
        for form in rest {
            out.push(expand_form(form, &reg, 0)?);
        }
        Ok(out)
    }

    // ─── Pure alias macro ───────────────────────────────────────────────

    #[test]
    fn alias_macro_expands_to_primitive() {
        let forms = expand(
            r#"
            (:wat::core::defmacro (:my::vocab::Concurrent (xs :AST<List<wat::holon::HolonAST>>) -> :AST<wat::holon::HolonAST>)
              `(:wat::holon::Bundle ,xs))
            (:my::vocab::Concurrent (:wat::core::vec :wat::holon::HolonAST a b c))
            "#,
        )
        .unwrap();
        assert_eq!(forms.len(), 1);
        // Expansion: (:wat::holon::Bundle (:wat::core::vec :wat::holon::HolonAST a b c))
        match &forms[0] {
            WatAST::List(items, _) => {
                assert_eq!(items.len(), 2);
                assert!(matches!(&items[0], WatAST::Keyword(k, _) if k == ":wat::holon::Bundle"));
            }
            _ => panic!("expected List after expansion"),
        }
    }

    // ─── Transforming macro with multiple params ────────────────────────

    #[test]
    fn subtract_macro_expansion() {
        let forms = expand(
            r#"
            (:wat::core::defmacro (:my::vocab::Subtract (x :AST<wat::holon::HolonAST>) (y :AST<wat::holon::HolonAST>) -> :AST<wat::holon::HolonAST>)
              `(:wat::holon::Blend ,x ,y 1 -1))
            (:my::vocab::Subtract foo bar)
            "#,
        )
        .unwrap();
        // (:wat::holon::Blend foo bar 1 -1)
        match &forms[0] {
            WatAST::List(items, _) => {
                assert_eq!(items.len(), 5);
                assert!(matches!(&items[0], WatAST::Keyword(k, _) if k == ":wat::holon::Blend"));
                assert!(matches!(&items[1], WatAST::Symbol(i, _) if i.as_str() == "foo"));
                assert!(matches!(&items[2], WatAST::Symbol(i, _) if i.as_str() == "bar"));
                assert!(matches!(items[3], WatAST::IntLit(1, _)));
                assert!(matches!(items[4], WatAST::IntLit(-1, _)));
            }
            _ => panic!("expected List"),
        }
    }

    // ─── Unquote-splicing ───────────────────────────────────────────────

    #[test]
    fn splice_list_arg_into_template() {
        let forms = expand(
            r#"
            (:wat::core::defmacro (:my::vocab::SumAll (xs :AST<List<wat::holon::HolonAST>>) -> :AST<wat::holon::HolonAST>)
              `(:wat::holon::Bundle ,@xs))
            (:my::vocab::SumAll (a b c))
            "#,
        )
        .unwrap();
        // (:wat::holon::Bundle a b c) — the list elements are spliced in.
        match &forms[0] {
            WatAST::List(items, _) => {
                assert_eq!(items.len(), 4);
                assert!(matches!(&items[0], WatAST::Keyword(k, _) if k == ":wat::holon::Bundle"));
                assert!(matches!(&items[1], WatAST::Symbol(i, _) if i.as_str() == "a"));
                assert!(matches!(&items[2], WatAST::Symbol(i, _) if i.as_str() == "b"));
                assert!(matches!(&items[3], WatAST::Symbol(i, _) if i.as_str() == "c"));
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
              `(:wat::holon::Atom ,x))
            (:my::outer 42)
            "#,
        )
        .unwrap();
        // (:wat::holon::Atom 42) after fixpoint.
        match &forms[0] {
            WatAST::List(items, _) => {
                assert_eq!(items.len(), 2);
                assert!(matches!(&items[0], WatAST::Keyword(k, _) if k == ":wat::holon::Atom"));
                assert!(matches!(items[1], WatAST::IntLit(42, _)));
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
            WatAST::List(items, _) => items,
            _ => panic!("expected list"),
        };
        // (((tmp :i64) 1)) — drill through the bindings list, the
        // binding, and the typed-name pair to reach tmp.
        let bindings = match &list[1] {
            WatAST::List(bs, _) => bs,
            _ => panic!("expected bindings list"),
        };
        let first_binding = match &bindings[0] {
            WatAST::List(b, _) => b,
            _ => panic!("expected binding pair"),
        };
        let typed_name = match &first_binding[0] {
            WatAST::List(tn, _) => tn,
            _ => panic!("expected (name :Type) pair"),
        };
        let template_tmp = match &typed_name[0] {
            WatAST::Symbol(i, _) => i,
            _ => panic!("expected Symbol"),
        };
        // The body position's `tmp` — user-supplied argument, not macro-origin.
        let user_tmp = match &list[2] {
            WatAST::Symbol(i, _) => i,
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
              `(:wat::holon::Atom ,v))
            (:my::wrap some-var)
            "#,
        )
        .unwrap();
        let list = match &forms[0] {
            WatAST::List(items, _) => items,
            _ => panic!("expected list"),
        };
        let v_arg = match &list[1] {
            WatAST::Symbol(i, _) => i,
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
            let outer = if let WatAST::List(items, _) = f {
                items.clone()
            } else {
                panic!("expected list")
            };
            let bindings = if let WatAST::List(b, _) = &outer[1] {
                b.clone()
            } else {
                panic!()
            };
            let pair = if let WatAST::List(b, _) = &bindings[0] {
                b.clone()
            } else {
                panic!()
            };
            let typed_name = if let WatAST::List(tn, _) = &pair[0] {
                tn.clone()
            } else {
                panic!()
            };
            if let WatAST::Symbol(i, _) = &typed_name[0] {
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
        let forms = expand(r#"(:wat::holon::Atom "hello") 42 "world""#).unwrap();
        assert_eq!(forms.len(), 3);
        assert!(matches!(forms[1], WatAST::IntLit(42, _)));
        assert!(matches!(&forms[2], WatAST::StringLit(s, _) if s == "world"));
    }

    // ─── Nested quasiquote — arc 029 slice 1 ────────────────────────────

    /// Helper: find the `:wat::core::quasiquote` body inside a
    /// `(:wat::core::defmacro ...)` form. Used by nested-quasi tests
    /// to assert the generated macro's body.
    fn find_defmacro_body(form: &WatAST) -> &WatAST {
        match form {
            WatAST::List(items, _) => {
                assert!(matches!(&items[0], WatAST::Keyword(k, _) if k == ":wat::core::defmacro"));
                // items[1] is the (name (param :T) ... -> :Ret) signature;
                // items[2] is the body — a (:wat::core::quasiquote ...).
                let body = &items[2];
                match body {
                    WatAST::List(b, _) => {
                        assert!(matches!(&b[0],
                            WatAST::Keyword(k, _) if k == ":wat::core::quasiquote"));
                        &b[1]
                    }
                    _ => panic!("expected quasiquote body"),
                }
            }
            _ => panic!("expected defmacro list"),
        }
    }

    /// Helper: assert `form` is `(:wat::core::unquote <arg>)` and
    /// return the inner arg.
    fn expect_unquote(form: &WatAST) -> &WatAST {
        match form {
            WatAST::List(items, _) if items.len() == 2 => {
                assert!(matches!(&items[0],
                    WatAST::Keyword(k, _) if k == ":wat::core::unquote"));
                &items[1]
            }
            _ => panic!("expected (:wat::core::unquote ...)"),
        }
    }

    #[test]
    fn nested_quasiquote_preserves_inner_unquote() {
        // Outer macro body contains a nested quasiquote with an
        // unquote referencing an INNER parameter (not bound at outer
        // expansion). The unquote should survive into the generated
        // defmacro's body.
        let forms = expand_keeping_defmacros(
            r#"
            (:wat::core::defmacro (:my::mkmac (name :AST<()>) -> :AST<()>)
              `(:wat::core::defmacro (,name (x :AST) -> :AST)
                 `(:wat::holon::Atom ,x)))
            (:my::mkmac :my::wrap)
            "#,
        )
        .unwrap();
        // After outer expansion: a defmacro registration for :my::wrap
        // whose body is (:wat::core::quasiquote (:wat::holon::Atom
        // (:wat::core::unquote x))) — the inner `,x` preserved.
        let body = find_defmacro_body(&forms[0]);
        // body = (:wat::holon::Atom (:wat::core::unquote x))
        let body_items = match body {
            WatAST::List(items, _) => items,
            _ => panic!("expected list body"),
        };
        assert_eq!(body_items.len(), 2);
        assert!(matches!(&body_items[0],
            WatAST::Keyword(k, _) if k == ":wat::holon::Atom"));
        let inner = expect_unquote(&body_items[1]);
        assert!(matches!(inner, WatAST::Symbol(i, _) if i.as_str() == "x"));
    }

    #[test]
    fn double_unquote_substitutes_at_outer_level() {
        // ,,X at depth 2: outer unquote drops to depth 1; inner
        // unquote at depth 1 substitutes X's outer binding. Result
        // is (:wat::core::unquote <value>) — the value sits wrapped
        // in an unquote that fires on the inner expansion pass.
        let forms = expand_keeping_defmacros(
            r#"
            (:wat::core::defmacro (:my::mkmac (v :AST<i64>) -> :AST<()>)
              `(:wat::core::defmacro (:my::configured -> :AST)
                 `(:wat::holon::Atom ,,v)))
            (:my::mkmac 42)
            "#,
        )
        .unwrap();
        let body = find_defmacro_body(&forms[0]);
        let body_items = match body {
            WatAST::List(items, _) => items,
            _ => panic!("expected list"),
        };
        assert_eq!(body_items.len(), 2);
        assert!(matches!(&body_items[0],
            WatAST::Keyword(k, _) if k == ":wat::holon::Atom"));
        // body_items[1] = (:wat::core::unquote 42) — the value
        // substituted at outer expansion.
        let inner = expect_unquote(&body_items[1]);
        assert!(matches!(inner, WatAST::IntLit(42, _)));
    }

    #[test]
    fn unquote_of_literal_returns_literal() {
        // Direct check on unquote_argument: if the arg is already a
        // concrete value (from a prior substitution pass), return
        // as-is. Supports the `,,X` two-pass resolution.
        let bindings = HashMap::new();
        // A literal int — not a symbol, no binding needed.
        let lit = WatAST::IntLit(99, Span::unknown());
        let out = unquote_argument(&lit, &bindings).unwrap();
        match out {
            WatAST::IntLit(n, _) => assert_eq!(n, 99),
            _ => panic!("expected IntLit"),
        }
        // A list — same path.
        let list = WatAST::List(
            vec![WatAST::IntLit(1, Span::unknown()), WatAST::IntLit(2, Span::unknown())],
            Span::unknown(),
        );
        let out = unquote_argument(&list, &bindings).unwrap();
        assert!(matches!(out, WatAST::List(_, _)));
    }

    #[test]
    fn unquote_splicing_at_depth_two_preserves() {
        // ,@X at depth 2: preserve the unquote-splicing wrapper,
        // walk X at depth 1. X is an inner-macro parameter, so
        // it should appear as-is (symbol) inside the preserved
        // wrapper.
        let forms = expand_keeping_defmacros(
            r#"
            (:wat::core::defmacro (:my::mkmac (name :AST<()>) -> :AST<()>)
              `(:wat::core::defmacro (,name (xs :AST) -> :AST)
                 `(:wat::holon::Bundle ,@xs)))
            (:my::mkmac :my::wrap)
            "#,
        )
        .unwrap();
        let body = find_defmacro_body(&forms[0]);
        let body_items = match body {
            WatAST::List(items, _) => items,
            _ => panic!("expected list"),
        };
        assert_eq!(body_items.len(), 2);
        assert!(matches!(&body_items[0],
            WatAST::Keyword(k, _) if k == ":wat::holon::Bundle"));
        // body_items[1] = (:wat::core::unquote-splicing xs)
        match &body_items[1] {
            WatAST::List(items, _) => {
                assert_eq!(items.len(), 2);
                assert!(matches!(&items[0],
                    WatAST::Keyword(k, _) if k == ":wat::core::unquote-splicing"));
                assert!(matches!(&items[1], WatAST::Symbol(i, _) if i.as_str() == "xs"));
            }
            _ => panic!("expected unquote-splicing wrapper"),
        }
    }

    // Note: ,,@X (double unquote-splicing) is NOT yet supported. The
    // combined shape (:wat::core::unquote (:wat::core::unquote-splicing X))
    // at depth 2 would need special-case handling that lets the outer
    // substitution hand a concrete list down to an outer-level splice
    // wrapper. `make-deftest`'s implementation uses `,,default-prelude`
    // (non-splicing double unquote) where the list value is placed as
    // deftest's prelude argument — the splicing happens inside deftest's
    // own template, not at make-deftest's level. If a future use case
    // forces `,,@`, extend `walk_template` to recognize
    // `(unquote (unquote-splicing X))` at depth 2 as "substitute + wrap
    // in unquote-splicing" (outer wrapper replaced by the inner).

    #[test]
    fn make_deftest_shaped_template_expands_through_two_passes() {
        // The canonical forcing case — a macro-generating-macro that
        // configures dims + mode + default-prelude and registers a
        // new macro; then the user calls the new macro.
        let forms = expand(
            r#"
            (:wat::core::defmacro
              (:my::make-mac
                (name :AST<()>)
                (dims :AST<i64>)
                (mode :AST<wat::core::keyword>)
                (extras :AST)
                -> :AST<()>)
              `(:wat::core::defmacro
                 (,name
                   (test-name :AST<()>)
                   (body :AST<()>)
                   -> :AST<()>)
                 `(:wat::holon::configured
                    ,test-name
                    ,,dims
                    ,,mode
                    ,,extras
                    ,body)))

            (:my::make-mac :my::tdef 1024 :error ((load-a) (load-b)))

            (:my::tdef :my::run-1 (body-expr))
            "#,
        )
        .unwrap();
        // After both expansions, the final form should be:
        // (:wat::holon::configured :my::run-1 1024 :error ((load-a) (load-b)) (body-expr))
        assert_eq!(forms.len(), 1);
        match &forms[0] {
            WatAST::List(items, _) => {
                assert_eq!(items.len(), 6);
                assert!(matches!(&items[0],
                    WatAST::Keyword(k, _) if k == ":wat::holon::configured"));
                assert!(matches!(&items[1],
                    WatAST::Keyword(k, _) if k == ":my::run-1"));
                assert!(matches!(&items[2], WatAST::IntLit(1024, _)));
                assert!(matches!(&items[3],
                    WatAST::Keyword(k, _) if k == ":error"));
                // items[4] = ((load-a) (load-b))
                match &items[4] {
                    WatAST::List(l, _) => assert_eq!(l.len(), 2),
                    _ => panic!("expected extras list"),
                }
                // items[5] = (body-expr)
                match &items[5] {
                    WatAST::List(l, _) => assert_eq!(l.len(), 1),
                    _ => panic!("expected body list"),
                }
            }
            _ => panic!("expected final list"),
        }
    }
}
