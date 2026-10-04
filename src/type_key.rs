//! A composite type key is structure: a head `Name`, a type variable, or an
//! application of a head to arguments. It is not the rendered text.

use std::sync::Arc;

use crate::scope::Name;
use crate::types::TypeExpr;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum TypeKey {
    Name(Name),
    Var(Arc<str>),
    Apply { head: Name, args: Vec<TypeKey> },
}

impl std::fmt::Display for TypeKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeKey::Name(n) => f.write_str(&name_as_keyword(n)),
            TypeKey::Var(v) => write!(f, ":{v}"),
            TypeKey::Apply { head, args } => {
                write!(f, "({}", name_as_keyword(head))?;
                for a in args {
                    write!(f, " {a}")?;
                }
                write!(f, ")")
            }
        }
    }
}

/// The keyword a diagnostic prints for a `Name`. Not a second identity.
pub(crate) fn name_as_keyword(n: &Name) -> String {
    if n.namespace() == wat_reader::identifier::BOUND_NAMESPACE {
        let name = n.name();
        if name.starts_with(':') {
            return name.to_string();
        }
        let mut s = String::with_capacity(name.len() + 1);
        s.push(':');
        s.push_str(name);
        return s;
    }
    crate::edn::render::ns_to_wat_path(n.namespace(), n.name())
}

/// A spelling becomes a key at the text boundary.
///
/// `Name::from_keyword` accepts a keyword with `::` — that is a `Name`.
/// A spelling with neither `::` nor `/` is a type variable (`:T`, `T`, `:i64`).
/// A `(` is rendered text: `None`, so an insert that still holds it panics
/// at the call site rather than being parsed back into a key.
pub(crate) fn type_key_from_spelling(s: &str) -> Option<TypeKey> {
    // Same hoist as registry lookup: a repeated spelling does not re-parse.
    std::thread_local! {
        static KEYS: std::cell::RefCell<std::collections::HashMap<String, Option<TypeKey>>> =
            std::cell::RefCell::new(std::collections::HashMap::new());
    }
    if let Some(hit) = KEYS.with(|slot| slot.borrow().get(s).cloned()) {
        return hit;
    }
    let built = type_key_from_spelling_uncached(s);
    KEYS.with(|slot| slot.borrow_mut().insert(s.to_string(), built.clone()));
    built
}

fn type_key_from_spelling_uncached(s: &str) -> Option<TypeKey> {
    if s.starts_with('(') {
        return None;
    }
    if let Some(name) = Name::from_keyword(s) {
        return Some(TypeKey::Name(name));
    }
    if !s.starts_with(':') && s.contains("::") {
        let mut kw = String::with_capacity(s.len() + 1);
        kw.push(':');
        kw.push_str(s);
        if let Some(name) = Name::from_keyword(&kw) {
            return Some(TypeKey::Name(name));
        }
    }
    if !s.contains("::") && !s.contains('/') {
        let bare = s.strip_prefix(':').unwrap_or(s);
        return Some(TypeKey::Var(Arc::from(bare)));
    }
    Name::enter(s).map(TypeKey::Name)
}

pub(crate) fn type_key_from_expr(t: &TypeExpr) -> Option<TypeKey> {
    match t {
        TypeExpr::Path(p) => type_key_from_spelling(p),
        TypeExpr::Parametric { head, args } => {
            let mut kw = String::with_capacity(head.len() + 1);
            if !head.starts_with(':') {
                kw.push(':');
            }
            kw.push_str(head);
            let head_name = Name::from_keyword(&kw).or_else(|| Name::enter(&kw))?;
            let mut out = Vec::with_capacity(args.len());
            for a in args {
                out.push(type_key_from_expr(a)?);
            }
            Some(TypeKey::Apply {
                head: head_name,
                args: out,
            })
        }
        TypeExpr::Fn { .. } | TypeExpr::Tuple(_) | TypeExpr::Var(_) => None,
    }
}

pub(crate) fn require_type_key_spelling(s: &str, site: &str) -> TypeKey {
    type_key_from_spelling(s).unwrap_or_else(|| {
        panic!("cannot build a TypeKey from a spelling at {site}: {s}");
    })
}

pub(crate) fn require_type_key_expr(t: &TypeExpr, site: &str) -> TypeKey {
    type_key_from_expr(t).unwrap_or_else(|| {
        panic!(
            "cannot build a TypeKey from a TypeExpr at {site}: {}",
            crate::check::format_type(t)
        );
    })
}

/// The head of an application, or the key itself.
pub(crate) fn type_key_base(k: &TypeKey) -> TypeKey {
    match k {
        TypeKey::Apply { head, .. } => TypeKey::Name(head.clone()),
        other => other.clone(),
    }
}
