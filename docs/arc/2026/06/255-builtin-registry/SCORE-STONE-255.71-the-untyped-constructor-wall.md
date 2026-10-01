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

---

# AMEND-2 resume (2026-09-30/10-01): the wall lands green

**Executor: a Sonnet subagent.** Continues from the two commits above plus the WEIGH `63a2518db`
and `AMEND-2-STONE-255.71-the-wall-lands-green.md`'s own M1 ruling: the recorded codemods are wat
programs the same runtime type-checks, so the wall reaches them with no path exemption, and the
same holds for wat written inside Rust string literals. Commits `0374d9c78`, `a3b2eeb3b`,
`ff192216d`, all local on `main`, not pushed.

## 1. The 84 recorded codemods — converted

**Measured correction to the AMEND's own "674 sites" figure.** A structural AST walk (the SAME
`is-hit?` shape `typed-constructors.wat`'s own census logic uses, run via a throwaway
`scratch/census-fixes-sites.wat`, deleted after use) over the 84 `wat-scripts/fixes/*.wat` files
`wat --check` flagged found **338 sites**, not 674. The AMEND's figure double-counted: its
`grep -c "untyped.*constructor call"` pattern matches BOTH the `CheckError`'s `:message` field and
its `:reason` field for the SAME error (both contain the literal text "untyped ... constructor
call"), so `674 / 2 = 337` unique flagged errors — confirmed by summing each file's own declared
`:message "N type-check errors"` count across all 84 files (337, matching the parsed-error count
exactly). The AST walk found one further genuinely-untyped site the wall does **not** flag at all:
`wat-scripts/fixes/response-record-to-enum.wat:100:21`, a `Tuple` constructor checked via
`check_tuple_constructor_against` against an already-known expected type (the enclosing
`:user::add-record`'s own declared `HashMap` value type), never reaching
`infer_tuple_constructor`'s bracket-less branch — typed anyway for "every collection constructor
has one shape" consistency. **337 + 1 = 338.**

**Types harvested via the stash-dance** (`wat/fix.wat`'s own BOOTSTRAP note, applied to the
WALL itself rather than a new verb): the wall's 4 `local_errors.push(untyped_constructor_error(…))`
call sites in `src/check.rs` (lines 15692/15797/15965/16551 — `infer_persistentmap_constructor`,
`infer_persistentvector_constructor`, `infer_tuple_constructor`, `infer_linked_list_constructor`'s
own `None =>` arms, each already "degrades gracefully" per their own doc comments, so commenting
the `push` out changes no other control flow) were commented out with a `STASH-DANCE-255.71-TEMP`
marker, `cargo build --release` rebuilt an old-checker binary, and `WAT_CHECK_TYPES=1 wat --check
<file>` was run over all 84 files — every one type-checks clean with the wall lifted — to harvest
the checker's own recorded type at each site's exact call span (`src/check/type_record.rs`'s
`Recording`, dumped via `src/distribution/check_output.rs`'s `emit_types`). Each site's own call
span (the `(`, matching `typed-constructors.wat`'s own `c69::table-key` convention) was joined
against the type record by exact (file, line, col) — 337 of 338 joined cleanly; the one miss
(`response-record-to-enum.wat:100:21`, never independently checked since it's reached via
`check_against`) was typed by reading the enclosing function's own declared signature.

The compact tuple notation `format_type_inner` renders (`:(A,B)`, confirmed not reader-parseable,
matching `typed-constructors.wat`'s own header note) was bridged **by hand** to the parseable
`(:wat::core::Tuple :- [A B])` form, recursively, restoring the leading colon a namespaced leaf's
compact rendering drops — only **10 distinct raw shapes** covered all 337 harvested sites
(verified: no site's raw type fell outside this set).

**Committed table:** `TABLE-STONE-255.71-fixes-typed-constructors.edn` — a SIBLING to the corpus's
own table, per the AMEND's own licensed shape ("append them to the committed table or a sibling
table beside it"). 338 `:typed` rows, no STOP-1/legacy-shape/positive-control/removed rows arose in
this population.

**Applied** via `typed-constructors.wat`'s existing table-driven path, wall restored
(`git checkout -- src/check.rs`, rebuilt) before applying (the codemod itself only needs to LOAD,
which it already does post-`2fb4578a9`'s self-conversion — no second stash-dance needed to apply).
Dry-run on `/tmp` copies first, diffed against hand-read expectations (`kill-make-deftest.wat`,
`bare-symbol-shorthand-to-fqdn.wat`, `hoist-where-into-condition.wat`'s nested-tuple case all
verified correct); then applied to the real 84 files. **Idempotent**: re-run over all 84 files
(both immediately after the dry-run and again after the real apply) — 0 changes both times.
**Verified**: all 84 converted files individually `wat --check` clean (rc 0) under the restored
wall.

Commit `0374d9c78`.

## 2. Inline wat in Rust string literals — censused and converted

**Census methodology:** `grep -rnP` over `src/**/*.rs` and `tests/**/*.rs` for `(` immediately
followed by one of the 5 codemod heads (either spelling) with a negative lookahead for `:- `,
then **excluded** (a) lines whose content is a `///`/`//!`/`//` doc comment (confirmed by direct
inspection: every hit in `src/intrinsic/vector.rs`, `src/intrinsic/linkedlist.rs`,
`src/intrinsic/map.rs`, `src/collection/eval.rs`, `src/types.rs`, `src/value/value.rs`,
`src/closure_extract.rs`, `src/record/access.rs`, `src/rete/{eval_insert,eval_test,
step_payload}.rs`, `tests/cli/type_record.rs`, `tests/collection/{list,
probe_arc216_stone7_tuple_roundtrip,probe_arc278_0c_persistent_parity}.rs`,
`tests/function/probe_stone255_71_list_bracketless_illegal.rs`, `tests/types/tuple.rs`,
`src/edn/render.rs`, `src/intrinsic/{holon/engram,holon/reckoner,mod,seq,list,special/ann_form,
special/rete_alias,rete}.rs`, `src/collection/infer.rs` is inside an `/// @example (…) #=> N`
rustdoc comment — illustrative text no reader ever parses as a program). This left **55 real
sites across 19 files**.

Converted all 55 (the `:lhs`/`:rhs` `PersistentVector<WatAST>` pattern from `wat/rete.wat:48-51`'s
own `Rule` record, and `compile`/`compile-all`'s `PersistentVector<Rule>`/`<Query>` args, repeated
across `src/rete/reachability.rs` (10), `src/rete/kernel/tests/{mod,arm_lease,pass_semantics,
binding_repr_bench,stratify_numbers,termination_verdict,strat_cost,rank_and_instrument,
harvest_cost,fanout_cost,cascade_cost,accum_cost,where_tree_branch_differential}.rs`,
`src/collection/transform.rs`, `src/runtime.rs`, `tests/rete/probe_arc278_{P6_delta_asymmetric_join,
deep_cascade,accumulate_from_types,8custom_native_differential,8b_accumulate_native_differential,
8a_accumulate_oracle,6b_ii_b_where_native_differential,seq1b_list_hofs}.rs` — one `probe_s3b`-style
nested-Tuple-of-Vector-of-Tuple case in `probe_arc278_P6_delta_asymmetric_join.rs` hand-verified
against `hoist-where-into-condition.wat`'s already-proven identical shape).

Three were found and left alone, each verbatim-confirmed, not assumed:
- `tests/function/probe_stone255_69_wat_type_scalar_dispatch.rs` (2) and
  `tests/resolve/probe_arc251_keyword_to_type_form.rs` (1): every occurrence sits inside a
  `panic!()`/`assert!()` message string illustrating what the ACTUAL wat source (a separate
  `.wat`/`.wat.bad` fixture read via `startup_from_file`/`startup_beside`) does — never wat source
  itself.
- `src/rete/kernel/tests/where_tree_branch_differential.rs`'s `MID_CORE` constant: a corpus-TEXT
  substring matched via `src.find(MID_CORE)` to classify OTHER files' raw text — never parsed as
  its own program; converting it would change what it matches, not fix a type.

**4 further sites, found in `src/runtime.rs`, are genuinely never type-checked** — fed through
`eval_expr`/`eval_with_ctx` (both: parse → macro-expand → `eval_inner`, no `check_program` call
anywhere in the path) rather than `startup_from_source`. Confirmed by reading both functions'
full bodies. Converted anyway (same "one shape" consistency; harmless since nothing validates the
bracket on that path).

Commit `ff192216d`.

## 3. This stone's own 62 reds — cured

Diagnosed from `.floor/2026-09-30T23-01-26Z/ARM.txt` (the AMEND's own red floor, 62 failed/6235),
captured verbatim per doctrine before any fix. Every root cause traced to code item 1 or item 2
above was already converting; the following were the DISTINCT mechanisms (not 62 — many shards
share one cause):

**(a) `rete_compile_gate` 16 shards + `stdlib_door_reads_a_set_as_one_world` (17 tests) — ONE
site.** `tests/lint/rete_compile_gate.rs`'s `driver_defn` builds a synthesized `:census::run` defn
APPENDED to every `wat-scripts/` file with a declared rete namespace; its own body contained an
untyped `(:wat::core::PersistentVector {calls})` (the per-namespace `CompileOutcome` collector) and
`(:wat::core::PersistentVector)` (each `compile-all` call's empty queries arg) — one shared site
firing on every file the driver touches. Verbatim (from the second re-run after other 255.71
corrections, `.floor/2026-09-30T23-01-26Z/ARM.txt`):
```
🔥 12 wat-scripts/ file(s) in shard 9/16 DECLARE a rete rule that does not COMPILE. ...
  wat-scripts/fixes/rename-core-set-and-list-to-their-homes.wat
      startup: #wat.check/CheckErrors {:message "2 type-check errors" ... :errors
      [#wat.check/MalformedForm {:message "malformed :wat::core::PersistentVector form: untyped
      `:wat::core::PersistentVector` constructor call — every `PersistentVector` needs its own
      type bracket, whatever the head's spelling; write `(wat.type/PersistentVector :- [T…] …)`"
      :location #wat.core/Span {:file "wat-scripts/fixes/rename-core-set-and-list-to-their-homes.wat"
      :line 246 :col 5 ...} ...}]}
```
Fixed: `driver_defn` bracketed both constructor calls (`:- [:wat::rete::Query]` /
`:- [:wat::rete::CompileOutcome]`).

**(b) `wat_scripts_fixes_load` + `every_recorded_migration_replays` 16 shards + `_positional_ctor`
(18 tests) — item 1 above.** E.g. shard 4's verbatim capture:
```
thread 'every_recorded_migration_replays::every_recorded_migration_replays_shard_4' panicked at
tests/cli/every_recorded_migration_replays.rs:541:5:
recorded-migration replay shard 4 failed:
break-kind-string-to-enum: first run rc=exit status: 3 ... #wat.check/CheckErrors {:message "1
type-check error" ... #wat.check/MalformedForm {:message "malformed :wat::core::Tuple form:
untyped `:wat::core::Tuple` constructor call ..." :location ... {:file
"wat-scripts/fixes/break-kind-string-to-enum.wat" :line 33 :col 24 ...} ...}
```
(and 5 more files in the same shard's block: `inline-constraint-per-type-spelling`,
`read-string-to-outcome`, `rename-four-families-to-their-homes`, `response-record-to-enum`,
`wrap-compile-in-compileoutcome`). Fixed by item 1's 338-site conversion.

**(c) `reachability_shard_0..5_of_6` (6 tests) — `src/rete/reachability.rs`.** Verbatim (shard 0):
```
thread 'rete::reachability::reachability_shard_0_of_6' panicked at
src/rete/reachability.rs:1012:5:
the GENERATOR is wrong for these cells, not rete — a TemplateDefect says the synthesized program
is malformed ...
  :wat::rete::linkedlist::get @ inline-constraint: Unattributed — #wat.check/CheckErrors
  {:message "2 type-check errors" ... #wat.check/MalformedForm {:message "malformed
  :wat::core::List form: untyped `:wat::core::List` constructor call ..." :location
  {:file "<entry>" :line 18 :col 104 ...}} ...
```
Root cause: 10 untyped `PersistentVector`/`List` value-literal fixtures (the `special_for`/
Cell-building tables' hit/miss facts for `fn`/`foldl`/`reduce`/`mapv`/`filterv`/`vector::*`/
`linkedlist::get`/`*::first` rows) fed as `{f}` substitutions, outside any expected-type context.
Fixed: 10 literal pairs bracketed with `:- [:wat::core::i64]`.

**(d) `arm_lease` 6 tests, `node_share_cost` 5 tests, `rank_and_instrument::cell_rank_after_fanout`,
`where_tree_branch_differential` — shared `SCOPED_WORK_WORLD`/`NODE_SHARE_WORLD` fixtures.**
Verbatim (`scoped_work_with_network_base_untouched`):
```
thread 'rete::kernel::tests::arm_lease::scoped_work_with_network_base_untouched' panicked at
src/rete/kernel/tests/arm_lease.rs:398:10:
scoped-work world should freeze: #wat.check/CheckErrors {:message "2 type-check errors" ...
#wat.check/MalformedForm {:message "malformed :wat::core::PersistentVector form: untyped
`:wat::core::PersistentVector` constructor call ..." :location {:file "<entry>" :line 13 :col 36
...}} #wat.check/MalformedForm {... :line 14 :col 36 ...}]}
```
Root cause: `SCOPED_WORK_WORLD`'s `:lhs (:wat::core::PersistentVector c1 c2) :rhs
(:wat::core::PersistentVector rhs)` and `NODE_SHARE_WORLD`'s identical pattern — `wat/rete.wat:
48-51`'s `Rule` record declares both fields `PersistentVector<WatAST>`. Fixed: both worlds'
`:lhs`/`:rhs` bracketed `:- [:wat::WatAST]`.

**(e) `pass_semantics::seed_batches_uniform_classes_and_defers_mixed_ones`.** Verbatim:
```
thread '...seed_batches_uniform_classes_and_defers_mixed_ones' panicked at
src/rete/kernel/tests/pass_semantics.rs:... D7_ERASURE_WORLD ... #wat.check/CheckErrors
```
Root cause: `D7_ERASURE_WORLD`'s `compile-all`'s rules/queries args and `:d7c::fire`'s
`PersistentVector<Record>` facts, untyped. Fixed, all 4 sites bracketed.

**(f) `binding_repr_bench::binding_cardinality_distribution`.** Verbatim:
```
thread '...binding_cardinality_distribution' panicked at
src/rete/kernel/tests/binding_repr_bench.rs:203:10:
join world should freeze: #wat.check/CheckErrors {:message "3 type-check errors" ...
#wat.check/MalformedForm {... :line 8 :col 70 ...} {... :line 8 :col 112 ...} {... :line 9 :col 76
...}]}
```
Root cause: `J`'s `rule`'s `:lhs`/`:rhs` (line 8, both element positions) and the direct
`(:wat::rete::compile (:wat::core::PersistentVector rule))` argument (line 9) — this last one
PROVES a direct-argument-position call is **not** uniformly exempt from the wall (a different
`compile`/`compile-all` call elsewhere, fed via `check_against`, is exempt — see item 1's
`response-record-to-enum.wat:100:21` finding — so each site had to be checked empirically, not
assumed from shape alone). Fixed, all 3 sites bracketed.

**(g) `collection::transform::{filter_native_tests,seqable_to_stream_tests}` (2 tests).**
Verbatim:
```
thread 'collection::transform::seqable_to_stream_tests::seqable_to_stream_keep_stays_under_wall_at_n4000'
panicked at src/collection/transform.rs:1763:14:
world should freeze: #wat.check/CheckErrors {:message "1 type-check error" ...
#wat.check/MalformedForm {:message "malformed :wat::core::PersistentVector form: untyped
`:wat::core::PersistentVector` constructor call ..." :location {:file "<entry>" :line 8 :col 2
...}} ...}
```
Root cause: both n4000 wall-regression fixtures' `foldl` accumulator seed
`(:wat::core::PersistentVector)` (empty). Fixed: `:- [:wat::core::i64]`.

**(h) `runtime::tests::step_tail_recursion_terminates_under_bound`.** Verbatim: the test's own
`(:wat::core::Tuple sum steps)` result tuple, untyped — `sum` is `:wat::WatAST` (via
`step-to-terminal`'s declared return), `steps` is `:wat::core::i64`; confirmed from the match arms
immediately below the construction (`Value::wat__WatAST(h)` / `Value::i64(n)`). Fixed.

**(i) `probe_arc278_seq1b_list_hofs::{list_hofs_preserve_container,list_hofs_typecheck_parametric}`
(2 tests).** Verbatim:
```
thread 'probe_arc278_seq1b_list_hofs::list_hofs_typecheck_parametric' panicked at
tests/rete/probe_arc278_seq1b_list_hofs.rs:86:5:
all 8 HOFs must type-check on a List/of (parametric). Got: Err(#wat.check/CheckErrors {:message
"9 type-check errors" ... #wat.check/MalformedForm {:message "malformed :wat::core::List form:
untyped `:wat::core::List` constructor call ..." ...} ...})
```
Root cause: the shared `L123` const (`"(:wat::core::List 1 2 3)"`) used by all 8 generated HOF
test programs, plus one inline `(:wat::core::List 4 5)` literal in `list_concat_nxm`. Fixed:
`L123` and the inline literal both `:- [:wat::core::i64]`.

**(j) `types::tuple::legacy_tuple_lowercase_redirects_via_pattern2_poison` — a genuine correction,
not new typing.** Verbatim (actual vs. golden mismatch at the point of failure):
```
left:  #wat.check/CheckErrors {:message "2 type-check errors" ... [TypeMismatch{...} ,
       MalformedForm{:message "malformed :wat::core::Tuple form: untyped `:wat::core::Tuple`
       constructor call ..." :location {... :line 2 :col 116 ...} :head ":wat::core::Tuple" ...}]}
right (the committed golden): #wat.check/CheckErrors {:message "1 type-check error" ...
       [TypeMismatch{...}]}
```
The wall now ALSO fires its own `MalformedForm` on the `.wat.bad` fixture's lowercase-`tuple` call
— genuinely untyped, unrelated to the retired-verb `TypeMismatch` already asserted there. The
golden EDN (`tests/types/tuple__legacy_tuple_lowercase_redirects_via_pattern2_poison.edn`) was
updated to the real 2-error `CheckErrors` the checker now produces, in the SAME field order and
indentation the file already used. The property under test (retired verb → `TypeMismatch` +
`Retirement` remedy) is unchanged; the `.wat.bad` fixture itself (a NEGATIVE control for a
DIFFERENT, pre-existing wall) was left untouched.

**(k) `services::probe_arc170_wrong_service_compile_error::swapped_colocation_tuple_is_compile_error`
— the AMEND's own named false-"pre-existing" claim; a genuine table-row correction.** Verified
against the PRIOR green floor per doctrine: `.floor/2026-09-28T22-13-36Z/clean.log:4253`:
```
PASS [   0.969s] (4221/6225) wat::services probe_arc170_wrong_service_compile_error::swapped_colocation_tuple_is_compile_error
```
— so 255.71 caused this, confirmed, not pre-existing. Verbatim failure:
```
no check error matched `CheckErrorKind::TypeMismatch { expected, got, .. } if
expected == ":((wat::kernel::Address :- [probe::Echo::Op probe::Echo::Reply]),
             (wat::kernel::Address :- [probe::Kv::Op probe::Kv::Reply]))"
&& got == ":((wat::kernel::Address :- [probe::Kv::Op probe::Kv::Reply wat::kernel::Transport.Wire]),
          (wat::kernel::Address :- [probe::Echo::Op probe::Echo::Reply wat::kernel::Transport.Wire]))"`;
errors were: #wat.check/CheckErrors {:message "5 type-check errors" ... [4 unrelated
MalformedForm "positional variant construction is retired" errors on this file's OWN untouched
lines 20/21/33/34, then] #wat.check/TypeMismatch {... :expected
":((wat::kernel::Address :- [probe::Echo::Op probe::Echo::Reply]),(wat::kernel::Address :- [probe::Kv::Op probe::Kv::Reply]))"
:got ":((wat::kernel::Address :- [probe::Kv::Op probe::Kv::Reply]),(wat::kernel::Address :- [probe::Echo::Op probe::Echo::Reply]))"
...}]}
```
Diagnosis: `2fb4578a9`'s own table row typed `good`/`bad`'s Tuple elements from
`:probe::dial-all`'s DECLARED 2-arg `Address` contract — but `:wat::capability::Dialable/coord`
(the actual value-producing call) is declared `-> (:wat::kernel::Address :- [S R T])`
(`wat/capability.wat:51`), T being `Transport.Wire` per Stone 255.21, confirmed by this exact
test module's OWN untouched sibling `wrong_service_coord_is_compile_error`, whose golden `got`
string already carries the Wire slot on this identical call. Before 255.71 (unbracketed), the
Tuple's element type was inferred structurally FROM `Dialable/coord`'s real 3-slot return (hence
the test's own, never-touched golden strings already expected Wire on both sides and PASSED); the
codemod's wrong 2-slot bracket forced the WRONG shape onto the constructor, turning the comparison
from a 3-slot one (matching the untouched golden) into a 2-slot one (mismatch). **Fixed by hand**
(the codemod cannot re-target an already-bracketed site — same class of exception as
`wat/bracket.wat:491`'s own precedent in this SCORE): `good`/`bad`'s brackets in
`tests/services/probe_arc170_wrong_service_colocation.wat.bad` corrected to the 3-slot
`[Op Reply Transport.Wire]` shape; the committed table's two corresponding rows corrected to match
and explain why.

Commits `a3b2eeb3b` (items a–k's fixes) and `ff192216d` (item 2's remaining census).

## STOP-1 sites (2, unchanged from `2fb4578a9`, re-confirmed by the wall's-reach gate below)

- `wat/rete/oracle/accum-pass.wat:135` — a custom accumulator's element type is whichever field a
  user-authored rule's own accumulator var binds; a real per-rule design choice, not statically
  knowable.
- `wat-scripts/probes/arc-170/probe-s3b-astsplice.wat:75` — the type is built at runtime via
  string-concat + AST splicing; no static text exists to read a type off of.

Both left unconverted, as the original SCORE already found and the AMEND does not revisit.

## Gates

**Clippy** — `cargo clippy --release --all-targets -- -D warnings`: rc 0, clean, both before and
after every commit in this resume.

**Idempotence** — `typed-constructors.wat` re-run over the 84 `wat-scripts/fixes/` files: 0
changes (checked immediately after the dry-run apply and again after the real apply).

**Census** (`scripts/replay/census.sh --diff`) — fresh whole-tree census
`.census/2026-10-01T00-15-17Z.txt` (2268 files) against the pre-stone baseline
`.census/2026-09-28T23-59-17Z.txt`:
```
census-diff: no STOP-8
```
Zero files flip rc 0 → non-zero — the 84-file STOP-8 the prior SCORE reported is fully resolved,
with no new regression anywhere else in the tracked `.wat` corpus.

**The wall's reach** — a corpus-wide search (every tracked `.wat` file outside `.wat.bad`,
`.wat.golden`, `wat-scripts/fixes/**`, 2147 files) for a bracket-less call of the 5 codemod heads,
excluding `;;`-comment lines and the `Head?`-predicate false match (`List?` is not the `List`
constructor): **exactly 2 hits, both the already-documented, licensed STOP-1 sites above** — the
same 2 the original SCORE found, re-confirmed after this resume's full conversion. The gate's
literal "0 outside…" wording is still not fully met, for the same reason the original SCORE
flagged: STOP-1's own "leave them unconverted, list them, finish the rest" directly licenses
leaving these two exactly where they are, and neither file falls inside the gate's named exemption
zones. Not re-litigated; the builder's call, not this resume's.

**Release floor** — `scripts/floor.sh`, one run at a time, in the foreground, never piped for exit
code.

*First run this resume* (after items 1–3 above, all commits landed):
`.floor/2026-10-01T00-13-03Z/`, **RED**:
```
Summary [ 448.033s] 6235 tests run: 6151 passed (30 slow), 84 failed, 24 skipped
```
**Diagnosed before any further action, per doctrine.** All 84 failures share the exact same shape
— verbatim (one of 84, representative; every other failure in this run is byte-identical in
structure, differing only in the deftest name and source `.wat` file):
```
thread 'test::deftest_wat_tests_edn_test_write_bool' panicked at
tests/kernel/test.rs:17:1:
deftest_wat_tests_edn_test_write_bool: exceeded time-limit of 5000ms — deftest
:wat-tests::edn::test-write-bool at wat-tests/edn/render.wat:10:1 (test thread leaked — process
exit will reap)
```
All 84/84 are `exceeded time-limit` wall-clock panics at the SAME harness line
(`tests/kernel/test.rs:17:1`), across 84 UNRELATED `wat-tests/*.wat` deftest files spanning
completely different subsystems (edn, gen, holon, …) — not a shared code path a type-check
regression could touch. **Root cause, found and owned**: this resume ran
`scripts/replay/census.sh` (32 parallel `wat --check` subprocesses, 2268 files) CONCURRENTLY with
this floor's own `cargo nextest run --release` execution — a doctrine violation (floor.sh's own
header: "one run at a time") committed by this resume itself, not a code defect. The two
processes' timestamps overlap exactly: the floor started `00:13:03Z`, ran 448s; `census.sh` was
launched at `00:15:17Z`, squarely inside that window. Machine-load contention from the extra 32
parallel heavy processes is sufficient and sole explanation for 84 UNRELATED deftests all missing
an identical 5-second wall-clock budget simultaneously. This is NOT the "pre-reproducible/
known-flake" dismissal doctrine forbids — it is a logged, concrete, singular external cause with a
timestamp trail, not a guess, and the fix is procedural (don't run anything else during a floor),
not a code change. The red was NOT re-run; this exact log is kept at `.floor/2026-10-01T00-13-03Z/`
as the record of the contaminated attempt.

*Clean run* (immediately after, nothing else running concurrently, confirmed via `ps aux` before
launch): `.floor/2026-10-01T00-21-52Z/`, **GREEN**:
```
Summary [ 388.083s] 6235 tests run: 6235 passed (24 slow), 24 skipped
```
This is the floor of record. 6235/6235, every one of this stone's own 62 reds cured, no new
failures anywhere. The pre-stone-71 baseline (`.floor/2026-09-28T22-13-36Z`) ran 6225; this
resume added no new `#[test]` functions (only fixture/golden corrections), so the +10 delta is
entirely `2fb4578a9`'s own "7 new tests" claim plus whatever else that commit's diff carries —
not independently re-verified here, since it predates this resume.

## Commits (this resume)

- `0374d9c78` — type the 84 `wat-scripts/fixes` codemods' 338 untyped-constructor sites; the
  sibling committed table.
- `a3b2eeb3b` — cure the 62 floor reds (items a–k above).
- `ff192216d` — finish the Rust-literal census (item 2's remaining 19-file, 55-site sweep).
