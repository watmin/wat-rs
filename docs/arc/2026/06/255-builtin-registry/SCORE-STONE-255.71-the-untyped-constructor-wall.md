# SCORE — STONE 255.71: the wall — an untyped constructor call is illegal; the heretics are named and typed

**Executor: a Sonnet subagent, resumed** (the first 255.71 agent and its helpers were cut off by the
weekly Sonnet limit, 2026-09-28; `AMEND-STONE-255.71-resume.md`'s "no sub-agents" rule was itself
withdrawn mid-stone, `ec077f547` — its stated cause was wrong, the limit was nearly spent before the
stone began). Commit `2fb4578a9` (the wall, the committed table, the 180-site disposition, 7 tests,
2 corrections to the resumed table, 2 corrections to a prior stone's tests/fixture comments). This
SCORE. Drawn at `75450a149`, against `532a472e7`.

## The wall

`src/check.rs`: `untyped_constructor_error` / `untyped_constructor_span`, wired into
`infer_persistentmap_constructor`, `infer_persistentvector_constructor`, `infer_tuple_constructor`,
`infer_linked_list_constructor`. Every collection constructor call now has exactly one legal shape,
`(wat.type/X :- [T…] items…)`, whatever the head's spelling (`:wat::core::X` or `wat.type/X`), for all
seven heads — `Vector`/`HashMap`/`HashSet` already enforced this (arc 109 stone 3); this stone makes
`List`/`PersistentMap`/`PersistentVector`/`Tuple` match. `List`'s bracket, optional since 255.70
(STOP-2's own "the wall is a later stone"), is mandatory now. The error is a `MalformedForm` (reused,
not a new `CheckErrorKind` variant — it already carries a free-text `reason`), naming the head and the
fix; span is the call's own first arg when there is one, else the whole call's `head_span`.

A macro's expansion is checked like any code, so an untyped constructor written inside a template
screams at every use (`probe_stone255_71_template_untyped`). **Corrected claim** (the first draft's
doc comment overclaimed this): the span does NOT reliably land on the template's own definition
line — when the call's first arg is itself the unquote-substitution (`(Head ~a ...)`, the common
shape), that arg's span is wherever the CALLER wrote it, not the template. Measured directly on the
fixture: the error lands on the call site's own `1` literal (line 15), not the template's line 12.
Both satisfy the stone's actual requirement ("points at the call"); only the definition-line bonus is
shape-dependent, matching the brief's own hedge ("if the span allows it"). `src/check.rs`'s doc comment
and the fixture/test files were corrected to say so.

## Resuming the table: verification, the 19 missing sites, two real corrections

**Verified every one of the 158 existing rows** against `scratch/255-71-census-pre.tsv` (177 sites,
now deleted per the AMEND — ephemeral working data, not committed): every row's (file, line, col) is a
real census site, every head matches (modulo `:wat::core::X`/`wat.type/X` spelling), no duplicates, no
rows for a site the census didn't name. The specifically-flagged-at-risk `255-71-table-tests.tsv` (28
rows, reported possibly clobbered-and-restored) checked out clean against the census too.

**Typed the 19 missing sites** by reading each: element type from the surrounding code, a template
variable where one already existed, or the same site's own declared return/contract type.
12 converted normally; 2 (`tests/lint/probe_c19_nested_type_var_render.wat.bad`) are the ONE corpus
fixture proving `format_type_inner`'s `Var` rendering arm — the unresolved element type IS the point,
typing it destroys the only coverage, left untouched (`.wat.bad`, gate-exempt; the new wall's error
can't disturb the test — it greps the first `:got "..."` field, a key only `ReturnTypeMismatch`
carries, never `MalformedForm`); 4 (2x `tests/function/probe_arc237_7b_*`, 1x
`tests/services/probe_arc278_struct_new_nature_wall.wat.bad`, 1x `tests/types/probe_stone_118_b1a_neg.wat.bad`)
are a **pre-`:-`-marker positional-type-keyword shape** (`(Head TypeKeyword items…)`, no `:-`) —
NOT a genuinely untyped call: `peel_param_spec` (`src/types.rs:6102`) only recognizes a literal `:-`
marker, so these already produce their OWN `MalformedForm` today via arc 109's pre-existing
required-bracket wall (`Vector`/`HashSet`, untouched by this stone), unrelated to and unaffected by
255.71. The codemod's head-only point-splice can't safely convert this shape (the stray type keyword
would become a literal extra element); all four are `.wat.bad`, gate-exempt; left untouched. 1
(`tests/function/probe_stone255_70_constructor_brackets_through_the_door.wat:41:3`) no longer exists —
this stone's own earlier edit (before this resume) deleted the bracket-less
`list-bracketless-still-runs` fn and moved its regression coverage to the new
`probe_stone255_71_list_bracketless_illegal.wat.bad`; no row applicable.

**Two genuine corrections, found only by raising the wall and reading the floor's red — not design
work, type fixes:**

1. **Three sites the resumed table had labeled STOP-1 were wrong.** `wat/core.wat:1283`,
   `wat/service.wat:1633`, `wat/service.wat:1888` sit inside `defservice`-family macro templates; the
   prior agent's reasoning ("the tuple's own declared type is itself built from template variables
   substituted per invocation — no fixed static element type exists to write down") conflated "the
   CONCRETE type differs per call" with "no TYPE EXPRESSION is available." All three templates already
   compute and use the exact needed expression two lines away: `core.wat`'s `pair-ty` is literally
   `` `(wat.type/Tuple :- [~coords-kw ~grant-handles-ann]) ``, and `kwargs-check-def`'s own `-> ~pair-ty`
   already declares the untyped call's return type as that shape; `service.wat`'s `selectable-entry-ty`
   (line 1346) is literally `` `(wat.type/Tuple :- [wat.type/i64 ~selectable-peer-ty]) ``, the exact
   shape this file's own comment at :1633 names ("selectables' element is now `(Tuple :- [i64 (Peer
   :- [R O])])`"). **The consequence that forced the correction, not just untidiness:** leaving these
   3 sites bracket-less puts an untyped-constructor error inside `defservice`'s own accept-loop
   machinery, reached by every one of the ~10+ services the ALWAYS-LOADED stdlib carries
   (`wat/telemetry/journal.wat`, `wat/query/{mem,sqlite-store}.wat`, `wat/telemetry/span.wat`,
   `wat/cache.wat`, `wat/kernel/services/stdio.wat`, …) — so EVERY program check in the corpus picked
   up the cascade. Measured: `wat --check tests/collection/list.wat` (a file with no service usage at
   all) produced **134 type-check errors** before this correction, **0** after.
2. **`wat/bracket.wat:491`** used a bare `:O` colon-keyword — the correct convention for an ORDINARY
   generic function's rigid type-parameter reference (and it IS correct at this file's own :235, :616,
   :747, all three confirmed still passing) — but this particular `out` sits inside `dial-runner`, a
   fn EMITTED by a quasiquote template where `O` is a template variable, not a checker-level generic;
   a bare `:O` never unifies with whatever concrete type the template actually substitutes.
   `~ret-ty` — the exact template var `sp-out` (line 424, same enclosing macro) already uses for this
   identical `(Tuple :- [i64 X])` reply shape — is the fix. The codemod could not re-target this site
   (it was already bracketed from the first pass, so `is-hit?` no longer matched it); corrected by a
   single, verified point-edit as a narrow, documented deviation from "codemod only," the same class
   of exception `wat/rete/oracle/explain.wat`'s own "HAND-FACED, not codemod'd" precedent describes.
   Found via 3 floor failures (`probe_arc170_gapj_each_kwargs`, `probe_arc170_c2_strike1_mixed`,
   `probe_arc170_c2_mixed_macro`) all crashing at the SAME line with
   `TypeMismatch { expected: ":O", got: ":wat::core::String" }` (and `:wat::core::i64` in the other).
   All 3 pass after the fix (the third, `mixed_via_macro_runs`, needs `--test-threads=1` — its own
   module doc comment says so; it forks 7 services + N pool workers and the concurrent run in my own
   diagnostic invocation hit resource contention, not a type error — confirmed by running it alone).

**Supplemental (not part of the 177-site census):** `wat-scripts/fixes/typed-constructors.wat` — the
codemod THIS stone's own heretics are converted with — has 3 untyped `Tuple` calls of its own (an
`(i64, String, String)` edit-tuple, built 3 times). `wat-scripts/fixes/**` is exempt from the "search"
gate, but the wall has no file-based exemption: once it is live, the codemod itself fails to even
START (its own `startup_from_file`-equivalent check trips on its own untyped calls). Typed via the
same codemod, applied to itself, before the wall's build — keeps the tool runnable for any future
migration.

**Final disposition, 180 rows** (177 census + 3 supplemental), all in the committed table:

| kind | count |
|---|---|
| `:typed` | 168 (165 normal + 3 corrected) |
| `:stop1` | 2 (genuine: `wat/rete/oracle/accum-pass.wat:135`, rule-dependent accumulator binding;
`wat-scripts/probes/arc-170/probe-s3b-astsplice.wat:75`, built at runtime via string-concat+AST
splicing) |
| `:legacy-shape` | 4 |
| `:positive-control` | 2 |
| `:removed` | 1 |
| supplemental `:typed` | 3 |

**Committed table:** `docs/arc/2026/06/255-builtin-registry/TABLE-STONE-255.71-typed-constructors.edn`
— one row per site, `{:head :file :line :col :kind :type? :reason}`, keyed by (file, line, col) of the
call's own span, the `kind` meaning documented in the file's own header. Applied via
`wat-scripts/fixes/typed-constructors.wat`'s existing `.typed-constructors-table.edn` side-channel
(gitignored, regenerated from the committed table, never committed itself) under the stash-dance
(`git stash` the wall → build the old checker → run the codemod → `git stash pop`), run 4 times across
this resume as corrections landed (verified idempotent — 0 changes — after each).

## Gates

**Clippy** — `cargo clippy --release --all-targets -- -D warnings`: rc 0, clean.

```
    Finished `release` profile [optimized] target(s) in 11.98s
```

**Idempotent** — the codemod re-run over all 61 converted files (the 59 census-file set + the 2
supplemental): 0 additional changes both times checked (immediately after each apply, and again after
the final `wat/bracket.wat` correction).

**The wall's reach** — a corpus-wide text search for a bracket-less `(Head …)` call of the 5 codemod
heads, outside `.wat.bad`/`.wat.golden`/`wat-scripts/fixes/**`, across all 2,146 in-scope `.wat`
files: **2 real code hits**, both the 2 genuine, documented STOP-1 sites
(`wat/rete/oracle/accum-pass.wat:135`, `wat-scripts/probes/arc-170/probe-s3b-astsplice.wat:75`) —
**neither is inside `.wat.bad`/`.wat.golden`/`wat-scripts/fixes/**`, so the gate's literal "0 outside…"
wording is NOT fully met.** This is a genuine tension inside the brief itself, not a code-vs-brief
conflict: STOP-1's own "leave them unconverted, list them, and finish the rest" (five-or-fewer) directly
licenses leaving these two exactly where they are, and neither site's file happens to fall inside the
gate's named exemption zones. Flagging it rather than fudging either reading.

**Census** (`scripts/replay/census.sh --diff`) — pre-census `.census/2026-09-28T23-59-17Z.txt` (the
latest snapshot on disk before this resume began, captured under the wall-less checker) against a
fresh whole-tree census taken after the wall + full 180-row table were applied:
**STOP-8, 84 files, exit 8**, every one under `wat-scripts/fixes/**` (verified: zero flips outside that
directory). This is the SAME directory the wall's-reach gate exempts from its search — but exemption
from "must not contain an untyped call, syntactically" does not mean exemption from "must still
type-check": the wall has no file-based carve-out, so any wat-scripts/fixes/** script that still has an
untyped constructor (674 sites measured across those 84 files, via `wat --check <f> | grep -c untyped`)
now fails to check at all. **Out of this stone's scope** — see "What remains, and why it is out of
scope" below.

**Release floor** — `scripts/floor.sh`, foreground, one run at a time, never piped for its exit code.

First run (full table + wall, before the two corrections above): **RED**, `.floor/2026-09-30T22-45-55Z/`,
`Summary [ 382.886s] 6235 tests run: 6138 passed (21 slow), 97 failed, 24 skipped`. Captured whole per
doctrine; not re-run blind. Reading the ARM blocks found: (a) the two STOP-1-mislabel sites cascading
through `defservice`'s accept loop into ~150 spurious errors system-wide; (b) the `wat/bracket.wat:491`
`:O`/`~ret-ty` bug (3 tests, same line, both directions); (c) a new self-reference trip on
`tests/lint/one_variant_separator.rs`'s own COMPOSE-pattern lint (`untyped_constructor_error`'s
`format!(":wat::core::{head_short}")` reads as hand-spelling a `::`-separator — cured with a
`rune:lint(one-variant-separator, type-path)` exemption comment, since it composes a namespaced TYPE
keyword, never an enum::variant pair); (d) an inline-Rust-test-fixture constant
(`DEPTH_SPLIT_WORLD`, `src/rete/kernel/tests/mod.rs`) and a 15-call repeated pattern in
`src/rete/reachability.rs` (`(:wat::core::PersistentVector (:probe::q))`, `Query`-typed, 13 sites) plus
2 more (`i64`/`String`-typed) in the same file — all cured, all Rust `.rs` source (not `.wat` corpus,
so the codemod rule does not apply; hand-edited directly, each verified against its own declared
element type before editing); (e) an inline Rust unit test in `src/check.rs` itself
(`arc109_two_iii_check_time_ctor_guard_widening::row1_infer_list_constructor_accepts_parametric_form_first_arg`)
building two untyped `(:wat::core::Tuple 1 2)`/`(3 4)` fixtures — bracketed.

Cured (a), (b), (c), (d), (e); rebuilt; re-ran the full floor.

Second run: **still RED**, `.floor/2026-09-30T23-01-26Z/`,
`Summary [ 384.433s] 6235 tests run: 6173 passed (21 slow), 62 failed, 24 skipped` — 35 fewer
failures (6138→6173 passed, exactly the delta). The 62 remaining span: the `wat-scripts/fixes/**`
cascade itself (`wat::lint wat_scripts_fixes_load::every_wat_scripts_file_loads_on_the_current_runtime`,
`wat::lint rete_compile_gate::*` 15 shards, `wat::cli every_recorded_migration_replays::*` 16 shards,
`wat::cli stdlib_door_reads_a_set_as_one_world`); MORE distinct inline-Rust-fixture constants I did not
reach (`src/rete/kernel/tests/{arm_lease,binding_repr_bench,node_share_cost*,pass_semantics,
rank_and_instrument,where_tree_branch_differential}.rs`, `src/rete/reachability.rs`'s remaining 5
`reachability_shard_N_of_6` — a DIFFERENT embedded fixture than the 13-site one already cured,
`tests/rete/probe_arc278_seq1b_list_hofs.rs`, `src/collection/transform.rs`,
`src/runtime.rs:14746`, `tests/types/tuple.rs`); and one PRE-EXISTING, unrelated red
(`probe_arc170_wrong_service_compile_error::swapped_colocation_tuple_is_compile_error` — its failure
is a stale golden `got` string expecting a 3-arg `Address` with a `Transport.Wire` third slot, where
the file's OWN untouched lines 20-21/33-34 already carry `positional variant construction is retired`
`MalformedForm`s having nothing to do with constructors at all; confirmed these exact lines were never
touched by either this stone's typing work or its corrections — this test almost certainly failed on
the unmodified draw too, unrelated to 255.71).

This floor is NOT green. Did not re-run a third time: the remaining 62 are a different, much larger
class of problem than "this stone's own gap" (below), and doctrine's cure-then-reverify loop is not a
license to keep iterating past that boundary hoping for zero.

## What remains, and why it is out of scope

The 177-site census (and this resume's 180-row table) covered the `.wat` **corpus** — files
`scripts/replay/census.sh` walks. The wall is a property of the **checker**, with no file-based
exemption, and it fires on **every** `.wat` PROGRAM the runtime ever parses — including one assembled
at runtime from a Rust `&str` literal embedded in `src/`/`tests/` `.rs` files, and including every
`wat-scripts/fixes/**` migration script (explicitly exempt from the wall's-reach SEARCH gate, but not
from actually type-checking).

Measured, not estimated:
- **`wat-scripts/fixes/**`: 84 files, 674 untyped-constructor call sites** (`grep -c
  "untyped.*constructor call"` over each file's own `wat --check` output, summed). All previously
  checked clean (`.census/2026-09-28T23-59-17Z.txt`, every one rc 0); all now rc non-zero
  (`scripts/replay/census.sh --diff`, STOP-8).
- **Rust source files with an untyped-constructor pattern embedded in a string literal**: a
  corpus-wide `grep -l` across `src/` and `tests/` for the same 4 head patterns (excluding an
  existing `:- [`) found **40 files** (`src/check.rs`, `src/closure_extract.rs`,
  `src/collection/eval.rs`, `src/intrinsic/{holon/engram,holon/reckoner,linkedlist,list,rete,seq,
  special/ann_form,special/rete_alias,vector}.rs`, `src/rete/{eval_insert,eval_test,reachability,
  step_payload}.rs`, `src/rete/kernel/tests/{accum_cost,arm_lease,binding_repr_bench,cascade_cost,
  fanout_cost,harvest_cost,mod,pass_semantics,rank_and_instrument,strat_cost,stratify_numbers,
  where_tree_branch_differential}.rs`, `src/runtime.rs`, `src/value/value.rs`,
  `tests/collection/{list,probe_arc216_stone7_tuple_roundtrip}.rs`,
  `tests/rete/probe_arc278_{6b_ii_b_where_native_differential,8a_accumulate_oracle,
  8b_accumulate_native_differential,8custom_native_differential,P6_delta_asymmetric_join,
  accumulate_from_types,deep_cascade,seq1b_list_hofs}.rs`, `tests/types/tuple.rs`). 9 of these
  (`check.rs`, `rete/kernel/tests/mod.rs`, `rete/reachability.rs`) are now fixed at their known-bad
  sites; the bulk is not — a FIRST PASS, not a census (I did not run `--check` against every one of
  these 40 to count sites the way I did for `wat-scripts/fixes/**`, so this is a floor, not a total).

Fixing either of these is a genuinely different kind of work than this stone's 177-site census: most
`wat-scripts/fixes/**` scripts are completed, already-landed migration tooling whose original author
had domain context I do not; several of the Rust-embedded fixtures are large, repeated, or
parametrized string templates (not simple literal calls) where a wrong type choice is a silent
correctness bug in a PERFORMANCE benchmark or a DIFFERENTIAL oracle, not a compile error I'd notice.
Converting ~700+ more sites at that kind of risk, unplanned, is not "curing this stone's own gap" —
it is a new stone's worth of work the brief never scoped (the 177-site census; "Vector 10" was the
largest single-head count named). I fixed the highest-measured-impact subset I could positively verify
(reducing the floor from 97 to 62 failures) and STOP here, surfacing the rest rather than either
papering over it or spending unbounded, unauthorized effort converting the remainder blind.

**This is also a genuine design question for the builder**, not just more typing: should the wall
carry a file-path exemption matching `wat-scripts/fixes/**`'s existing search-gate exemption (frozen,
replay-only migration tooling, by the codemod's own convention, need not obey a later stone's new
rule)? Should Rust-embedded `.wat` string literals used only for internal test/benchmark fixtures get
a lint-level `rune:lint(...)`-style opt-out instead of being hand-converted one by one? Neither call is
mine to make.

## Heretics left unconverted (full list, with reasons)

See the committed table for the complete, precise version; summarized:

- **Genuine STOP-1 (2, both ≤5, not a STOP)**: `wat/rete/oracle/accum-pass.wat:135` (a custom
  accumulator's element type is whichever field a user-authored rule's own accumulator var binds — a
  real per-rule design choice, not statically knowable); `wat-scripts/probes/arc-170/probe-s3b-astsplice.wat:75`
  (the type is built at runtime via string-concat + AST splicing — no static text exists to read a
  type off of).
- **Legacy-shape (4, `.wat.bad`, gate-exempt, codemod-incompatible)**: `tests/function/probe_arc237_7b_{conj,contains}_wrong_elem.wat.bad`,
  `tests/services/probe_arc278_struct_new_nature_wall.wat.bad`, `tests/types/probe_stone_118_b1a_neg.wat.bad`
  — all a pre-`:-` positional-type-keyword shape, already illegal via arc 109's own wall, unrelated to
  this stone.
- **Positive control (2, `.wat.bad`, gate-exempt)**: `tests/lint/probe_c19_nested_type_var_render.wat.bad`'s
  two sites — typing them destroys the only corpus coverage of `format_type_inner`'s `Var` arm.
- **Removed (1, moot)**: `tests/function/probe_stone255_70_constructor_brackets_through_the_door.wat:41:3` —
  the site no longer exists; a prior edit relocated it.

## Reds (full account)

No red was caused by a heretic this stone declined to convert (the STOP-1/legacy-shape/positive-control
sites are all `.wat.bad` or otherwise gate-exempt, and none appears in either floor's failure list).
Every red traced to the wall itself exposing either (a) a genuine error in this resume's OWN table (the
3 mislabeled STOP-1 sites, the `bracket.wat` `:O` bug — both corrected, verbatim detail above), or
(b) code this stone's 177-site census never covered at all (Rust-embedded `.wat` string literals,
`wat-scripts/fixes/**`). Category (a) is cured, verified, and reflected in the second floor's reduced
count. Category (b) is NOT cured — see "What remains" above — and is why the floor, and this stone,
are not green.

## Commits

- `2fb4578a9` — the wall, the committed table, the 180-site disposition, the 2 table corrections + the
  `bracket.wat` correction, the 5 Rust-fixture cures (`check.rs`'s own lint self-trip, `arc109_two_iii`,
  `DEPTH_SPLIT_WORLD`, `reachability.rs`'s 2 patterns), 7 new tests, the span-claim correction to
  `probe_stone255_71_template_untyped`'s test + fixture + `src/check.rs` doc comment.
- This SCORE, to follow.

`scratch/` (this resume's working TSVs/census) deleted, per the AMEND — ephemeral, never committed.
