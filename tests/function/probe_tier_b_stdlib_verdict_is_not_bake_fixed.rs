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
//! - `a_user_restriction_on_user_main_reddens_stdlib_bodies` — the WITNESS. If a future Tier B
//!   silences it, this goes red rather than the errors going quietly missing.
//! - `the_user_declarable_surface_inside_stdlib_bodies_is_exactly_two` — the SURFACE, pinned at
//!   the two names that make the witness possible. A third one appearing is a new attack
//!   surface of exactly this class, and it must be a deliberate decision, not a silent one.
//!
//! ⚠ Neither test blesses the behaviour. The witness's diagnostics blame stdlib functions for
//! "calling" a name they only quote; that is an adjacent defect, named in the SCORE and NOT
//! repaired here.

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

/// ⛔⛔ ONE USER FORM, NINE ERRORS, EVERY ONE OF THEM INSIDE A STDLIB FILE.
///
/// This is the negative control every future Tier B must survive. The sweep that produces
/// these errors is `check:restricted-call(ALL fns)`, one of the four Tier B set out to elide;
/// eliding it would make this program compile clean and drop nine located errors on the floor.
#[test]
fn a_user_restriction_on_user_main_reddens_stdlib_bodies() {
    let err = freeze("_neg_restricted.wat.bad")
        .expect_err("a `:restricted-to` on `:user::main` must still redden the stdlib bodies \
                     that quote that name — if this passes, a check sweep stopped running");
    let rendered = format!("{err:?}");

    for needle in [
        "DefRestrictedCallerNotAllowed",
        ":user::main",
        // the enclosing fn the walker blames is a STDLIB function …
        ":wat::kernel::stderr-svc::service-forms",
        // … and the located span is inside a STDLIB file, not the consumer's
        ":file \"wat/kernel/services/stdio.wat\"",
    ] {
        assert!(
            rendered.contains(needle),
            "the refutation must carry {needle:?} — got: {rendered}"
        );
    }

    // The count is the measurement, not decoration: nine call sites across six stdlib files.
    // A change to it is a change to the surface and should be read, not rounded away.
    let hits = rendered.matches("DefRestrictedCallerNotAllowed").count();
    assert_eq!(
        hits, 9,
        "the witness moved nine stdlib verdicts when it was measured (2026-09-18); it now \
         moves {hits}. Re-read `the_user_declarable_surface_inside_stdlib_bodies_is_exactly_two` \
         before adjusting this number — got: {rendered}"
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
