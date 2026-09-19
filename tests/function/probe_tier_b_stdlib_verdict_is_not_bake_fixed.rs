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
//! - `no_stdlib_body_names_a_user_declarable_name_in_evaluated_position` — the stone's
//!   number: evaluated-position user-declarable leaves in stdlib bodies is **zero**.
//!   Path and signature channels empty; quoted channel is the same two names (9+10).
//! - `the_restricted_call_sweep_still_walks_every_wat_body` — 8c still walks every
//!   `FunctionBody::Wat` at `src/check.rs:750`. Companion process:
//!   `probe_tier_b_restricted_sweep_still_runs`.
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
//! before because the sweep fired; it passes now because nothing is left for it to fire on.
//! The fresh control is `the_restricted_call_sweep_still_walks_every_wat_body` in this file
//! (source shape of `src/check.rs:750`) plus
//! `tests/function/probe_tier_b_restricted_sweep_still_runs.rs` (boot-census hits for the
//! `8c` phase). Deleting `:750` reddens both. This file's inverted witness remains a control
//! for the quoted-mention repair, not for the sweep.
//!
//! ⚠ **And Tier B is still REFUTED, not re-opened.** The repair removes the only two names by
//! which a user write could change a stdlib verdict *through this door on today's corpus*; it
//! does not establish the general claim, and `the_user_declarable_surface_inside_stdlib_bodies_is_exactly_two`
//! still reports a surface of two (it counts raw leaves, quoted or not, on purpose). Re-deciding
//! Tier B is Tier B's job, not this file's.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use wat::ast::WatAST;
use wat::freeze::{startup_bare, startup_from_source, StartupError};
use wat::load::loader::InMemoryLoader;
use wat::types::TypeExpr;
use wat::value::FunctionBody;
use wat::{is_unquote_escape, quote_boundary, Boundary};

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
/// on, and it would keep passing if the sweep were elided entirely. The fresh control is
/// `the_restricted_call_sweep_still_walks_every_wat_body` plus
/// `probe_tier_b_restricted_sweep_still_runs`.
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
        world
            .symbols
            .functions_iter()
            .any(|(p, _)| p == ":user::main"),
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

// ── ⭐ THE EVALUATED-POSITION GATE + FOUR-CHANNEL CENSUS ─────────────────────────────────

/// User-declarable = namespaced AND not a reserved prefix. Same predicate as
/// the two-name surface above, and the complement of what `Privilege::User`
/// can write (`resolve::gate`: Unnamespaced refused, Reserved refused).
fn user_declarable(n: &str) -> bool {
    n.contains("::") && !wat::is_reserved_prefix(n)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Channel {
    OwnPath,
    Signature,
    Evaluated,
    Quoted,
}

impl Channel {
    fn as_str(self) -> &'static str {
        match self {
            Channel::OwnPath => "own-path",
            Channel::Signature => "signature",
            Channel::Evaluated => "evaluated-body",
            Channel::Quoted => "quoted-body",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Site {
    channel: Channel,
    name: String,
    enclosing: String,
    file: String,
    line: i64,
    kind: &'static str,
}

/// Four-channel census of user-declarable names inside stdlib functions, taken
/// from the frozen AST — never from a grep over `wat/*.wat`.
///
/// The stone's number is the **evaluated-position** count. The walk mirrors
/// `walk_for_restricted_call`'s four-way `quote_boundary` split exactly
/// (`src/check.rs:1701`): only `AllData` and `Quasiquote` are data regions;
/// `Match` / `MatchesSubject` / `MakeRule` / `Ordinary` stay live. Unquote
/// escapes inside a quasiquote template resume the evaluated walk.
///
/// Counts **both** `Keyword` and `Symbol` leaves. The restriction walker
/// looks up only `Keyword` (`check.rs:1677`); Symbols are counted so a
/// namespaced symbol-ref the normalizer missed cannot hide. A user-declarable
/// Symbol in evaluated position is the same door.
#[test]
fn no_stdlib_body_names_a_user_declarable_name_in_evaluated_position() {
    let world = startup_bare().expect("bare stdlib world must freeze");

    let fns = world.symbols.functions_iter().count();
    assert!(
        fns > 1000,
        "only {fns} functions in the bare world — the instrument is broken or the stdlib \
         stopped loading, and this gate would then pass vacuously"
    );

    let mut sites: Vec<Site> = Vec::new();
    let mut wat_bodies: usize = 0;
    let mut eval_keyword_leaves: usize = 0;
    let mut eval_symbol_leaves: usize = 0;

    for (path, func) in world.symbols.functions_iter() {
        if user_declarable(path) {
            sites.push(Site {
                channel: Channel::OwnPath,
                name: path.clone(),
                enclosing: path.clone(),
                file: String::new(),
                line: 0,
                kind: "path",
            });
        }
        for t in func
            .param_types
            .iter()
            .chain(std::iter::once(&func.ret_type))
            .chain(func.rest_param_type.iter())
        {
            for n in type_expr_names(t) {
                if user_declarable(&n) {
                    sites.push(Site {
                        channel: Channel::Signature,
                        name: n,
                        enclosing: path.clone(),
                        file: String::new(),
                        line: 0,
                        kind: "type",
                    });
                }
            }
        }
        if let FunctionBody::Wat(body) = &func.body {
            wat_bodies += 1;
            walk_eval(
                body,
                path,
                &mut sites,
                &mut eval_keyword_leaves,
                &mut eval_symbol_leaves,
            );
        }
    }

    assert!(
        wat_bodies > 1000,
        "only {wat_bodies} FunctionBody::Wat in the bare world (fns={fns}) — the walk \
         matched nothing, and an empty evaluated surface would then be vacuously true"
    );
    assert!(
        eval_keyword_leaves > 1000,
        "only {eval_keyword_leaves} evaluated-position Keyword leaves (eval_symbol_leaves=\
         {eval_symbol_leaves}) — the walk never descended into bodies (or quote_boundary \
         exempted everything)"
    );

    let names_on = |ch: Channel| -> BTreeSet<&str> {
        sites
            .iter()
            .filter(|s| s.channel == ch)
            .map(|s| s.name.as_str())
            .collect()
    };
    let own_path = names_on(Channel::OwnPath);
    let signature = names_on(Channel::Signature);
    let evaluated = names_on(Channel::Evaluated);
    let quoted = names_on(Channel::Quoted);

    assert!(
        own_path.is_empty(),
        "stdlib own-path channel is not closed — a user-declarable function path is \
         registered in the bare world: {own_path:?}"
    );
    assert!(
        signature.is_empty(),
        "stdlib signature channel is not closed — a user-declarable name in param/ret \
         types: {signature:?}"
    );
    assert_eq!(
        quoted.iter().copied().collect::<Vec<_>>(),
        vec![":user::main", ":user::spawn::service-locus"],
        "quoted-body user-declarable names moved (channel 4). The two-name assertion \
         still pins the union; this pin is the quoted half."
    );

    if !evaluated.is_empty() {
        let mut by_name: BTreeMap<&str, Vec<String>> = BTreeMap::new();
        for s in sites.iter().filter(|s| s.channel == Channel::Evaluated) {
            by_name.entry(&s.name).or_default().push(format!(
                "{}:{} {} {} in {} ({})",
                s.file,
                s.line,
                s.channel.as_str(),
                s.name,
                s.enclosing,
                s.kind
            ));
        }
        panic!(
            "evaluated-position user-declarable surface is NON-ZERO — a user declaration \
             of any of these names can change a stdlib verdict through \
             check:restricted-call (8c). This stone lands nothing and reports. names={:?}\n{}",
            evaluated.iter().copied().collect::<Vec<_>>(),
            by_name
                .values()
                .flatten()
                .cloned()
                .collect::<Vec<_>>()
                .join("\n")
        );
    }

    // Keyword vs Symbol split for the quoted pin — both names are Keywords
    // (colon-FQDNs). A Symbol here would mean the normalizer left a namespaced
    // symbol-ref in a quoted template; still a name, still pinned.
    let quoted_kinds: BTreeSet<&str> = sites
        .iter()
        .filter(|s| s.channel == Channel::Quoted)
        .map(|s| s.kind)
        .collect();
    assert_eq!(
        quoted_kinds.iter().copied().collect::<Vec<_>>(),
        vec!["keyword"],
        "quoted user-declarable leaves were expected to be Keywords; got {quoted_kinds:?} \
         (eval_symbol_leaves={eval_symbol_leaves})"
    );

    let mut quoted_occ: BTreeMap<&str, usize> = BTreeMap::new();
    for s in sites.iter().filter(|s| s.channel == Channel::Quoted) {
        *quoted_occ.entry(s.name.as_str()).or_default() += 1;
    }
    // ⭑ 9 → 10 on 2026-09-19: `wat/queue.wat` was promoted into the stdlib and carries one
    // `defservice` (`:wat::queue::queue`, queue.wat:214), so there is a TENTH `…::service-forms`
    // body quoting `:user::main` into its child-program template. The pin FIRED on that
    // promotion, which is what it is for — the surface grew deliberately, not silently.
    // ⛔ The evaluated-position count is still ZERO and the two-name surface is unchanged: this
    // is one more OCCURRENCE of a name already on the quoted channel, not a new name.
    assert_eq!(
        quoted_occ.get(":user::main").copied().unwrap_or(0),
        10,
        "quoted `:user::main` occurrence count moved (was 10 service-forms mentions)"
    );
    assert_eq!(
        quoted_occ
            .get(":user::spawn::service-locus")
            .copied()
            .unwrap_or(0),
        11,
        "quoted `:user::spawn::service-locus` occurrence count moved (was 11)"
    );
}

/// ⭐ Fresh 8c control: the `check:restricted-call(ALL fns)` sweep is still
/// the ALL-fns walk of every `FunctionBody::Wat` through `walk_for_restricted_call`.
///
/// Distinguishes *"the sweep ran and found nothing"* from *"the sweep did not
/// run"* by pinning the source of the phase at `src/check.rs:750`. Deleting
/// that phase (or emptying the closure, or skipping stdlib bodies) removes
/// this snippet and this test goes RED. A control that only froze a program
/// with no remaining restricted mentions would stay green through that
/// deletion — that is the inverted witness, and it is not this test.
///
/// Runtime confirmation that the phase actually executed lives in
/// `probe_tier_b_restricted_sweep_still_runs.rs` (boot-census hits); that
/// file is a separate process because `force_mode` is a process-global
/// `OnceLock`.
#[test]
fn the_restricted_call_sweep_still_walks_every_wat_body() {
    let src = include_str!("../../src/check.rs");
    let start = src.find("P_CHECK_RESTRICTED").unwrap_or_else(|| {
        panic!(
            "src/check.rs no longer names P_CHECK_RESTRICTED — the phase at :750 was \
             deleted. That is the elision this control exists to catch."
        )
    });
    let window = &src[start..src.len().min(start + 900)];
    for needle in [
        "functions_iter()",
        "FunctionBody::Wat",
        "walk_for_restricted_call(body, name, func.synthesized_for.as_deref()",
    ] {
        assert!(
            window.contains(needle),
            "the check:restricted-call(ALL fns) phase still exists but no longer {needle} \
             in its window — emptying the closure, skipping stdlib bodies, or filtering \
             by reserved prefix must break this rather than stay green. window:\n{window}"
        );
    }
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

/// Every `Keyword` / `Symbol` leaf under `node`, quoted or not.
///
/// Over-approximating on purpose for the PATH, SIGNATURE, and QUOTED channels:
/// a name entering by any of those is still a deliberate decision, and none of
/// path/signature is exempted by `quote_boundary`. `walk_for_restricted_call`
/// **does** know the difference now — since the quoted-mention stone it routes
/// through `quote_boundary` and does not fire on a leaf in quoted data — so this
/// walk is no longer a model of that walker. The position-aware census
/// (`no_stdlib_body_names_a_user_declarable_name_in_evaluated_position`) is.
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

fn note_site(
    channel: Channel,
    name: &str,
    kind: &'static str,
    enclosing: &str,
    span: &wat::Span,
    sites: &mut Vec<Site>,
) {
    if user_declarable(name) {
        sites.push(Site {
            channel,
            name: name.to_string(),
            enclosing: enclosing.to_string(),
            file: span.file.to_string(),
            line: span.line,
            kind,
        });
    }
}

/// Evaluated-position descent. Mirrors `walk_for_restricted_call` (`check.rs:1670`):
/// Keyword/Symbol leaves here are live mentions; `quote_boundary` splits AllData
/// and Quasiquote off as data; every other `Boundary` variant stays live.
fn walk_eval(
    node: &WatAST,
    enclosing: &str,
    sites: &mut Vec<Site>,
    eval_kw: &mut usize,
    eval_sym: &mut usize,
) {
    match node {
        WatAST::Keyword(k, span) => {
            *eval_kw += 1;
            note_site(Channel::Evaluated, k, "keyword", enclosing, span, sites);
            return;
        }
        WatAST::Symbol(id, span) => {
            *eval_sym += 1;
            note_site(
                Channel::Evaluated,
                id.as_str(),
                "symbol",
                enclosing,
                span,
                sites,
            );
            return;
        }
        WatAST::List(items, _) => {
            if let Some(WatAST::Keyword(head, _)) = items.first() {
                match quote_boundary(head) {
                    // `quote` / `forms` / `literal` — head is a live mention; items[1..]
                    // is data. Same early-return as check.rs:1706–1708.
                    Boundary::AllData => {
                        walk_eval(&items[0], enclosing, sites, eval_kw, eval_sym);
                        for child in items.iter().skip(1) {
                            walk_quoted(child, enclosing, sites);
                        }
                        return;
                    }
                    // `quasiquote` — head live; template is data EXCEPT unquote escapes.
                    // Same early-return as check.rs:1713–1718.
                    Boundary::Quasiquote => {
                        walk_eval(&items[0], enclosing, sites, eval_kw, eval_sym);
                        for child in items.iter().skip(1) {
                            walk_qq_template(child, enclosing, sites, eval_kw, eval_sym);
                        }
                        return;
                    }
                    // Match / MatchesSubject / MakeRule / Ordinary: arc-198 width.
                    // Their "data" regions are DSL in THIS program, not child-program
                    // source. ⛔ Do not exempt them — that would manufacture a zero.
                    Boundary::Match
                    | Boundary::MatchesSubject
                    | Boundary::MakeRule
                    | Boundary::Ordinary => {}
                }
            }
        }
        _ => {}
    }
    match node {
        WatAST::List(items, _) | WatAST::Vector(items, _) | WatAST::Set(items, _) => {
            items
                .iter()
                .for_each(|c| walk_eval(c, enclosing, sites, eval_kw, eval_sym));
        }
        WatAST::Map(pairs, _) => pairs.iter().for_each(|(k, v)| {
            walk_eval(k, enclosing, sites, eval_kw, eval_sym);
            walk_eval(v, enclosing, sites, eval_kw, eval_sym);
        }),
        _ => {}
    }
}

/// Quoted-data descent (`quote` / `forms` / `literal` arguments). No unquote
/// escape: those heads have none. Nested quasiquote inside quote is still data.
fn walk_quoted(node: &WatAST, enclosing: &str, sites: &mut Vec<Site>) {
    match node {
        WatAST::Keyword(k, span) => {
            note_site(Channel::Quoted, k, "keyword", enclosing, span, sites);
        }
        WatAST::Symbol(id, span) => {
            note_site(
                Channel::Quoted,
                id.as_str(),
                "symbol",
                enclosing,
                span,
                sites,
            );
        }
        WatAST::List(items, _) | WatAST::Vector(items, _) | WatAST::Set(items, _) => {
            items.iter().for_each(|c| walk_quoted(c, enclosing, sites));
        }
        WatAST::Map(pairs, _) => pairs.iter().for_each(|(k, v)| {
            walk_quoted(k, enclosing, sites);
            walk_quoted(v, enclosing, sites);
        }),
        _ => {}
    }
}

/// Quasiquote-template descent. Mirrors `walk_restricted_quasiquote_template`
/// (`check.rs:1757`): template text is data; `is_unquote_escape` heads resume
/// the full evaluated walk on their arguments. Nested quasiquote stays template
/// (still scanned for escapes). Exempting the whole template would reopen `~`.
fn walk_qq_template(
    node: &WatAST,
    enclosing: &str,
    sites: &mut Vec<Site>,
    eval_kw: &mut usize,
    eval_sym: &mut usize,
) {
    if let WatAST::List(items, _) = node {
        if let Some(WatAST::Keyword(head, span)) = items.first() {
            if is_unquote_escape(head) {
                // Walker does not mention-check the escape head itself
                // (check.rs:1766–1771); it is a `:wat::` keyword anyway.
                note_site(Channel::Quoted, head, "keyword", enclosing, span, sites);
                for arg in items.iter().skip(1) {
                    walk_eval(arg, enclosing, sites, eval_kw, eval_sym);
                }
                return;
            }
        }
    }
    match node {
        WatAST::Keyword(k, span) => {
            note_site(Channel::Quoted, k, "keyword", enclosing, span, sites);
        }
        WatAST::Symbol(id, span) => {
            note_site(
                Channel::Quoted,
                id.as_str(),
                "symbol",
                enclosing,
                span,
                sites,
            );
        }
        WatAST::List(items, _) | WatAST::Vector(items, _) | WatAST::Set(items, _) => {
            items
                .iter()
                .for_each(|c| walk_qq_template(c, enclosing, sites, eval_kw, eval_sym));
        }
        WatAST::Map(pairs, _) => pairs.iter().for_each(|(k, v)| {
            walk_qq_template(k, enclosing, sites, eval_kw, eval_sym);
            walk_qq_template(v, enclosing, sites, eval_kw, eval_sym);
        }),
        _ => {}
    }
}
