//! Arc 293.3-core — `parse_defsurface`.
//!
//! A surface declares a structural interface: a set of required named members
//! with types. Structs satisfy a surface by having (at least) those members
//! with assignable types (width subtyping). No `:satisfies`, no `:parent`,
//! no declaration at the use site.
//!
//! `parse_defsurface` mirrors `parse_defstruct` but is simpler: name + fields only
//! (no metadata-map).
//!
//! Arc 293.4a — member list now carries both Field members (`name <- :T` triples)
//! and Method members (`(name [args...] -> :R)` lists). `struct_satisfies_surface`
//! takes a `resolve_method` closure (supplied by the `check` layer at the call site
//! in `assignable`) so that Method satisfaction can consult `defn :T/<name>` sigs.
//!
//! Arc 170 C2 — the surface itself may carry type params (`defsurface :Holds<T> …`,
//! parsed into `SurfaceDef.type_params` via `parse_declared_name`, same as
//! `defenum`/`defrecord`). A bare reference to one of those params in a member's
//! declared type (e.g. `-> :T`) is a PLACEHOLDER: `struct_satisfies_surface` treats it
//! as satisfied by any concrete type (`is_surface_type_param_ref`), and the concrete
//! per-satisfier binding is resolved separately — at `extend-type` registration time
//! (`register_extend_type_surface_impls`, runtime.rs) and at the surface-method
//! call site (`check.rs`'s surface-method-call arm). Monomorphic surfaces
//! (`type_params` empty) take the identity path throughout — unaffected.

use crate::ast::WatAST;
use crate::span::Span;

use super::{Nature, SurfaceDef, SurfaceMember, TypeDef, TypeExpr, TypeError, TypeErrorKind};

const HEAD: &str = ":wat::core::defsurface";

/// True iff `struct_fields` satisfies every member of `surface` (width-open: extras OK).
///
/// Row-polymorphic width subtyping for Field members: for each `Field { name, ty }` in
/// `surface.members`, `struct_fields` must contain `(fname, fty)` where `fname == name`
/// and `is_assignable(fty, ty)` holds. Extra fields in `struct_fields` are fine.
///
/// Arc 293.4a — Method members: a `Method { name, ret, .. }` is satisfied iff
/// `resolve_method(name)` returns a `(arg_types, defn_ret)` whose `defn_ret` is
/// assignable to the member's `ret`. The resolver is supplied by the caller (typically
/// built in `check::assignable` from `env.schemes`) so this module does not depend on
/// the check layer.
///
/// `is_assignable` is a caller-supplied check (typically `check::assignable`),
/// passed as a closure so this module does not depend on `check`.
pub fn struct_satisfies_surface<F, R>(
    struct_fields: &[(String, TypeExpr)],
    surface: &SurfaceDef,
    mut is_assignable: F,
    mut resolve_method: R,
) -> bool
where
    F: FnMut(&TypeExpr, &TypeExpr) -> bool,
    R: FnMut(&str) -> Option<(Vec<TypeExpr>, TypeExpr)>,
{
    surface.members.iter().all(|member| match member {
        SurfaceMember::Field { name: mname, ty: mty } => {
            // Arc 293.4d — Field member: satisfied by a struct field with an assignable type
            // OR by a `:<T>/<name>` accessor (method / extend-type) that returns an assignable
            // type. This lets a foreign type back a Field member with an extend-type method.
            //
            // Arc 170 C2 — a bare reference to one of the SURFACE's OWN type params (e.g. `:T`
            // for `Holds<T>`) is a placeholder, satisfied by ANY concrete type — the per-satisfier
            // binding is resolved separately, at the call site (`check.rs`'s surface-method-call
            // arm), not here. `is_surface_type_param_ref` is the identity-no-op when
            // `surface.type_params` is empty (monomorphic surfaces unaffected).
            let has_struct_field = struct_fields.iter().any(|(fname, fty)| {
                fname == mname
                    && member_type_satisfied(fty, mty, &surface.type_params, &mut is_assignable)
            });
            if has_struct_field {
                return true;
            }
            // Fall through to the method resolver (for foreign types with extend-type).
            if let Some((_, defn_ret)) = resolve_method(mname) {
                return member_type_satisfied(&defn_ret, mty, &surface.type_params, &mut is_assignable);
            }
            false
        }
        SurfaceMember::Method { name: mname, args: margs, ret: mret, .. } => {
            // Method member: a `defn :T/<name>` must exist with an assignable sig.
            // The resolver forms the key `":<T>/<name>"` from the candidate type context
            // and returns (defn_arg_types, defn_ret) from env.schemes.
            if let Some((defn_arg_types, defn_ret)) = resolve_method(mname) {
                // Return type must be assignable — UNLESS the surface declares it (in whole
                // or in part, e.g. `Address'<S,R>`) from one of its own type params (Arc 170
                // C2: a placeholder, satisfied by construction — `member_type_satisfied`
                // recurses through parametric shapes to find embedded placeholders).
                if !member_type_satisfied(&defn_ret, mret, &surface.type_params, &mut is_assignable) {
                    return false;
                }
                // If the surface member declared explicit arg-type constraints (non-empty
                // ArgSpec.fixed_params), check that the defn's arg types are assignable
                // per-position. An empty fixed_params means the surface only constrains
                // the return type (e.g. bare `[self]` with no annotation).
                // Arc 293 K0c — SKIP position 0 (self). A surface method's `self` is the receiver,
                // tautologically the surface; comparing it (`is_assignable(defn_self, surface_self)`)
                // re-enters satisfaction of the surface — which wrongly rejects, or, on a heavier
                // graph, recurses to a stack overflow. The real per-position constraints are args 1..;
                // self-to-self is never checked. (Bare `[self]` left `fixed_params` empty, masking this.)
                if margs.fixed_params.len() > 1 {
                    let member_arg_types: Vec<&TypeExpr> =
                        margs.fixed_params.iter().skip(1).map(|(_, ty)| ty).collect();
                    let defn_rest: Vec<&TypeExpr> = defn_arg_types.iter().skip(1).collect();
                    if defn_rest.len() < member_arg_types.len() {
                        return false;
                    }
                    for (defn_ty, member_ty) in defn_rest.iter().zip(member_arg_types.iter()) {
                        if !member_type_satisfied(defn_ty, member_ty, &surface.type_params, &mut is_assignable) {
                            return false;
                        }
                    }
                }
                true
            } else {
                false // no matching defn → not satisfied
            }
        }
    })
}

/// Arc 170 C2 — true iff `ty` is a bare `Path` naming one of the surface's OWN declared
/// type params (e.g. `Path(":T")` when `type_params` contains `"T"`). Such a reference is
/// a PLACEHOLDER in the surface's declaration — satisfied by any concrete type; the actual
/// per-satisfier binding is resolved at the call site, not during structural satisfaction.
/// Always `false` when `type_params` is empty (monomorphic surfaces — pure no-op).
fn is_surface_type_param_ref(ty: &TypeExpr, type_params: &[String]) -> bool {
    if type_params.is_empty() {
        return false;
    }
    match ty {
        TypeExpr::Path(p) => {
            let bare = p.strip_prefix(':').unwrap_or(p);
            type_params.iter().any(|tp| tp == bare)
        }
        _ => false,
    }
}

/// Arc 170 C2 (receiver-check gap fix) — true iff the RESOLVED `defn_ty` (a satisfier's
/// concrete method return/arg type, e.g. `(Address' :- [Echo::Op Echo::Reply])`) satisfies the
/// surface's RAW, unresolved `member_ty` (e.g. `Address'<S,R>` where `S`/`R` are the
/// surface's own type params — not real types).
///
/// `is_surface_type_param_ref` only recognizes a WHOLE-position bare placeholder (`-> :T`).
/// A member type that EMBEDS a placeholder inside a parametric shape (`-> Address'<S,R>`,
/// as opposed to `-> :T` directly) fell through to `is_assignable(defn_ty, member_ty)`,
/// which compares the resolved concrete type against the literal unresolved symbols `:S`/
/// `:R` — meaningless names with no type-def — and always fails. This is the divergence
/// between return-type resolution (check.rs's surface-method-call arm, which substitutes
/// `S,R` with the satisfier's concrete binding via `register_extend_type_surface_impls`'s
/// `rename()`) and receiver-satisfaction (this module), which never substituted.
///
/// The fix: walk `member_ty` structurally. At any node that is itself a placeholder
/// (`is_surface_type_param_ref`), accept ANY `defn_ty` at that position (the concrete
/// binding is resolved elsewhere, at the call site). At a `Parametric` node with a
/// concrete (non-placeholder) head, require the same head + arity in `defn_ty` and recurse
/// pairwise into the args. Everywhere else, fall back to `is_assignable` (unchanged
/// behavior for members with no placeholder at all — monomorphic surfaces, or a
/// generic surface's non-generic members, are byte-for-byte unaffected: the first
/// `is_surface_type_param_ref` check is identical to before, and the `Parametric` recursion
/// only fires when in the OLD code it would have `unify`'d/`is_assignable`'d bare symbol
/// paths that can never match a real type anyway).
fn member_type_satisfied<F>(
    defn_ty: &TypeExpr,
    member_ty: &TypeExpr,
    type_params: &[String],
    is_assignable: &mut F,
) -> bool
where
    F: FnMut(&TypeExpr, &TypeExpr) -> bool,
{
    if is_surface_type_param_ref(member_ty, type_params) {
        return true;
    }
    match (defn_ty, member_ty) {
        (
            TypeExpr::Parametric { head: dh, args: dargs },
            TypeExpr::Parametric { head: mh, args: margs },
        ) if dh == mh && dargs.len() == margs.len() => dargs
            .iter()
            .zip(margs.iter())
            .all(|(d, m)| member_type_satisfied(d, m, type_params, is_assignable)),
        _ => is_assignable(defn_ty, member_ty),
    }
}

/// Parse a method-member list `(name [args...] -> :RetType)` into a `SurfaceMember::Method`.
///
/// Arc 293.4a — per-sig parsing adapted to `TypeError` (`defsurface` returns `TypeError`).
///
/// A method with type params is spelled `(launch :- [S R] [args] -> ret)` — the `:-` binder,
/// peeled via `crate::types::peel_param_spec`. The bare name is stored; `type_params` carries
/// the extracted param names (e.g. `["S" "R"]`). Monomorphic methods store `vec![]`.
///
/// STONE reap-the-angle-machinery (arc 109) — the earlier inline `make<T>` angle spelling and
/// its dedicated splitter `split_method_name_type_params` are gone: `<` opening a type head
/// is a LEXER-level error since "annihilate the angle bracket" (verified directly —
/// `(make<T> [self] -> :T)` fails at `crates/wat-reader/src/parser.rs` before this parser ever
/// runs), so the `name_raw.contains('<')` branch that called it was unreachable, not merely
/// unexercised.
///
/// The argspec vector `[args...]` is parsed via `parse_argspec_triples`, keeping the full
/// `ArgSpec` (not flattened to `Vec<TypeExpr>`). Arc 293 K0b — ALL binders must be typed
/// (`name <- :Type`), including `self`. Bare untyped binders (e.g. `[self]` without `<-`)
/// are a `MalformedDecl`; write `[self <- :TheSurface  …]` (the surface's own name as the
/// self type). Typed args (e.g. `[self <- :Shape  n <- :i64]`) produce a populated `ArgSpec`
/// whose `fixed_params` are checked per-position at satisfaction time.
fn parse_method_member_sig(
    sig_items: &[WatAST],
    sig_span: &Span,
) -> Result<SurfaceMember, TypeError> {
    // A generic method member is `(launch :- [S R St Sh Lu] [args] -> ret)` — the `:-`
    // binder, γ-i's shape (siblings, NO parens around the params). The earlier inline
    // `make<T>` angle spelling is gone (arc 109 "annihilate the angle bracket" — see the
    // STOP-1 note on this function's doc comment).
    // sig_items[0] = method name (Symbol)
    // then EITHER `:- [T U ...]` (binder) OR nothing, followed by:
    //   argspec Vector [...], `->` (Symbol), :RetType.
    if sig_items.len() < 4 {
        return Err(TypeError::new(
            sig_span.clone(),
            TypeErrorKind::MalformedDecl {
                head: HEAD.into(),
                reason: format!(
                    "method member sig must have at least 4 elements \
                     `(name [args] -> :R)`; got {}",
                    sig_items.len()
                ),
            },
        ));
    }

    // Item 0: method name (a bare Symbol; type params, if any, come from the `:-` binder
    // below, never from the name itself).
    let name_raw = match &sig_items[0] {
        WatAST::Symbol(s, _) => s.as_str(),
        other => {
            return Err(TypeError::new(
                other.span().clone(),
                TypeErrorKind::MalformedDecl {
                    head: HEAD.into(),
                    reason: format!(
                        "method member first element must be a Symbol name; got {}",
                        other.variant_name()
                    ),
                },
            ))
        }
    };

    // Arc 109 stone "the last comma lives in a symbol" — the door is the `:-` binder
    // (γ-i's shape, already used by `defn`/`fn`: `src/function/metadata.rs::peel_type_binder`).
    // A BARE name followed by `:- [T U ...]` peels the binder; no binder → monomorphic
    // method, `type_params` stays empty.
    //
    // STONE-finish-the-param-spec (arc 109) — routed through the one door
    // (`crate::types::peel_param_spec`) rather than this site's own hand-rolled
    // `k == ":-"` test + `[Vector, rest @ ..]` peel (found as a TENTH instance of
    // the class this stone exists to close — not among the brief's original nine,
    // but the exact same shape, so the rune below would refuse it either way).
    let tail = &sig_items[1..];
    let has_marker = tail.first().is_some_and(crate::types::is_binder_marker);
    let (method_name, type_params, rest): (String, Vec<String>, &[WatAST]) =
        match crate::types::peel_param_spec(tail) {
            (Some(items), after) => {
                // Mirrors `src/function/metadata.rs::peel_type_binder` (γ-i's shape) —
                // `!id.is_reference()` keeps only local binder names.
                let params: Vec<String> = items
                    .iter()
                    .filter_map(|item| match item {
                        WatAST::Symbol(id, _) if !id.is_reference() => Some(id.as_str().to_string()),
                        _ => None,
                    })
                    .collect();
                (name_raw.to_owned(), params, after)
            }
            (None, _) if has_marker => {
                return Err(TypeError::new(
                    tail[0].span().clone(),
                    TypeErrorKind::MalformedDecl {
                        head: HEAD.into(),
                        reason: format!(
                            "method member `{}`: `:-` binder must be followed by a \
                             `[T U ...]` Vector of type-param names",
                            name_raw
                        ),
                    },
                ));
            }
            (None, _) => (name_raw.to_owned(), Vec::new(), tail),
        };

    if rest.len() < 3 {
        return Err(TypeError::new(
            sig_span.clone(),
            TypeErrorKind::MalformedDecl {
                head: HEAD.into(),
                reason: format!(
                    "method member `{}` sig must have `[args] -> :R` after the name \
                     (and binder, if any); got {} remaining element(s)",
                    method_name,
                    rest.len()
                ),
            },
        ));
    }

    // Item 1 (of `rest`): argspec Vector — parse via parse_argspec_triples, keeping the
    // full ArgSpec. Arc 293 K0b — ALL binders in a surface method member MUST be typed
    // (`name <- :Type`), including `self`. A bare untyped binder (e.g. `[self]` without
    // `<-`) is a MalformedDecl; write `[self <- :TheSurface  …]` (the surface's own name
    // as the self type).
    let args = match &rest[0] {
        WatAST::Vector(items, vec_span) => {
            match crate::argspec::parse_argspec_triples(
                items,
                HEAD,
                vec_span,
                crate::argspec::ParseOptions { allow_rest_binder: false },
            ) {
                Ok(spec) => spec,
                Err(_) => {
                    // Arc 293 K0b — bare untyped binders are no longer accepted.
                    return Err(TypeError::new(
                        vec_span.clone(),
                        TypeErrorKind::MalformedDecl {
                            head: HEAD.into(),
                            reason: format!(
                                "all binders in a surface method member must be typed (`name <- :Type`); \
                                 `self` must be written `[self <- :TheSurface  …]` (the surface's own \
                                 name); bare untyped binders (e.g. `[self]`) are not accepted in \
                                 method member `{}`",
                                method_name
                            ),
                        },
                    ));
                }
            }
        }
        other => {
            return Err(TypeError::new(
                other.span().clone(),
                TypeErrorKind::MalformedDecl {
                    head: HEAD.into(),
                    reason: format!(
                        "method member second element must be an argspec Vector `[...]`; got {}",
                        other.variant_name()
                    ),
                },
            ))
        }
    };

    // Item 2 (of `rest`): `->` arrow Symbol.
    match &rest[1] {
        WatAST::Symbol(s, _) if s.as_str() == "->" => {}
        other => {
            return Err(TypeError::new(
                other.span().clone(),
                TypeErrorKind::MalformedDecl {
                    head: HEAD.into(),
                    reason: format!(
                        "expected `->` symbol after argspec in method member `{}`; got {}",
                        method_name,
                        other.variant_name()
                    ),
                },
            ))
        }
    }

    // Item 3 (of `rest`): the return TYPE.
    //
    // Arc 109 Stone ②-iii — a type node, not necessarily a keyword: a parametric return is
    // the form `(:wat::cache::Cache::GetResponse :- [V])` since the `:-` migration, and a
    // function return is the bracket `[arg… :-> ret]`. `parse_type_node` is the substrate's
    // one door for all four spellings — the same door the argspec slot above already uses,
    // which is why the ARGUMENT types migrated cleanly while the RETURN type did not.
    let ret = super::parse_type_node(&rest[2]).map_err(|e| TypeError::new(
        rest[2].span().clone(),
        TypeErrorKind::MalformedDecl {
            head: HEAD.into(),
            reason: format!(
                "bad return type in method member `{}`: {}",
                method_name, e
            ),
        },
    ))?;

    // Arc 278 #16 Stone 16.0 — OPTIONAL kwargs OPTIONS MAP after `-> :RetType`.
    // Everything past `rest[3]` is an order-INDEPENDENT sequence of
    // `:keyword value` PAIRS (kwargs — NOT positional; the design needs a second option
    // `:max-page-bytes` in a later stone, and a positional parse could not hold it). The
    // loop is a general kwargs reader: adding a recognized key later touches only the
    // match arm below, nothing structural.
    //
    // Recognized keys (only one today):
    //   `:max-request-bytes` → a positive i64 literal → `max_request_bytes`.
    // Absent from the map → DEFAULT_MAX_FRAME_BYTES (512 KiB) cast to i64.
    //
    // Every malformation is a LOCATED `MalformedDecl` (no-hidden-failures), matching the
    // surrounding surface-parse error shape: an UNKNOWN key, an odd tail (a key with no
    // value), a non-keyword where a key is expected, a DUPLICATE key, a non-i64 value, or
    // a value <= 0. (Enforcement / checker rule / codegen are a LATER stone — parse only.)
    let mut max_request_bytes: Option<i64> = None;
    let opts = &rest[3..];
    let mut i = 0usize;
    while i < opts.len() {
        // Option KEY — must be a keyword.
        let key = match &opts[i] {
            WatAST::Keyword(k, _) => k.as_str(),
            other => {
                return Err(TypeError::new(
                    other.span().clone(),
                    TypeErrorKind::MalformedDecl {
                        head: HEAD.into(),
                        reason: format!(
                            "method member `{}`: options after `-> :RetType` are `:keyword value` \
                             pairs; expected an option keyword, got {}",
                            method_name,
                            other.variant_name()
                        ),
                    },
                ))
            }
        };
        // Option VALUE — must be present (no dangling key at the tail).
        let val = opts.get(i + 1).ok_or_else(|| TypeError::new(
            opts[i].span().clone(),
            TypeErrorKind::MalformedDecl {
                head: HEAD.into(),
                reason: format!(
                    "method member `{}`: option `{}` has no value — options are `:keyword value` pairs",
                    method_name, key
                ),
            },
        ))?;
        match key {
            ":max-request-bytes" => {
                if max_request_bytes.is_some() {
                    return Err(TypeError::new(
                        opts[i].span().clone(),
                        TypeErrorKind::MalformedDecl {
                            head: HEAD.into(),
                            reason: format!(
                                "method member `{}`: duplicate option `:max-request-bytes`",
                                method_name
                            ),
                        },
                    ));
                }
                let n = match val {
                    WatAST::IntLit(n, _) => *n,
                    other => {
                        return Err(TypeError::new(
                            other.span().clone(),
                            TypeErrorKind::MalformedDecl {
                                head: HEAD.into(),
                                reason: format!(
                                    "method member `{}`: `:max-request-bytes` must be a positive \
                                     i64 literal; got {}",
                                    method_name,
                                    other.variant_name()
                                ),
                            },
                        ))
                    }
                };
                if n <= 0 {
                    return Err(TypeError::new(
                        val.span().clone(),
                        TypeErrorKind::MalformedDecl {
                            head: HEAD.into(),
                            reason: format!(
                                "method member `{}`: `:max-request-bytes` must be POSITIVE; got {}",
                                method_name, n
                            ),
                        },
                    ));
                }
                max_request_bytes = Some(n);
            }
            unknown => {
                return Err(TypeError::new(
                    opts[i].span().clone(),
                    TypeErrorKind::MalformedDecl {
                        head: HEAD.into(),
                        reason: format!(
                            "method member `{}`: unrecognized option `{}` — recognized options: \
                             `:max-request-bytes`",
                            method_name, unknown
                        ),
                    },
                ))
            }
        }
        i += 2;
    }
    // Arc 278 #16 Stone 16.3 — capture explicitness BEFORE defaulting: this is what
    // `synthesize_surface_protocol`'s mandatory-budget lock consults for `:nature :Peer'`
    // surfaces (a non-serviceable surface's methods legitimately ride the default forever).
    let max_request_bytes_explicit = max_request_bytes.is_some();
    // Unset → the DEFAULT_MAX_FRAME_BYTES (512 KiB) default, cast to i64.
    let max_request_bytes: i64 =
        max_request_bytes.unwrap_or(crate::edn::render::DEFAULT_MAX_FRAME_BYTES as i64);

    Ok(SurfaceMember::Method {
        name: method_name,
        args,
        ret,
        type_params, // Arc 293.4e-pre.ii — extracted by split_method_name_type_params above
        max_request_bytes, // Arc 278 #16 Stone 16.0 — kwargs option `:max-request-bytes N` (default: 512 KiB)
        max_request_bytes_explicit, // Arc 278 #16 Stone 16.3 — was the key actually written?
    })
}

/// Parse a `(:wat::core::defsurface :Name [name <- :T ...])` declaration.
///
/// Positional form after the head keyword:
///   args[0]  — name keyword (e.g. `:geo::Shape`)
///   args[1]  — member-vector `[name <- :T ...]` (WatAST::Vector)
///
/// Empty member list is legal (zero-member surface — every struct satisfies it).
///
/// Arc 293.4a — the member vector may now mix field triples `name <- :T`
/// (sequences of Symbol, Symbol("->"), Keyword) and method lists
/// `(name [args...] -> :RetType)` (WatAST::List elements). The walker groups
/// consecutive non-List items as field-triple sub-runs and passes each to
/// `parse_argspec_triples`; List items are parsed as Method members.
pub(crate) fn parse_defsurface(args: Vec<WatAST>, decl_span: Span) -> Result<TypeDef, TypeError> {
    // Valid shape (arc 293 K0a + arc 278 S4c):
    //   (:wat::core::defsurface :Name :nature :<nature-root> [:messages [msgs]] :features [members])
    // :nature is MANDATORY and MUST precede :messages/:features.
    // :messages is OPTIONAL and, when present, MUST precede :features. It is FORBIDDEN unless the
    //   surface's nature is `:wat::kernel::Peer` — a peer surface OWNS its protocol
    //   `defrecord`/`defenum` forms so a `:satisfies` service can ship them across a process fork
    //   (arc 278 S4c). The forms are registered (as external defrecords are) by `register_types`;
    //   here we only parse the clause, validate it, and validate feature/message completeness.
    // :features is MANDATORY — a member vector not introduced by :features is a MalformedDecl.
    let mut iter = args.into_iter().peekable();

    // Slot 0 — name keyword.
    let name_kw = iter.next().ok_or_else(|| TypeError::new(
        decl_span.clone(),
        TypeErrorKind::MalformedDecl {
            head: HEAD.into(),
            reason: "expected :Name after (:wat::core::defsurface ...)".into(),
        },
    ))?;
    let (name, name_params) = super::parse_declared_name(HEAD, &name_kw, &decl_span)?;
    let type_params = super::take_declared_binder(HEAD, name_params, name_kw.span(), &mut iter)?;

    // `:nature :<root>` — MANDATORY.
    let next = iter.next().ok_or_else(|| TypeError::new(
        decl_span.clone(),
        TypeErrorKind::MalformedDecl {
            head: HEAD.into(),
            reason: "`:nature` is mandatory — write \
                     (:wat::core::defsurface :Name :nature :<nature-root> :features [members])"
                .into(),
        },
    ))?;
    match &next {
        WatAST::Keyword(k, _) if k == ":nature" => {}
        WatAST::Keyword(k, _) if k == ":features" => {
            return Err(TypeError::new(
                next.span().clone(),
                TypeErrorKind::MalformedDecl {
                    head: HEAD.into(),
                    reason: "`:nature` is mandatory — found `:features` where `:nature` was expected; \
                             write (:wat::core::defsurface :Name :nature :<nature-root> :features [members])"
                        .into(),
                },
            ));
        }
        other => {
            return Err(TypeError::new(
                other.span().clone(),
                TypeErrorKind::MalformedDecl {
                    head: HEAD.into(),
                    reason: "expected `:nature :<kw>` after the surface name".into(),
                },
            ));
        }
    }
    // nature value keyword.
    let val_node = iter.next().ok_or_else(|| TypeError::new(
        decl_span.clone(),
        TypeErrorKind::MalformedDecl {
            head: HEAD.into(),
            reason: ":nature needs a value keyword".into(),
        },
    ))?;
    let nature_val = match &val_node {
        WatAST::Keyword(v, _) => match Nature::from_root_keyword(v.as_str()) {
            Some(h) => h,
            None => return Err(TypeError::new(
                val_node.span().clone(),
                TypeErrorKind::MalformedDecl {
                    head: HEAD.into(),
                    reason: format!(
                        ":nature value must be a nature-root symbol (:wat::core::Struct, :wat::core::Record, :wat::holon::Record, or :wat::kernel::Peer); got {}",
                        v
                    ),
                },
            )),
        },
        other => {
            return Err(TypeError::new(
                other.span().clone(),
                TypeErrorKind::MalformedDecl {
                    head: HEAD.into(),
                    reason: ":nature value must be a keyword (:wat::core::Struct, :wat::core::Record, :wat::holon::Record, or :wat::kernel::Peer)".into(),
                },
            ));
        }
    };

    // OPTIONAL `:messages [ <defrecord/defenum forms> ]` — arc 278 S4c.
    // Collect the declared message TYPE NAMES (form's slot-1 keyword) for the completeness check.
    let mut message_names: Vec<String> = Vec::new();
    let mut messages_span: Option<Span> = None;
    if matches!(iter.peek(), Some(WatAST::Keyword(k, _)) if k == ":messages") {
        let msg_kw = iter.next().unwrap();
        messages_span = Some(msg_kw.span().clone());
        // Arc 278 S4c — `:messages` is ONLY meaningful on a peer surface: it holds the wire
        // protocol a `:satisfies` service ships across a process fork. On any aggregate nature
        // (Struct/Record/HolonRecord) there is no protocol to own → FORBIDDEN (located error).
        if nature_val != Nature::Peer {
            return Err(TypeError::new(
                msg_kw.span().clone(),
                TypeErrorKind::MalformedDecl {
                    head: HEAD.into(),
                    reason: format!(
                        ":messages is permitted ONLY on a :nature :wat::kernel::Peer surface \
                         (it holds the peer's own protocol records/enums so a :satisfies service \
                         ships them across a process fork); surface {} has :nature {} — remove :messages",
                        name,
                        nature_val.root_keyword()
                    ),
                },
            ));
        }
        let msg_vec = iter.next().ok_or_else(|| TypeError::new(
            decl_span.clone(),
            TypeErrorKind::MalformedDecl {
                head: HEAD.into(),
                reason: ":messages needs a `[ <defrecord/defenum forms> ]` vector".into(),
            },
        ))?;
        let msg_items = match msg_vec {
            WatAST::Vector(items, _) => items,
            other => {
                return Err(TypeError::new(
                    other.span().clone(),
                    TypeErrorKind::MalformedDecl {
                        head: HEAD.into(),
                        reason: ":messages value must be a Vector `[ (defrecord …) (defenum …) … ]`".into(),
                    },
                ));
            }
        };
        for m in &msg_items {
            // Each message is a type-decl `(<head> :Name …)`; slot-1 keyword is the name. Post
            // the arc-294 flip, defrecord/defstruct expand to a `(do (recordtype :Name …) …)`
            // companion, so unwrap a leading `do` to its declaration child first.
            if let WatAST::List(mi, _) = unwrap_message_decl(m) {
                if let Some(WatAST::Keyword(mn, _)) = mi.get(1) {
                    message_names.push(mn.clone());
                }
            }
        }

        // Arc 278 S4c WALL 2 — TRANSITIVE completeness of the `:messages` block itself.
        // The direct check below verifies a FEATURE's `req <-`/`-> ret` types are declared; this
        // walks EACH message TypeDef's OWN referenced type paths (a Response enum's variant payload
        // records, a Request record's non-primitive field types, …) and requires every non-stdlib
        // reference to ALSO be declared in `:messages`. Because every message form is walked, a
        // required-membership check on each form's DIRECT refs closes the transitive graph: if A
        // references B, B must be in `:messages`; B's own refs are checked when B's form is walked.
        // Without this, a response enum referencing a user error record absent from `:messages` would
        // fail to resolve at the forked child's fresh startup (the fork-failure class, one level deeper).
        for m in &msg_items {
            let mut refs: Vec<String> = Vec::new();
            collect_message_form_type_refs(unwrap_message_decl(m), &mut refs);
            for r in refs {
                // Only namespaced user types are protocol messages; skip stdlib + type vars.
                if !r.contains("::") || r.starts_with(":wat::") {
                    continue;
                }
                if !message_is_declared(&message_names, &r) {
                    return Err(TypeError::new(
                        msg_kw.span().clone(),
                        TypeErrorKind::MalformedDecl {
                            head: HEAD.into(),
                            reason: format!(
                                "surface {} :messages type references {} which is not declared in \
                                 this surface's :messages — a peer surface that owns :messages must \
                                 declare EVERY non-stdlib type reachable from its protocol \
                                 records/enums (a response enum's variant payload record, a request \
                                 record's non-primitive field type, …), so a :satisfies service \
                                 ships them ALL across a process fork (arc 278 S4c). Add a \
                                 (defrecord {} …) to :messages, or remove the reference.",
                                name, r, r
                            ),
                        },
                    ));
                }
            }
        }
    }

    // Arc 278 S4c WALL 1 — `:messages` is MANDATORY on a peer surface (peer ⇔ has :messages).
    // A `:nature :wat::kernel::Peer` surface OWNS its request/response protocol; a `:satisfies`
    // service ships those `:messages` records/enums across a process fork so the forked child can
    // resolve them at its fresh startup. A peer surface with NO `:messages` clause has no protocol
    // to ship → the fork cannot carry a wire vocabulary → located compile error. (Non-peer natures
    // remain FORBIDDEN from `:messages`, enforced above; together: peer ⇔ has :messages.)
    if nature_val == Nature::Peer && messages_span.is_none() {
        return Err(TypeError::new(
            decl_span.clone(),
            TypeErrorKind::MalformedDecl {
                head: HEAD.into(),
                reason: format!(
                    "a :nature :Peer surface must declare :messages (its own request/response \
                     protocol records/enums) so a :satisfies service ships them across a process \
                     fork; surface {} has no :messages",
                    name
                ),
            },
        ));
    }

    // `:features [members]` — MANDATORY.
    let features_kw = iter.next().ok_or_else(|| TypeError::new(
        decl_span.clone(),
        TypeErrorKind::MalformedDecl {
            head: HEAD.into(),
            reason: "expected :features clause — \
                     (:wat::core::defsurface :Name :nature :<kw> [:messages [msgs]] :features [members])"
                .into(),
        },
    ))?;
    match &features_kw {
        WatAST::Keyword(k, _) if k == ":features" => {}
        other => {
            return Err(TypeError::new(
                other.span().clone(),
                TypeErrorKind::MalformedDecl {
                    head: HEAD.into(),
                    reason: "expected :features clause after :nature (and optional :messages) — \
                             (:wat::core::defsurface :Name :nature :<kw> [:messages [msgs]] :features [members])"
                        .into(),
                },
            ));
        }
    }

    // The member-vector: the next arg after the :features keyword.
    let members_node = iter.next().ok_or_else(|| TypeError::new(
        decl_span.clone(),
        TypeErrorKind::MalformedDecl {
            head: HEAD.into(),
            reason: ":features needs a `[members]` vector".into(),
        },
    ))?;

    // Arc 293.4d-fix — STRUCTURAL invariant: the member vector is the LAST arg; nothing follows it.
    if let Some(extra) = iter.next() {
        return Err(TypeError::new(
            extra.span().clone(),
            TypeErrorKind::MalformedDecl {
                head: HEAD.into(),
                reason: "unexpected form after the member vector — every surface member (a field \
                         `name <- :T` AND a method `(name [self] -> :ret)`) goes INSIDE the single \
                         `[...]` member vector; nothing follows it"
                    .into(),
            },
        ));
    }

    let nature: Option<Nature> = Some(nature_val);

    let (member_items, member_span) = match members_node {
        WatAST::Vector(items, span) => (items, span),
        other => {
            return Err(TypeError::new(
                other.span().clone(),
                TypeErrorKind::MalformedDecl {
                    head: HEAD.into(),
                    reason: "member-vector must be a Vector `[name <- :T ...]`".into(),
                },
            ));
        }
    };

    // Arc 293.4a — walk member_items: List elements are Method members; everything else
    // is collected as field-triple sub-runs and parsed by parse_argspec_triples.
    //
    // ⛔ Arc 109 Stone ②-iii — THE DISCRIMINATOR IS THE SLOT, NOT THE NODE KIND. A method
    // member `(name [args] -> :R)` is a List, and so is a parametric field TYPE
    // `(:wat::core::Vector :- [:wat::core::Error])` since the `:-` migration. Reading "List
    // ⇒ method" tore the type out of its own triple and handed it to
    // `parse_method_member_sig`, leaving `causes <-` as a two-item run — reported as
    // `triple is incomplete`, naming the FIELD as the defect when the type was fine.
    //
    // Position inside the current field run decides: a List at run-offset 0 opens a method
    // member; at offset 2 it fills the type slot. (Offset 1 is the arrow — a List there is
    // malformed, and `parse_argspec_triples` says so precisely, so it accumulates.)
    let mut members = Vec::<SurfaceMember>::new();
    let mut field_items: Vec<WatAST> = Vec::new();

    for item in member_items {
        let at_triple_start = field_items.len().is_multiple_of(3);
        match item {
            WatAST::List(sig_items, sig_span) if at_triple_start => {
                // Flush any accumulated field-triple items first.
                if !field_items.is_empty() {
                    flush_field_items(&field_items, &member_span, &mut members)?;
                    field_items.clear();
                }
                // Parse the method member.
                members.push(parse_method_member_sig(&sig_items, &sig_span)?);
            }
            other => {
                // Accumulate field-triple items (Symbol / arrow / type node — the type slot
                // holds a Keyword, a `wat.type/X` Symbol, a parametric List, or a
                // `[arg… :-> ret]` Vector; `parse_argspec_triples` reads all four).
                field_items.push(other);
            }
        }
    }

    // Final flush for any trailing field-triple items.
    if !field_items.is_empty() {
        flush_field_items(&field_items, &member_span, &mut members)?;
    }

    // Arc 278 S4c — COMPLETENESS: when a peer surface OWNS its messages (`:messages` present),
    // every user (non-`:wat::`) protocol type a feature method references — its request payload
    // (`req <-`, i.e. `args[1]`) and its response (`-> :T`) — MUST be declared in `:messages`.
    // A feature that names an undeclared, non-stdlib message type is the FORK-FAILURE class made a
    // located compile error: the forked child boots a universe of stdlib + the shipped `:messages`,
    // so any protocol type NOT in `:messages` is an unresolved reference at the child's startup.
    // (Type variables `:T` and stdlib `:wat::…` types are exempt; only namespaced user types are
    // checked — a message type always carries a `::`.)
    if let Some(msgs_span) = &messages_span {
        for m in &members {
            if let SurfaceMember::Method { name: mname, args, ret, .. } = m {
                let mut refs: Vec<String> = Vec::new();
                // request payload is the arg AFTER `self` (args[1]); `self` (args[0]) is the
                // surface itself and is never a message — exclude it.
                if let Some((_, req_ty)) = args.fixed_params.get(1) {
                    collect_user_type_paths(req_ty, &mut refs);
                }
                collect_user_type_paths(ret, &mut refs);
                for r in refs {
                    // Only namespaced user types are protocol messages; skip stdlib + type vars.
                    if !r.contains("::") || r.starts_with(":wat::") {
                        continue;
                    }
                    if !message_is_declared(&message_names, &r) {
                        return Err(TypeError::new(
                            msgs_span.clone(),
                            TypeErrorKind::MalformedDecl {
                                head: HEAD.into(),
                                reason: format!(
                                    "surface {} feature `{}` references protocol type {} which is not \
                                     declared in this surface's :messages — a peer surface that owns \
                                     :messages must declare EVERY non-stdlib request/response type it \
                                     uses, so a :satisfies service ships them across a process fork \
                                     (arc 278 S4c). Add a (defrecord {} …) to :messages, or remove the \
                                     reference.",
                                    name, mname, r, r
                                ),
                            },
                        ));
                    }
                }
            }
        }
    }

    Ok(TypeDef::Surface(SurfaceDef {
        name,
        type_params,
        members,
        nature,
    }))
}

/// Arc 278 S4c WALL 2 — walk a `:messages` type-decl form (`recordtype`/`defenum`, post-expansion)
/// and collect every type it references in a FIELD position. A field type is written `name <- :Type`
/// (both record fields and enum tagged-variant fields use this triple), so every `:Type` keyword
/// IMMEDIATELY following a `<-` symbol is a referenced type. Each is parsed via `parse_type_expr` and
/// its leaves collected with `collect_user_type_paths` (so `(:wat::core::Vector :- [my::ns::Thing])` yields
/// both the stdlib head and the user arg). Type NAMES (slot-1 keyword), enum variant keywords, and the
/// `:wat::enum::*` purity marker are NOT `<-`-preceded, so they are correctly skipped. Recurses into
/// every nested List/Vector so enum variant vectors are reached.
/// Arc 294 item 9a — a `:messages` decl is a defrecord/defstruct/defenum, and after the
/// aggregate flip a defrecord/defstruct EXPANDS to a kwargs-companion `(:wat::core::do
/// (:wat::core::recordtype :Name …) (:wat::core::defmacro :Name …))`. Unwrap a leading
/// `do` to its first declaration child so both the type NAME (slot-1 keyword) and the
/// field type refs are read from the recordtype/structtype/defenum, not the `do` head or
/// the companion defmacro. defenum (no companion) passes through unchanged.
fn unwrap_message_decl(form: &WatAST) -> &WatAST {
    if let WatAST::List(items, _) = form {
        if let Some(WatAST::Keyword(h, _)) = items.first() {
            if h == ":wat::core::do" {
                if let Some(child) = items.get(1) {
                    return unwrap_message_decl(child);
                }
            }
        }
    }
    form
}

fn collect_message_form_type_refs(form: &WatAST, out: &mut Vec<String>) {
    let children = match form {
        WatAST::List(items, _) => items,
        WatAST::Vector(items, _) => items,
        _ => return,
    };
    for (i, child) in children.iter().enumerate() {
        if let WatAST::Symbol(s, _) = child {
            if s.as_str() == "<-" {
                if let Some(WatAST::Keyword(k, _)) = children.get(i + 1) {
                    if let Ok(te) = super::parse_type_expr(k) {
                        collect_user_type_paths(&te, out);
                    }
                }
            }
        }
        // Recurse into nested collections (enum tagged-variant vectors, etc.).
        collect_message_form_type_refs(child, out);
    }
}

/// Arc 278 — is a referenced protocol type path DECLARED in this surface's `:messages`?
///
/// Originally, the two sides of this comparison spelled a PARAMETRIC message differently
/// (third in the series: `7336464e` box-svc<T>::Record, `10107da9` the flat type-arg split —
/// each one side normalized and the other not): the DECLARATION side stored the `:messages`
/// slot-1 keyword VERBATIM, params included — `":ns::Cache::GetRequest<K>"` — while the
/// REFERENCE side (a walked `TypeExpr`, via `collect_user_type_paths`) emitted a `Parametric`
/// leaf as its HEAD alone — `":ns::Cache::GetRequest"`. Raw string equality reported a
/// correctly-declared parametric message as undeclared, so this helper asked the question of
/// the BASE name on both sides instead.
///
/// STONE reap-the-angle-machinery (arc 109) — EXAMINED per the brief's STOP-3. The base-strip
/// (`split_type_params_pub`) is gone: the asymmetry above was CLOSED, not merely hidden, by
/// arc 109 "annihilate the angle bracket" — a message's own declared name can no longer embed
/// `<K>` either (a parametric message's params live in its own `:- [K]` binder, a separate form
/// slot: `(defrecord :ns::Cache::GetRequest :- [K] [...])`, verified against `wat/cache.wat`'s
/// live `Cache<K,V>` surface). So `message_names` (the raw declared name keyword) and
/// `referenced` (always base-only by construction) are now BOTH unconditionally bare, and
/// `split_type_params_pub(s).0` on a `<`-free `s` was already returning `s` itself — a no-op,
/// not a wrong answer (confirmed with a positive/negative probe pair: a matching parametric
/// message passes this wall, an undeclared one is still correctly refused). Plain equality is
/// what the strip degenerated to; this is now written directly as that.
///
/// Both walls (the direct feature-reference wall and WALL 2 — transitive completeness) route
/// through this ONE helper so the twins cannot drift.
fn message_is_declared(message_names: &[String], referenced: &str) -> bool {
    message_names.iter().any(|mn| mn == referenced)
}

/// Arc 278 S4c — collect the type-path leaves of a `TypeExpr` (for the surface `:messages`
/// completeness check). `Path` leaves and `Parametric` heads are emitted with a leading `:`
/// (matching how `:messages` names are written); container args are walked recursively.
fn collect_user_type_paths(t: &TypeExpr, out: &mut Vec<String>) {
    match t {
        TypeExpr::Path(p) => out.push(p.clone()),
        TypeExpr::Parametric { head, args } => {
            // Parametric heads are stored without the leading colon; re-add it for a uniform check.
            out.push(super::parametric_head_fqdn(head));
            for a in args {
                collect_user_type_paths(a, out);
            }
        }
        TypeExpr::Tuple(ts) => {
            for t in ts {
                collect_user_type_paths(t, out);
            }
        }
        TypeExpr::Fn { args, ret } => {
            for a in args {
                collect_user_type_paths(a, out);
            }
            collect_user_type_paths(ret, out);
        }
        TypeExpr::Var(_) => {}
    }
}

/// Pass a slice of field-triple items through `parse_argspec_triples` and append
/// the resulting `SurfaceMember::Field` entries to `members`.
fn flush_field_items(
    field_items: &[WatAST],
    member_span: &Span,
    members: &mut Vec<SurfaceMember>,
) -> Result<(), TypeError> {
    let argspec = crate::argspec::parse_argspec_triples(
        field_items,
        HEAD,
        member_span,
        crate::argspec::ParseOptions { allow_rest_binder: false },
    )
    .map_err(TypeError::from)?;

    for (id, ty) in argspec.fixed_params {
        members.push(SurfaceMember::Field {
            name: id.as_str().to_owned(),
            ty,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    //! Arc 278 #16 Stone 16.0 — the optional `:max-request-bytes N` annotation on a
    //! `:features` method member parses into `SurfaceMember::Method.max_request_bytes`.
    use super::*;
    use crate::edn::render::DEFAULT_MAX_FRAME_BYTES;

    /// Parse a single defsurface source into its `SurfaceDef` (strips the head keyword and
    /// routes through `parse_defsurface`, the same path `register_types` uses).
    fn parse_surface(src: &str) -> Result<SurfaceDef, TypeError> {
        let form = crate::parser::parse_one_with_file(src, "max_request_bytes_test")
            .expect("test source must read cleanly");
        let (items, span) = match form {
            WatAST::List(items, span) => (items, span),
            other => panic!("expected a defsurface List, got {}", other.variant_name()),
        };
        // Strip the head keyword (`:wat::core::defsurface`); parse_defsurface takes the rest.
        let args: Vec<WatAST> = items.into_iter().skip(1).collect();
        match parse_defsurface(args, span)? {
            TypeDef::Surface(s) => Ok(s),
            other => panic!("expected TypeDef::Surface, got {:?}", other),
        }
    }

    fn method_budget(surf: &SurfaceDef, name: &str) -> i64 {
        surf.members
            .iter()
            .find_map(|m| match m {
                SurfaceMember::Method { name: n, max_request_bytes, .. } if n == name => {
                    Some(*max_request_bytes)
                }
                _ => None,
            })
            .unwrap_or_else(|| panic!("no method member named {name:?}"))
    }

    #[test]
    fn declared_max_request_bytes_is_parsed_and_undeclared_defaults() {
        let surf = parse_surface(
            "(:wat::core::defsurface :t::Svc :nature :wat::core::Struct :features [\
               (write-logs [self <- :t::Svc] -> :t::Resp :max-request-bytes 300)\
               (stats [self <- :t::Svc] -> :t::Resp)])",
        )
        .expect("defsurface with a declared + an undeclared op must parse");

        // Declared op → its literal value.
        assert_eq!(
            method_budget(&surf, "write-logs"),
            300,
            "declared `:max-request-bytes 300` must land in max_request_bytes"
        );
        // Undeclared op → the DEFAULT_MAX_FRAME_BYTES (512 KiB) default, cast to i64.
        assert_eq!(
            method_budget(&surf, "stats"),
            DEFAULT_MAX_FRAME_BYTES as i64,
            "undeclared op must default to DEFAULT_MAX_FRAME_BYTES as i64"
        );
    }

    #[test]
    fn nonpositive_max_request_bytes_is_a_located_error() {
        let err = parse_surface(
            "(:wat::core::defsurface :t::Bad :nature :wat::core::Struct :features [\
               (write-logs [self <- :t::Bad] -> :t::Resp :max-request-bytes -5)])",
        )
        .expect_err("`:max-request-bytes -5` (non-positive) must be a LOCATED error, not silently accepted");
        // It is a MalformedDecl carrying the surface head (the surrounding surface-parse shape).
        match err.kind() {
            TypeErrorKind::MalformedDecl { .. } => {}
            other => panic!("expected a MalformedDecl for a non-positive budget; got {other:?}"),
        }
    }

    #[test]
    fn noninteger_max_request_bytes_is_a_located_error() {
        let err = parse_surface(
            "(:wat::core::defsurface :t::Bad2 :nature :wat::core::Struct :features [\
               (write-logs [self <- :t::Bad2] -> :t::Resp :max-request-bytes :nope)])",
        )
        .expect_err("`:max-request-bytes :nope` (non-integer) must be a LOCATED error");
        match err.kind() {
            TypeErrorKind::MalformedDecl { .. } => {}
            other => panic!("expected a MalformedDecl for a non-integer budget; got {other:?}"),
        }
    }

    #[test]
    fn unknown_option_key_is_a_located_error() {
        // An unrecognized kwargs key is NEVER silently ignored (no-hidden-failures) — this is
        // what keeps the options map extensible: a future stone adds `:max-page-bytes` to the
        // recognized set and this test's key stays rejected.
        let err = parse_surface(
            "(:wat::core::defsurface :t::Bad3 :nature :wat::core::Struct :features [\
               (write-logs [self <- :t::Bad3] -> :t::Resp :max-frobnicate 5)])",
        )
        .expect_err("an unrecognized option key must be a LOCATED error, not silently ignored");
        match err.kind() {
            TypeErrorKind::MalformedDecl { .. } => {}
            other => panic!("expected a MalformedDecl for an unknown option key; got {other:?}"),
        }
    }

    #[test]
    fn duplicate_max_request_bytes_is_a_located_error() {
        // The options loop is order-INDEPENDENT (not observable with a single recognized key);
        // a repeated key is a located error rather than a silent last-wins overwrite.
        let err = parse_surface(
            "(:wat::core::defsurface :t::Bad4 :nature :wat::core::Struct :features [\
               (write-logs [self <- :t::Bad4] -> :t::Resp :max-request-bytes 300 :max-request-bytes 400)])",
        )
        .expect_err("a duplicate `:max-request-bytes` must be a LOCATED error");
        match err.kind() {
            TypeErrorKind::MalformedDecl { .. } => {}
            other => panic!("expected a MalformedDecl for a duplicate option; got {other:?}"),
        }
    }

    // ── Arc 278 — the `:messages` completeness walls compare BASE names ──────────────────────
    //
    // Third in the series of "one side normalized, the other not" string comparisons
    // (`7336464e` box-svc<T>::Record, `10107da9` the flat type-arg split). The declaration side
    // stores `:messages` names VERBATIM (`":…::GetRequest<K>"`); the reference side walks a
    // `TypeExpr` and emits a `Parametric` leaf as its HEAD alone (`":…::GetRequest"`). Raw
    // equality therefore reported a correctly-declared PARAMETRIC message as undeclared.
    // `message_is_declared` asks the question of the BASE on both sides.

    #[test]
    fn parametric_message_is_recognized_as_declared() {
        // RED before the base-normalization, verbatim:
        //   malformed :wat::core::defsurface declaration: surface :t::Cache feature `get`
        //   references protocol type :t::Cache::GetRequest which is not declared in this
        //   surface's :messages …
        // — note the REPORTED name carries no `<K>` while `:messages` declares `GetRequest<K>`.
        // Arc 109 ③ — angle-bracket decl-names and references retired: `Head :- [args]`
        // siblings for a declaration's own name; `(Head :- [args])` in parens for a reference.
        let surf = parse_surface(
            "(:wat::core::defsurface :t::Cache :- [K V] :nature :wat::kernel::Peer \
               :messages \
               [(:wat::core::recordtype :t::Cache::GetRequest :- [K] [probes <- (:wat::core::Vector :- [K])]) \
                (:wat::core::defenum :t::Cache::GetResponse :- [V] :wat::enum::Pure \
                  :Ok              [results <- (:wat::core::Vector :- [(:wat::core::Option :- [V])])] \
                  :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64])] \
               :features \
               [(get [self <- (:t::Cache :- [K V])  req <- (:t::Cache::GetRequest :- [K])] \
                  -> (:t::Cache::GetResponse :- [V]) :max-request-bytes 1024)])",
        )
        .expect(
            "a surface whose :messages declare PARAMETRIC types must recognize a feature's \
             reference to them — the two sides spell the params differently, so membership is \
             asked of the BASE name",
        );
        assert_eq!(surf.type_params, vec!["K".to_string(), "V".to_string()]);
    }

    #[test]
    fn base_normalization_does_not_weaken_the_wall() {
        // The safety property has TWO halves. This is the second: base-normalization must not
        // turn the wall into a rubber stamp. A genuinely undeclared message — differing from a
        // declared one in its BASE, not merely its params — is still a located error.
        // Arc 109 ③ — angle-bracket decl-names and references retired (same spelling rule as
        // `parametric_message_is_recognized_as_declared` above).
        let err = parse_surface(
            "(:wat::core::defsurface :t::Cache2 :- [K] :nature :wat::kernel::Peer \
               :messages \
               [(:wat::core::recordtype :t::Cache2::GetRequest :- [K] [probes <- (:wat::core::Vector :- [K])])] \
               :features \
               [(get [self <- (:t::Cache2 :- [K])  req <- (:t::Cache2::PutRequest :- [K])] \
                  -> :t::Cache2::GetResponse :max-request-bytes 1024)])",
        )
        .expect_err(
            "a feature referencing a message whose BASE is absent from :messages must STILL be a \
             located error — base-normalization closes a spelling gap, it does not relax the wall",
        );
        match err.kind() {
            // Byte-identical, not a `contains` probe: the reason is a deterministic scalar, so
            // the whole of it is asserted (a loose check would pass on a wall that named the
            // WRONG message — precisely the failure mode base-normalization could introduce).
            TypeErrorKind::MalformedDecl { head, reason } => {
                assert_eq!(head, HEAD);
                assert_eq!(
                    reason,
                    "surface :t::Cache2 feature `get` references protocol type \
                     :t::Cache2::PutRequest which is not declared in this surface's :messages — \
                     a peer surface that owns :messages must declare EVERY non-stdlib \
                     request/response type it uses, so a :satisfies service ships them across a \
                     process fork (arc 278 S4c). Add a (defrecord :t::Cache2::PutRequest …) to \
                     :messages, or remove the reference."
                );
            }
            other => panic!("expected a MalformedDecl for an undeclared message; got {other:?}"),
        }
    }
}
