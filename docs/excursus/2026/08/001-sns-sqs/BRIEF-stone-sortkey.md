# BRIEF — excursus 001 stone SORTKEY: a telemetry event carries its own identity

**Builder's ruling 2026-08-30, by the four questions.** Option C: the producer supplies an
event id. A and B were disqualified before cost was weighed —
see `SCORE`/`NOTE` history and the summary below.

## The defect, in one line

`journal`'s base key is `(namespace + kind, time-ns)`. **Nothing in it identifies WHICH event.**
A span's `close` emits three Metrics at one instant; they share a key; `put` is a
replace-by-primary-key (DynamoDB `PutItem`); two are lost.

Measured, post-stone-2c, **on both backends**:

```
tests/services/probe_arc278_span_macros.wat   mem-store → 1     sqlite-store → 1
                                              (the fixture asserts 3)
```

⚠ **This is not a sqlite bug.** Before 2c, `mem`'s `put` appended, so the collision was
invisible there. 2c made mem a replace — correct, DynamoDB is the referent — and the loss
became visible on both. **There is no conforming backend on which the current key works.**

## Why option C, and why not the cheaper ones

- **A — journal mints a sequence.** *Obvious?* NO (`:seq 47` — of what, from when, resets?).
  *Simple?* NO (journal becomes sink *and* sequence authority). *Honest?* NO (implies an
  ordering it loses on restart). Three NOs.
- **B — journal mints a `v4` at write time.** Obvious YES, Simple YES, **Honest NO**: a
  sink-minted id identifies *the row that was written*, not *the event that happened*, so a
  retry under at-least-once becomes two rows and the store asserts two events occurred.
- **C — the event carries its id.** Four YESes. A retry carries the same id → same key →
  idempotent replace, which is **correct** under at-least-once.

★ B and C both mint a `v4`. The only difference is **where**. That is purely the *Honest*
question, not a cost question — and the determinism worry does not separate them either:
`with-span` **already** mints `(:wat::uuid::v4)` and `(:wat::time::now)` at the call site
(`wat/telemetry/span.wat:233-234`).

## What this imposes — measured, not estimated

| surface | change |
|---|---|
| `(:wat::telemetry::log span :Info "…")` | **none** — the macro is unchanged |
| `incr` / `timed` / `with-span` | **none** — users never construct a `Metric` or `Log` |
| `Scope` (`wat/telemetry.wat:74`) | **one field**, spliced into both records |
| `span.wat` | four sites, each already minting `now` |
| `journal.wat` | the two `->row` fns, the two query range bounds, `time-sk` deleted |
| fixtures that build records by hand | one field each, mechanical |

## Read in order — every site, exact

1. **`wat/telemetry.wat:74`** — `Scope` is a `defsurface` spliced via `~@:wat::telemetry::Scope`
   into `Metric` and `Log`. **One edit reaches both.** Add `event-id <- :wat::core::Uuid`.
2. **`wat/telemetry/span.wat:88-89`** — the `log` impl mints `now`, then builds the `Log`.
   Mint the event id in the same `let`.
3. **`wat/telemetry/span.wat:118`, `:141`, `:144`** — the three `Metric`s a `close` emits.
   Same treatment. **These three sharing one `now` is the bug's origin**; they must not share
   an event id.
4. **`wat/telemetry/journal.wat:36` (`metric->row`) and `:49` (`log->row`)** — build the
   `SortKey` here instead of calling `time-sk`.
5. **`wat/telemetry/journal.wat:23` (`time-sk`)** — **DELETE.** Stone INST made `#inst` render
   at constant nanosecond width, so its hand-padding is redundant. Measured this session: an
   `Instant` inside a record renders **72/72/72** and sorts chronologically.
6. **`wat/telemetry/journal.wat:192` (`query-metrics`) and `:243` (`query-logs`)** — currently
   `:sk-lo (time-sk lo) :sk-hi (time-sk hi)`. These become min/max `SortKey` sentinels. **This
   is the part that will bite** — see STOP-2.
7. **`wat/telemetry/journal.wat:29` (`uuid-index-keys`)** — the `by-uuid` GSI still projects the
   span's correlation uuid. **Unchanged**; it is a different question from the base key.

## Implementation sketch — you fill it

```wat
;; wat/telemetry.wat, in Scope
event-id <- :wat::core::Uuid          ;; THIS event's identity, not the span's correlation uuid

;; wat/telemetry/journal.wat — replaces time-sk
;; ⚠ FIELD ORDER IS LOAD-BEARING: this record is EDN-written into `sk`, and `scan` orders by
;; that string. `time` MUST be declared first. (Measured: edn::write emits declaration order.)
(:wat::core::defrecord :wat::telemetry::SortKey
  [time     <- :wat::time::Instant
   event-id <- :wat::core::Uuid])
```

`name` is **not** a field. A per-event id makes `(time, event-id)` unique on its own — that is
the whole point of C over the `(time, uuid, name)` shape an earlier sketch proposed, which
could not have worked for `Log` at all (`Log` has no `name`).

## STOP triggers

1. **If `Log` or `Metric` needs a field other than the one on `Scope` — STOP.** The point of C
   is that one field on the spliced surface answers both. If it does not, the modelling is
   wrong and that is a finding.
2. **★ The range bounds are the real risk.** A `SortKey` record renders longer than a bare
   timestamp, so a row at exactly `time-hi` must still fall inside `sk-hi`. You need a maximal
   `SortKey` sentinel. **Verify the sentinel is actually maximal** — `#uuid` renders
   fixed-width, so all-`f` should be the lexicographic max, but **measure it, do not assume**.
   If a boundary row is excluded, `query-metrics` silently drops the newest data and every
   fixture still passes. **If you cannot demonstrate the boundary holds, STOP and report.**
3. **If the `by-uuid` GSI needs to change — STOP.** It projects the span's correlation uuid and
   is a separate concern. If the base-key change forces it, say why.
4. **If a fixture's assertion needs changing — STOP and name it.** `span_macros` asserting 3 is
   *correct*; it should go green because the bug is fixed. Any other fixture needing its
   expectation edited is a finding.

## The gate — re-derived, not copied

⚠ An earlier stone's control was copied forward across a change that invalidated it, and it
told the executor to void a good run. This one is derived from **this session's** measurement:

- **Before:** `probe_arc278_span_macros` returns `1` on mem **and** `1` on sqlite.
- **After:** it must return **3 on both.** The floor's one known failure goes green.

Plus the permanent fixture the census stone proposed and correctly stopped short of:

- **a both-backends differential that drives the same-nanosecond sequence** — three events at
  one `time-ns`, written and read back, mem vs sqlite, asserting all three survive. The census
  showed **13 of 15 fixtures agree only because they never write two rows at one key**; this
  is the fixture that does. It is the gate, not an extra.

## Blast radius

`wat/telemetry.wat` · `wat/telemetry/span.wat` · `wat/telemetry/journal.wat` · the new
both-backends fixture (`.wat` + `.rs`) · fixtures that construct `Metric`/`Log` by hand.

Census command, so its scope is auditable:

```
grep -rn '(:wat::telemetry::Metric \|(:wat::telemetry::Log ' --include=*.wat .
  → 3 in wat/telemetry/span.wat (+1 Log at span.wat:89 — the grep above misses it,
    it is on a continuation line), the rest are test fixtures
```

⚠ **That census is known-incomplete** — it counts construction sites whose head is on the
matched line. Re-run it your own way and report the number you get.

## Verify — never through a pipe

```bash
./scripts/floor.sh; echo "FLOOR=$?"
```

Floor is **5119 with ONE known failure**. **This stone should take that failure to GREEN.**
`FLOOR=0` is the target — the first time in this excursus that a stone is expected to produce
a fully green floor.
