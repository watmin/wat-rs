//! Fenced macro evaluator — DEFAULT-DENY pure-combinator restriction of `runtime::eval`.
//!
//! `macro_eval(form, env, sym)`:
//!   1. `validate_pure_total(form)?`  — walk the form; any keyword head not on the
//!      blessed allow-list (or not handled by the quote/quasiquote data-skip) → Err.
//!   2. `crate::runtime::eval(form, env, sym)` — run the already-validated form
//!      through the existing evaluator. No new interpreter, no duplicated arithmetic.
//!
//! # DEFAULT-DENY
//!
//! The allow-list is the PURE-TOTAL subset of `dispatch_keyword_head` /
//! `dispatch_keyword_head_value` (runtime.rs: fn dispatch_keyword_head / fn dispatch_keyword_head_value). Every head NOT
//! explicitly blessed is refused. This means:
//!   - a newly-added effectful prim is AUTOMATICALLY refused (the F5 class
//!     structurally cannot recur) — it stays denied until someone deliberately blesses it.
//!   - a forgotten pure head causes a false-refusal (a RED test), never a
//!     silently-admitted effect. The suite teaches completeness.
//!
//! # Skipped forms (data, not code)
//!
//! `(:wat::core::quote X)` is skipped entirely (X is pure literal data, never evaluated).
//!
//! `(:wat::core::quasiquote X)` is NOT simply skipped: `X` is a template that is
//! **data** (the literal parts) PLUS **code** (the `~`/`~@` unquote sub-expressions).
//! `validate_pure_total` descends into `X` tracking quasiquote depth (mirroring
//! `walk_quasiquote`'s depth logic): nested `quasiquote` bumps depth +1; an
//! `unquote` or `unquote-splicing` that *fires* at depth 1 means its sub-expression
//! is real code — `validate_pure_total` recurses into it. Literal template nodes
//! and material below depth 1 (inside a nested quasiquote) stay skipped (data).
//! This closes the F5-redux hole: an impure computed unquote in a program-body
//! quasiquote is refused before eval, not silently executed at expand time.
//!
//! # Load-bearing invariant — expand runs BEFORE user-defn registration
//!
//! `validate_pure_total` gates keyword *heads*. A blessed HOF (`map`/`foldl`/…)
//! takes a function ARG; an inline `(:wat::core::fn …)` lambda's body IS
//! validated (the walk recurses into it), but a bare reference to a top-level
//! user `defn` passed as that arg would NOT be head-checked. That vector is
//! closed not here but by the **freeze pipeline order**: `expand_all`
//! (freeze.rs:~745/751 — where this evaluator runs) precedes
//! `register_defines` (freeze.rs:~807), so **user/stdlib `defn`s are not yet
//! registered at expand time** — a reference to one does not resolve (it
//! errors), it cannot run an impure body. Only blessed builtins + inline
//! lambdas (body-validated) are reachable.
//!
//! **If that order ever changes** (defn-registration moved before macro
//! expansion), this evaluator alone no longer guarantees totality/purity — a
//! gate at `apply_function` (refuse applying a top-level user `defn` in macro
//! mode; the DESIGN's "gate 2") becomes necessary. Mark this dependency before
//! touching the freeze order.
//!
//! # Provenance
//!
//! Surface names (`macro_eval`, `validate_pure_total`, `RefusedInMacro`) were
//! RATIFIED by an intueri cast (arc 249.2b; rationale recorded in
//! `error.rs:51` at the `RefusedInMacro` variant). Arc 249 Stone 249.2b-i.
//!
//! Arc 249 Stone 249.2b-i — F5 is now CLOSED here (gated by this module).

use crate::ast::WatAST;
use crate::runtime::{Environment, SymbolTable};

use super::error::{MacroError, MacroErrorKind};

// ─── Public surface ──────────────────────────────────────────────────────────

/// Fenced expand-time evaluator: validate purity, then delegate to the
/// existing `runtime::eval`. Closes F5 (arc 249 finding): any impure keyword
/// head in `form` is refused before eval is invoked.
///
/// Returns `Err(MacroError)` (wrapping `MacroErrorKind::RefusedInMacro`) for
/// any keyword-headed sub-form whose head is not on the blessed allow-list.
/// Returns `Err(MacroError { kind: MalformedTemplate })` if the underlying
/// `runtime::eval` fails.
pub(crate) fn macro_eval(
    form: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<crate::value::TrackedValue, MacroError> {
    // Pre-walk validation — DEFAULT-DENY purity gate.
    validate_pure_total(form)?;
    macro_eval_pre_validated(form, env, sym)
}

/// Evaluate a form that has ALREADY been validated by `validate_pure_total`
/// at definition time (via `validate_macro_definition` in `parse_defmacro_form`).
///
/// WHY this variant exists: `expand_program_body` passes the immutable definition
/// body to `macro_eval`. That body was validated ONCE at definition time (the
/// hoist — arc 249 stone O). Re-running `validate_pure_total` on every invocation
/// would be redundant. Substituted forms in `unquote_argument` and
/// `splice_argument` are NOT definition-body forms — they carry fresh AST built
/// at call time and MUST be validated; those sites use `macro_eval` (with
/// validation).
///
/// INVARIANT: callers must guarantee the form was validated by
/// `validate_pure_total` before calling this function. The only sanctioned
/// call site is `expand_program_body`.
pub(super) fn macro_eval_pre_validated(
    form: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<crate::value::TrackedValue, MacroError> {
    // Delegate to the existing evaluator — no new interpreter.
    // Thread `e.span` (the runtime's precise failing-site span) into MacroError.
    // Arc 298.2: every span is real; always use e.span directly.
    crate::runtime::eval(form, env, sym).map_err(|e| {
        let span = e.span().clone();
        // Arc 258 Stone 258.2b: MacroAbort surfaces clean — user message only,
        // no "macro_eval: runtime::eval failed:" prefix noise.
        // Arc 296: non-MacroAbort failures carry the typed RuntimeError cause
        // instead of collapsing to a prose string.
        match e.kind() {
            crate::runtime::RuntimeErrorKind::MacroAbort { message } => MacroError {
                span,
                kind: MacroErrorKind::MalformedTemplate { reason: message.clone() },
            },
            _ => MacroError {
                span,
                kind: MacroErrorKind::MacroEvalRuntimeFailed { cause: Box::new(e) },
            },
        }
    })
}

// ─── Validator ───────────────────────────────────────────────────────────────

/// Walk `form` recursively. At each `List` with a `Keyword` head:
///   - head == `quote` → return Ok (skip contents; pure literal data, never evaluated)
///   - head == `quasiquote` → descend into the template tracking depth (see below)
///   - head is on the blessed allow-list → recurse into args
///   - otherwise → `Err(MacroError { kind: RefusedInMacro { head } })`
///
/// Non-List nodes and Lists without a Keyword head are walked recursively
/// (their children may contain keyword-headed sub-forms).
///
/// # Quasiquote depth tracking (mirrors `walk_quasiquote`)
///
/// A quasiquote template mixes data (literal template nodes) and code
/// (`~`/`~@` unquote sub-expressions). When `validate_pure_total` enters a
/// `(:wat::core::quasiquote X)`, it calls `validate_quasiquote_template(X, 1)`.
/// That helper walks `X` at depth 1:
///   - nested `(:wat::core::quasiquote …)` → bump depth +1, recurse (data below)
///   - `(:wat::core::unquote E)` or `(:wat::core::unquote-splicing E)` at depth 1
///     → `E` is real code; recurse `validate_pure_total(E)`
///   - same unquote forms at depth > 1 → peel depth, recurse (still inside a
///     nested quasiquote; E is data at this level)
///   - everything else → recurse `validate_quasiquote_template` (template data)
pub(super) fn validate_pure_total(form: &WatAST) -> Result<(), MacroError> {
    match form {
        WatAST::List(items, span) => {
            match items.first() {
                Some(WatAST::Keyword(head, _)) => {
                    // Pure literal data: skip entirely.
                    // Arc 294.b — `:wat::holon::literal` is also pure data; skip.
                    if head == ":wat::core::quote" || head == ":wat::holon::literal" {
                        return Ok(());
                    }
                    // Quasiquote: descend into the template with depth tracking.
                    // The template mixes data (literal nodes) and code (unquote
                    // sub-expressions); validate only the code parts.
                    if head == ":wat::core::quasiquote" {
                        if let Some(template) = items.get(1) {
                            validate_quasiquote_template(template, 1)?;
                        }
                        return Ok(());
                    }
                    // Check against the blessed allow-list.
                    if is_expand_time_legal(head) {
                        // BEWARE: `fn` forms are NOT blanket-opaque here. A fn in a
                        // program body can be INVOKED at expand time (blessed HOFs —
                        // map, foldl — take fns), so its body is expand-time code and
                        // MUST be pure-validated. Blanket fn-opacity reopens F5 (a
                        // kernel-send hidden in a HOF'd fn body would fire during
                        // expansion). Caught + reverted at the 245 long-tail scoring.
                        //
                        // THE ONE sound contextual exception: a literal fn form as
                        // the argument of `:wat::runtime::signature-of-fn`. That verb
                        // only CREATES the closure (pure) and reads its SIGNATURE —
                        // the body never executes on this path, so its heads (user
                        // fns destined for the expansion's runtime code) are not
                        // expand-time code. Shared macro-purity infra (arc 249) — not
                        // run-threads-specific, even though run-threads (since retired)
                        // was the originating consumer that reflected on its caller's
                        // coordinator/factory fns this way.
                        if head == ":wat::runtime::signature-of-fn" {
                            for child in items.iter().skip(1) {
                                let is_literal_fn = matches!(
                                    child,
                                    WatAST::List(fn_items, _)
                                        if matches!(
                                            fn_items.first(),
                                            Some(WatAST::Keyword(k, _)) if k == ":wat::core::fn"
                                        )
                                );
                                if !is_literal_fn {
                                    validate_pure_total(child)?;
                                }
                            }
                            return Ok(());
                        }
                        // Recurse into all args (skip the head keyword itself).
                        for child in items.iter().skip(1) {
                            validate_pure_total(child)?;
                        }
                        Ok(())
                    } else {
                        Err(MacroError {
                            span: span.clone(),
                            kind: MacroErrorKind::RefusedInMacro { head: head.clone() },
                        })
                    }
                }
                // List with non-Keyword head (e.g. a nested list, or no head):
                // recurse into all children.
                _ => {
                    for child in items {
                        validate_pure_total(child)?;
                    }
                    Ok(())
                }
            }
        }
        // Vectors: recurse into elements (may contain keyword-headed sub-forms).
        WatAST::Vector(items, _) => {
            for child in items {
                validate_pure_total(child)?;
            }
            Ok(())
        }
        // Arc 257 slice 1 — Map/Set literals are pure (recurse into k/v and elements).
        WatAST::Map(pairs, _) => {
            for (k, v) in pairs {
                validate_pure_total(k)?;
                validate_pure_total(v)?;
            }
            Ok(())
        }
        WatAST::Set(items, _) => {
            for item in items {
                validate_pure_total(item)?;
            }
            Ok(())
        }
        // Leaf nodes — no sub-forms to check.
        WatAST::IntLit(_, _)
        | WatAST::FloatLit(_, _)
        // Arc 300 stone B — rational literal is a leaf, same as int/float.
        | WatAST::RationalLit(_, _)
        // Arc 300 stone C1 — bigint literal is a leaf too.
        | WatAST::BigIntLit(_, _)
        // Arc 300 stone D — char literal is a leaf too.
        | WatAST::CharLit(_, _)
        | WatAST::BoolLit(_, _)
        | WatAST::StringLit(_, _)
        | WatAST::NilLit(_)
        | WatAST::Keyword(_, _)
        | WatAST::Symbol(_, _) => Ok(()),
    }
}

/// Depth-tracked walk of a quasiquote template, called by `validate_pure_total`
/// when it encounters `(:wat::core::quasiquote X)`. Mirrors the depth logic of
/// `walk_quasiquote` in runtime.rs:
///
///   - nested `quasiquote` → bump depth +1 (still inside data; recurse)
///   - `unquote` / `unquote-splicing` at depth 1 → E is real code; validate it
///   - same forms at depth > 1 → peel depth (still data within nested quasiquote)
///   - everything else → recurse at current depth (template data node)
///
/// Arc 249 Stone 249.3a — closes the F5-redux hole: an impure computed unquote
/// inside a program-body quasiquote is refused here, before `runtime::eval` runs.
///
/// rune:solvere(load-bearing-coupling) — qq depth-walk is mirrored in 3 sites
/// (walk_template / validate_quasiquote_template / walk_quasiquote); the depth
/// rule (nested +1, fire-at-depth-1, peel-deeper) is one contract that must
/// change in all three in sync; a unifying visitor would obscure three readable
/// single-purpose walkers.
fn validate_quasiquote_template(form: &WatAST, depth: u32) -> Result<(), MacroError> {
    match form {
        WatAST::List(items, _) => {
            if let Some(WatAST::Keyword(head, _)) = items.first() {
                // Nested quasiquote: bump depth, recurse into template.
                if head == ":wat::core::quasiquote" {
                    if let Some(inner) = items.get(1) {
                        validate_quasiquote_template(inner, depth + 1)?;
                    }
                    return Ok(());
                }
                // Unquote or unquote-splicing.
                if head == ":wat::core::unquote" || head == ":wat::core::unquote-splicing" {
                    if depth == 1 {
                        // Fires at this level: E is real code. Validate it fully.
                        if let Some(code) = items.get(1) {
                            validate_pure_total(code)?;
                        }
                    } else {
                        // Still inside a nested quasiquote: peel depth, stay in template mode.
                        if let Some(inner) = items.get(1) {
                            validate_quasiquote_template(inner, depth - 1)?;
                        }
                    }
                    return Ok(());
                }
            }
            // Plain list in the template: walk children at the same depth.
            for child in items {
                validate_quasiquote_template(child, depth)?;
            }
            Ok(())
        }
        WatAST::Vector(items, _) => {
            for child in items {
                validate_quasiquote_template(child, depth)?;
            }
            Ok(())
        }
        // Arc 257 slice 1 — Map/Set template nodes: walk all children at the same depth.
        WatAST::Map(pairs, _) => {
            for (k, v) in pairs {
                validate_quasiquote_template(k, depth)?;
                validate_quasiquote_template(v, depth)?;
            }
            Ok(())
        }
        WatAST::Set(items, _) => {
            for child in items {
                validate_quasiquote_template(child, depth)?;
            }
            Ok(())
        }
        // Leaf nodes in the template — no sub-forms to check.
        _ => Ok(()),
    }
}

// ─── Expand-time legality — DERIVED from the registry, a residue left ────────
//
// Decides EXPAND-TIME LEGALITY: which `dispatch_keyword_head` /
// `dispatch_keyword_head_value` heads (runtime.rs) may be CALLED from inside a
// `defmacro` body while it is being expanded. This is NOT "the pure-total
// subset" — arc 255 Stone expand-1 audited all 202 entries against their own
// registered `@Purity`/`@Determinism`/`@Totality` and found expand-time-legality
// is an independent property that purity and totality each bear on but
// neither one settles alone.
//
// Arc 255 Stone expand-T4b: this predicate now DERIVES its answer from each verb's
// OWN `@ExpandTime <Variant>` at its registration site (`registry().lookup_entry`) —
// the same shape `total-T4b` (`rete/purity.rs`'s `intrinsic_meta`) built one stone
// earlier for the totality axis. A `matches!` here duplicating a fact the registry
// already holds is exactly the shape 255.1c retired as "a gate reading a copy of the
// truth" (`intrinsic/mod.rs:988`, `[[feedback_a_gate_over_two_hand_lists_is_a_hand_list]]`):
//
//   Legal | Preserving   -> true.  `Preserving` means "I contribute no illegality of
//                           my own; my sub-forms carry theirs" — the same reading
//                           `pure`/`deterministic` already give it
//                           (`intrinsic/mod.rs:1038`'s `matches!(purity, Pure | Preserving)`).
//   RuntimeOnly           -> false. Denied by declaration. No verb has been ruled
//                           `RuntimeOnly` yet — this stone's DESIGN names ruling one
//                           as out of scope; that verdict needs a maker.
//   Unreviewed | None     -> the residue `matches!` below: a name that is either
//                           registered but not yet reviewed on this axis, or has no
//                           registration site at all — either way, unhomed.
//
// The historical findings that shaped which verbs are expand-time legal still apply.
// They are now facts the REGISTRY carries at each verb's own site, not a second copy
// kept here:
//
//   - Effectful heads (`:wat::kernel::*`, IO, spawning, time `now`, random
//     UUIDs, signal queries, `:wat::core::apply`, `:wat::core::eval-ast!`,
//     etc.) are denied by default — arc 255 Stone expand-1's audit found zero
//     exceptions across all 202 entries that were legal.
//   - `:wat::i64::/` (and `mod`/`rem`/`quot`) is legal despite being
//     `@Totality Partial` (undefined at a zero divisor). A partial verb can
//     still be expand-time-legal: dividing by zero during expansion raises a
//     deterministic, located MacroError — a compile-time failure instead of
//     a runtime one, which is strictly better. Totality and expand-time
//     legality are different axes.
//   - `:wat::core::fresh-symbol` and `:wat::kernel::macro-call-site` are legal
//     despite being Nondeterministic, and correctly so. `fresh-symbol` mints
//     a different capture-proof gensym on every call — that nondeterminism
//     IS what makes hygienic expansion possible. `macro-call-site` reads the
//     current invocation's own source span, stable for the duration of that
//     expansion. Neither makes a given macro SOURCE expand to different code
//     across runs.
//   - `:wat::hashmap::keys` / `:wat::hashmap::values` are legal, also
//     Nondeterministic, and also correctly — now annotated `@ExpandTime Legal`
//     at their registration site (`src/intrinsic/hashmap.rs`), not carried in
//     this file's residue. ⛔ arc 255 Stone expand-1 FIRST REMOVED THEM and
//     that was wrong; a stale copy of this very bullet then went on claiming
//     the removal had happened for a full stone afterward, until expand-T4a's
//     rider refused to transcribe a blessing this file's CODE gave while this
//     COMMENT denied it
//     (`[[feedback_a_walls_paperwork_can_claim_a_door_it_did_not_close]]`).
//     A `HashMap` is a pure data collection and `keys` is a pure PROJECTION of
//     it: the same map yields the same SET of keys every time, and only their
//     ORDER is unspecified ("deliberately NOT part of the contract" —
//     src/value/pmap.rs). The hazard people reach for this gate to prevent is a
//     macro whose EXPANSION varies between runs — but that is a property of a
//     USE, not of a verb, and a verb-level gate cannot tell the two apart.
//     ★ Measured: removing them made `:wat::core::format` (wat/core.wat:1639)
//     undefinable and took 247 of 415 targeted tests RED. And `format`'s use is
//     order-independent — the fold at wat/core.wat:1939 carries a `nil`
//     accumulator, its result is bound to `_unused-chk` and DISCARDED, and the
//     emitted code never references the keys. Only WHICH unused kwarg gets
//     named in an error could vary, and only for a program that is already
//     broken. The generated code is identical either way.
//     ⚠ Order-dependence IS a real hazard — `(foldl conj [] (keys m))` inside a
//     macro would emit varying code — but the honest instrument for it is
//     EXPANDING A MACRO TWICE AND COMPARING THE OUTPUT, which tests the
//     property where it actually lives, for every verb, without a curated list.
//     A determinism gate on the verb cannot see it and refuses the innocent.
//
// The suite still teaches completeness: a false-refusal (a legal head whose
// registration reads anything but `Legal`/`Preserving`, that ALSO has no arm in the
// residue below) makes a stdlib test RED — add it to the residue, or better, home it
// and annotate it `@ExpandTime Legal`/`Preserving` at its registration site. A missing
// effectful head is harmless: it defaults `Unreviewed`/`None` and is denied unless the
// residue explicitly admits it, which it never does for an effectful verb.
//
// rune:struere(invariant-coupling) — this predicate mirrors the expand-time-legal arm
// of fn dispatch_keyword_head / fn dispatch_keyword_head_value (runtime.rs); the suite
// enforces completeness (default-deny makes over-restriction the only drift direction).
fn is_expand_time_legal(head: &str) -> bool {
    if let Some(e) = crate::intrinsic::registry().lookup_entry(head) {
        // `matches!` does NOT go non-exhaustive when `ExpandTime` grows a variant — it
        // silently returns `false` — so a new pole is REFUSED here unless named
        // explicitly. `ExpandOnly` (arc 255 Stone expand-only-the-missing-pole) is an
        // expand-time-only verb, by definition legal inside a macro body (its ONLY
        // legitimate call site — `macro-error` is the one verb that declares it); omitting
        // it here would refuse that verb at the one place it is allowed to be called.
        return matches!(
            e.expand_time,
            wat_doc::ExpandTime::Legal | wat_doc::ExpandTime::ExpandOnly | wat_doc::ExpandTime::Preserving
        );
    }
    // ★ THE RESIDUE — arc 255 Stone expand-T4b. NOT a hand-list of "which verbs are
    // expand-time legal": every name below is one for which `registry().lookup_entry`
    // returns `None` — no registration site exists yet to carry an `@ExpandTime`
    // ruling — so the verdict for exactly these 20 stays HERE until one exists. This
    // is a HOMING BACKLOG, not the 202-name hand-list it replaced: each row retires
    // the moment its verb gets a registration site — move the reasoning there (the
    // same motion `:wat::hashmap::keys` / `:wat::hashmap::values` just made above)
    // and delete the arm. A REGISTERED verb does not belong here — if one is ever
    // added below alongside a real registration, the derivation above is being
    // shadowed by a copy, which is the exact defect this stone exists to remove.
    // Measured at 59 (arc 255 Stone expand-T4a's count, reconfirmed by this stone via
    // `lookup_entry` on every one, not re-guessed). Down to 58 — arc 255 Stone A-2-ii-b
    // homed `:wat::core::sort$native` into `#[wat_intrinsic]` with its own
    // `@ExpandTime Legal`, so `lookup_entry` now answers for it above and the
    // hand-list row would have been a shadowing copy. Down to 21 — arc 255 Stone
    // `1c-c-the-residues-cannot-shadow-the-registry`: 33 of the 54 rows this residue
    // actually carried had drifted the same way, one at a time, unnoticed — see the
    // retirement bullet below. Down to 20 — arc 255 Stone 1c-e: `str` registered
    // (`#[wat_intrinsic]`, `@ExpandTime Legal`), same shadowing-copy defect. Down to
    // 18 — arc 255 Stone 1c-g: `=`/`not=` registered (`#[wat_intrinsic]` wrappers in
    // `src/runtime.rs`, each `@ExpandTime Legal`), same shadowing-copy defect.
    //
    // Grouped only for a reader's sake — the residue's DEFINITION is `lookup_entry ==
    // None`, nothing about these groupings:
    //   value/control-flow ops with no per-verb home yet —
    //     `i64/to-f64` (dual spelling of the homed `i64::to-f64`), `i64/to-string`,
    //     `subtype?`, `List?`
    //   ~~str~~ — DELETED arc 255 Stone 1c-e: `#[wat_intrinsic]`-registered
    //     (`src/runtime.rs`'s `eval_str`, `@ExpandTime Legal`), so the `registry().lookup_entry`
    //     door above answers first and this arm was unreachable dead text — the identical
    //     "shadowed by a copy" defect the header above names.
    //   ~~=/not=~~ — DELETED arc 255 Stone 1c-g: both `#[wat_intrinsic]`-registered
    //     (`src/runtime.rs`, each `@ExpandTime Legal`), so the `registry().lookup_entry` door
    //     above answers first and these arms were unreachable dead text — the identical
    //     "shadowed by a copy" defect the header above names.
    //   collection constructors — `Vector`, `HashMap`, `HashSet`
    //   collection / sequence ops still on the pre-registry dispatch path —
    //     `count`, `into`, `filterv`, `reduce`, `reduce-stream`, `doall`, `dorun`,
    //     `stream->pvec`
    //   ~~Option/Result unwrappers~~ — DELETED 2026-08-31. All four (`Option/expect`,
    //     `Option/try`, `Result/expect`, `Result/try`) are now `#[wat_intrinsic]`-registered,
    //     so the `registry().lookup_entry` door above answers first and these arms were
    //     unreachable dead text — precisely the "shadowed by a copy" defect this residue
    //     list's own header names. Found by a rider that was homing three of them and
    //     noticed the fourth had been stale since earlier the same day.
    //   ~~and/or/not/do/fn/match/bool::to-string/show/type/conforms?/record?/macro-error,
    //     Tuple, assoc/conj/contains?/first/second/third/get/drop/take/filter/map/mapv/
    //     foldl/find-last-index/stream::lazy/stream->vec, and all four homoiconic AST
    //     helpers (forms/with-children/write-forms/struct->form)~~ — DELETED arc 255
    //     Stone `1c-c-the-residues-cannot-shadow-the-registry`. All 33 are now
    //     `#[wat_intrinsic]`-registered, so the `registry().lookup_entry` door above
    //     answers first and these arms were unreachable dead text — the identical
    //     "shadowed by a copy" defect the 2026-08-31 deletion above names, this time
    //     found by a GATE (`the_residues_cannot_shadow_the_registry`,
    //     `src/intrinsic/mod.rs`) that asserts the rule this header states, rather than
    //     by a rider noticing.
    //   ReadOutcome / Error field access — `ReadOutcome::Forms`,
    //     `ReadOutcome::Malformed`, `Error/message`
    matches!(
        head,
        | ":wat::core::i64/to-f64"
        | ":wat::core::i64/to-string"
        | ":wat::core::subtype?"
        | ":wat::core::List?"
        | ":wat::core::Vector"
        | ":wat::core::HashMap"
        | ":wat::core::HashSet"
        | ":wat::core::count"
        | ":wat::core::into"
        | ":wat::core::filterv"
        | ":wat::core::reduce"
        | ":wat::core::reduce-stream"
        | ":wat::core::doall"
        | ":wat::core::dorun"
        | ":wat::core::stream->pvec"
        | ":wat::core::ReadOutcome::Forms"
        | ":wat::core::ReadOutcome::Malformed"
        | ":wat::core::Error/message"
    )
}

// ─── The mirror wall — arc 255 Stone expand-only-the-mirror-wall ─────────────
//
// `is_expand_time_legal` (above) refuses a head found INSIDE a macro body unless the
// registry names it `Legal`/`ExpandOnly`/`Preserving`. This is its mirror: refuse a head
// found OUTSIDE one — i.e. anywhere in ordinary program code — when the registry names it
// `ExpandOnly`. `RuntimeOnly` is the OTHER half's mirror (refused inside a macro body,
// since it has no expand-time behaviour); `ExpandOnly` is THIS half's target (refused
// outside one, since it has no runtime call site at all). `macro-error` is, today, the
// sole `ExpandOnly` declarer (measured — `crate::intrinsic::macro_error`).
//
// Per DESIGN-STONE-expand-only-the-mirror-wall.md's probe A, this walk needs no "am I
// inside a macro body?" context. A `defmacro` form's ENTIRE shape — name, argspec, body —
// is a declaration, not a value-producing expression: it stays verbatim in the tree after
// registration (`hoist_top_level_form`'s doc, this file's own module) but is never walked
// as program code, the same way `check.rs`'s `:4871-4883` returns for the same head
// without descending. A `quasiquote` template is skipped the same way — data, not code,
// reusing `resolve::boundary`'s own AllData/Quasiquote classification rather than a second
// hand-rolled copy of the same language fact. Both skips are decided by the CURRENT node's
// head alone; no flag threads across the recursion. The one place an `ExpandOnly` verb is
// legal is therefore structurally unreachable by this walk — it can only ever see misuse:
// a direct call in ordinary code, or a macro template that QUOTED the call into its own
// expansion (a real defect, invisible until the emitted code ran and raised).
pub(super) fn refuse_expand_only_in_program(form: &WatAST) -> Result<(), MacroError> {
    match form {
        WatAST::List(items, span) => {
            if let Some(WatAST::Keyword(head, _)) = items.first() {
                // Declaration form: the WHOLE form is not walked as program code. This is
                // the one place an ExpandOnly verb is legal, and it is unreachable from here.
                if head == ":wat::core::defmacro" {
                    return Ok(());
                }
                // Data, not code — reuse resolve::boundary's established classification
                // (quote/forms/holon::literal are AllData; quasiquote is its own pole).
                if matches!(
                    crate::resolve::boundary::quote_boundary(head),
                    crate::resolve::boundary::Boundary::AllData
                        | crate::resolve::boundary::Boundary::Quasiquote
                ) {
                    return Ok(());
                }
                let is_expand_only = crate::intrinsic::registry()
                    .lookup_entry(head)
                    .is_some_and(|e| matches!(e.expand_time, wat_doc::ExpandTime::ExpandOnly));
                if is_expand_only {
                    return Err(MacroError {
                        span: span.clone(),
                        kind: MacroErrorKind::ExpandOnlyOutsideMacro { head: head.clone() },
                    });
                }
            }
            for child in items {
                refuse_expand_only_in_program(child)?;
            }
            Ok(())
        }
        WatAST::Vector(items, _) => {
            for child in items {
                refuse_expand_only_in_program(child)?;
            }
            Ok(())
        }
        WatAST::Map(pairs, _) => {
            for (k, v) in pairs {
                refuse_expand_only_in_program(k)?;
                refuse_expand_only_in_program(v)?;
            }
            Ok(())
        }
        WatAST::Set(items, _) => {
            for item in items {
                refuse_expand_only_in_program(item)?;
            }
            Ok(())
        }
        // Leaf nodes — no sub-forms to check.
        WatAST::IntLit(_, _)
        | WatAST::FloatLit(_, _)
        | WatAST::RationalLit(_, _)
        | WatAST::BigIntLit(_, _)
        | WatAST::CharLit(_, _)
        | WatAST::BoolLit(_, _)
        | WatAST::StringLit(_, _)
        | WatAST::NilLit(_)
        | WatAST::Keyword(_, _)
        | WatAST::Symbol(_, _) => Ok(()),
    }
}
