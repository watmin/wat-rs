//! ⛔⛔ **TIER B IS REFUTED, AND THIS FILE IS THE REFUTATION.**
//!
//! DESIGN: `docs/excursus/2026/08/001-sns-sqs/tier-b-the-check-skips-what-it-already-proved/DESIGN.md`
//! SCORE:  the sibling `SCORE.md`
//! ROOT:   `docs/excursus/2026/08/001-sns-sqs/can-a-user-def-change-a-stdlib-verdict/FINDING.md`
//!
//! Tier B wanted to restrict `check_program`'s four `ALL fns` sweeps to functions a boot-cache
//! snapshot did not supply, on the claim:
//!
//! > for every stdlib function `F`, the verdict of `check(F)` is determined solely by state
//! > fixed at bake time — no user-supplied state can reach it.
//!
//! **That claim is false.** `walk_for_restricted_call` — the `check:restricted-call(ALL fns)`
//! sweep — reads `CheckEnv::binding_metadata`, a map a user program writes with an ordinary
//! `{:restricted-to […]}` metadata-map, keyed by the binding's own name. Nine stdlib
//! `…::service-forms` bodies MENTION `:user::main` inside a quoted `(:wat::core::forms …)`
//! child-program template, and the walker fires on every `WatAST::Keyword` leaf it walks,
//! quoted or not. So one metadata-map on the user's own `:user::main` reddens nine bodies
//! across six stdlib files — and Tier B would have skipped exactly those bodies.
//!
//! ⛔ **`binding_metadata` is an EIGHTH door.** The spike (`src/spike_probe.rs`) instrumented
//! seven, all inside `check:body-infer`; this one is read by a different sweep and was never in
//! the census. That is why the census's "exactly three `:repl::` names" answer — true for its
//! own sweep — did not settle Tier B.
//!
//! ## What each test holds
//!
//! - `a_user_restriction_on_user_main_no_longer_reddens_stdlib_bodies` — the witness, INVERTED
//!   2026-09-18 when the defect it reproduced was repaired. See its doc.
//! - `the_user_declarable_surface_inside_stdlib_bodies_is_exactly_two` — the SURFACE, pinned at
//!   the two names that make the witness possible. A third one appearing is a new attack
//!   surface of exactly this class, and it must be a deliberate decision, not a silent one.
//!
//! ⚠ Neither test blesses the behaviour. The witness's diagnostics blame stdlib functions for
//! "calling" a name they only quote; that is an adjacent defect, named in the SCORE and NOT
//! repaired here.
//!
//! ## ⛔ 2026-09-18 — THE DEFECT THE REFUTATION RESTED ON IS REPAIRED, AND TIER B IS NOT
//!
//! `docs/excursus/2026/08/001-sns-sqs/a-mention-in-a-quoted-form-is-not-a-call/` struck the
//! quoted-mention half: `walk_for_restricted_call` now routes through
//! `resolve::boundary::quote_boundary` and does not fire on a `WatAST::Keyword` sitting in
//! quoted data (`quote` / `forms` / `literal`, or a quasiquote template outside an unquote
//! escape). The witness below therefore FREEZES GREEN and its assertion is inverted.
//!
//! ⚠ **What that costs this file, stated plainly rather than quietly:** the inverted witness no
//! longer proves the `check:restricted-call(ALL fns)` sweep RUNS over stdlib bodies. It passed
//! before because the sweep fired; it passes now because nothing is left for it to fire on. A
//! future Tier B must derive a NEW control for that sweep — this one has become a control for
//! the repair, not for the sweep, and reading it as the latter is how an elision would ship
//! unnoticed.
//!
//! ⚠ **And Tier B is still REFUTED, not re-opened.** The repair removes the only two names by
//! which a user write could change a stdlib verdict *through this door on today's corpus*; it
//! does not establish the general claim, and `the_user_declarable_surface_inside_stdlib_bodies_is_exactly_two`
//! still reports a surface of two (it counts raw leaves, quoted or not, on purpose). Re-deciding
//! Tier B is Tier B's job, not this file's.

use std::sync::Arc;

use wat::ast::WatAST;
use wat::freeze::{startup_bare, startup_from_source, StartupError};
use wat::load::loader::InMemoryLoader;
use wat::types::TypeExpr;
use wat::value::FunctionBody;

const DIR: &str = "tests/function/";
const STEM: &str = "probe_tier_b_stdlib_verdict_is_not_bake_fixed";

fn fixture(suffix: &str) -> String {
    let path = format!("{DIR}{STEM}{suffix}");
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("fixture {path:?} must exist (run from crate root): {e}"))
}

fn freeze(suffix: &str) -> Result<wat::freeze::FrozenWorld, StartupError> {
    startup_from_source(
        &fixture(suffix),
        Some("consumer.wat"),
        Arc::new(InMemoryLoader::new()),
    )
}

// ── ⭐ THE WITNESS ───────────────────────────────────────────────────────────────────

/// NON-VACUITY for the witness below: the same program without the metadata-map freezes.
#[test]
fn the_baseline_without_the_metadata_map_freezes_green() {
    freeze("_base.wat").expect("a plain `:user::main` must freeze");
}

/// ⛔⛔ INVERTED 2026-09-18 — WHAT WAS NINE ERRORS IS NOW ZERO, AND THAT IS THE REPAIR.
///
/// **What this asserted until 2026-09-18:** exactly NINE `DefRestrictedCallerNotAllowed`
/// errors, across SIX stdlib files, from one `{{:restricted-to [:my::]}}` on the consumer's own
/// `:user::main` — every error located in a file the author had never seen. That was the Tier B
/// refutation, and it was correct: `walk_for_restricted_call` fired on every `WatAST::Keyword`
/// leaf, including the `:user::main` that nine `…::service-forms` bodies QUOTE into a
/// `(:wat::core::forms …)` child-program template.
///
/// **What it asserts now:** the program freezes. A mention inside a quoted template is data —
/// a name resolved in a different program, at a different time, by a different caller — so a
/// CALLER-restriction on it was checking the wrong thing at the wrong time. Arc 198's width
/// over POSITION (head, `let` binding, `apply` argument, map value) is untouched; only quoted
/// data is exempt, and only when not unquoted. See
/// `tests/kernel/wat_arc198_def_restricted.rs`'s `quoted_mention_*` tests for that boundary.
///
/// ⚠ **This is no longer a control on the sweep.** It passed before because
/// `check:restricted-call(ALL fns)` FIRED; it passes now because nothing remains for it to fire
/// on, and it would keep passing if the sweep were elided entirely. Any future Tier B needs a
/// fresh control — see the module doc.
///
/// ⚠ The fixture keeps its `_neg_restricted.wat.bad` name (the suffix is historical, naming the
/// defect it reproduced) so Tier B's SCORE.md stays followable by path.
#[test]
fn a_user_restriction_on_user_main_no_longer_reddens_stdlib_bodies() {
    let world = freeze("_neg_restricted.wat.bad").unwrap_or_else(|err| {
        panic!(
            "a `:restricted-to` on the consumer's OWN `:user::main` must not redden stdlib \
             bodies that merely QUOTE that name into a child-program template — got: {err:?}"
        )
    });

    // NON-VACUITY: the world really did freeze with the restricted `:user::main` in it, rather
    // than the fixture having quietly stopped declaring one.
    assert!(
        world.symbols.functions_iter().any(|(p, _)| p == ":user::main"),
        "the fixture must still declare `:user::main`, or this gate passes on an empty premise"
    );
}

// ── ⭐ THE SURFACE, PINNED ───────────────────────────────────────────────────────────

/// Every name reachable from a stdlib function — its own path, every type its signature
/// mentions, and every `Keyword`/`Symbol` leaf in its body — that a USER PROGRAM COULD LEGALLY
/// DECLARE: namespaced (`resolve::gate` returns `Unnamespaced` otherwise) and outside the
/// reserved prefixes (`Reserved` otherwise, for `Privilege::User`).
///
/// Those two walls are exactly the complement of the key space a user can write into, so this
/// set IS the surface on which a user declaration and a stdlib body can meet. It is **two**.
#[test]
fn the_user_declarable_surface_inside_stdlib_bodies_is_exactly_two() {
    let world = startup_bare().expect("bare stdlib world must freeze");

    // NON-VACUITY, first — a walk over an empty world proves nothing.
    let fns = world.symbols.functions_iter().count();
    assert!(
        fns > 1000,
        "only {fns} functions in the bare world — the instrument is broken or the stdlib \
         stopped loading, and this gate would then pass vacuously"
    );

    let mut surface: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let note = |n: &str, out: &mut std::collections::BTreeSet<String>| {
        // `is_namespaced` is containment, not a leading-colon test — parametric heads drop the
        // colon (`wat::kernel::Peer`). Mirrors `resolve::registration::is_namespaced`.
        if n.contains("::") && !wat::is_reserved_prefix(n) {
            out.insert(n.to_string());
        }
    };
    for (path, func) in world.symbols.functions_iter() {
        note(path, &mut surface);
        for t in func
            .param_types
            .iter()
            .chain(std::iter::once(&func.ret_type))
            .chain(func.rest_param_type.iter())
        {
            for n in type_expr_names(t) {
                note(&n, &mut surface);
            }
        }
        if let FunctionBody::Wat(body) = &func.body {
            walk_names(body, &mut |n| note(n, &mut surface));
        }
    }

    let found: Vec<&str> = surface.iter().map(String::as_str).collect();
    assert_eq!(
        found,
        vec![":user::main", ":user::spawn::service-locus"],
        "the user-declarable surface inside stdlib bodies has MOVED. Every name here is one a \
         user `def` / `defn` / `defclause` / type / `extend-type` can land on, in a map a \
         stdlib body reads — which is how `{{:restricted-to […]}}` on `:user::main` reddens \
         nine stdlib bodies (see the witness above) and how `:repl::turn` reddened five before \
         it was renamed. A name ADDED here is a new attack surface; a name REMOVED is progress \
         that should be recorded. Either way, read \
         docs/excursus/2026/08/001-sns-sqs/tier-b-the-check-skips-what-it-already-proved/SCORE.md \
         before editing this list."
    );
}

/// Every name a [`TypeExpr`] mentions, however nested. `Var` carries an integer id, not a name.
fn type_expr_names(t: &TypeExpr) -> Vec<String> {
    let mut out = Vec::new();
    fn go(t: &TypeExpr, out: &mut Vec<String>) {
        match t {
            TypeExpr::Path(p) => out.push(p.clone()),
            TypeExpr::Var(_) => {}
            TypeExpr::Tuple(ts) => ts.iter().for_each(|x| go(x, out)),
            TypeExpr::Parametric { head, args } => {
                out.push(head.clone());
                args.iter().for_each(|x| go(x, out));
            }
            TypeExpr::Fn { args, ret } => {
                args.iter().for_each(|x| go(x, out));
                go(ret, out);
            }
        }
    }
    go(t, &mut out);
    out
}

/// Every `Keyword` / `Symbol` leaf under `node`. Over-approximating on purpose: a name used as
/// DATA counts too, because `walk_for_restricted_call` does not know the difference either.
fn walk_names(node: &WatAST, f: &mut impl FnMut(&str)) {
    match node {
        WatAST::Keyword(k, _) => f(k),
        WatAST::Symbol(id, _) => f(id.as_str()),
        WatAST::List(items, _) | WatAST::Vector(items, _) | WatAST::Set(items, _) => {
            items.iter().for_each(|c| walk_names(c, f))
        }
        WatAST::Map(pairs, _) => pairs.iter().for_each(|(k, v)| {
            walk_names(k, f);
            walk_names(v, f);
        }),
        WatAST::IntLit(..)
        | WatAST::FloatLit(..)
        | WatAST::RationalLit(..)
        | WatAST::BigIntLit(..)
        | WatAST::CharLit(..)
        | WatAST::BoolLit(..)
        | WatAST::StringLit(..)
        | WatAST::NilLit(..) => {}
    }
}
