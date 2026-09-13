# REFUTE 2 — 2a1: the door still drops types silently. Ask each macro what it emits, one step at a time

Verified by the orchestrator on `101626eea`, uncontended:
- floor 5425/5425 (`.floor/2026-09-13T08-15-43Z`), clippy 0;
- **R1 ✓** c3 through `bootstrap/era/probe-R/door.wat`: `:demo::Req [n]`, `:demo::Op` `Go [req]`;
- **R3 ✓** including a failure INSIDE a user macro's expansion: `Refused.form` is the call
  `(:r3/bad :x)`, not the defmacro (`bootstrap/era/probe-R/r3-macro.wat`);
- **R4 ✓** for Colour, Point, Ping `::Op`, CreateWebACL;
- corpus (the 1489-file step-24 input, one wat process): 1457 Ok, 32 Refused, 0 unreadable, 13.2 s.
  No refused form is a user-macro call, so R1 added no refusal.

Two things keep 2a1 from closing.

## R5 — the admitted set is still a hand list, and it drops real types today

Measured on the step-24 corpus:
- **`:wat::holon::defrecord`** is a stdlib macro whose template emits `(:wat::core::recordtype …)`
  (`wat/Record.wat:224`). It is on neither list. **All 22** top-level holon records are missing from
  the door's `Ok`; the control, the 17 `:wat::core::defrecord` records in the same files, is 17/17
  found. `declared_types_filter_admits_pre_expansion_type_macros` checks one hand list against another,
  so it cannot see a head that is on neither.
- **`:wat::core::defn` declares types.** A kwargs defn emits `(:wat::core::defstruct <fn>::Kwargs …)`
  (`wat/core.wat:888-889`); a capability defn emits `::GrantHandles` and `::Coords`
  (`wat/core.wat:1188`, `:1195`). The door excludes defn, so those types are dropped too.
- **A declaration in a top-level `let` body** is dropped (e.g.
  `tests/macros/probe_register_types_splice_aware_let_typealias.wat`'s `:diag::LetAlias`). Startup
  registers it: `register_types`' do-flattens / let-wraps rule (arc 170 slice 3 Gap J).
- **A static derivation does not fix it** (measured, `bootstrap/era/probe-R/probes-2a1-refute2.diff`,
  test `zz_probe_derived_type_minting_macros`): walking stdlib templates for emitted type heads derives
  `defn` (correctly: it emits types), and through defn `deftest`, `deftest-hermetic`, `run-hermetic`,
  `rete::defrule`, `rete::defquery`, whose bodies must not expand.

**The shape — measured exact** (`zz_probe_one_step_walk_vs_door`, same diff). After
`register_defmacros` and `preregister_acronyms`, walk each top-level form:
1. `crate::types::classify_type_decl(form)` is `Some` → keep the form.
2. The head is a container → walk its BODY children only. Use `src/macros/expand.rs:250`
   `container_body_start` (do → from 1, let → from 2; a let's bindings are never walked). Make it
   `pub(crate)` and call it; one copy.
3. The head is a keyword naming a registered macro (stdlib or this program's) →
   `crate::macros::expand_once` (`src/macros/expand.rs:366`: macroexpand-1, no recursion, no fixpoint)
   and walk the result.
4. Anything else is not a declaration: it is not kept and NEVER expanded. A defn's one step yields its
   `::Kwargs` defstruct (kept) and a `:wat::core::def` (not kept, so its body is never expanded).

Then the existing tail on the kept forms: `expand_all` → `register_types_with_acronyms` →
`register_variant_types`.

- **No name list remains in the door.** `PRE_EXPANSION_TYPE_FORMS`, `keeps_declared_types_form`,
  `declared_types_filter_derives_type_decl_heads_from_classify` and
  `declared_types_filter_admits_pre_expansion_type_macros` are deleted. The probe measured that keeping
  `derive`/`extend-type`/`declare-acronyms` changes nothing (identical report), because none adds a type
  and `preregister_acronyms` reads the whole program first.
- **The measured bar** (the orchestrator re-runs it on your commit): over the step-24 corpus, against
  `101626eea`'s door, **0 files lose a type, 47 files gain** (`bootstrap/era/probe-R/one-step-vs-door.txt`
  lists every one), **0 new refusals**, 1457 Ok / 32 Refused.
- **Pins, on TRACKED files:**
  - `tests/types/probe_arc234_stone2a_record_primitives.wat` returns `:myapp::Voltage [magnitude]` and
    `:myapp::Point [x y]`;
  - a kwargs defn returns its `::Kwargs` with its fields (pick a tracked file; name it in the SCORE);
  - `tests/macros/probe_let_splice_enum_via_macro.wat` returns `:my::probe::Event` with its variants;
  - c3 still returns `:demo::Req` and `:demo::Op`;
  - **a body is never expanded:** an inline program whose `defn` body holds a malformed declaration
    call (e.g. `(:wat::core::defstruct)`), which would refuse if expanded, returns `Ok` with the
    program's own declared types.

## R6 — two committed tests read the orchestrator's IGNORED instrument directory

`declared_types_register_despite_stale_bodies` (`src/check.rs:23529`) parses
`bootstrap/era/probe-L/w25/…` (`:23532`, `:23563`). `bootstrap/` is in `.gitignore` and holds 0 tracked
files: the test passes only on this machine, and is RED on any clone, including the DR site. (The
orchestrator's brief cited those files as evidence; tests need tracked ground.)
- **Timing:** measure on the TRACKED `tests/services/probe_arc278_sift_rules.wat` (it loads today,
  `--check` exit 0); keep the milliseconds assertion.
- **The stale-body property:** it is R5's "a body is never expanded" pin, inline in the test source.
- **The wall:** a lint test that FAILS when committed Rust source under `src/` or `tests/` contains a
  string literal beginning `"bootstrap/`, naming file:line. It must go RED once before it counts.

## R7 — the oracle covers the newly reported kinds

Extend `declared_types_oracle_matches_type_of_after_startup` with each R5 pin that loads normally:
the holon records and the kwargs `::Kwargs` (`type-of` after `startup_from_source` == the door's
`type_info_value`). For any that cannot load, say why in the SCORE.

## STOP triggers

- **STOP-1:** a top-level form that `101626eea`'s door answered `Ok` now refuses. Report the file and
  the verbatim error.
- **STOP-2:** the floor is red. Paste the whole block verbatim, and do not re-run.

## Tier

Commit on green (floor + clippy 0). **Do not push.** Yield with `SCORE-2a1-REFUTE-2.md`, one row per
R5, R6, R7, each pin its own line.
