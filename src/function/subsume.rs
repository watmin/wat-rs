//! Arc 109 Stone — the fn-form home's SUBSUME role: runtime type-matching for defclause dispatch.
//!
//! Split by ACT, never by declaration FORM (per DESIGN's one contract decision — see
//! `docs/arc/2026/04/109-kill-std/DESIGN-STONE-the-defclause-into-function-home.md`).
//! `declared_type_subsumes` is the reachability predicate `parse.rs`'s `parse_defclause_form`
//! uses to refuse a clause no input can ever reach; `value_matches_type_by_name` and
//! `val_type_path` are the runtime matcher `eval.rs`'s `select_defclause_clause` dispatches
//! through to pick a clause at call time. Moved verbatim out of `src/runtime.rs` (arc 109 the
//! defclause-into-function-home stone). Behaviour is unchanged; only the location moved.
//!
//! ★ Kept apart from `infer.rs` on purpose: these three are RUNTIME type-matching consulted at
//! clause-selection time, not check-time inference — `infer.rs` is the type-CHECKER's `infer_fn`,
//! a different tier entirely, and grouping this trio there on the strength of both saying "type"
//! would be exactly the FORM-shaped grouping the home's contract decision forbids.
//!
//! All three are `pub(in crate::function)`: `declared_type_subsumes` is called cross-file from
//! `parse.rs`; `value_matches_type_by_name`/`val_type_path` are called cross-file from `eval.rs` —
//! mirroring the scope `mod.rs`'s `FN_HEAD` constant and `parse.rs`'s `ParsedFnSignature` already
//! use for this same home-internal, non-external sharing.
//!
//! Siblings: `parse.rs` (fn-form + defclause parsers), `eval.rs` (fn-form + defclause
//! evaluators), `infer.rs` (check-tier inference), `metadata.rs` (binding-metadata peel).

use crate::types::Nature;
use crate::value::{SymbolTable, Value};

/// Stone 118.B2c strike 1 — does declared type `earlier` accept EVERY value that `later` accepts?
///
/// This is the REACHABILITY predicate the registration wall is built on, and it is the exact
/// mirror of [`value_matches_type_by_name`]: `earlier` subsumes `later` iff, for every value `v`,
/// `matches(v, later)` implies `matches(v, earlier)`.
///
/// ⚠ **SUBSUMPTION, NOT INTERSECTION — and the difference is the whole rule.** Two arms whose
/// domains merely intersect (an earlier concrete arm, a later catch-all) are a legitimate
/// FALLBACK: the later arm still fires for the rest of its domain, deterministically. That shape
/// is in production and documented — see `wat/bracket.wat:314-316`, which names first-match-wins,
/// calls the generic arm a "PERMISSIVE catch-all", and states that ordering is load-bearing.
/// Refusing intersection would outlaw it. Only CONTAINMENT means dead code.
///
/// CONSERVATIVE BY CONSTRUCTION. It answers `true` only for the cases where
/// `value_matches_type_by_name` is provably universal (a bare type-var Path, and the `_ => true`
/// arm covering fn/tuple/var types) or where the two types are literally identical. Anything
/// subtler — a Parametric head, a record supertype — answers `false`, so the wall refuses only
/// what is PROVABLY unreachable. A wall that guesses is worse than a wall that under-fires.
pub(in crate::function) fn declared_type_subsumes(earlier: &crate::types::TypeExpr, later: &crate::types::TypeExpr) -> bool {
    use crate::types::TypeExpr;
    // Identical declared types: trivially, the earlier arm takes every value the later would.
    if crate::check::format_type(earlier) == crate::check::format_type(later) {
        return true;
    }
    match earlier {
        // Mirrors the `is_type_var` early-return in `value_matches_type_by_name`: a bare
        // uppercase path with no `::`/`.` is a wildcard at dispatch and accepts anything.
        TypeExpr::Path(p) => {
            let s = p.strip_prefix(':').unwrap_or(p);
            !s.contains("::")
                && !s.contains('.')
                && s.chars()
                    .find(|c| c.is_alphabetic())
                    .is_some_and(|c| c.is_uppercase())
        }
        // Mirrors the matcher's `_ => true` arm (fn / tuple / var types are permissive).
        TypeExpr::Fn { .. } | TypeExpr::Tuple(_) | TypeExpr::Var(_) => true,
        // Everything else — concrete paths, parametrics — accepts a restricted set. Not proven
        // universal, so not proven to subsume.
        _ => false,
    }
}

/// Stone 237.2 — runtime type match for defclause dispatch.
///
/// Matches a concrete `Value` against a declared `TypeExpr`. Returns `true`
/// when the value's type is compatible with the declared type. This is the
/// runtime mirror of the check-time `unify(clause_arg_type, call_arg_type)`.
///
/// For typeunion types, we check if the value's type is a member of the union.
/// For plain Path types, we compare against the value's type_name.
pub(in crate::function) fn value_matches_type_by_name(
    val: &Value,
    ty: &crate::types::TypeExpr,
    sym: &SymbolTable,
) -> bool {
    match ty {
        crate::types::TypeExpr::Path(p) => {
            // Arc 251 Stone — bare uppercase paths are type-vars (same rule as
            // `collect_free_type_vars::is_type_var`): no `::` or `.` and first
            // alpha char is uppercase. Type-vars are wildcards at runtime
            // dispatch (the checker already validated the type; runtime is
            // defensive). Mirrors the `_ => true` arm below.
            let s = p.strip_prefix(':').unwrap_or(p);
            let is_type_var = !s.contains("::")
                && !s.contains('.')
                && s.chars()
                    .find(|c| c.is_alphabetic())
                    .is_some_and(|c| c.is_uppercase());
            if is_type_var {
                return true;
            }
            // Arc 259 S2c-ii.0 — a Record::def value's val_type_path() returns
            // the generic static ":wat::core::Record"; its SPECIFIC class lives in
            // class_fqdn. Dispatch on the specific class so a defclause keyed on
            // e.g. `:user::Tag` matches the corresponding record value.
            // All non-record values keep the existing val_type_path() comparison.
            match val {
                // Arc 293.R2.1 — Aggregate (record/struct): compare stripped path vs class.
                Value::Aggregate(a) => {
                    // p may carry a leading colon; strip it to compare bare FQDN.
                    let bare_p = p.strip_prefix(':').unwrap_or(p.as_str());
                    // Arc 278 — the RECORD-TOP must dispatch, or the runtime disagrees with the
                    // checker. `:wat::core::Record` roots every record for `is_subtype` (R7's
                    // record-top), so the checker ACCEPTS a call passing a concrete record to a
                    // param declared as the top — and this arm then refused it, because no real
                    // record's `class` is ever literally "wat::core::Record". The result was a
                    // program that type-checks and dies at runtime with `NoMatchingClause`,
                    // reporting `expected :wat::core::Record, got :wat::core::Record` (a declared
                    // TYPE against a concrete CLASS, which could never be equal).
                    //
                    // Dispatching on the specific class stays exactly as it was — that is what
                    // lets a clause keyed on `:user::Tag` match a `Tag`. This only ADDS the
                    // supertype, so it can never make a call that dispatches today stop
                    // dispatching; and the checker still gates which calls are legal at all.
                    bare_p == a.class.as_ref()
                        || (bare_p == "wat::core::Record" && a.nature != Nature::Struct)
                }
                _ => {
                    // Map the value's runtime type to its canonical type-keyword path.
                    let val_type = val_type_path(val);
                    if p.as_str() == val_type {
                        return true;
                    }
                    // Strike 2, the MONOMORPHIC half: a surface with no type params (e.g.
                    // `:wat::spawn::Locus`) names a top exactly as a parametric one does, and
                    // leaving it out would ship the fix with a seam. Same additive shape, same
                    // one door.
                    if let Some(types) = sym.types_deref() {
                        let surface = crate::types::parametric_head_fqdn(p);
                        if matches!(types.get(&surface), Some(crate::types::TypeDef::Surface(_)))
                            && crate::types::family_extends(val_type, &surface, types)
                        {
                            return true;
                        }
                    }
                    false
                }
            }
        }
        // Arc 118.2a — container-polymorphic defclause dispatch (`:wat::core::into`/
        // `reduce`/`filter`) needs clauses that differ ONLY in a Parametric container head
        // ((Vector :- [T]) vs (List :- [T]) vs (PersistentVector :- [T]) vs (Stream :- [T])) to actually discriminate
        // at runtime. Before this arc, EVERY defclause's competing clauses at a shared arity
        // differed only in bare Path types (i64 vs f64) — a Parametric param was always the
        // SOLE clause at that position, so the old unconditional `true` never mis-dispatched.
        // With multiple same-arity clauses differing in container kind, the permissive
        // fallback made the FIRST declared clause win regardless of the value's real shape
        // (silently wrong — e.g. a Stream fed to a `(Vector :- [T])` clause "matched" and ran the
        // Vector body unchanged). Fix: for container heads the seq-container registry knows
        // about, require the value's ACTUAL classification to agree with the declared head;
        // every other Parametric shape ((HashMap :- [K V]), (Option :- [T]), (Result :- [T E]), etc. — never
        // multi-clause-competing on container kind) keeps the old permissive behavior.
        crate::types::TypeExpr::Parametric { head, .. } => {
            // ── Stone 118.B2c strike 2 — A SURFACE IS THE CONTAINER-TOP ─────────────────────
            //
            // B1a (`eab12e05`) taught the CHECKER that a concrete instantiation satisfies a
            // parametric surface. This selector never learned it: the container match below
            // resolves the value to a `StreamContainer` and demands the declared head equal that
            // container's canonical name, so `wat::core::Seqable` could never match ANYTHING. A
            // `defclause` arm typed with a surface type-checked and then died at runtime with
            // `NoMatchingClause` — the checker and the runtime disagreeing about the same call.
            //
            // ★ THIS IS THE ARC-278 RECORD-TOP FIX, ONE ARM DOWN, and that fix's own comment
            // (just above) supplies the safety argument verbatim: "This only ADDS the supertype,
            // so it can never make a call that dispatches today stop dispatching; and the checker
            // still gates which calls are legal at all." That the same function needed this twice,
            // for two different tops, is the finding — the arm enumerates concrete heads, so every
            // new top arrives as a fresh instance of the same bug.
            //
            // ONE DOOR: `family_extends` (src/types.rs) is the CHECKER's own answer to
            // "does this type's family satisfy this surface", walking the `extend-type` edges
            // `register_subtype` laid down. The runtime now asks the same question of the same
            // registry instead of keeping a second, narrower opinion.
            if let Some(types) = sym.types_deref() {
                let surface = crate::types::parametric_head_fqdn(head);
                if matches!(types.get(&surface), Some(crate::types::TypeDef::Surface(_)))
                    && crate::types::family_extends(val_type_path(val), &surface, types)
                {
                    return true;
                }
            }
            use crate::collection::seq_container::StreamContainer;
            match StreamContainer::of_value(val) {
                Some(container) => {
                    let canonical_head = match container {
                        StreamContainer::Vector => "wat::core::Vector",
                        StreamContainer::List => "wat::core::List",
                        StreamContainer::PersistentVector => "wat::core::PersistentVector",
                        StreamContainer::Stream => "wat::stream::Stream",
                        StreamContainer::HashSet => "wat::core::HashSet",
                        // Tuple/WatAstList aren't declared via a Parametric head with a type
                        // arg the way the others are (Tuple is structural; WatAstList is the
                        // bare `:wat::WatAST` Path) — never competes at this arm; permissive.
                        StreamContainer::Tuple | StreamContainer::WatAstList => return true,
                    };
                    head.as_str() == canonical_head
                }
                // Not a seq-container value at all (HashMap, Option, Result, a bare fn, …) —
                // permissive fallback unchanged from before this arc.
                None => true,
            }
        }
        // For fn / tuple / var types: accept (permissive fallback, type-checker already
        // validated; runtime is defensive).
        _ => true,
    }
}

/// Map a runtime `Value` to its canonical type-keyword path for defclause dispatch.
pub(in crate::function) fn val_type_path(val: &Value) -> &'static str {
    match val {
        Value::i64(_) => ":wat::core::i64",
        Value::u8(_) => ":wat::core::u8",
        Value::f64(_) => ":wat::core::f64",
        Value::bool(_) => ":wat::core::bool",
        Value::String(_) => ":wat::core::String",
        Value::Unit => ":wat::core::nil",
        Value::wat__core__keyword(_) => ":wat::core::keyword",
        Value::wat__core__fn(_) => ":wat::core::fn",
        Value::wat__core__clauses(_) => ":wat::core::clauses",
        Value::wat__WatAST(_) => ":wat::WatAST",
        Value::holon__HolonAST(_) => ":wat::holon::HolonAST",
        Value::Vec(_) => ":wat::core::Vector",
        Value::Tuple(_) => ":wat::core::Tuple",
        Value::Option(_) => ":wat::core::Option",
        Value::Result(_) => ":wat::core::Result",
        // Arc 293.R2.1 — Aggregate: Struct nature → "<struct>" (dynamic class); others → ":wat::core::Record".
        // Arc 293 S3-Nature-2 — `Peer` is never the nature of a constructed `AggregateValue` (a peer is
        // a `RustOpaque`, not an aggregate); exhaustiveness only, unreachable at runtime.
        Value::Aggregate(a) => match a.nature {
            Nature::Struct => "<struct>",
            Nature::Record | Nature::HolonRecord => ":wat::core::Record",
            Nature::Peer => unreachable!("AggregateValue never carries Nature::Peer"),
        },
        Value::Enum(_) => "<enum>",
        // Arc 278 Stone A — foreign dynamic values dispatch on their own kind.
        Value::ForeignRecord(_) => ":wat::edn::ForeignRecord",
        Value::ForeignVariant(_) => ":wat::edn::ForeignVariant",
        Value::wat__std__HashMap(_) => ":wat::core::HashMap",
        Value::wat__core__PersistentMap(_) => ":wat::core::PersistentMap",
        Value::wat__core__PersistentVector(_) => ":wat::core::PersistentVector",
        Value::wat__std__HashSet(_) => ":wat::core::HashSet",
        // Arc 214 Stone 4.6a-i — peer RustOpaques carry their specific type_path
        // (e.g. ":wat::kernel::Thread" / ":wat::kernel::Process"); report it
        // so the defclause dispatcher sees the real peer type, not the generic fallback.
        // One authority: type_name() delegates to inner.type_path for RustOpaque;
        // val_type_path mirrors it as &'static str (inner.type_path IS &'static str).
        Value::RustOpaque(inner) => inner.type_path,
        Value::io__IOReader(_) => ":wat::io::IOReader",
        Value::io__IOWriter(_) => ":wat::io::IOWriter",
        Value::Vector(_) => ":wat::holon::Vector",
        Value::OnlineSubspace(_) => ":wat::holon::OnlineSubspace",
        Value::Reckoner(_) => ":wat::holon::Reckoner",
        Value::Engram(_) => ":wat::holon::Engram",
        Value::EngramLibrary(_) => ":wat::holon::EngramLibrary",
        Value::Hologram(_) => ":wat::holon::Hologram",
        Value::Instant(_) => ":wat::time::Instant",
        Value::Duration(_) => ":wat::time::Duration",
        Value::wat__core__Uuid(_) => ":wat::core::Uuid",
        // Stone 242.1 — renamed from :wat::core::Char to :wat::core::char
        // (scalar types lowercase per Doctrine 2).
        Value::wat__core__Char(_) => ":wat::core::char",
        // Arc 300 stone B — FQDN-only (mirrors Uuid, not the bare-primitive char).
        // Stone C1 lowercased the surface (Doctrine 2: scalar types are lowercase).
        Value::wat__core__Rational(_) => ":wat::core::rational",
        // Arc 300 stone C1 — arbitrary-precision integer.
        Value::wat__core__BigInt(_) => ":wat::core::bigint",
        Value::wat__core__List(_) => ":wat::core::List",
        Value::wat__stream__Stream(_) => ":wat::stream::Stream",
        Value::wat__kernel__Sender(_) => ":wat::kernel::Sender",
        Value::wat__kernel__Receiver(_) => ":wat::kernel::Receiver",
        Value::wat__kernel__HandlePool { .. } => ":wat::kernel::HandlePool",
        Value::wat__kernel__ChildHandle(_) => ":wat::kernel::ChildHandle",
        // Arc 232 Stone 232.1 — registry carriers (not dispatch-callable at runtime).
        Value::wat__core__extend_def(_) => ":wat::core::extend-def",
    }
}
