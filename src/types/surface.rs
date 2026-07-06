//! Arc 293.3-core — `parse_defsurface`.
//!
//! A surface declares a structural interface: a set of required named members
//! with types. Structs satisfy a surface by having (at least) those members
//! with assignable types (width subtyping). No `:satisfies`, no `:parent`,
//! no declaration at the use site.
//!
//! `parse_defsurface` mirrors `parse_defstruct` but is simpler: name + fields
//! only (no metadata-map), v1 monomorphic (no `<T>` type params shipped here).
//!
//! Arc 293.4a — member list now carries both Field members (`name <- :T` triples)
//! and Method members (`(name [args...] -> :R)` lists). `struct_satisfies_surface`
//! takes a `resolve_method` closure (supplied by the `check` layer at the call site
//! in `assignable`) so that Method satisfaction can consult `defn :T/<name>` sigs.

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
            let has_struct_field = struct_fields
                .iter()
                .any(|(fname, fty)| fname == mname && is_assignable(fty, mty));
            if has_struct_field {
                return true;
            }
            // Fall through to the method resolver (for foreign types with extend-type).
            if let Some((_, defn_ret)) = resolve_method(mname) {
                return is_assignable(&defn_ret, mty);
            }
            false
        }
        SurfaceMember::Method { name: mname, args: margs, ret: mret, .. } => {
            // Method member: a `defn :T/<name>` must exist with an assignable sig.
            // The resolver forms the key `":<T>/<name>"` from the candidate type context
            // and returns (defn_arg_types, defn_ret) from env.schemes.
            if let Some((defn_arg_types, defn_ret)) = resolve_method(mname) {
                // Return type must be assignable.
                if !is_assignable(&defn_ret, mret) {
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
                        if !is_assignable(defn_ty, member_ty) {
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

/// Split a bare Symbol name like `"make<T>"` into `("make", vec!["T"])`.
/// A name with no `<` returns `(name.to_owned(), vec![])`.
/// Returns `TypeError` (adapted from `split_name_and_type_params` in runtime.rs, which returns
/// `EvalBreak` — not reachable from surface.rs; STOP-1 copy per the brief).
fn split_method_name_type_params(name: &str, sig_span: &Span) -> Result<(String, Vec<String>), TypeError> {
    match name.find('<') {
        None => Ok((name.to_owned(), Vec::new())),
        Some(lt_index) => {
            if !name.ends_with('>') {
                return Err(TypeError {
                    span: sig_span.clone(),
                    kind: TypeErrorKind::MalformedDecl {
                        head: HEAD.into(),
                        reason: format!(
                            "method member name {:?} opens '<' but does not close '>'",
                            name
                        ),
                    },
                });
            }
            let bare = name[..lt_index].to_string();
            let inside = &name[lt_index + 1..name.len() - 1];
            let params: Vec<String> = inside
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            Ok((bare, params))
        }
    }
}

/// Parse a method-member list `(name [args...] -> :RetType)` into a `SurfaceMember::Method`.
///
/// Arc 293.4a — copies the shape of `parse_defprotocol_form`'s per-sig logic but adapted
/// to `TypeError` (STOP-3 resolution: no shared helper because `defprotocol` returns
/// `RuntimeError` while `defsurface` returns `TypeError`).
///
/// Arc 293.4e-pre.ii — method names with type params (`make<T>`) are now split via
/// `split_method_name_type_params` (a local copy of runtime.rs's `split_name_and_type_params`
/// adapted to `TypeError` — STOP-1 per the brief). The bare name is stored; `type_params`
/// carries the extracted param names (e.g. `["T"]`). Monomorphic methods store `vec![]`.
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
    // (name [args...] -> :RetType)
    // sig_items[0] = method name (Symbol)
    // sig_items[1] = argspec Vector [...]
    // sig_items[2] = -> (Symbol)
    // sig_items[3] = :RetType (Keyword)
    if sig_items.len() < 4 {
        return Err(TypeError {
            span: sig_span.clone(),
            kind: TypeErrorKind::MalformedDecl {
                head: HEAD.into(),
                reason: format!(
                    "method member sig must have at least 4 elements \
                     `(name [args] -> :R)`; got {}",
                    sig_items.len()
                ),
            },
        });
    }

    // Item 0: method name (bare Symbol, possibly with type params e.g. `make<T>`).
    // Arc 293.4e-pre.ii — split via split_method_name_type_params to extract the bare name
    // and any type_params; monomorphic names (no `<`) produce an empty type_params vec.
    let (method_name, type_params) = match &sig_items[0] {
        WatAST::Symbol(s, _) => split_method_name_type_params(s.as_str(), sig_span)?,
        other => {
            return Err(TypeError {
                span: other.span().clone(),
                kind: TypeErrorKind::MalformedDecl {
                    head: HEAD.into(),
                    reason: format!(
                        "method member first element must be a Symbol name; got {}",
                        other.variant_name()
                    ),
                },
            })
        }
    };

    // Item 1: argspec Vector — parse via parse_argspec_triples, keeping the full ArgSpec.
    // Arc 293 K0b — ALL binders in a surface method member MUST be typed (`name <- :Type`),
    // including `self`. A bare untyped binder (e.g. `[self]` without `<-`) is a MalformedDecl;
    // write `[self <- :TheSurface  …]` (the surface's own name as the self type).
    let args = match &sig_items[1] {
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
                    return Err(TypeError {
                        span: vec_span.clone(),
                        kind: TypeErrorKind::MalformedDecl {
                            head: HEAD.into(),
                            reason: format!(
                                "all binders in a surface method member must be typed (`name <- :Type`); \
                                 `self` must be written `[self <- :TheSurface  …]` (the surface's own \
                                 name); bare untyped binders (e.g. `[self]`) are not accepted in \
                                 method member `{}`",
                                method_name
                            ),
                        },
                    });
                }
            }
        }
        other => {
            return Err(TypeError {
                span: other.span().clone(),
                kind: TypeErrorKind::MalformedDecl {
                    head: HEAD.into(),
                    reason: format!(
                        "method member second element must be an argspec Vector `[...]`; got {}",
                        other.variant_name()
                    ),
                },
            })
        }
    };

    // Item 2: `->` arrow Symbol.
    match &sig_items[2] {
        WatAST::Symbol(s, _) if s.as_str() == "->" => {}
        other => {
            return Err(TypeError {
                span: other.span().clone(),
                kind: TypeErrorKind::MalformedDecl {
                    head: HEAD.into(),
                    reason: format!(
                        "expected `->` symbol after argspec in method member `{}`; got {}",
                        method_name,
                        other.variant_name()
                    ),
                },
            })
        }
    }

    // Item 3: `:RetType` keyword.
    let ret = match &sig_items[3] {
        WatAST::Keyword(k, _) => {
            super::parse_type_expr(k).map_err(|e| TypeError {
                span: sig_items[3].span().clone(),
                kind: TypeErrorKind::MalformedDecl {
                    head: HEAD.into(),
                    reason: format!(
                        "bad return type in method member `{}`: {}",
                        method_name, e
                    ),
                },
            })?
        }
        other => {
            return Err(TypeError {
                span: other.span().clone(),
                kind: TypeErrorKind::MalformedDecl {
                    head: HEAD.into(),
                    reason: format!(
                        "method member return type must be a keyword after `->` in `{}`; got {}",
                        method_name,
                        other.variant_name()
                    ),
                },
            })
        }
    };

    Ok(SurfaceMember::Method {
        name: method_name,
        args,
        ret,
        type_params, // Arc 293.4e-pre.ii — extracted by split_method_name_type_params above
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
    // Valid shape (arc 293 K0a — `:nature` is MANDATORY; the 3-arg bare-:features form is retired):
    //   (:wat::core::defsurface :Name :nature :<nature-root> :features [members])  — 5 args only
    // :nature is mandatory and MUST precede :features.
    // :features is MANDATORY — a member vector not introduced by :features is a MalformedDecl.
    if args.len() != 5 {
        return Err(TypeError {
            span: decl_span.clone(),
            kind: TypeErrorKind::MalformedDecl {
                head: HEAD.into(),
                reason: format!(
                    "expected (:wat::core::defsurface :Name :nature :<nature-root> :features [members]); \
                     :nature is mandatory — the bare :features-only form is retired; \
                     got {} args after head",
                    args.len()
                ),
            },
        });
    }

    let mut iter = args.into_iter();

    // Slot 0 — name keyword.
    let name_kw = iter.next().unwrap();
    let (name, _type_params) = super::parse_declared_name(HEAD, &name_kw, &decl_span)?;

    // Slot 1 — either `:nature` keyword or `:features` keyword.
    let next = iter.next().unwrap();
    let nature: Option<Nature> = match &next {
        WatAST::Keyword(k, _) if k == ":nature" => {
            // Slot 2 — nature-value keyword.
            let val_node = iter.next().unwrap();
            let nature_val = match &val_node {
                WatAST::Keyword(v, _) => {
                    match Nature::from_root_keyword(v.as_str()) {
                        Some(h) => h,
                        None => return Err(TypeError {
                            span: val_node.span().clone(),
                            kind: TypeErrorKind::MalformedDecl {
                                head: HEAD.into(),
                                reason: format!(
                                    ":nature value must be a nature-root symbol (:wat::core::Struct, :wat::core::Record, :wat::holon::Record, or :wat::kernel::Peer'); got {}",
                                    v
                                ),
                            },
                        }),
                    }
                },
                other => {
                    return Err(TypeError {
                        span: other.span().clone(),
                        kind: TypeErrorKind::MalformedDecl {
                            head: HEAD.into(),
                            reason: ":nature value must be a keyword (:wat::core::Struct, :wat::core::Record, or :wat::holon::Record)".into(),
                        },
                    });
                }
            };
            // Slot 3 — REQUIRE :features keyword (nature MUST precede features).
            let features_kw = iter.next().unwrap();
            match &features_kw {
                WatAST::Keyword(k, _) if k == ":features" => {}
                other => {
                    return Err(TypeError {
                        span: other.span().clone(),
                        kind: TypeErrorKind::MalformedDecl {
                            head: HEAD.into(),
                            reason: "expected :features clause after :nature value — \
                                     (:wat::core::defsurface :Name :nature :<kw> :features [members])"
                                .into(),
                        },
                    });
                }
            }
            Some(nature_val)
        }
        WatAST::Keyword(k, _) if k == ":features" => {
            // Arc 293 K0a — :nature is mandatory; the bare :features-only form is retired.
            return Err(TypeError {
                span: next.span().clone(),
                kind: TypeErrorKind::MalformedDecl {
                    head: HEAD.into(),
                    reason: "`:nature` is mandatory — found `:features` where `:nature` was expected; \
                             write (:wat::core::defsurface :Name :nature :<nature-root> :features [members])"
                        .into(),
                },
            });
        }
        other => {
            return Err(TypeError {
                span: other.span().clone(),
                kind: TypeErrorKind::MalformedDecl {
                    head: HEAD.into(),
                    reason: "expected :features clause (or :nature :<kw> :features) after surface name \
                             — (:wat::core::defsurface :Name :features [members]); \
                             a member vector not introduced by :features is a malformed declaration"
                        .into(),
                },
            });
        }
    };

    // The member-vector: the next arg after the :features keyword.
    let members_node = iter.next().unwrap();

    // Arc 293.4d-fix — STRUCTURAL invariant: the member vector is the LAST arg; nothing follows it.
    if let Some(extra) = iter.next() {
        return Err(TypeError {
            span: extra.span().clone(),
            kind: TypeErrorKind::MalformedDecl {
                head: HEAD.into(),
                reason: "unexpected form after the member vector — every surface member (a field \
                         `name <- :T` AND a method `(name [self] -> :ret)`) goes INSIDE the single \
                         `[...]` member vector; nothing follows it"
                    .into(),
            },
        });
    }

    let (member_items, member_span) = match members_node {
        WatAST::Vector(items, span) => (items, span),
        other => {
            return Err(TypeError {
                span: other.span().clone(),
                kind: TypeErrorKind::MalformedDecl {
                    head: HEAD.into(),
                    reason: "member-vector must be a Vector `[name <- :T ...]`".into(),
                },
            });
        }
    };

    // Arc 293.4a — walk member_items: List elements are Method members; everything else
    // is collected as field-triple sub-runs and parsed by parse_argspec_triples.
    let mut members = Vec::<SurfaceMember>::new();
    let mut field_items: Vec<WatAST> = Vec::new();

    for item in member_items {
        match item {
            WatAST::List(sig_items, sig_span) => {
                // Flush any accumulated field-triple items first.
                if !field_items.is_empty() {
                    flush_field_items(&field_items, &member_span, &mut members)?;
                    field_items.clear();
                }
                // Parse the method member.
                members.push(parse_method_member_sig(&sig_items, &sig_span)?);
            }
            other => {
                // Accumulate field-triple items (Symbol / Symbol("<-") / Keyword).
                field_items.push(other);
            }
        }
    }

    // Final flush for any trailing field-triple items.
    if !field_items.is_empty() {
        flush_field_items(&field_items, &member_span, &mut members)?;
    }

    Ok(TypeDef::Surface(SurfaceDef {
        name,
        type_params: vec![],
        members,
        nature,
    }))
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
