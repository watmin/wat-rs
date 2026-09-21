# SCORE-AMEND — STONE 251.8d-i-b: rete accepts `:-` only

Folded into the stone landing (was `c0508e87b` / originally `120c67a4a`). **Not pushed.**
Parent: `AMEND-251.8d-i-b-the-arrow-is-GONE.md` (the 2026-09-21 no-dual-accept ruling).
Floor / workspace clippy / census: orchestrator. Crate clippy + lint + rete suite run here.

## The ruling, as implemented

Rete field/fact/accum bindings use `is_binder_marker` (`:-`) **only**. A live
`(?k <- :k)` is now `MalformedClause`. Param annotations `[x <- :T]` still
use `is_param_annotation_arrow` (`<-` or `:-`) — that is 255.2, not rete.

Converted binding: `(?k <- :k)` → `(?k :- :k)` (arrow converts; `:k` stays).

## Stash-dance: not used, and why

The converter (`rete-bind-arrow-to-binder.wat`) rides **existing** `fix.wat`
verbs (`rete-var?`, `carry-rete-var?`, `fix-text-apply`). No new
`include_str!` verb. Sequence:

1. Convert rete-binding `<-` → `:-` **while the checker still accepted both**
2. Switch the three rete field-bind sites to `is_binder_marker`
3. Teach the wat oracle (`wat/rete/compile.wat` `bind-arrow?`) the same spelling
4. Rebuild — stdlib `wat/fmt.wat` / `wat/grep.wat` already had `:-`

The chicken-and-egg does not apply: the old form was still legal at convert
time. Freeze after the checker change: **green**.

## Corpus writes (the new ruling required them)

The AMEND's old line *"NOT ONE corpus `.wat` converted"* is **superseded**
by *"the tool change and the corpus conversion become one landing."* This is
**not** the 8d-ii dialect flip (heads, types, all arrows). It is a surgical
rewrite of rete-var-preceded `<-` only.

| | |
|---|---|
| tracked `.wat` rewritten | **321** (330 candidates; comment-only no-ops skipped) |
| `.wat.bad` rewritten | **25** |
| rust-embedded rete strings | **42 files / 338 occ** (same `?var` + arrow pattern; extra-whitespace 2) |
| stdlib | `wat/fmt.wat`, `wat/grep.wat`; oracle `wat/rete/compile.wat` |
| param `[x <- :T]` | **untouched** |

Driver: `wat-scripts/fixes/rete-bind-arrow-to-binder.wat` (replay fixtured).
Idempotent on the 330-file `/tmp` copies.

Negative control: `(?k <- :k)` → `MalformedClause`. Cascade with `(?loc :- :location)` `--check` **rc=0**.

## ⭐ FINDING: expr_ir:1035 is a param skip, not a field bind

The AMEND listed four `"<-"` sites. Three are rete field/fact/accum binds and
now call `is_binder_marker`. `expr_ir/mod.rs:1035` skips **`fn` param types**
(`[x <- :T]` / `[x :- :T]`) while lowering a literal fn. Routing that through
`is_binder_marker` only reds user-fold arity (`call-user expected 4, got 2`).
**Left on `is_param_annotation_arrow`.** Report, not forced.

## Gate: the 179-file delta — ONE tree

Full-dialect copies (`to-faithful-clojure`) vs live originals (now with
surgical `:-` binds). Timed one file first: grep copy **0.33 s**.

| | this tree at 255.5 | this AMEND |
|---|---|---|
| originals clean | **161 / 179** | **161 / 179** |
| still clean after conversion | 95 | **95** |
| ⛔ **REGRESSIONS** | **66** | **66** |

**0 closed. 0 newly_broken.** `ReteCheckErrors` **21** still — symbol-headed
`(vrm/F …)` / `(?fact :- weather/Type)` on the **full dialect** copies. Live
rust-scheme files with `:-` binds load. Allowed; not forced.

## Non-vacuity

Replay `rete-bind-arrow-to-binder`: `(?k <- :k)` → `(?k :- :k)`; `[x <- :T]`
stays; already-`:-` stays. `to_faithful_clojure` leftover-arrow gate still
green. `one_param_spec` green.

## Test count

`cargo nextest list --release -p wat`: **5320**.

## Walls I ran

- crate clippy `-p wat --all-targets -D warnings` — **0**
- `--test rete` — **514 passed**
- `--lib rete::kernel` — **133 passed**
- `--test lint` — **356 passed** (incl. `one_param_spec`)
- `--test cli every_recorded_migration` — 18 passed

Floor + workspace clippy + `census.sh --diff`: **not run** (orchestrator).
Do not push. Do not start 8d-ii.
