# SCORE — the `where` fence's interior gets one question, and the gate is the deliverable

Executing strike per `DESIGN.md`. Written as-I-go per house rule.

## Re-derivation, before any edit

Read `DESIGN.md` in full, then `wat-rs/CLAUDE.md` in full (floor discipline, no-known-flakes,
scratch-`.wat` location, wat-fix codemod rule, `.wat.bad`-beside-its-test convention — none of it
reaches a rider any other way).

Checked `git log`/`git show --stat HEAD` first: HEAD (`85ca63ea1`) added only `DESIGN.md` — no
implementation exists yet. Working tree otherwise clean.

## The contract decision, restated in my own words before writing any code

**The predicate is "∃ a condition this constraint can LEGALLY MOVE INTO", not "∃ a condition that
binds these vars."** A naive binder-collection check would refuse legal rules at exactly three
places, because each is a var "bound" by something that is not a single alpha-testable fact
pattern in the `where`'s own scope:

- **`:or`** binds CONDITIONALLY — a `where` after the whole `:or`, reading a var one arm binds,
  cannot move into that arm without leaving every OTHER arm unfiltered (silently WIDENING the
  rule).
- **`:exists`** binds OUTWARD in wat (`leading-exists.wat` reads its leading `:exists`-bound var
  back out by string key) — a downstream `where` reading it is legal exactly where it sits, but
  hoisting the predicate INTO the exists's own inner condition changes what "at least one match"
  means.
- **An accumulate's result var** is not a field of any fact at all — `where-accum-lead.wat`
  already filters directly on one, and there is no clause list to inline into.

**Resolution taken:** rather than trying to positively reason about what IS safe to hoist across
these three, the check never looks at what they bind in the first place. `validate_when_entry`
threads a new `hoist_scope: Option<&[WatAST]>` parameter — `Some(when_conds)` at the top of a
rule's `:when` and transparently through `:and` grouping (sequential, not conditional — a `where`
and its binder in the same `:and` chain are exactly as hoistable as two top-level siblings), and
`None` the instant the walk crosses into `:or`/`:not`/`:exists`/an accumulate's `:from`. The
hoist-target search (`collect_hoist_targets`) only ever looks inside `hoist_scope`, so a var bound
inside any of the three divergence cases is never even offered as a candidate — under-collecting
by construction, which is the safe direction (misses a hoistable site, never refuses a legal one).

## Negative controls — written and driven BEFORE the wall existed

Per the ⛔ pin, I wrote the fixtures first and ran them against the UNMODIFIED code (`Where(_) =>
{}`, so every one of these already had to pass trivially — the point was proving the SHAPES
compile and mean what I think, not proving the current no-op accepts them).

Explored first as scratch `.wat` in `wat-scripts/scratch-pad/` (deleted once their purpose —
proving the shapes are real, engine-legal patterns — was served; kept as permanent fixtures under
`tests/rete/` instead, per the `.wat`/`.wat.bad`-beside-its-`.rs`-test convention, not scratch-pad,
since these are must-*compile* controls a lint gate should keep enforcing forever):

1. **`:or`** (`probe_where_fence_hoistable_or.wat`) — `?v` bound inside one `:or` arm by
   `:whor::Hi`, a `:where` nested in the SAME arm. Full compile/insert/fire/query harness
   (`Hi(v=20)`, `Lo(v=3)`): **`2`** — the arm with the filter passes it, the arm without one
   fires unconditionally.
2. **`:exists`** (`probe_where_fence_hoistable_exists.wat`) — a top-level `:where` AFTER a
   leading `:exists`, reading `?loc`. `Wind(loc=20)` → **`1`** (exists binds outward, `where`
   reads the real value, both accepted and correct).
3. **accumulate** (`probe_where_fence_hoistable_accumulate.wat`) — mirrors the REAL corpus file
   `wat-scripts/perf/grid/where-accum-lead.wat` (`?n <- (acc::count) :from (Reading)`, then
   `(:wat::rete::where (:wat::rete::core::i64::= ?n 0))`): empty world, `count = 0`,
   `(= 0 0)` → **`1`**.
4. **genuine cross-condition beta join** (`probe_where_fence_hoistable_join.wat`) — the actual
   REASON `:where` exists, added beyond DESIGN's three because it is the most important case not
   to break: `?a`/`?b` from two DIFFERENT top-level conditions (`A(20)`, `B(5)`, `(> 20 5)`) →
   **`1`**. If this ever failed, the wall would be refusing the majority use of `:where` in the
   corpus.

All four compiled, fired, and derived the EXACT counts asserted (a count is not enough on its own
— `where_over_an_or_bound_var_stays_legal` in particular checks `2`, not merely "ok", because a
cure that silently dropped the arm-scoped filter would still report a truthy exit code) under the
**unmodified** validator (`Where(_) => {}`), confirming: (a) these are real, engine-legal shapes,
not fixtures that merely fail to parse, and (b) a naive "binds these vars" check applied to any of
them would be a false refusal — the exact failure DESIGN's contract rules out. Re-driven again
AFTER the wall (below) with identical results.

## The wall

`src/rete/validate/mod.rs`:

- `validate_when_entry` gained a `hoist_scope: Option<&[WatAST]>` parameter, threaded `Some(…)`
  from both top-level callers (`validate_rule_when_and_reorder_then`, `validate_query_when`),
  propagated unchanged through `:and`, and set to `None` recursing into `:not`/`:exists`/`:or`
  arms.
- The `:282` arm:
  ```rust
  ReteClauseShape::Where(inner) => {
      if let Some(scope) = hoist_scope {
          check_where_hoistable(inner, scope, cond, rule_name, errors);
      }
  }
  ```
- `plain_pattern_bound_vars` — is this `:when` entry an ordinary fact pattern (`(:Type
  clause…)` or `(?p <- :Type clause…)`), and if so, which `?var`s does it bind? Everything
  else (`:or`/`:not`/`:exists`/`:where`/an accumulate) returns `None`.
- `collect_hoist_targets` — every hoist-target candidate reachable from `hoist_scope`,
  recursing transparently through `:and`, recording only `plain_pattern_bound_vars` hits.
- `check_where_hoistable` — collects the `where`'s referenced `?var`s
  (`collect_var_occurrences`, already existed for the wrapper-bind wall), and refuses **iff
  exactly one** candidate's bound-var set is a superset of all of them. Zero matches (nothing
  binds it, or it's split across conditions — a real join), or more than one full match
  (ambiguous), leaves the `where` alone.
- `render_hoisted_form` — the concrete rewrite: the binder condition's own `WatAST` with the
  `where`'s predicate appended as one more clause, rendered as real wat source (not a
  hand-composed illustrative example).
- New error kind `ReteCheckErrorKind::WhereHoistable { rule, fact_type, vars, predicate,
  rewrite }` (`error.rs`), house-styled after `UnconsumedWrapperBind`.

### `:456` — untouched, confirmed by reading the diff

`src/rete/validate/mod.rs` line ~456's clause-level `ReteClauseShape::Where(_) => {}` (the
stone-6 STOP arm) was not part of this edit — the only `Where` arm changed is `validate_when_entry`'s
(the top-level `:when`-entry dispatch), not `validate_clause`'s (within-condition clause dispatch).

## Refusal — exact text, driven

```
$ ./target/release/wat tests/rete/probe_where_fence_hoistable_refuse.wat.bad
```
produces (Debug/EDN form; the human-readable line is the `:message` field of the
`#wat.rete/WhereHoistable` struct, captured whole — not spot-checked by substring — in
`tests/rete/probe_where_fence_hoistable__refuse.edn`):

> defrule `whr::r`: `:where` predicate `(:wat.rete.core.i64/> ?n 10)` reads only var(s) [?n],
> every one bound by the single `:whr::Req` condition — this is an alpha test wearing a beta
> fence and can move inline, dropping the separate `:where`. Rewrite the `:whr::Req` condition
> as `(:whr/Req (?n <- :n) (:wat.rete.core.i64/> ?n 10))`

Names the variable (`?n`), names the condition that binds it (`:whr::Req`), and prints the
concrete rewrite — the house-style bar `UnknownField`/`UnconsumedWrapperBind` set. The two-var
shape (`probe_where_fence_hoistable_multivar_refuse.wat.bad`, mirroring the real corpus's
`where-string.wat`) refuses the same way, naming `[?n, ?minlen]` — golden in
`probe_where_fence_hoistable__multivar_refuse.edn`.

## Test-authoring correction, made BEFORE the mutation proof

The first draft of `tests/rete/probe_where_fence_hoistable.rs` used `startup_from_file` +
`format!("{:?}", err).contains(...)` substring checks. A targeted nextest run for two OTHER
house lints — not something I set out to check, they showed up when the full floor ran — caught
this as a self-inflicted defect, not a corpus finding:

- `no_loose_string_assert::tests_carry_no_loose_string_assert` — 7 offending lines in my file
  (loose `.contains()` where an exact structural comparison belongs).
- `no_inlined_wat_in_tests::tests_carry_no_inlined_wat` — my assertion strings
  (`"(:whr/Req (?n <- :n) ...)"`) are themselves wat-reader-parseable forms sitting in `.rs`
  source, which is exactly what that lint exists to catch.

Fixed by switching to the house `run()` (spawn `CARGO_BIN_EXE_wat`, same idiom
`probe_arc278_D10_then_field_types.rs`/`probe_arc278_field_span.rs` use) +
`wat::assert_edn_matches_file!` against captured `.edn` goldens for the two refusals, and
`assert_eq!(out.trim(), "N", ...)` for the four legal-shape controls (added full
compile/insert/fire/query `:user::main` harnesses to all four `.wat` fixtures so `run()` has
something to assert against — `startup_from_file` alone doesn't exercise "fires correctly",
only "compiles"). Re-ran both lints clean afterward (verbatim below, "Lint re-check").

## Mutation proof

Emptied the arm back to `ReteClauseShape::Where(_) => {}` (removing the `check_where_hoistable`
call), rebuilt, re-ran the full `where_fence_hoistable` cluster:

**RED, verbatim** (`cargo nextest run --release -E 'test(where_fence_hoistable)'`, mutated tree):

```
        PASS [   0.355s] (1/6) wat::rete probe_where_fence_hoistable::where_over_an_exists_bound_var_stays_legal
        FAIL [   0.377s] (2/6) wat::rete probe_where_fence_hoistable::where_reading_only_its_own_conditions_var_is_refused
  stdout ───

    running 1 test
    test probe_where_fence_hoistable::where_reading_only_its_own_conditions_var_is_refused ... FAILED

    failures:

    failures:
        probe_where_fence_hoistable::where_reading_only_its_own_conditions_var_is_refused

    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 497 filtered out; finished in 0.37s

  stderr ───

    thread 'probe_where_fence_hoistable::where_reading_only_its_own_conditions_var_is_refused' (3506120) panicked at /home/john/work/holon/wat-rs/tests/rete/probe_where_fence_hoistable.rs:68:33:
    a hoistable where must be refused: FrozenWorld { config: Config { ... — the full multi-KB
    Debug dump of the successfully-frozen world (every registered type, incl. builtins) follows;
    elided here as noise, not evidence — the load-bearing fact is that `startup_from_file`
    returned `Ok(FrozenWorld)` at all, which `.expect_err()` turned into this panic. Full text on
    disk at the time: /home/john/.claude/projects/-home-john-work-holon/fd01e281-0457-4e4a-a481-
    acd7beca46ad/tool-results/bzdldup25.txt }
        PASS [   0.544s] (3/6) wat::rete probe_where_fence_hoistable::where_over_an_or_bound_var_stays_legal
        PASS [   0.545s] (4/6) wat::rete probe_where_fence_hoistable::where_joining_two_conditions_stays_legal
        PASS [   0.548s] (5/6) wat::rete probe_where_fence_hoistable::where_over_an_accumulate_result_stays_legal
        FAIL [   0.575s] (6/6) wat::rete probe_where_fence_hoistable::where_reading_two_vars_from_the_same_condition_is_refused
    ...
     Summary [   0.726s] 6 tests run: 4 passed, 2 failed, 5509 skipped
        FAIL [   0.377s] (2/6) wat::rete probe_where_fence_hoistable::where_reading_only_its_own_conditions_var_is_refused
        FAIL [   0.575s] (6/6) wat::rete probe_where_fence_hoistable::where_reading_two_vars_from_the_same_condition_is_refused
```

(This RED run predates the `startup_from_file`→`run()` test-authoring correction above — the
assertions were still `.expect_err(...)`-shaped at the moment of mutation, hence the
`FrozenWorld { .. }` panic text rather than the current file's `!ok` assertion. The mechanism —
both refusal tests go red, all four legal-shape controls stay green — is what the proof is
about, and it is unaffected by which assertion style ran; the mutation was re-proven is not
needed a second time under the corrected file because the corrected file changed HOW the result
is checked, not WHAT `check_where_hoistable` does.)

Both positive-refusal tests fail exactly as predicted (`startup_from_file` — later `run()` — sees
no refusal), all four negative controls stay green (confirming the mutation didn't accidentally
change engine behavior, only remove the new check). Restored the arm, rebuilt: **GREEN**, all six
tests pass (verbatim below, "Final green run").

## Population census — RE-DERIVED here, and it does NOT match DESIGN's own re-derived numbers

DESIGN.md's table (measured on this same `grok-rete` branch, same day): wat-scripts 71 files /
277 hits, tests 36 files / 78 hits. I measured independently, with two different tools
(`grep -rl`/`grep -ro` and, separately, `rg`) before trusting either:

| tree | files (this measurement) | `:wat::rete::where` hits |
|---|---|---|
| `wat-scripts/` (excl. `scratch-pad/`) | **38** | **192** |
| `tests/*.wat` | **36** | **78** |
| `docs/arc/**/*.wat` | **3** | **3** |
| `.rs` string literals | 24 | — hand work, out of scope |
| `wat/` | 2 | 7 |

⚠ **`docs/arc/` is a FOURTH tree neither DESIGN nor my first pass named.** I only found it because
the floor's own `docs_wat_loads_or_declares_why_not` lint (which walks a curated ledger of 22
`docs/arc/**/*.wat` files, not the whole tree) went red from my change — this is not a file I
went looking for, it is one the floor handed me. Re-derived directly: `grep -rl
":wat::rete::where" docs/arc --include="*.wat"` finds exactly 3 files, all 3 refusable (below).

`tests/` matches DESIGN's figure exactly. **`wat-scripts/` does not: 38/192 measured here, not
71/277.** I checked this discrepancy rather than picking a number: `rg -l ... --glob
'!scratch-pad/**'` first appeared to confirm DESIGN's ballpark (74 files), but the glob relative
path was wrong — it was NOT excluding `scratch-pad/` at all (verified with `diff` against the
plain `grep` file list: every file `rg` had beyond `grep`'s 38 was one of the 36
`scratch-pad/probe-*`/`rules-corpus-*` files already in the tree, or my own throwaway additions
before I deleted them). With the exclusion actually applied, `rg` and `grep` agree exactly at 38
files. I found no evidence for 71 anywhere in the current tree — either DESIGN's own re-derivation
had the same glob-exclusion mistake in reverse, or it was measured against a different working-tree
state that no longer matches HEAD. Reporting my own re-measurement as of `85ca63ea1`, not
DESIGN's, per the file's own instruction not to reuse figures without checking.

`wat/` — 7 hits in 2 files (`compile.wat:92,126,370,376,917`, `oracle/pass.wat:542,608`),
independently re-confirmed by reading every line: all seven are `(:wat::core::= head-nm
":wat::rete::where")`-shaped string comparisons against the head name, never a live `:where`
usage. No bootstrap hazard, same conclusion as DESIGN, reached independently by reading rather
than trusting the prior claim.

## Refusable population — derived by actually RUNNING the check, not by reading source

⛔ A hit is not a refusable site. I built the check first, then ran every corpus file containing a
`:wat::rete::where` hit through the real compiled validator (`./target/release/wat <file> </dev/null`)
and classified by what actually happened — not by guessing from the AST shape:

- **`WhereHoistable` in stderr** → REFUSABLE.
- No error, or an error unrelated to freeze (a `readln: end of input` runtime panic from a
  `:user::main` that reads stdin interactively; a `MainSignatureError` for a fixture with no
  `:user::main` at all, meant to be driven by `startup_from_file`/`startup_beside` from its own
  Rust test, not the CLI) → clean. Verified for every such case that the OTHER error's own
  location (`freeze.rs` main-check / `:user::main` body) is strictly AFTER a successful
  `build_env` — i.e., freeze, and this check inside it, already ran and passed before the
  unrelated failure happened. Confirmed by reading `src/freeze.rs`'s `startup_from_forms`
  ordering, not assumed from the message text alone.

Counted DISTINCT refusable sites (not files) via `#wat.rete/WhereHoistable {:rule` occurrences —
this exact anchor was needed because the EDN error's OWN rendering embeds a full copy of itself
inside its `:message` string, so a naive `grep -c WhereHoistable` double-counts every site; caught
by checking a known single-`where` fixture and finding count=2 where it should be 1, then
re-deriving the count with an anchor unique to the un-escaped, structural copy.

| tree | files w/ hit | REFUSABLE files | clean files | REFUSABLE sites |
|---|---|---|---|---|
| `wat-scripts/` (excl. scratch-pad) | 38 | **25** | 13 | 143 |
| `tests/*.wat` | 36 | **32** | 4 | 75 |
| `docs/arc/**/*.wat` | 3 | **3** | 0 | 3 |
| **total** | **77** | **60** | 17 | **221** |

**60 of 77 files (78%), 221 individual `:where` sites, are refusable under this check as it
stands.** This is the blast radius DESIGN asked to have measured before anything else changed —
not migrated, not weakened, just measured and reported. Two of the three `docs/arc/` files
(`harness-experiri/experiri-when-match.wat`, `vigilia-2026-09-05/probes/probe_vig_left_idx_latch.wat`)
were previously undeclared and now trip `docs_wat_loads_or_declares_why_not` fresh; the third
(`probes/red-acc-refire-native-vs-oracle.wat`) already carried a `rune:lint(red-by-design)` for an
unrelated reason (it deliberately prints two disagreeing lines) and so was already "declared", even
though it is now ALSO refusable for this new reason — that lint's bookkeeping records only
loads-or-declared, not which of possibly several reasons a file is red for.

## Floor — run TWICE, both in the FOREGROUND, and both reds are reported (neither discarded)

`pgrep -af 'cargo|nextest'` checked clear immediately before each run (the only matches were
unrelated `wat --mcp` processes whose path contains the substring `.cargo`, not an actual
cargo/nextest invocation). Per house rule, NEITHER red below was re-run to "see if it clears" —
the first was captured whole, read, understood (self-inflicted test-authoring defects mixed with
expected corpus fallout), FIXED (the self-inflicted part only), and the second run is the honest
post-fix state, still red, exactly as DESIGN predicted it would be.

### Run 1 — before the `no_loose_string_assert`/`no_inlined_wat_in_tests` fix

```
     Summary [ 438.223s] 5496 tests run: 5364 passed (2 slow), 132 failed, 19 skipped
```
Captured whole at `.floor/2026-09-09T22-08-51Z/` (`ARM.txt`, `clean.log`, `raw.log`). 132 failing
tests. Read every failure name; classified each by cause before changing anything:

- **2 self-inflicted** (my own new test file, not a corpus finding):
  `wat::lint no_inlined_wat_in_tests::tests_carry_no_inlined_wat` and
  `wat::lint no_loose_string_assert::tests_carry_no_loose_string_assert` — my draft test file's
  `.contains()` substrings ARE the two things those lints exist to catch (verbatim offender lists
  were in the ARM block, all pointing at `tests/rete/probe_where_fence_hoistable.rs` lines
  69–88). I initially miscounted a third
  (`docs_wat_loads_or_declares_why_not::every_docs_wat_loads_or_declares_why_not`) as
  self-inflicted too, on the assumption that everything red in the same run must share one cause
  — checked its ARM block directly and it is corpus fallout (2 genuinely refusable `docs/arc/`
  files), not caused by my test file at all. Corrected before writing this table: **2**
  self-inflicted, not 3.
- **130 corpus fallout** (132 − 2, including `docs_wat_loads_or_declares_why_not`) — every other
  failure is a `probe_arc278_*`/`probe_arc300_*`/
  `wat_scripts_grid_*`/`sift_rules_arena`/`every_wat_scripts_file_loads_on_the_current_runtime`
  test whose `.wat` fixture is one of the 60 refusable files in the census above, now correctly
  refused at freeze. Spot-verified one from a cluster that looked unrelated at a glance
  (`probe_arc278_export::a_well_formed_user_call_still_runs` — "export" sounds like it might not
  touch `where` at all) by reading its ARM block directly: it fails because
  `tests/rete/probe_arc278_export.wat` itself carries two genuinely hoistable `:where`s
  (`exp::cool` on `?c`, `sn::mark-bad` on `?k`), both named verbatim in the panic. Did not assume
  the pattern from the name; checked it.

**Fixed the 2 real self-inflicted lint defects** (see "Test-authoring correction" above);
re-checked those two lints alone, both green
(`no_inlined_wat_in_tests::tests_carry_no_inlined_wat` PASS,
`no_loose_string_assert::tests_carry_no_loose_string_assert` PASS). The third
(`docs_wat_loads_or_declares_why_not`) re-checked RED on its own — confirmed it is corpus fallout
too (the 2 `docs/arc/` files above), not something my test file caused; left untouched per the
"do not migrate the corpus" instruction, exactly like every other corpus site.

### Run 2 — final state, after the fix, nothing else changed

```
     Summary [ 438.800s] 5496 tests run: 5366 passed (1 slow), 130 failed, 19 skipped
```
Captured whole at `.floor/2026-09-09T22-27-23Z/` (`.floor/latest`). **Exactly 132 − 2 = 130** —
the two self-inflicted lints now pass. Checked "same corpus failures, not a different red" by
diffing the two ARM.txt FAIL-line sets directly (run 1's list with the 2 self-inflicted test
names filtered out, vs. run 2's list, both sorted): **`diff` reports zero lines of difference** —
the exact same 130 named tests failed both times. None of the six `where_fence_hoistable::*`
tests appear in either failure list — they pass in both runs.

The 130 remaining failures, by category (all corpus fallout, none touched, none migrated):

| category | count | example |
|---|---|---|
| `wat::rete probe_arc278_*` / `probe_arc300_*` / `probe_fence_names_the_head` | 111 | `probe_arc278_export::*` (13), `probe_arc278_vsa_where_native_differential::*` (10), `probe_arc278_where_is_positionally_free::*` (4), … |
| `wat rete::kernel::tests::*` / `wat rete::reachability::*` | 15 | `termination_verdict::*` (7), `reachability::*` (6) |
| `wat::lint docs_wat_loads_or_declares_why_not` | 1 | 2 `docs/arc/` files, named above |
| `wat::lint wat_scripts_fixes_load::every_wat_scripts_file_loads_on_the_current_runtime` | 1 | the 25 refusable `wat-scripts/` files, batched into one test |
| `wat::services probe_arc278_sift_rules_arena::*` | 4 | rule-arena fixtures with hoistable `where`s |

This is not a novel or surprising red — it is the exact, predicted consequence of DESIGN's own
warning ("Expect the floor to RED if the corpus contains refusable sites — that is the check
working"), reproduced identically in both runs (same failing test SET both times, modulo the 2
lint fixes), which is itself a form of mutation-adjacent evidence: the redness tracks the corpus
population I measured, not something flaky.

## Lint re-check — the 2 self-inflicted fixes, confirmed green in isolation before the final floor

```
$ cargo nextest run --release -E 'test(no_inlined_wat) or test(no_loose_string_assert) or test(docs_wat_loads_or_declares_why_not)'
        PASS [ ...] wat::lint no_inlined_wat_in_tests::tests_carry_no_inlined_wat
        PASS [ ...] wat::lint no_loose_string_assert::tests_carry_no_loose_string_assert
        FAIL [ 4.668s] ( 13/13) wat::lint docs_wat_loads_or_declares_why_not::every_docs_wat_loads_or_declares_why_not
     Summary [ 4.677s] 13 tests run: 12 passed, 1 failed, 5502 skipped
```
Confirms the fix worked (both self-inflicted lints green) and confirms `docs_wat_loads_or_declares_why_not`
is corpus fallout, not something the test-file fix could or should touch (same 2 `docs/arc/`
files named, unchanged).

## Files changed

```
 M src/rete/validate/error.rs                                          (+ WhereHoistable kind)
 M src/rete/validate/mod.rs                                            (the wall)
?? tests/rete/probe_where_fence_hoistable.rs                           (6 tests)
?? tests/rete/probe_where_fence_hoistable_or.wat                       (negative control 1)
?? tests/rete/probe_where_fence_hoistable_exists.wat                   (negative control 2)
?? tests/rete/probe_where_fence_hoistable_accumulate.wat               (negative control 3)
?? tests/rete/probe_where_fence_hoistable_join.wat                     (negative control 4)
?? tests/rete/probe_where_fence_hoistable_refuse.wat.bad               (positive case)
?? tests/rete/probe_where_fence_hoistable_multivar_refuse.wat.bad      (positive, 2-var)
?? tests/rete/probe_where_fence_hoistable__refuse.edn                  (golden)
?? tests/rete/probe_where_fence_hoistable__multivar_refuse.edn         (golden)
?? docs/arc/2026/06/278-rules-engine/strike-where-fence-hoistable/SCORE.md  (this file)
```
No `wat-scripts/` file was added or changed — the three scratch-pad reconnaissance files used to
prove the negative-control shapes were real before committing to them were deleted once their
purpose was served (per `wat-rs/CLAUDE.md`'s "delete it if it's truly dead" clause), not left as
scratch residue.

## What I did NOT do

- Did **not** touch `:456`'s clause-level `ReteClauseShape::Where(_) => {}` (the stone-6 STOP
  arm) — confirmed by re-reading the diff before writing this file.
- Did **not** migrate any corpus file, in ANY of the three trees (`wat-scripts/`, `tests/`,
  `docs/arc/`). All 60 refusable files remain exactly as they were; this is measured and
  reported, not fixed. A `wat-fix` codemod over these sites is its own strike.
- Did **not** annotate the 2 newly-red `docs/arc/` files with `;; rune:lint(red-by-design)` to
  quiet `docs_wat_loads_or_declares_why_not` — that lint's own failure is corpus fallout like
  every other one in the 130, not a defect in this strike's own deliverable, and DESIGN's
  instruction is to report the redness, not silence it.
- Did **not** weaken the check to make the floor green — the floor is still RED (130, both runs),
  exactly as predicted.
- Did **not** re-run either red floor to "see if it clears." Run 1's red was captured whole,
  read, and diagnosed BEFORE anything changed; Run 2 followed a specific, understood, disclosed
  code change (the 2-line test-authoring fix), not a hopeful re-run of the same code.
- Did **not** touch the 24 `.rs` string-literal sites — DESIGN's explicit scope carve-out (hand
  work).
- Did **not** reuse DESIGN's wat-scripts census figure once independent re-measurement
  disagreed with it, twice, with two different tools (`grep` and `rg`).
- Did **not** leave the scratch-pad reconnaissance `.wat` files in the tree once they had served
  their purpose (proving the three shapes are real, engine-legal patterns) — deleted, not
  committed, per the scratch-`.wat` convention's own "delete it if it's truly dead" clause.

---

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_018ntHDMRNCKDKNr2gVfzXmP
