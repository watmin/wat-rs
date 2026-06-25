//! Arc 293.3-core — `parse_defsurface`.
//!
//! A surface declares a structural interface: a set of required named members
//! with types. Structs satisfy a surface by having (at least) those members
//! with assignable types (width subtyping). No `:satisfies`, no `:parent`,
//! no declaration at the use site.
//!
//! `parse_defsurface` mirrors `parse_defstruct` but is simpler: name + fields
//! only (no metadata-map), v1 monomorphic (no `<T>` type params shipped here).

use crate::ast::WatAST;
use crate::span::Span;

use super::{SurfaceDef, TypeDef, TypeExpr, TypeError, TypeErrorKind};

const HEAD: &str = ":wat::core::defsurface";

/// True iff `struct_fields` satisfies every member of `surface` (width-open: extras OK).
///
/// Row-polymorphic width subtyping: for each `(mname, mty)` in `surface.members`,
/// `struct_fields` must contain a `(fname, fty)` where `fname == mname` and
/// `is_assignable(fty, mty)` holds. Extra fields in `struct_fields` are fine.
///
/// `is_assignable` is a caller-supplied check (typically `check::assignable`),
/// passed as a closure so this module does not depend on `check`.
pub fn struct_satisfies_surface<F>(
    struct_fields: &[(String, TypeExpr)],
    surface: &SurfaceDef,
    mut is_assignable: F,
) -> bool
where
    F: FnMut(&TypeExpr, &TypeExpr) -> bool,
{
    surface.members.iter().all(|(mname, mty)| {
        struct_fields
            .iter()
            .any(|(fname, fty)| fname == mname && is_assignable(fty, mty))
    })
}

/// Parse a `(:wat::core::defsurface :Name [name <- :T ...])` declaration.
///
/// Positional form after the head keyword:
///   args[0]  — name keyword (e.g. `:geo::Shape`)
///   args[1]  — member-vector `[name <- :T ...]` (WatAST::Vector)
///
/// Empty member list is legal (zero-member surface — every struct satisfies it).
pub(crate) fn parse_defsurface(args: Vec<WatAST>, decl_span: Span) -> Result<TypeDef, TypeError> {
    if args.len() != 2 {
        return Err(TypeError {
            span: decl_span.clone(),
            kind: TypeErrorKind::MalformedDecl {
                head: HEAD.into(),
                reason: format!(
                    "expected (:wat::core::defsurface :Name [members]); got {} args after head",
                    args.len()
                ),
            },
        });
    }

    let mut iter = args.into_iter();

    // Slot 0 — name keyword.
    let name_kw = iter.next().unwrap();
    let (name, _type_params) = super::parse_declared_name(HEAD, &name_kw, &decl_span)?;

    // Slot 1 — member-vector.
    let members_node = iter.next().unwrap();
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

    let argspec = crate::argspec::parse_argspec_triples(
        &member_items,
        HEAD,
        &member_span,
        crate::argspec::ParseOptions { allow_rest_binder: false },
    )
    .map_err(TypeError::from)?;

    let members: Vec<(String, TypeExpr)> = argspec
        .fixed_params
        .into_iter()
        .map(|(id, ty)| (id.as_str().to_owned(), ty))
        .collect();

    Ok(TypeDef::Surface(SurfaceDef {
        name,
        type_params: vec![],
        members,
    }))
}
