//! Arc 109 Stone 2 — the declare home's TYPEVAR phase.
//!
//! Split by PHASE, never by declaration FORM (see
//! `docs/arc/2026/04/109-kill-std/DESIGN-STONE-the-declare-home.md`). Free/bound
//! type-variable walking is a helper family neither `register.rs` nor `parse.rs` owns outright —
//! `parse.rs`'s `try_parse_*_def_fn_form` fns call [`collect_free_type_vars`] to build a
//! signature's `type_params`, but the walk itself (the three-lexical-classes var test) is its own
//! concern. `angle_minted_name_reason` ships here rather than with `register.rs`/`parse.rs`
//! because its only sibling at the two real call sites (`src/edn/render.rs`) is
//! [`angle_type_head_in_name`], already in this file — no in-module caller exists to measure it
//! by; placement follows the shared-purpose adjacency instead (see this stone's report for the
//! caller evidence). Moved verbatim out of `src/runtime.rs` (arc 109 Stone 2). Behaviour is
//! unchanged; only the location moved.
//!
//! Siblings: `register.rs` (populate the SymbolTable), `parse.rs` (read a declaration form's
//! shape), `preregister.rs` (the earlier stub-before-bodies pass).

use crate::declare::parse::is_type_var_path;
use crate::rust_deps::UseDeclarations;
use crate::types::{TypeEnv, TypeExpr};

/// Arc 109 — the lexer's type-head predicate, applied to a MINTED name.
///
/// `<` opens a type head only when preceded by an identifier character (`Vector<`,
/// `make<`, `Thread'<`). An operator `<` follows `::` or leads its token
/// (`:wat::core::<`, `<-`, `<=`) and never matches. This is the SAME predicate
/// `crates/wat-reader/src/lexer.rs` uses to refuse the spelling in SOURCE — the
/// wall now stands at both doors a name can come through, written or minted.
pub(crate) fn angle_type_head_in_name(s: &str) -> bool {
    let b = s.as_bytes();
    (1..b.len()).any(|i| {
        b[i] == b'<' && {
            let p = b[i - 1] as char;
            p.is_ascii_alphanumeric() || p == '_' || p == '\''
        }
    })
}

/// The one refusal message for a minted angle name, shared by both doors.
pub(crate) fn angle_minted_name_reason(name: &str) -> String {
    format!(
        "angle-bracket type parameters are illegal in a name (arc 109, \"annihilate the \
         angle bracket\") — and that holds for a name BUILT at expand time exactly as it \
         holds for one written in source: {name:?}. `:-` is the ONE parameterization \
         operator. A macro must emit the type-application FORM `(Head :- [A B])`, not \
         concatenate `Head` + \"<\" + args + \">\" into a keyword. A name is an atom; \
         structure encoded inside one has to be re-parsed by every consumer, and that \
         second parser is what this wall exists to make impossible."
    )
}

/// Stone 251.7 — collect free type-variable names from a function signature.
///
/// Walks each `TypeExpr` in `param_types` and `ret_type`, returning in
/// first-occurrence order (deduped) every type-variable name WITHOUT the
/// leading `:`.
///
/// **The var test (three-lexical-classes rule):** a `TypeExpr::Path(p)` is a
/// type variable iff, after stripping a leading `:`:
///   - the result contains neither `"::"` nor `'.'`   (bare, not FQDN), AND
///   - the result's first alphabetic character is **Uppercase**.
///
/// This includes `K`, `V`, `T`, `W`, `A`, `B`, … and excludes lowercase
/// legacy bare primitives (`:i64`, `:bool`, `:f64`, `:nil`) and FQDN
/// named types (`:wat::core::i64`, `:user::Foo`).
///
/// Recursion mirrors `check::rename`: `Parametric.args`, `Fn.args`,
/// `Fn.ret`, `Tuple` elements.  `Var(_)` is synthetic (never parsed) —
/// ignored.  `Path` with no match also ignored.
///
/// Arc 109 (param-spec-must-be-consumed) — the single recursive walk lives
/// in the free fn [`walk_free_type_vars`] below, hoisted out of this
/// function so [`collect_free_type_vars_in`] can share it without a second
/// walker. This function is now a thin wrapper: walk `param_types` via the
/// slice-taking sibling, then walk `ret_type` with the same accumulator.
pub(crate) fn collect_free_type_vars(
    param_types: &[crate::types::TypeExpr],
    ret_type: &crate::types::TypeExpr,
) -> Vec<String> {
    let mut seen = collect_free_type_vars_in(param_types);
    walk_free_type_vars(ret_type, &mut seen);
    seen
}

/// Arc 109 (param-spec-must-be-consumed) — sibling entry point over a plain
/// slice of `TypeExpr`, with no function-shaped `(param_types, ret_type)`
/// split. Used by the type-declaration consumption wall (`types.rs`,
/// `parse_type_decl`), which has no "return type" — every declared
/// `type_params` entry must appear somewhere in the def's member types
/// (fields, variants, inner/body/members), and this is where "somewhere"
/// is decided. Delegates to the same [`walk_free_type_vars`] recursion
/// [`collect_free_type_vars`] uses, so nested consumption
/// (`[x :- (Vector :- [T])]`) is handled identically in both callers —
/// deliberately not a second walker (stone 251.8a already collapsed four
/// hand-rolled versions of this question into one door).
pub(crate) fn collect_free_type_vars_in(types: &[crate::types::TypeExpr]) -> Vec<String> {
    let mut seen = Vec::new();
    for ty in types {
        walk_free_type_vars(ty, &mut seen);
    }
    seen
}

/// The one recursive walk shared by [`collect_free_type_vars`] and
/// [`collect_free_type_vars_in`]. See both callers' docs for the var test
/// and the recursion shape (`Parametric.args`, `Fn.args`/`Fn.ret`, `Tuple`
/// elements; `Var(_)` synthetic, ignored).
///
/// Arc 296 P-1 — the match lives in [`walk_type_expr`]; this is the free-var
/// visitor over that recursion, not a second walker.
fn walk_free_type_vars(ty: &TypeExpr, seen: &mut Vec<String>) {
    walk_type_expr(ty, &mut |p| {
        if is_type_var_path(p) {
            let name = p.strip_prefix(':').unwrap_or(p).to_string();
            if !seen.contains(&name) {
                seen.push(name);
            }
        }
    });
}

/// Arc 296 P-1 — the ONE recursion over a `TypeExpr` tree (`Path` / `Parametric.args` /
/// `Fn.args`+`Fn.ret` / `Tuple` elements; `Var(_)` synthetic, ignored).
/// [`walk_free_type_vars`] and [`first_unknown_named_type`] are visitors, not walkers.
fn walk_type_expr(ty: &TypeExpr, visit_path: &mut dyn FnMut(&str)) {
    match ty {
        TypeExpr::Path(p) => visit_path(p),
        TypeExpr::Parametric { args, .. } => {
            for a in args {
                walk_type_expr(a, visit_path);
            }
        }
        TypeExpr::Fn { args, ret } => {
            for a in args {
                walk_type_expr(a, visit_path);
            }
            walk_type_expr(ret, visit_path);
        }
        TypeExpr::Tuple(elements) => {
            for e in elements {
                walk_type_expr(e, visit_path);
            }
        }
        TypeExpr::Var(_) => {}
    }
}

/// Arc 296 P-2 prereq — first `TypeExpr::Path` in `ty` that is a NAMED type
/// and is in none of the membership stores:
/// `TypeEnv::contains` ∪ `is_builtin_primitive` ∪ `UseDeclarations::covers`
/// ∪ `TypeEnv::is_subtype_parent` (derive markers).
///
/// The `covers` arm was removed and restored (STOP-3): seeding only this
/// program's `use!` into TypeEnv (STOP-1) leaves stdlib `use!` of
/// `:rust::sqlite::*` in `use_decls` but not in `contains`. Type variables
/// (`is_type_var_path`) are accepted without asking any store.
pub(crate) fn first_unknown_named_type(
    ty: &TypeExpr,
    bound: &[String],
    env: &TypeEnv,
    use_decls: &UseDeclarations,
) -> Option<String> {
    let mut found = None;
    walk_type_expr(ty, &mut |p| {
        if found.is_some() {
            return;
        }
        if is_type_var_path(p) {
            return;
        }
        let stripped = p.strip_prefix(':').unwrap_or(p);
        if bound.iter().any(|b| b == stripped) {
            return;
        }
        if env.contains(p)
            || crate::runtime::is_builtin_primitive(stripped)
            || use_decls.covers(p)
            || env.is_subtype_parent(p)
        {
            return;
        }
        found = Some(p.to_string());
    });
    found
}

