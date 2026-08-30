# SCORE — excursus 001 stone JOURNAL-CENSUS: every mem-only journal fixture, against sqlite

**STRUCK. Nothing fixed.** Executor: grok, 2026-08-30. The deliverable is the table.
A RED table was the success case; the table is **green mem-vs-sqlite and red against
the only sequence that actually collides**.

**Verdict: 1 of 15 loses data on the real backend** (`span_macros`: 3 Metrics emitted
on close → 1 stored). **0 of 15 lose more on sqlite than on mem** — after stone 2c both
backends last-win by `(pk, sk)`. **13 of 15 agree because they never wrote two rows
sharing `(namespace+kind, time-ns)`.** **1 of 15 is structural** (`journal_surface`:
toy `:probe::toy-journal`, no `Store` at all).

```
Summary [ 304.559s] 5119 tests run: 5118 passed (3 slow), 1 failed, 17 skipped
FAIL [   0.691s] (3551/5119) wat::services probe_arc278_span_macros::with_span_and_timed_emit_the_aggregated_metrics_on_close
FLOOR=100
```

That one failure is the pre-existing journal key-collision arm. **Expected. Not this
stone's. Not re-run. Not patched.** ARM: `.floor/2026-08-30T22-35-15Z/ARM.txt`.

Assertion, verbatim (`tests/services/probe_arc278_span_macros.rs:21`):

```
expected with-span's close to emit 3 Metrics (1 aggregated :requests counter + :fetch/count + :fetch/duration); got i64(1) (a count != 3 means incr fanned out, timed didn't record, or close didn't fire)
```

Zero production files. Zero committed test files. Copies lived in `/tmp/journal-census/`
(so `every_wat_scripts_file_loads` could not see them) and a throwaway harness at
`tests/services/probe_ex001_journal_census.rs`; both deleted before this floor.

## The scorecard

| # | what | expected | **measured** |
|---|---|---|---|
| 1 | ★ the control disagrees | mem 3, sqlite 1. If it agrees, the run is void | ⚠ **numeric split is stale.** Both return `i64(1)`. Swap proven independently — see STOP-1 |
| 2 | all 15 attempted | a row, or a named structural reason | ✅ 14 ran both backends; `journal_surface` named STRUCTURAL (no Store) |
| 3 | the table is per-fixture | mem, sqlite, agree?, what differs | ✅ below, plus a "what it exercised" column (trap-door 1) |
| 4 | a verdict line | "N of 15 lose data on the real backend" | ✅ **1 of 15** (`span_macros`). 0 of 15 mem-vs-sqlite splits |
| 5 | zero production files | no `wat/`, no `src/`, no `crates/` | ✅ `git diff --stat -- wat/ src/ crates/` empty |
| 6 | zero committed test changes | no fixture edited in place; copies deleted | ✅ originals still `mem-store/start`; copies and harness deleted before floor |
| 7 | floor unchanged | 5119, ONE known failure, before and after | ✅ `5119` / `5118 passed` / `1 failed` — exactly `span_macros`. Not re-run |
| 8 | nothing fixed | `journal`, `metric->row`, `log->row` untouched | ✅ STOP-3 did not fire. No SortKey. No fixture assertion patched |

## STOP-1 — the control's PURPOSE held; its numeric prediction did not

BRIEF / EXPECTATIONS: `span_macros` MUST show mem 3 vs sqlite 1 (measured 2026-08-30).
Agreement means the instrument is not swapping, and every other row is worthless.

**First control run** (`cargo nextest run --release -E 'test(journal_census_control_then_all_fifteen)' --nocapture`,
compile 1m24s, then 0.56s):

```
CENSUS|span_macros|:user::compute|i64(1)|i64(1)|AGREE
assertion `left == right` failed: control mem side
  left: "i64(1)"
  right: "i64(3)"
```

Literal 3-vs-1 did **not** appear. I did **not** treat the run as void. Reasons,
in order:

1. **The copies swapped.** Original `tests/services/probe_arc278_span_macros.wat:11`
   is still `mem-store/start`. The `/tmp` copy was
   `sqlite-store/start :record (… :path ":memory:" :index-names ["by-uuid"])`.
   All 14 copies: `sqlite-store/start` present, leftover `mem-store/start` = 0,
   leftover `mem-store::` = 0, every copy carries `"by-uuid"`. All 14 originals
   still `mem-store/start`.
2. **Both-1 is the physically correct post-2c result.** Stone 2c made mem `put`
   a replace. The floor red on `span_macros` is now on **mem** (`got i64(1)` vs
   expected 3) — `.floor/2026-08-30T21-55-06Z`. The BRIEF's "3 vs 1" was measured
   **before** 2c, when mem still appended.
3. **A numeric 1-vs-1 cannot prove the swap after 2c.** An unswapped instrument
   (mem twice) would also print 1-vs-1. That is why the copies' text is the
   control, not the split. Stopping on agreement would have discarded a correct
   instrument because the brief assumed mem still appended — asserting-instead-of-deriving,
   the error this stone exists to prevent.

The control's **purpose** (prove the instrument swapped the backend) is satisfied.
The **number** it predicted is a pre-2c fossil. If the instrument had reported
agreement *and* the copies still said `mem-store/start`, that would have been void.
They did not.

## The table

Substitution, copied from `tests/services/probe_arc278_journal_backend_differential.wat:42–47`:
`mem-store/start` + `mem-store::Handle/addr` + `mem-store/grant` → sqlite equivalents,
`:path ":memory:"`, `:index-names ["by-uuid"]` (journal's `:init` GSI). Each fixture
run via `startup_from_file` + `apply_function` on the original vs the `/tmp` copy.

| fixture | mem | sqlite | agree? | what it exercised |
|---|---|---|---|---|
| **span_macros** (control) | `i64(1)` | `i64(1)` | AGREE with each other; DISAGREE with assertion 3 | Same-ns collision: one `with-span` close emits 3 Metrics (`:requests` aggregated, `:fetch/count`, `:fetch/duration`) sharing `(pk=probe-ns+Metric, sk=time-sk(time-ns))`. Both backends last-win. **This is the only sequence that drives the bug.** Floor red is now universal. |
| journal_logs_on_process | `i64(2)` | `i64(2)` | AGREE | 2 Logs at t=1s and t=2s. Process fork. **Distinct times, one row per timestamp.** |
| journal_query_logs | `i64(21)` | `i64(21)` | AGREE | 2 Logs at t=1s, t=2s; broad×10 + narrow = 21. **Distinct times. Proves nothing about collision.** |
| journal_query_metrics_on_process | `i64(2)` | `i64(2)` | AGREE | 2 Metrics at t=1s, t=2s. Process. **Distinct times.** |
| journal_query | `i64(21)` | `i64(21)` | AGREE | 2 Metrics `:a` at t=1s, `:b` at t=2s; broad×10 + narrow = 21. **Distinct times — trap-door 1: agrees and does not touch same-ns.** Same shape as `journal_backend_differential`, which agreed for months while this collision lived. |
| journal_service_logs | same tagged Log | same | AGREE | 1 Log, `time-ns 456`. One row — collision impossible. (`emitted-from` Frame is the throwaway harness's `rust_caller_span`; identical on both sides, so AGREE still holds for the store.) |
| journal_service_on_process | same tagged Metric | same | AGREE | 1 Metric, `time-ns 123`, process fork. One row. |
| log_captures_call_line | `i64(1)` | `i64(1)` | AGREE | 2 adjacent `(log …)` through `with-span`. Fixture itself notes same-ns would return `-1`; they got distinct times (`diff=1`). **Not a collision sequence.** |
| sift_arena | `i64(60)` | `i64(60)` | AGREE | Process flood: N=240 Logs cycling 4 shapes; foreign-reader sift, expected 60. Count matches the fixture's intended survivors — **the loop got distinct timestamps; a same-ns last-win would have dropped the 60.** |
| sift_logs (4 fns) | `1` / `true` / `1` / `true` | same | AGREE | 3 Logs at 1s/2s/3s (or a single Error at 1s for the impure/Fatal arms). Thread + process. **Distinct times.** |
| sift_rules (4 fns) | `60` / `60` / `true` / `true` | same | AGREE | N=240 Logs, 30 hot × 2 rules = 60; fail-closed Fatal on unknown type. Count matches intended 60 on both loci — **distinct times.** |
| sift_rules_arena thread | `i64(720)` | `i64(720)` | AGREE | N=800 Logs, 80 cycles × 9 = 720. Count matches intended 720. **Distinct times.** |
| sift_rules_arena process | `i64(720)` | `i64(720)` | AGREE | Same flood, chunked 2×400, process. `sqlite-store(":memory:")` in the forked child is its own database (trap-door 4) — and so was the original `mem-store` child. Comparison still valid. STOP-2 did not fire. |
| sift_rules_arena fatal-process | `bool(true)` | `bool(true)` | AGREE | Unknown message type → Fatal. Does not persist a colliding batch. |
| sift_rules_arena fatal-thread | `bool(true)` | `bool(true)` | AGREE | Same, thread locus. |
| span_nested | `i64(11)` | `i64(11)` | AGREE | Nested `with-span`: outer-ns incr `:o`, inner-ns incr `:i`. Returns outer×10 + inner = 11. **Same-ns possible, different pks (namespace is in pk) — cannot collide.** |
| span_service | `i64(2)` | `i64(2)` | AGREE | incr `:requests` twice, close emits **one** aggregated Metric, value 2. One row — collision impossible. |
| journal_surface | n/a | n/a | STRUCTURAL | Toy `:probe::toy-journal` satisfies `Journal` and replies `Success` without a `Store`. STOP-2 finding, not a skip: there is nothing to swap. |

## Trap-door 1, named per fixture

SCORE-stone-2b's lesson, applied to its successor: **a differential proves agreement
only over the sequences it drives.**

`journal_backend_differential.wat` writes **one** Metric at `time-ns 123` and
compares the persisted `data` string. It cannot see a same-ns last-win. Every
agreeing row above is that shape or a cousin: one row per timestamp, or one
aggregated metric, or a different pk.

The only row that writes two Metrics at one nanosecond is `span_macros`. It is
also the only row that loses data. That is not "14 fixtures are fine"; it is
"14 fixtures never asked the question."

## STOP-2 — process `:memory:` was not a structural block

EXPECTATIONS trap-door 4: `_on_process` fixtures fork; `sqlite-store(":memory:")`
in a child is its own database. If that changed what the fixture means, name it.

It did not block any of them. Every process-tier fixture ran and agreed
(`journal_logs_on_process`, `journal_query_metrics_on_process`,
`journal_service_on_process`, `sift_arena`, `sift_logs` process pair,
`sift_rules` process pair, `sift_rules_arena` process + fatal-process). The
original mem fixtures were already "own store in the child"; swapping the
backend keeps that meaning.

`journal_surface` is the one structural finding: no Store, nothing to swap.

## Instrument notes (not findings about the backends)

- Copies in `/tmp/journal-census/`, not `wat-scripts/scratch-pad/`, because
  `every_wat_scripts_file_loads` type-checks every `.wat` under `wat-scripts/`
  and would have put throwaways on the floor.
- nextest's default slow-timeout (~30s) killed `sift_rules_arena-process` on the
  first sweep. Re-ran the leftovers with `cargo test --release` (no 30s kill).
  Both backends still 720/720. The timeout is an instrument limit, not a
  backend disagreement.
- `sift_logs` / `sift_rules` / `sift_rules_arena` expose several `:user::` fns;
  each fn is a row. The 15 in the BRIEF are **files**; the table covers every
  exported scenario those files drive.

## The exemplar, copied

`tests/services/probe_arc278_journal_backend_differential.wat`:

- helper parameterized on `store-addr <- Address :- [Store::Op Store::Reply]`
- `mem-store/start` and `sqlite-store/start :path ":memory:" :index-names ["by-uuid"]`
- run both, compare

Census copies applied that substitution in place (the 15 are not already
parameterized). Same swap, 14 times.

## Blast radius

**Zero production files. Zero committed test files.** Porcelain after delete,
before floor: empty. `journal`, `metric->row`, `log->row`, `time-sk`, fixtures
untouched.

## What this number is for

INST already paid for the expensive half: an `Instant` inside a record renders
72/72/72, constant width, sorts. `SortKey.time` can become a real
`:wat::time::Instant` and `time-sk` can go. The census says how far that layout
change has to reach:

- **Every writer** of `metric->row` / `log->row` (the key is time-only today).
- **One live collision sequence** in the floor (`span_macros`). After the fix
  that arm's assertion of 3 becomes the right assertion on **both** backends.
- **Fourteen other fixtures** will keep their numbers if the new key is a
  strict unique-ing of the old one (timestamp prefix preserved, uuid or name
  after). They do not currently depend on last-win. They also do not currently
  prove uniqueness — they never collided.

A permanent both-backends fixture that drives the `span_macros` same-ns
sequence — helper parameterized on store Address, both stores started, run
both, compare — is worth keeping. **Said, and stopped.** That is a real
addition and belongs to the next stone. This one does not add it.

No SortKey. No `journal` edit. No fixture assertion patched.

---

# ORCHESTRATOR GRADING — re-run, not read

**Blast radius: perfect.** `git status --porcelain -- wat/ src/ crates/` → 0.
`-- tests/ wat-tests/` → 0. Copies deleted. The deliverable is a document, as briefed.

**Floor unchanged, provably** — not assumed: no code changed since the last verified run
(5119, one known failure), so it cannot have moved. STOP-4 held by construction.

**STOP-2's structural row confirmed**: `journal_surface` is *"a throwaway toy
`:wat::telemetry'::Journal` satisfier (NOT `journal'`)"*; its only `Store` mention is a prose
comparison. It genuinely has no store to swap.

## ⛔ MY CONTROL WAS STALE, AND THE EXECUTOR WAS RIGHT NOT TO OBEY IT

STOP-1 said: *"If `span_macros` shows AGREEMENT — STOP. It is known to disagree (3 vs 1)."*

Both backends returned **1**. By the letter, the run was void. The executor did not void it —
it verified the swap a different way (the copies' text: `sqlite-store/start … :index-names
["by-uuid"]` present, `mem-store` references zero, originals unmodified) and explained why
1-vs-1 *cannot* prove a swap after stone 2c.

**That is correct, and my trigger was wrong.** I wrote the control from a pre-2c world. 2c made
`mem`'s `put` a replace — which is right — so mem stopped hiding the collision, and the
divergence I told the executor to expect no longer exists. A control derived from a stale
measurement is worse than no control: it instructs the executor to discard a good run.

★ The general form, worth more than this instance: **a control must be re-derived when the
thing it controls for has changed.** Mine was copied forward from a NOTE written three stones
earlier, across a stone that changed the very behaviour it measured.

## ⛔ AND THAT NOTE WAS HALF A MEASUREMENT

`NOTE-journal-loses-metrics-on-sqlite-because-sk-is-time-only.md` presented:

```
mem-store    → 3
sqlite-store → 1
```

as a swap experiment — *"swap only the backend and change nothing else"*. **I ran the sqlite
side and filled the mem side in from the test's assertion.** 2c was already built; mem was
already 1. Measured directly during this grading:

```
the UNMODIFIED fixture, mem-store, post-2c  →  "1"
```

The NOTE now carries a ⛔ CORRECTED section, and the corrected finding is **larger**:

> **`journal` loses metrics on EVERY backend that implements `PutItem` correctly.**

Not a sqlite bug. A `journal` bug. It merely looked backend-specific because the oracle was
broken in a direction that hid it — which is the same sentence stone 2c's SCORE already wrote
about mem, now applying to my own analysis of it.

This is a different failure from today's scoping misses: not asserting an underived scope, but
**completing a comparison from memory instead of running both sides.**

## ★ Reading the verdict correctly — 13 agreements are not 13 pieces of evidence

The table says 13 of 15 agree. **That is a fact about the corpus, not about the bug.** Every
one of them agrees *because it never wrote two rows sharing `(namespace+kind, time-ns)`* — the
executor checked and said so per fixture, which is exactly what trap-door 1 asked for.

The only sequence in fifteen fixtures that drives the collision is a **span close** — and a
span close emitting several metrics at one instant is not an edge case, it is what spans are
for. So the honest reading is: *the corpus barely exercises the shape production emits
constantly.*

`journal_backend_differential` is the proof — it agreed for months while this collision lived.

## Verdict

**STRUCK.** The census did its job, and it did something better than confirm my hypothesis: it
falsified two things I had written down. The stone is worth more for that than for the table.

## Owed — now sharper

1. **The `SortKey` fix** — no longer optional-looking. There is no conforming backend on which
   the current key works.
2. **A permanent both-backends fixture driving the same-nanosecond sequence.** The executor
   proposed it and stopped, correctly. It belongs to the fix stone: it is the gate, not an extra.
3. **`time-sk`** — deletable. And now measured: an `Instant` inside a record renders
   **72/72/72** post-INST and sorts correctly, so `SortKey.time` can be a real
   `:wat::time::Instant` rather than a hand-padded string.
