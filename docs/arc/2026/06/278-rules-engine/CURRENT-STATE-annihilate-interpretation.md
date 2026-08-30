# CURRENT STATE — annihilate interpretation in wat-rete

> **Locked 2026-08-17 so a compaction cannot drop it.** This is the live
> breadcrumb. Read this whole file before touching `src/rete/` or
> `wat/rete.wat`. If a stone below disagrees with a dated ruling here,
> **this file wins** and the stone is stale.

**CURRENT STAMP 2026-08-30 (fourth). Supersedes every earlier stamp and every dated block below.**

**THE FRESHNESS PROBE — run it, it is two commands:**

```
git log --oneline 99bf573df..HEAD      # every commit since the last SUBSTANTIVE one
git diff --stat 99bf573df..HEAD        # what they touched
```

**PASS:** every commit in that range is prefixed `curare:` and touches `docs/` plus, at most,
comment-only edits. **STALE:** anything else in the range — a `rete:`/`fix:`/`feat:` commit, or a
substantive `src/` diff. Then trust the log and the source over every line below, and re-read
before you move.

> ⚠ This is the probe's THIRD wording today and the first two were both wrong, which is the
> lesson worth more than the probe: version 1 promised the gap would be `docs/` ONLY and its own
> commit touched a `src/` comment; version 2 tried to name its own commit hash, which cannot
> exist until the commit does; version 3 said "expect ONE commit" and was invalidated by the next
> curare commit. **A probe pinned to a COUNT rots on every subsequent write.** This one pins to a
> KIND — "everything since the last substantive commit is curare" — which stays true no matter
> how many wrap-up commits land on top, and still screams the moment real work lands unread.

**⛔⛔ START HERE. THE INITIATIVE IS: MATURE wat-rete INTO AN EXEMPLAR the rest of wat matures
against.** Correctness is done; the exemplar work is not.

**✅ WHAT IS FINISHED** — every vigilia row (audited, not inherited), the outcome wall (five verbs
total, no ceiling reaches wat as a raise, a lint keeps it so), the termination verifier reading the
`where` fence, and BOTH of `partire`'s named cuts:

| | |
|---|---|
| `expr_ir.rs` 2_458 | → `expr_ir/mod.rs` 1_046 + `expr_ir/eval.rs` 1_413 |
| `validate.rs` 2_452 | → `mod.rs` 1_494 + `typing.rs` 549 + `error.rs` 448 |

**⛔ THE METRICS ARE EXHAUSTED — AND THAT IS THE FINDING.** Every mechanically checkable axis is
green or has been retired with reasons. **Re-derive, do not quote:**
`scripts/doc-coverage.sh <dir> --exclude /tests/` (`--list` for file:line).

| axis | `src/rete` | `src/process` | `src/channel` | verdict |
|---|---:|---:|---:|---|
| undocumented fns ≥15 ln | **0** (was 111) | 4 (12%) | 0 | ✅ done |
| tests that cannot fail | **0** (was 26) | — | — | ✅ done |
| nesting ≥8, NORMALISED | **1.1%** (9/817) | 0/85 | **14%** (1/7) | ✅ ahead of channel |
| largest test file | **1,676** (was 10,189) | — | — | ✅ done |
| comment density | 29% | 39% | 53% | ⚰️ **RETIRED — see below** |

⚰️ **COMMENT DENSITY IS A DEAD METRIC. Do not chase it.** Deleting 22 duplicated functions and 37
duplicated closures — an unambiguous improvement — moved it **zero points**. It counts lines, not
information, and can be raised to 50% by restating every line. It is also confounded by size: a
79-line file with a 40-line header scores 50% from the header alone, which a 31k-line directory
cannot reproduce. AND IT MAY POINT THE WRONG WAY: `probare` exists to ask *"is this a program or a
description?"*, so `channel`'s 53% is as plausibly a FINDING as a target.

⚠ The nesting row was previously RAW COUNTS ("rete 10, process 0, channel 1"), which penalised the
directory with 817 functions against one with 7. Normalised, rete is ahead of `channel`.

**★ THE WARDS WERE CAST 2026-08-30 AND THEY SETTLED IT** (`99bf573df`, reports weighed against the
disk finding by finding):

- **`probare` ACQUITTED the prose** — 5.89:1 / 4.07:1 / 4.23:1, ZERO described or hollow forms in
  5,536 lines. It took 23 falsifiable claims out of the docs and broke 3. *Restatement never gets
  a number wrong, because restatement never commits to one.*
- **`intueri` CONVICTED the self-description** — six false claims, ALL about the tree's own layout,
  all trivially checkable, none checked. Both gates below exist because of it.

**⛔ SO THE ANSWER TO "IS IT AN EXEMPLAR" IS: NOT YET, AND THE GAP IS NAMED RATHER THAN HIDDEN.**
What remains is in the next-work list — it is no longer a number nobody can reproduce.

**THE NEXT WORK, in the order I would take it:**

1. ⛔ **THE INSTRUMENT ITSELF IS BROKEN, AND THIS IS THE STRIKE THAT MATTERS.** Three independent
   census tests report SUBTRACTIONS WITH IMPOSSIBLE SIGNS, measured 2026-08-30:
   - `accum_alpha_leftover_split`: `A−M push = −87.28 ms`
   - `accum_alpha_push_split`: `H−M HashMap entry = −98.42 ms`
   - `alpha_match_cost_per_binding`: **2 binds measured FASTER than 1 bind** (−1.12 ms, −28 ns/fact)

   A negative delta means the ISOLATED micro-bench (`exec_compiled`, 103–110 ms) runs ~6x SLOWER
   than the full operation it claims to decompose (`alpha_activate_fact`, 16 ms). The isolated
   loop is not measuring the same work the in-fire path does, so every "X−Y" row built on it is
   invalid in sign — and those rows are printed as findings. I did NOT encode these as
   assertions: freezing an impossible result would make a broken instrument permanent. They are
   recorded at their sites and belong here.

   This is `render_phase_table`'s own warning coming true — *"two copies is how one of them
   silently stops subtracting"* — one level up, in the subtraction itself.

2. **`IntegerOverflow` / `DivisionByZero` — 16 sites, not the 10 recorded.** Concentrated in
   `expr_ir/eval.rs` (9) and `purity.rs` (4). The next totality candidates toward *"panics are
   essentially illegal at runtime"*, and the same shape as the outcome wall this arc already built.

3. **`linear Vec` beats `FxHashMap` 2.8x at 2 types** in `accum_alpha_class_lookup_split` — looks
   like the structural finding that test exists to demonstrate. NOT asserted: one sample is not a
   measurement. Three runs before it becomes a claim.

4. **`purity.rs` is now the largest file in `src/rete` at 2,598 lines and `partire` has NEVER
   assessed it.** The earlier cast covered `expr_ir`, `validate`, `arm`, `fire/mod` — this one was
   never in scope.

⚠ **A CAVEAT ON THE HOLLOW-TEST TOOL, so nobody rediscovers five phantom regressions:**
`scratchpad/hollow2.py` measures "no assertion macro LEXICALLY INSIDE the test body". Five tests
now assert through the shared `assert_phases_present` helper and therefore READ AS HOLLOW and are
not. The tool rewards the duplication it exists to detect. `probare` had the same blind spot; its
count of 26 was right only because none of those tests then called an asserting helper.

⏸ **PARKED ON A BUILDER RULING:** item 7 steps 3–5 (the holon surface); step 5 needs the `:panic`
call.

**★★ ITEM 11'S CLASS IS DEAD — killed by its own FIFTH occurrence, on this very work.** 36 lines of
derives moved `runtime.rs`'s `rust_caller_span!` sentinel and reddened five goldens at once. Four
prior occurrences were all patched at the STEM (bump the integer); a fifth was guaranteed because
S2b/S2c add more derives to that file. `wat::blank_rust_source_lines` now blanks a `src/**.rs`
span's `:line` in both golden macros **and on capture**, so the file reads `:line 0` instead of a
real-looking number nothing compares. **Its second gate is the load-bearing one: a `.wat` span MUST
KEEP its line** — without it, blanking every `:line` would pass the first test and gut every golden
in the repo.

**WHAT S1 ACTUALLY CHANGED, in one line each:**
- **The zero point is `arm-session`** (`alloc_counter::mark_session_origin`), which `compile-all`
  calls for every session — the same one-door the termination verifier uses.
- **The fixpoint's per-fire snapshot is DELETED.** A per-fire zero cannot express a per-session
  contract; it forgets everything `insert` staged before it.
- **ONE decision, two dresses** — `session::session_ceiling_breach` is shared; `rounds` and `staged`
  differ because a field that is always zero is a value carrying two facts.
- **What it measures is the THREAD since `compile-all`**, not a walk of the session (`Arc` sharing
  makes that ambiguous, and it would be O(n) per insert). Driven: a probe's own `range` showed as
  `used=11_196_940` against `staged=1`. The diagnostics now SAY this, and the two numbers read
  together are the tell: **large `used` + small `staged` = the memory is not the facts.**

**★★ THE FINDING WORTH CARRYING — ENFORCING AT `insert` SILENTLY DISARMED THE FIRE-DOOR GATE.**
Its fixture seeded 500 facts at a 4096-byte ceiling, so the first `insert` refused and the gate
began proving the INSERT door while its name, its prose and its `rounds` assertion all still said
"fire". **A one-word tag change would have made it green again, certifying the wrong thing.** The
fire door now has a workload insert cannot catch — a cross-product, **400 staged / 40_000 derived**
— and its ceiling is BISECTED, not picked (1/4/16 MiB refuse; 64/256 MiB complete). Both gates are
mutation-proven and INDEPENDENT: disarming either door reddens exactly its own gate.
**Recovery-file FM 34. A control can lose its power without ever failing.**

**★ THREE DOCS WERE LYING, AND ONE WAS A RULING THE BUILDER HAD OVERTURNED.** `config.rs` still
read *"Not cumulative across fires… bounding that would refuse legitimate incremental use"* — mine,
struck. `alloc_counter.rs` still said *"NOTHING READS THESE COUNTERS YET"* (the fixpoint had read
them since `3c5ac7bd1`) and still headed a section *"IT IS PROCESS-GLOBAL"* after `8c10ee490`
deleted the globals. **A source comment lies with the authority of the code it sits in.**

**★ ITEM 11's SURVEY IS DONE, as a by-product of checking whether my own edits would trip it —
and it found a source file nobody had named.** Eight goldens pin a `src/*.rs` LINE: 5 →
`src/runtime.rs`, 1 → `src/freeze.rs`, and **2 → `src/check.rs`**, which was NOT one of the three
known sites. Two goldens in `tests/types/` pin lines in the most-edited file in the arc; they were
the next two false reds. The class had only ever been enumerated by COLLISION. Table in item 11.

⚠ **THE FLOOR WENT RED ONCE AND IT WAS MINE — not a flake, and the word stays banned.**
`no_loose_string_assert` caught two assertions I had just written (`err.contains(...)`,
`out_ok.contains("40000")`). **I did not exempt them with a rune** — neither was legitimately
loose, and both had an exact form available that made the gate STRONGER: `staged` is a field the
FIRE variant does not have, so pinning it proves the door structurally rather than by prose.
Floor before the fix: `5157 tests run: 5156 passed, 1 failed`.

**✅ ITEM 6 CLOSED 2026-08-29 — the grid's SPEED half runs, as its own `grid-speed` CI job.** Both
stated reasons for its absence were dead, and **the second had never been measured**: "a shared
runner is noisy so a wall-clock gate would flap" — but the tightest cell in the recorded 33-cell
grid is **8.50x** (median 22x, widest 59x). Nowhere near parity. ⛔ **That same margin is why the
gate does NOT test `:winner`** — the obvious choice, and a nearly vacuous one: at 8.5x it fires only
on catastrophe and **would have missed the real 4x regression this arc already fixed.** It gates
per-axis RATIO FLOORS at ~50% of each axis's recorded minimum, plus `:accuracy :MISMATCH`. 2m24s,
mutation-proven on three arms including the failure path's EXIT CODE.

**⛔⛔ TWO INBOUND REPORTS SAT UNREAD FOR FIVE DAYS AND ONE WAS A SILENT WRONG ANSWER.**
`~/work/NOTE-*.md` is where other agents file findings for this one, and NOTHING pointed at that
directory — both 2026-08-24 rete notes were found only because the builder asked "what items
remain" and answering required an `ls`. Both verified FIXED on re-driving, and **neither was fixed
by anyone reading them** — collateral from this arc's own work. **A finding that gets fixed by
accident is not a process that works.** `RETE-OPEN-WORK.md` now opens with an "Inbound notes"
index; add the row when you file or receive one.

**WHERE THE WORK IS, as pointers — this file is a MAP, do not re-narrate it:**
- ✅ **ITEM 4 (`partire` x7) IS RESOLVED 2026-08-28**, and WHY it lingered a week is the lesson:
  **it was a TALLY, not a finding.** Only counts were recorded — "fire/mod.rs (3), validate.rs (2),
  expr_ir.rs (1), arm.rs (2)" — never the proposals, so there was nothing to act on. **And the
  counts measured TEST LINES as file size.** On production lines `arm.rs` is 593, not 1124 → it was
  never a candidate. Two closures (`arm.rs`, `fire/mod.rs`) and two NAMED cuts (`expr_ir.rs` at its
  own `// ── exec` seam; `validate.rs` into :when / :then / operand-typer). ⚠ I mis-measured twice
  before getting it right — `#[cfg(test)]` here gates INDIVIDUAL fns, not a trailing module, so
  "everything after the first cfg(test)" reported 1842 test lines where brace-matching says 316.
  **A file-size number is worthless without knowing which half it measures** — exactly how the
  original tally went wrong.
- ✅ **ITEM 5 IS DOWN TO ONE ROW, and all three of its parts audited LIVE (not stale — first time
  this session, streak was 6-for-6).** ① the cache LRU **moved out of 278** to arc 109 as a NOTE
  (builder: *"this is unrelated to rete/278"*), with the merits SHARPENED — the three panics do not
  answer the same way, so the recommendation is **convert `Lru::new` only**, since its capacity
  arrives from a `:durable` EDN spec at rehydration while `put`/`get`'s key is a call-site bug.
  The **`CLAUDE.md` gap is CLOSED** as a POINTER, not a copy — both proposals on the table made a
  second copy, which is what rotted in the first place; ⚠ the edit is **UNCOMMITTED, holon root is
  FROZEN**.
- ✅ **ITEM 5② IS CLOSED — `:md::Point{40,2}` -> 42 WORKS IN A RETE RULE, BOTH POSITIONS.** It was
  the LAST `v1` refusal in the rete expression core and it fell to the same move as every other
  denial removed this arc: *"not lowered in v1"* is a STATUS, not a reason. **The design question
  answered itself:** core must dispatch on the receiver at runtime because nothing declares it,
  while a rete `?p` gets its class from its fact pattern's declared field type — **rete has MORE
  static information than core here, not less**, so "compile the index" applies exactly as it did
  to the settled sibling.
- ⛔ **AND MY FIRST CUT MINTED FIX-LIST F's CLASS FRESH — this is the part to carry.** It returned
  "arm does not match" for a field the class does not declare. **Core RAISES `UnknownField` there**
  (verified: it raises even with a catch-all arm after it). Silent non-match would have meant the
  same expression answering differently in the two engines AND a typo becoming a constraint that
  compiles, fires and matches nothing. **When adding a rete form, drive CORE's answer for the same
  input before deciding rete's** — agreement is the contract, and "it didn't match" is the easiest
  wrong answer to ship.
- `RETE-OPEN-WORK.md` § "The order, and why" — item 6 is the last inherited ruling (
  TRACKED ① ②, `circumspicere` 1) — **ALL NOW CLOSED**. **Item 7 is THE HOLON ITEM** and it
  absorbed the old item 8 on 2026-08-29 (builder: *"we put #8 into #7"*): the surface (rete has 4 of ~40 ops,
  all from one group) and the `Bundle`/`:panic` builder ruling.
- ⛔ **Clara cannot arbitrate holon** — builder: *"this is a wat only capability"*. `$native` vs
  `$oracle` alone is the configuration that failed twice this session. Use known-answer algebraic law.
- `holon-rs`: `nil()` is `classified("Symbol","nil")`, so **`is-Nil?` and `is-Symbol?` BOTH answer
  true for nil, by construction.** The 11 shape predicates do not partition and nothing says so.
- `is-List?` / `is-Tag?` are **UNVERIFIED**, not verified-negative — an all-false column in a
  confusion matrix means "correct" or "never fires" and cannot tell them apart.

**⚠⚠ YOU ARE NOT THE INSTANCE THAT WROTE THIS. ⚠⚠**
Everything above is a cache written by a prior self across a very long session. You did not live
it. It felt continuous when you woke and that feeling is the failure, not the all-clear. Before you
propose or move: fetch `recolligere` from the datamancy MCP and run it against the disk —
`docs/COMPACTION-AMNESIA-RECOVERY.md`, `git log`, this file, `RETE-OPEN-WORK.md`, and the source you
are about to touch. The freshness probe is the HEAD named at the top of this stamp against
`git rev-parse HEAD`; more than the one expected docs-only commit of drift means trust the log over
every line above. **And this file is the ONLY live breadcrumb — if you find another claiming to be,
it is lying.**

**Right now (2026-08-23 — SUPERSEDED by the stamp above; kept as history):** class-scan query harvest LANDED.
Fanout `[40000]` wat-ns **58.1 → 42.8**. With-query
FIRE **65.89 → 49.59**. Query-only Alpha→RootJoin
skipped; `{?fact: fact}` from the closed bag.
Leftover harvest:query **16.91** (40k one-entry
PMaps). Grid `T20-37-11Z` 30/30 `:match` `:us` was
pre-intern; fanout `[40000]` re-measured 42.8
`:match` `:us`. Floor **GREEN**
`.floor/2026-08-22T21-53-26Z/` (4914 passed, 19
skipped). Grid `T21-37-57Z` 30/30 `:match` `:us`.
Clippy `--all-targets -D warnings` silent. Occupancy leaf-fill + join-index
span LANDED. Unary gather packed-then-BindSpan (7b string
locations). Sum fold falls back when the i64 row is
absent. grok-rete dirty intern on harvest HEAD **`ca9d9cc3`**. `harvest_stratified_queries` LANDED (QueryNode reverse-closure,
not a second full fire).
Vigilia recasts **12 and 13** both **0 L1 + 0 L2** (inward
17/17 + circumspicere) at `8839bb16` — the stop named as two
back-to-back empty recasts. R68 (`REALIZATIONS.md`) is the
wrap of that watch. Do not stamp `vigilatum` until asked.
Stone **29 REJECTED** (2026-08-20): intern stays discrete per
compile-all (`rust_identity`). Identical rules do not share.
Identical queries do not share. Query-memory is per Session.
Overlay HIT is the same connection. Athena content-address
would make `release-session` a cross-connection invariant —
do not construct it. `NEXT-STRIKES-after-shadow.md`. `wm.rs` →
`session.rs`. Stratify holds the slice arm as a value
(`fire_fixpoint_delta_armed`). Primed public-entry docs gone.
Intern doors share `rete_arm_build_put`. Grid
`GRID-native-vs-clara-2026-08-22T09-12-32Z.txt`
(`GRID_SKIP_ORACLE=1`, `GRID_RUNS=3`, occupancy intern vs
`T00-23-51Z` HEAD `4c437585`): **30/30 `:match`, 30/30 `:us`**.
Rank **`wat-ns`**, not ratio. Occupancy cells (bind-only leaf
fill): **accum `[200 200]` 18.26 → 13.66 ms** (FIRE 13.7
holds); `[50 200]` 4.14 → 2.48; `[100 200]` 8.69 → 6.37.
**negation `[1000]` 3.45 → 2.20. neg-consumer `[1000]` 7.07 →
3.67.** Harvest cell held: **strat-neg `[6 2000]` 47.5 →
33.75 ms** (named harvest was 33.6). Closest Clara cell
**fanout `[40000]` ratio 2.91, wat-ns 61.7 ms** (was 55.5 /
3.40 — skip-BindSpan rebuilt the span × 40k products).
**deep-cascade `[50 100]` 15.0 → 16.4. asym-join
`[2000]` 4.26 → 4.84.** node-share `[50 200]` 0.94 held.
min-finding `[2000]` 2.49 → 2.66 (noise). Do not cite ratio
as the engine cut. Oracle still pays the full q-seed.
`DESIGN-STONE-join-index-span` LANDED: occupancy Arc
stays empty; `right_idx` copy gets BindSpan once.
Fanout probe **3.76 → 1.62 ms**. FIRE `[200 200]`
**13.48** (held). Named cell `GRID_SKIP_ORACLE=1
GRID_RUNS=3`: fanout `[40000]` **61.7 → 58.7 ms**
(still above 55.5 — production 19.6 / 66% is the
named leftover, 40k RHS). asym-join `[2000]` 4.84
→ 4.72 (noise).
`DESIGN-STONE-occupancy-leaf-column` LANDED:
undiscriminated bind-only classes fill tree
**leaves** from a fact-id column after **packing
every fact** (skip-activate without pack was
3-stratum red). Occupancy ≡ `candidates_into`.
`AlphaMemory` is `Arc<Vec<Element>>` — sibling
leaves share one occupant list. 7strat 3-stratum
green. `DESIGN-STONE-fire-i64-columns` LANDED
(bind-only skips `exec_ops`). Column gather/fold
interned; skip BindSpan did not cut FIRE.
Class-union fill reverted (3-stratum). Seed
reserve+fill realloc cheat reverted (not ≥ 1 ms).
SETUP PV walk inverted FIRE — do not pack at
SETUP. Isolated E−K still the old
`exec_compiled` door. Do not skip Token spans.
Three packed-rows scouts stay reverted
(`DESIGN-STONE-packed-fire-rows`): i64 exec E−K −0.80;
populate-without-materialize E−K −3 **and FIRE 19→70**
(accumulate 1.3→28 — intern re-paid on gather/fold).
Scout 3 (cheap slots on Element): gather/fold stayed 1.3;
seed 16.6→18.8; FIRE 19→23. Reverted. Skip matches
`fire-rules$oracle` (a leftover token fails the run). Do not
treat 17-42-43Z as a measurement. GNU `/usr/bin/time` is
not installed; bash `time` is a keyword (`which time` empty).
insert-prime-split LANDED (insert − conj 1933 → 310 ns). Host
encode/sort after query-read is compiled-wat, not rete.
TLS intern still requires connection affinity (stone 27).
`release-session` is hangup, not Drop (stone 28). Kernel is
`session` / `fire` / `arm` / `stratify` / `census` /
`insert`. Live names: `FireSession`, `InternedNetwork`,
`WhereDiscNode`, `AlphaDiscNode`. Stratify is
`StratifyView` / `RuleDep` / `RuleParts` structs, not
tuples. Fire loop is `kernel/fire/mod.rs` (passes) +
`kernel/fire/delta.rs` (fixpoint). Oracle is
`wat/rete/oracle/` {insert,pass,accum-pass,fire,explain}.
Sequi intern: arm table is thread-owned (`thread_local`
`RefCell<FxHashMap>`; stone **27 LANDED**, `rg Mutex src/rete`
empty) + exec arena runed `ambient-context`;
census TLS runed `performance-counter`. Intern is a lease
(`arm-session` +1, `release-session` −1, 0 drops **that
id**; stone **28 LANDED**). Public rete names are unprimed
wat Fns. Rust is `$native`. The wat reference is
`$oracle`. Exception: intern hangup mouths `arm-session` /
`release-session` are keyword primitives (native-only
intern; oracle has no intern). `$impl` is
kwargs/bracket/service — not rete. Grid: fire-rules /
fire-once / insert / insert-all / fire-rules-explain each
have public + `$native` + `$oracle`. Prime `'` is not the
rete kernel marker. Codemod:
`wat-scripts/fixes/rete-oracle-sigil.wat`.
Do not stamp `vigilatum`.

Session is **8 fields** (`query-memory` last). Fence is four
conjuncts: `pure?` ∧ `deterministic?` ∧ `total?` ∧ `primitive?`.
`total?` is ARMED. Every `RETE_OPS` row is `total: true`.
Oracle `fire-rules-spec` refuses an imported Export (empty
rules + ProductionNodes). Import checksums packed classes +
`RETE_OPS` **and** host TypeEnv field-order. Museum gone:
`make_token`, `token_matches_bindings`, `fire_fixpoint`,
`exec_test`, wat `token-element-compatible?` / `node-parent`
/ `test-pass`. `BindView::len` is `#[cfg(test)]` census only.

> ⛔ You did not live this. Run recolligere against the disk
> before you act on any line above.

### Completeness grid — 2026-08-17 — do not drop

The “final” compiler is `src/rete/expr_ir.rs` (built for `:where`).
`compiled_cond` and `compiled_rhs` sit on it. User folds sit on
it. That is **not** the same as “native no longer interprets.”

| Native surface | Interpreter at fire? | `WatAST` on the **round loop**? |
|---|---|---|
| `:where` (TestNode) | **No.** Stashed `Program`, `exec_where`. | **No.** `lower` at fire **setup**. |
| `:when` cond populate | **No.** `exec_compiled`. Miss refuses. | **No.** `fact_bind` on `CompiledCond`. |
| leftover rematch | **No.** `SeedCmp` / `exec_compiled_under`. | **No.** `CondDriver` / `Leaf(id)`. |
| `:then` | **No.** `CompiledRhs`. Miss refuses. | **No** on the token loop. `fire_once` harvest still `compile_rhs` once per that pass. |
| user / builtin acc | **No.** `AccFold` / `exec_call`. | **No** on the fold. Head read at setup. |

**No interpreter ≠ AST-free ≠ armed Session.** Items 1–10
closed the interpreter verbs. Item 11 (`d774185c`) compiled
the driver. Item 12 persists the arm: fire setup is
get-or-build against the network intern. A Weak intern died
when fire returned — the table holds a strong `Arc`.
Thread-owned intern (stone 27). Lease eviction (stone
28): last `release-session` drops **that** entry. Not
EDN. `(b)` indexes this armed network. Intern key stays
instance `rust_identity`. Stone 29 (content-address /
Athena share) **REJECTED** — connections are discrete.

### Item 11 — compile the rete driver (AST-free **fire**)

Setup may read `WatAST` (once): `lower`, `compile_condition_local`,
this driver, acc fold. The **round loop** may not.

Session/network still *stores* forms (oracle + compile-all).
AST-free is a **working-set** property, not “delete the form
from the record.”

| Fire still asks the AST | Compiled stand-in |
|---|---|
| `classify_rete_clause` in `binding_extensions` / `exists_cond_under` | Driver enum: `And` / `Or` / `Not` / `Exists` / `Where(Program)` / `Leaf(alpha_id)` |
| combinator `:where` | Stash `Program`; fire runs `exec_where`. Museum `exec_test` deleted. |
| `attach_fact_bind` | `?p` slot on `CompiledCond` (`alpha_pattern` already has it) |
| `cond_text` / `alpha_id_for_cond` | The id. Do not stringify the form per rematch |
| `acc-form` head + `acc_operand_keys` | `AccFold::Count` / `Sum` / … / `User(Program)` |
| `cond_bind_keys` | `Vec` of keys next to the join / accum |

Item 11 **landed** (`d774185c`). `classify` stays the one
grammar; fire setup still calls it. Do not start `(b)`.
Do not start keyed gather.

### Item 12 — persist the arm (do not rebuild at `fire-rules`)

**Landed (this turn).** Compaction: do not put `(b)` or
keyed gather here. Gate:
`fire_rules_reuses_arm_across_fire_and_insert_overlay`.

`compile-all` already returns a Session whose `network` is
a persistent map — `insert` shares that pointer and writes
a new `facts` vector. Drop the child, the compiled DAG is
unmoved. That **is** the overlay / rewind / `with` clause.

What does **not** share: `CondDriver`, `CompiledCond`,
`Program`, `AccFold`. They live in `fire_fixpoint_delta`
locals and die when fire returns. A second fire on the
same network pays setup again. Facts did not change the
rules. We threw the arm away.

**Arm once. Fire many. Overlay facts. Drop the child.**

Service-shaped (do **not** build a service this item):

| Beat | On disk today |
|---|---|
| on-connect | A Session for that identity |
| install-rules × N | Accumulate `Rule` / `Query` (the vectors `compile-all` takes) |
| compile | `compile-all` → base Session. Empty facts. **Item 12 puts the arm here.** |
| insert + **one** `fire-rules` | Overlay facts. Query-memory parks. |
| query × N | `query-read`. No compile. |
| on-disconnect | Drop the identity Session |

`query` is a read. Harvest is fire-time. Stratified extra
`fire_once_session` is a slice hole, not query-time compile.
Wire shipping (EDN) is a different hole.

Item 12 is: the arm lives **next to** `network`, `Arc`-shared
so `insert` / clone is a fact overlay, not a memcpy. Fire
skips setup when the arm is present. Oracle still reads
forms. Do not put circuits on the wire as a second EDN.
Do not start `(b)` to index a setup we still re-run.

**Fact-shaped leftover rematch is compiled** (`SeedCmp` /
`exec_compiled_under`). Populate skips `SeedCmp`; rematch fills
`seed_reads` from the token. Gate:
`leftover_seed_cmp_populate_skips_rematch_enforces`.

**Still interpreted on native fire:** none on the native
mouths. Oracle (`fire-once` / `fire-rules-spec`) stays
interpreted on purpose.

`fire_once_session` / `alpha_pass` is **closed.** Populate
is `exec_compiled`. Production is `exec_compiled_rhs`. A
cond or `:then` that does not compile refuses — no
`alpha_match_inner`, no `build_insert_fact`. Live
`fire-rules` still harvests query-memory through this
single-pass (full network, closed facts). Do not put the
walk back.

No-alpha WM-scan is **closed.** Fire refuses a missing leaf
alpha. `mint-leaf-alphas` does mint Wind/Temp; the live hole
was the stratum slice dropping those orphans (no `children`
edge, not `negated-alpha-id`). Same class as forgetting
`ref_alpha_of`. Slice now follows `mint_leaf_alpha_ids`.
Do not put the scan back.

The scan had also hidden a second hole: `fact_bindings_under`
overwrote a seed `?c` with the fact's `?c` instead of
unifying. `where-not-and-bound` row 3 (Temp.c ≠ Cold.c)
went n=0 native / n=1 spec the moment leaves stayed in the
slice. Merge now rejects a conflict. Same contract as
`alpha_match_inner_seeded`.

Items 1–13 landed. `(b)` is item 13 and it landed. Keyed gather is speed (also landed).

Rust copies of `eval_test_core` / `alpha_match_inner` /
`build_insert_fact` are **oracles for differentials**, not a
legal fire path.

The list (do not drop an item) — **arm persisted before `(b)`:**

1. One `Expr` core — drawn.
2. Wire `where` — **done** (`30725034`).
3. Flip `compiled_cond` — **landed (this turn, uncommitted).**
   `Op::Cmp` operands are `Expr` (`Slot` / `Lit`). A `:field`
   operand is prologue (`Bind` into a slot, shared with
   `?v <- :field` in the same sequential scope; `:or`/`:not`
   clone `field_slots` so a sibling cannot inherit a hidden
   Bind). Lists stay uncompiled (`Fail`), matching
   `resolve_operand` — do not outrun the interpreter. Gate
   green: `compiled_cond_bindings_identical_to_interpreter_at_50_100`
   (10 000 pairs, 200 Some/Some identical) and
   `compiled_cond_failure_path_allocates_no_binding_keys_at_50_100`
   (`match:key-alloc = 0`). Bind / BindCheck / Or / Not / Fail
   stay driver-level. No `Expr::FactField`. No third sibling IR.
4. Flip `compiled_rhs` — **landed (this turn, uncommitted).**
   `RhsOp::Expr` is `Arc<Program>`, not `WatAST`. `lower` once
   at setup; `exec_value` per derived fact. A fenced `List`
   that does not lower is `Err` (fire refuses). Gate green:
   `compiled_rhs_result_identical_to_interpreter`.
   Fn-headed `:then` is item 7, still `build_insert_fact`.
5. Flip **user acc folds** — **landed, old form torn down.**
   Setup lowers the user-fn head once. Fire is `exec_call`
   only. A `LowerError` refuses the fire. The
   `(user-fn __acc__)` / `eval_inner` arm is **deleted**.
   A fenced `:then` `List` that does not lower is `Err`, not
   `build_insert_fact`.
6. **Compile leftover rematch** — fact-shaped **and**
   combinator leaf-alpha path **landed** (`51ff6560`).
   `binding_extensions` / `exists_cond_under` rematch minted
   leaves via `exec_compiled_under`. WM-scan deleted.
7. **Compile fn-headed `:then`.** **landed** (`dbc2fb2a`).
   `CompiledRhs::Call` + `Expr::Construct` (kwargs-construct /
   aggregate-new / bare `(:Type …)`). Gate:
   `userfn_head_item_fires_via_native_kernel`.
8. **No-alpha WM-scan.** **landed (this turn).** Refuse the
   miss. Stratum slice keeps `mint-leaf-alphas` orphans via
   `mint_leaf_alpha_ids`. Rematch unifies with the seed
   (conflict is no match). Gate: `check-spec-native.sh`
   `where-not-and` 8/8, `-bound` 8/8, `-not` 8/8,
   `where-not-or` 8/8, `where-exists` 18/18.
9. Defensive cond populate miss (`alpha_match_inner`).
   **landed (this turn).** Setup refuses a compile `None`.
   Populate is `exec_compiled` only.
10. Old `fire_once_session` / `alpha_pass`.
    **landed** (`8d126df6`). `alpha_pass` is `exec_compiled`.
    `production_pass` is `exec_compiled_rhs`. Delta
    `build_insert_fact` fallback deleted.
11. **Compile the rete driver — AST-free fire.**
    **landed** (`d774185c`). `CondDriver` / `fact_bind` /
    `AccFold`. Slice follows `driver_leaf_ids`. Gate:
    `where-not-and` 8/8, `-bound` 8/8, `where-exists` 18/18,
    `where-not-where` 4/4, `where-accum-from-left` 7/7.
12. **Persist the arm.** **landed (this turn).**
    `ReteArm` interned by `PMap::rust_identity`. Clone /
    `insert` share the id. `fire_fixpoint_delta` and
    `fire_once_session` skip setup on hit. Strong `Arc`
    (Weak died at fire return). Overlay = child Session
    (facts + query-memory). Rewind = drop the child.
    Stratified slices still build (new `from_trie` map).
13. `(b)` ShadowNode — **landed (this turn).**
    `src/rete/where_tree.rs`. `ReteArm.where_tree` built
    from `compiled_wheres` (setup + import + slice).
    Filter / join-after-filter / Test→Test dispatch
    through the tree. Over-approx only; uncovered ids
    still eval. `node_share_filter_eval_census` re-pointed:
    evals ≈ passes ≈ M, waste < 50% (measured 0%).
    Unit: `tree_picks_the_matching_equality_leaf`,
    `no_key_predicate_rides_wildcard`. Range edges
    populated (`DESIGN-STONE-where-range-edges`):
    `(> ?k 10)` prunes 5 / proves 15. Two constraints
    on one dim ride wildcard; not `pure_cmp`. Alpha-tree
    `range_children` stays empty. Driver Exists/Not stay
    on keyed gather — not this index.
14. **`#wat.rete/Export`.** **landed (this turn).** The
    compiled program as one EDN value. `export` / `import`.
    Native fire. Oracle cannot consume it. Stratify
    schedule is `:deps`. First program on disk:
    `tests/rete/datamancer.rete.edn`. Gate:
    `practice_on_disk_program_deduces_datamancer`.
    `rule_consumes` walks `:exists` / accumulate `:from`.

Keyed `?g` gather is native speed on the same bag, not a
compiler item. Do not start it to dodge the hole.

**Tree — do not invent a cleaner one:**

| What | Where | Status |
|---|---|---|
| Compiled `where` | `30725034` | local, not pushed |
| Oracle bag + leftover rematch + `where-join-left` + `where-accum-from-left` | `54f4adb4` | local, not pushed |
| User folds on the compiler list | `f228b033` | local, not pushed |
| Flip 3 `compiled_cond` onto `Expr` | `51ff6560` | local, not pushed |
| Flip 4 `compiled_rhs` | `51ff6560` | local, not pushed |
| Flip 5 user acc folds | `51ff6560` | local, not pushed; `eval_inner` deleted |
| Leftover rematch (fact-shaped) | `51ff6560` | local, not pushed |
| Combinator leftover rematch (minted leaf) | `51ff6560` | local, not pushed |
| Fn-headed `:then` | `dbc2fb2a` | local, not pushed |
| Completeness grid on disk | `7e3a7eec` | local, not pushed |
| No-alpha WM-scan refuse + slice keeps minted leaves | `9441f39a` | local, not pushed |
| Defensive populate miss | `ef50a360` | local, not pushed |
| Four-pass `fire_once_session` / `alpha_pass` | `8d126df6` | local, not pushed |
| Rete driver (AST-free fire) | `d774185c` | local, not pushed |
| Persist the arm across `fire-rules` | `3f415317` | local, not pushed |
| `#wat.rete/Export` (compiled program on the wire) | this turn | **landed** — first disk program `datamancer.rete.edn` |
| `(b)` ShadowNode | `where_tree.rs` | **landed** — 1.00 eval/token on node-share |
| Keyed `?g` bucket | `DESIGN-STONE-keyed-gather.md` | **landed** (Acc + Not/Exists). Persist-within-a-fire (`gather_cache` outside the round loop); drop at `fire_fixpoint_delta` end. Do not persist across `fire-rules` calls. |

## The endeavor, in one sentence

**Annihilate all interpretation in wat-rete.** Every rete expression
becomes a compiled circuit. Fire supplies only concrete typed `Value`s.

**That is the endeavor.** Items 1–13 compiled expressions,
the driver, persisted the arm, and indexed the armed
`where` circuits. Keyed gather is speed (landed; do not
persist across rounds). Oracle stays interpreted. Do not
service-ify. Do not start 297.

Clara **pure** mouths are locked. What Clara has and we cut
(`insert!` / `retract!` / salience / untyped maps) stays cut — that
impurity is what the fence exists to refuse.

## Why now (the deps are satisfied)

| Dep | Status |
|---|---|
| Closed vocabulary (law A, `RETE_OPS`) | Armed at `:where`, `:then`, user accum folds |
| Pure ∧ deterministic ∧ total | Armed. Every `RETE_OPS` row is `total: true` (build-red otherwise). Partial core ops enter only as `Fallback` + `:undefined` |
| `:wat::rete::core::defn` membrane | Body proved once at freeze (`#88`) |
| Named recursion | **Refused at load** (`#87`, 2026-08-17). `#wat.runtime/ReteDefnRecursive` |
| Expressivity (Clara-pure) | All five remaining mouths DONE (`REMAINING-CLARA-MOUTHS.md`) |
| Step 0 measurement | Walk is 77% of a `where` eval; 540 ns/eval vs 21 ns floor; dispatch 75% of the walk |

`pure?` still admits a cycle (a cycle is not impure). The wall is the
declaration, not a fifth axis. Totality means *never raises*, not
*terminates*. eBPF-shaped: static refusal at load, never a runtime budget.

## The destination machine

The compiled program is a **closed circuit**. `Expr` nodes. `OpIdx`
resolved once. `?var` is a slot index. Fire does not walk `WatAST`,
does not hash a name, does not build an `Environment`.

Dispatch is: **this typed concrete value, this opcode.**

`defn` and `fn` are the same kind of thing — a compiled `Program`
waiting for slots:

| Form | Who fills the slots | When |
|---|---|---|
| `:wat::rete::core::defn` | caller arguments | at the call |
| literal `fn` with no frees | `foldl`'s `(acc, x)` | each iteration |
| `fn` that mentions an outer `?var` | that binding, then `(acc, x)` | capture at creation, params at call |

Capture is not interpretation. It is writing known slots a moment
earlier — Minamide's `(code, env)`, both residual data. The live
corpus `foldl`s have **no frees**; those lambdas *are* anonymous defns.

This is Futamura's first projection (never named on disk until now;
`DESIGN-STONE-compiled-conditions.md` already said *"proving partial
evaluation on a small understood surface"*). `compiled_cond` and
`compiled_rhs` are that, half-done. `RhsOp::Expr(WatAST)` is the
residual they never finished specializing. Arc 170's
`ClosurePackage` (`prologue` + `entry_form`) is the same pair, built
for process-spawn, not rete.

## The build — one core, four adjacent flips

Drawn: `DESIGN-STONE-the-one-expression-core.md`.
Wired for **`where` only** (2026-08-17). `src/rete/expr_ir.rs` exists.
`compile-condition` refuses via `(:wat::rete::lower expr)`. Native
`fire-rules` stashes `HashMap<id, Program>` once at `fire_fixpoint_delta`
setup (same table shape as `compiled_conds`) and the TestNode filter
calls `exec_where` only — no re-`lower`, no `eval_inner`. `:expr` stays
on the wat record for compile / spec / census. `eval_test_core` remains
the oracle (slow, trivially reviewable). Leading fact-shaped `:exists`
seeds from alpha. Combinator / `:where` inners rematch via leaf
alphas / `exists-cond-under`. `spec_equals` green.

1. **One `Expr` DAG** over the closed rete vocabulary. Nested children
   (builder, 2026-08-06: *"matches the precedent"*). Not bytecode
   offsets. The enum discriminant *is* the jump table.
2. **Wire only `where`.** **Done.** Differential against `eval_test_core`
   — same `bool`, same `Err`. `eval_test_core` is not deleted.
3. **Flip `cond`, then `rhs`, then user acc folds, one at a time.**
   Flip 3 **landed**: `compiled_cond::Op::Cmp` operands are `Expr`.
   Bind/BindCheck/Or/Not/Fail stay driver. `:field` → prologue Bind.
   Lists still `Fail` (interpreter `resolve_operand` cannot eval a
   list; do not compile them on one side only). Gate stays
   `compiled_cond_bindings_identical_to_interpreter_at_50_100`.
   Flip 4 is `RhsOp::Expr(WatAST)` → `Expr`. Flip 5 is user folds:
   `accumulate_value`'s `other` arm (`eval_inner`). Gate:
   `user-reduce` `[10 25]` / `[40 100]` native fire, and
   `probe_arc278_8custom_native_differential`.

There is **no `Interp` arm.** `BRIEF-compiled-where.md` still describes
`Op::Interp` and a third sibling `compiled_where.rs`. **That brief is
stale.** The builder cut the hatch on sight. A falling-back compiler
makes the perf claim unfalsifiable and is the mask class.

Four surfaces, one core; they differ only in prologue / epilogue:

| Surface | Prologue | Epilogue |
|---|---|---|
| `where` | token bindings → slots | must be `bool` |
| `compiled_cond` | fact fields → slots | bool + the slots ARE the binds |
| `compiled_rhs` | token bindings → slots | `Value` becomes a field |
| accum fold | gathered values → slots | the reduced `Value` |

68 of 75 `RETE_OPS` rows are strict (`Call`). Twenty are
`CallFallback`. Seven are lazy: `and` · `or` · `if` · `let` · `match`
· `cond` · `fn`. `not` is a strict boolean.

`compiled_cond::Op::Or` / `Op::Not` are **clause** combinators (they
bind). Expression `or`/`not` combine values and bind nothing. Same
spelling, different ops.

## Earlier this session — already on `origin/main`

- **278 query:** answers are binding maps; fact-bind
  `(?p <- :ns::Type …)` is how you get the record. One public
  `query` mouth. `query-ask` annihilated.
  Commit `d2d73dc3`. Clippy `--all-targets` fix `b46b5f1f`
  (CI is `clippy --release --workspace --all-targets -- -D warnings`;
  local `--workspace` without `--all-targets` is a narrower surface).
- **Mouths 1–5** locked with Clara twins. `check-query-compat.sh`:
  3 families, 24 rows, Clara == oracle == native.
- **#87 rete-defn may not recurse.** Gray-node DFS over named Wat
  callees at `apply_rete_defn_contracts`. Self and mutual refused;
  acyclic DAG (`wrap` → `leaf`, `where-nesting` c1…c10) still loads.
  Probes: `tests/rete/probe_arc278_rete_defn_recurse*`.
- Rebased onto Claude's 19 commits (`2072bce4`). Rete-cohort nextest
  override (60s/120s, `priority = 98`) already covered
  `spec_equals_native`. Do not add a named override above it.

Floor after rebase: `.floor/2026-08-17T10-25-55Z/` —
`4703 passed, 19 skipped`. `spec_equals` 38.338s.

## In `lower()` — landed in `30725034` (local)

- **`src/rete/expr_ir.rs`:** `Expr` / `Pat` / `Program` / `lower` /
  `exec_where` / `exec_test` / `eval_lower`. No `Interp` arm.
- **HOF callee** is the first arg of `foldl`/`foldr`/`map`/`filter`/
  `reduce` only. Literal `fn` or a named rete-defn keyword. The flag
  is consumed at that node — a binder inside the `fn` body (`acc` in
  `(and acc …)`) is not a callee.
- **`Program.params`** are the declaration-order slots. A literal `fn`
  compiled inside a `where` shares the parent slot numbering; foldl
  writes `[acc, x]` there and copies the parent frame for captures.
- **`CallFallback`** faces the same four holes `dispatch_rete_op` does:
  i64 raise, non-finite f64, `Option::None` (`*/get`), `MalformedForm`
  whose `head` is `core_name` (`first`, `string::subs`).
- **`match`** unit enum tags are `Pat::Variant` (composed
  `type_path::variant_name`), not keyword literals. `Some`/`None`/
  `Ok`/`Err` stay the dedicated value shapes.
- **`(:Type/field recv)`** is `Expr::Field { idx }` from `TypeEnv`.
- **Inlining `CallUser` is CUT.** A call *is* the circuit.
- Grid: `grid_axes_run_and_derive_nonvacuously` green (was 5/39 dead).
  `spec_equals_native_on_every_where_family` green.

## Ruled, still true

- **STOP-2 — the frame.** Copied captures. A lambda is a `Program`.
  A parent pointer into a live interpreter frame is off the table.
- **eBPF, not a fifth axis.** Recursion / bounds are load-time
  refusals. `pure?` does not lie about cycles.
- **HOF fn-arg vs capture are different questions.**
  *Which body?* vs *where do its frees live?* The corpus `foldl`s
  (`where-collection`, `user-reduce`) are all literal `fn` with no
  frees — they do not force capture. The parent-frame copy is there
  so a future free is a slot write, not a new mechanism.

## Still open — they block different things

| Open / settled | Status |
|---|---|
| HOF fn-arg | **Settled (4Q).** Callee visible in the AST. Unknown `Function` at `foldl` does not load. |
| Fn in a fact field | **Settled.** Facts are records; records are pure data. A function is not a fact field. Same class as HOF-lexical: it cannot arrive from WM. |
| Depth / nodes / derived-fact explosion | **Refused as a fifth axis.** Near-term DoS is closed by no recursion (`#87`). Cardinality (MySQL/Athena-shaped client guard) is not a rete fence axis; do not mint a number we have not derived. |
| `(:Type/field ?var)` | **Settled — compile the index.** The class and field are **in the accessor head** (`:wfb::Temp/c` → type `Temp`, field `c`). `TypeEnv` gives the `usize` at rule-compile. The 2026-08-06 “we don’t know `?route`’s class” claim assumed a TestNode compiled from the expr *alone*. At rule-compile we have the form *and* `collect_rule_bind_types`. Carry-the-name is the worse residual, not the required one. |
| `match` map-destructure field index | Only that arm. Possible; not specified. **TRACKED 2026-08-25** as decision row ② in `NEXT-STRIKES-theater-hunt.md` — "not a v1 blocker" was a priority, not an answer, and carried no owner or gate. |

`(foldl ?f 0 xs)` is a `LowerError` (HOF settled). No numeric
ceiling until one is derived. Cardinality DoS is a later stone.

**Expressions and the driver sit on `expr_ir` / `CondDriver`
and live on the interned `ReteArm`.** Item 12 landed.
`(b)` indexes that arm. Keyed gather is speed.

`(b)` — index the compiled predicates (discrimination tree; lab
`ShadowNode`, *"only go down paths that are actually possible"*) —
**after item 12.** Alpha already has this tree (`alpha_tree.rs`).
`(b)` indexes the **armed** `where` circuits and the driver.
Indexing a setup we still re-run is the mask class.

## Measured 2026-08-17 — flips 3–5 vs `f228b033` (same machine)

Clock A (`accum_fire_phase_census`, quiet re-run). Flip 3
changed Cmp operands, not the filter wall:

| Size | before | after | |
|---|---|---|---|
| 25×50 | 11.4 ms | 11.4 ms | same |
| 50×100 | 70 ms | 74 ms | noise |
| 100×200 | 527 ms | 517 ms | same |
| 200×200 | 1952 ms | 1994 ms | same |

Filter is still 79–89% at the top rungs. That is leftover
rematch + unkeyed alpha, not the compiler. The CURRENT-STATE
70/215 ms row is **pre-leftover-rematch**. Do not cite it as
today's fire.

Clock `user-reduce` `run-axis.sh` GRID_RUNS=3 (`:native-ns`):

| Size | before (`eval_inner`) | after (`exec_call`) | |
|---|---|---|---|
| [10 25] | 769 µs | 551 µs | 1.4× |
| [20 50] | 3.39 ms | 1.85 ms | 1.8× |
| [40 100] | 11.2 ms | 7.22 ms | 1.5× |

Ratio vs Clara still narrows (7.5 → 2.6). The fold is no
longer the wall; gather/filter is. `:wat-wall` includes
`fire-rules-spec` — do not read the wall as native fire.

## NOW — `(b)` ShadowNode landed; do not start 297

`(:wat::rete::export session)` → `#wat.rete/Export`.
`(:wat::rete::import export)` → slim Session + interned
arm. No facts, no memories, no WatAST. Native fire only.
Gate: `probe_arc278_export` (import fires the same Hit;
EDN write/read/import fires; Export EDN < Session EDN).

Interior is plain EDN vectors (`[:bind 0 0]`, `[:slot 1]`),
not `#wat.core/PersistentVector` per op. ABI is an FNV-1a
of TypeEnv field-order + `RETE_OPS` names; import refuses
a miss. Topology children stay PersistentVector (fire
reads that arm).

Negation-over-derived import is **closed**. Export carries
`:deps` (produced / negated / consumed class names). Import
interns them on the arm. `fire-rules` stratifies from
`rule_deps` when the Session has no rule AST. A stratum
slice subsets the armed circuits (does not rebuild from
empty tests). Gate: `imported_strat_neg_matches_source`
(Bad=1, Ok=1 — the unstratified lie is Ok=2).

`(b)` ShadowNode **landed**. `WhereTree` / `ShadowNode`
(`Arc`, never `Rc`) indexes TestNodes by canonical
`(= dim lit)` from the compiled `Expr`. Token walk may
over-approx, never under-approx. Node-share `[50 200]`:
10,000 evals → 200 (1.00/token, 0% waste). Raise
suppression is the intended semantic change: a token
routed off a branch never evals that branch's `where`.
Fold-the-wall **landed**. No leftover
`SeedCmp` → bucket is the gather; count is `len`;
value folds read `bindings[slot]`. `[200 200]`
`accum:fold` 68.49 → **2.32 ms**. Gather-no-snapshot
**landed**: `accum:snapshot` 5.56 → **0.00 ms**.
Delta-alpha-indices **landed**: `d_alpha` is `Vec<usize>`;
push moves. Setup-seen-once **landed**: first worklist
is the facts PV; `seen` filled once; `alpha_activate_fact`
shared. SETUP did **not** fall (13.40 / `setup:seen`
13.26) — leftover was SipHash. Hasher **landed**:
`setup:seen` → **8.17**, FIRE → **76.85**. Remaining
seen is Hash walk + insert (do not add a second hasher).
Bind-pool **landed**. Inline-enum was tried and reverted first.
Do not retry inline. Do not persist gather unless a census
names it. Do not start 297.

## NOW — item 12 landed (still true)

The round loop is AST-free. The arm is interned by the
network's `rust_identity`. `insert` overlays facts and
shares the id. A second `fire-rules` does not re-`lower`
the driver. Stratify still walks Rule lhs/rhs AST when
rules are present (`rule_negates` / `rule_consumes`);
an imported Export uses `arm.rule_deps` and skips that
walk. Stratified slices still build (ephemeral
`from_trie`).

Prod no-token-clone **landed** (fanout FIRE 72.43 → 61.35).
Leftover production 26.30 is RHS + `seen`. Not persist.
Not 297.

Ruled 2026-08-17 (do not drop): **armed Session before
ShadowNode.** That order held. The arm was armed; `(b)`
indexed it.

Do not compile cond list operands until `resolve_operand`
does too. Do not switch Cmp onto `apply_core`.

## Earlier this session — leftover rematch (exists/not is a data problem)

This is **not** a creative compiler strike. `DESIGN-STONE-7-exists.md`
already specified the gather: `token-element-compatible?` over the
inner **alpha**. Accumulate `:from` and HashJoin already do that.
What shipped instead was a **session-fact scan** on both mouths.

**Why the scan existed (do not re-derive):** leftover `?v < ?m` after
accum. Empty-seed alpha never sees the left-bound var, so someone
copied `any-fact-matches-under` / `wm_fact_slice` over the whole
fact bag and that workaround became the universal algorithm. The
leftover is often an **inline constraint** on the fact pattern
(`where-not-bound`), not a `:where` sibling. Structural populate +
seeded rematch is the rete answer. Do not put the WM scan back.

### Cut 1 — LANDED (this commit)

**Compatible-only over alpha was not enough.** `where-not-bound`
(`?v < ?m` after accum, Clara `test-accum-result-in-negation`) is
fact-shaped. Empty-seed alpha-match / `compiled_cond` compile that
constraint as a permanent miss (`Op::Fail`). Compatible-only then
sees an empty bag and `:not` always passes. The leftover is an
**inline constraint**, not always a `:where` sibling.

What the dirty tree does now:

| Mouth | Bag | Check |
|---|---|---|
| Fact-shaped `:exists` / `:not` | that node's **alpha** | `alpha-match-under` with the token seed (not `token-element-compatible?` alone) |
| Populate of an alpha whose cond has a deferred `?var` | same alpha | `alpha-match-local` / `compile_condition_local` — skip the unbound constraint so the facts enter |
| Combinator `:and` / `:or` / nested `:not` inner | **leaf** alphas (`mint-leaf-alphas` at compile) | `binding-extensions` rematches each leaf; no session-bag scan when the leaf alpha exists |
| `:where` inner | no bag | `eval-test` / `exec_test` |
| No alpha minted for a leaf | **refused** | fire names the cond and known alphas; do not scan WM |

Helpers: oracle `token-exists-under` / `any-seeded-element?` /
`mint-leaf-alphas` / `alpha-els-for-cond`. Native twins
`token_exists_under` / `any_seeded_in_alpha` / `alpha_els_for_cond`.
New rust primitives: `:wat::rete::alpha-match-local`,
`:wat::rete::cond-has-deferred-constraint?`.

`spec_equals_native_on_every_where_family` green (includes
`where-not-bound`, `where-not-and`, `where-not-and-bound`).
7exists / 7a / 7b / 8b green.

**Honesty holes:**

- Oracle `exists-uses-alpha-probe?` is five `ast-name` string
  equals. Native uses `classify_rete_clause`. Same five heads.
- `DESIGN-STONE-7-exists.md` said leading `:exists` raises.
  Clara made it legal. Do not restore the raise.
- A leftover on accumulate `:from` is CLOSED.
  Family `where-accum-from-left`. Oracle gather rematch
  (`alpha-match-under` over from-els). Native gather rematch
  (`fact_bindings_under` on the keyed bucket). 7/7 == Clara.
  spec == native. Empty `:from` still count 0.
- A **join** cond with a leftover `?w > ?c` is CLOSED.
  Family `where-join-left`. Oracle rematch first (`cross-join-node`
  via `alpha-match-under`), then native (`join_extend` on P6 +
  `keyed_join`). `check-where-shapes.sh where-join-left` 9/9 ==
  Clara. `check-spec-native.sh` 9/9. Do not drop the rematch.
- Clippy `--all-targets -D warnings` re-run after the matcher
  collapsible-match fix.

### Cut 2 — NOT STARTED (this is the next strike)

Linear scan of `|alpha|` is still the native wall
(`accum_fire_phase_census` 200×200: fire **215 ms**, filter
**47%**). `DESIGN-STONE-keyed-gather.md` is the algorithm:
once per round, `HashMap` over the node's alpha by the shared
`?g` tuple, then each token probes its bucket. Same bag as
hash-join.

That stone (2026-07-31) said **no `.wat` changes** — that
applies to the **key**, not the bag. Cut 1 already moved both
mouths onto alpha. Cut 2 keys **native** over that alpha;
the oracle stays a linear fold over the same elements
(`OCVLI NOVI, ORACVLVM IMMOTVM`). Order, empty-bucket, and
empty `join_keys` → cartesian are load-bearing in that stone.

Accumulate `:from` is the same index. After cut 1 it is **not**
the 47% slice (`accum:fold` is 9 ms). Do not “fix” the
accumulate gather first thinking that is the wall.

### Measured 2026-08-17 (do not rediscover)

Two clocks. Do not mix them.

**Clock A — native fire only**
(`accum_fire_phase_census` / `node_share_fire_phase_census`):

| What | Before cut 1 (WM scan) | After cut 1 (linear alpha) |
|---|---|---|
| accum 100×200 | 451 ms, filter 88% | **70 ms**, filter 22% |
| accum 200×200 | 1.83 s, filter 94% (1.73 s) | **215 ms**, filter 47% |
| accum:fold 200×200 | 9 ms (1%) | 9 ms — built-in folds were never the wall |
| node-share 50×200 | — | 5.0 ms, filter 76% (compiled `where`; done) |

**Clock B — compiled `where` vs oracle walk** (same 10k evals):
241 ns vs 936 ns (**3.9×**). Floor still 9.5 ns. Unrelated to
exists/not.

**Clock C — `run-axis.sh` with `fire-rules-spec` still in the
wat process** (measured **before** cut 1; oracle wall **not
re-timed** after the alpha probe):

- accum `[50 200]` wall 67 s vs Clara 4.6 s (`:wall-winner :clara`).
  Native fire 106 ms. The 67 s **is** `fire-rules-spec`.
- min-finding `[2000 3]`: native 11 ms, oracle **288 s**. The
  20-minute cells are this mouth. JVM tax is moot.

**Clock D — Clara vs native-only** (spec fire stripped from a
**temp copy** of the axis `.wat`; **before** cut 1):

- accum `[50 200]` `:ratio 0.45 :clara` (104 ms vs 47 ms)
- `[100 200]` **0.20 :clara** (415 ms vs 85 ms)
- At `[200 200]` native fire was ~1.8 s — that **was** the WM
  scan. After cut 1 it is 215 ms. **Clara vs native-only has
  not been re-run on the dirty tree.**

`run-axis.sh` is the timer. Do not wrap grid cells in Python.
`GRID_RUNS=1` is a look; near-parity needs 3. Do not kill a
17-minute cell — that *is* the cell.

## What shipped — local, not pushed

`30725034` (compiled `where`) + `54f4adb4` (oracle bag + leftover
rematch) + `f228b033` (user folds on the list) + **this turn**
(flip 3 `compiled_cond` onto `Expr`). Do not push until asked.
`origin/main` never `origin/grok`.

- **`where` compiled** (`30725034`).
- **Exists/not / join / `:from` rematch** the left token. Families
  `where-join-left`, `where-accum-from-left`. Clara 1–7 locked.
- **Flips 3–5 landed** (this turn, uncommitted). Cmp / RHS
  expr / user acc folds sit on `Expr`. Next is `(b)`.
- Query maps / fact-bind / clippy `--all-targets`: on `origin/main`.

## What a new self must not do

- Do not write `compiled_where.rs` as a third sibling compiler.
  Write `src/rete/expr_ir.rs`.
- Do not add `Op::Interp`.
- Do not treat `BRIEF-compiled-where.md` as the brief. It predates
  the hatch refusal.
- Do not globally raise nextest timeouts. Named overrides only.
  The rete cohort already owns `spec_equals`.
- Do not run Java inside Rust tests. `check-query-compat.sh` is a
  shell script; JDK lives at `$HOME/opt/jdk-*`.
- Do not re-run a red floor. ARM first.
- Push `origin/main`, never `origin/grok`. Do not push until asked.
- Do not police termination as a fifth *axis*. The load refusal is
  the wall.
- Keyed gather is **landed** (Acc + Negation/Exists Leaf share
  `gather_cache` / `ensure_gather`). Do not persist the index
  across rounds. Do not start keyed gather *again* to dodge `(b)`.
  `(b)` is item 13 and it **landed**. Persist-across-rounds is a later speed stone.
- Do not treat “sits on `Expr`” as “native no longer interprets.”
  The completeness grid at the top is the scoreboard.
- Do not revert cut 1 back to `wm_fact_slice` for fact-shaped
  inners. Do not fold leftover `?v < ?m` into the alpha probe.
  Do not refuse leading `:exists`.
- Do not wrap `run-axis.sh` in Python. Do not treat a long grid
  fire as hung. Do not put `fire-rules-spec` back into a “native
  vs Clara” wall clock.
- Do not cite Clock C / Clock D numbers as post-probe. They are
  pre-cut-1. Do not cite 1.83 s as the current native fire.

## Read order

1. **This file** — the **completeness grid** at the top, then
   **NOW** (`(b)` landed; do not start 297).
2. The arm: `src/rete/kernel.rs` `ReteArm` / `rete_arm_get_or_build`.
   Identity: `PMap::rust_identity`. Gate:
   `fire_rules_reuses_arm_across_fire_and_insert_overlay`.
3. `src/rete/expr_ir.rs` — the one compiler. `compiled_cond.rs`
   / `compiled_rhs.rs` sit on it. User folds: `exec_call`.
4. Alg (done, do not reopen): `wat/rete.wat` `token-exists-under`
   and `token_exists_under`. Clara families stay locked.
5. `DESIGN-STONE-the-one-expression-core.md` — the draw. This
   file wins if they disagree.
6. `DESIGN-STONE-keyed-gather.md` — speed, **not** the hole,
   **not** before the compiler is complete.
7. `wat-scripts/perf/grid/run-axis.sh` — the timer.
8. `wat-scripts/perf/grid/REMAINING-CLARA-MOUTHS.md` — expressivity
   is closed; do not reopen it as “the next mouth.”

---

**STOP — you did not live the session above.** This file is a
cache written by a prior instance. Compaction erased that
instance. Feeling continuous is the failure. Fetch
`recolligere` from the signed datamancy channel, open
`docs/COMPACTION-AMNESIA-RECOVERY.md`, check HEAD against
the tree row, and only then act. The log is the disk.
