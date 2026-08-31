# SCORE — excursus 001 stone SORTKEY: a telemetry event carries its own identity

**STRUCK.** Executor: grok, 2026-08-30. Option C. First fully green floor in this excursus.

```
Summary [ 302.320s] 5121 tests run: 5121 passed (3 slow), 17 skipped
FLOOR=0
```

Log: `.floor/2026-08-30T23-37-58Z/`. The known red is gone.

**The control, re-derived this session:** `span_macros` was `1` on mem and `1` on sqlite
(JOURNAL-CENSUS, post-2c). It now returns **3** on mem (floor, still asserts 3) and the
same-nanosecond sequence returns **3 on both backends** (`count=3;names=a,b,c`).

**STOP-2, demonstrated not argued:**

```
hi=2;wide=3;nil<=max=1;mid<=max=1;high<=max=1;next>max=1;helper=1
```

`query-metrics [T, T]` returns both rows at T, including one whose event-id is *not* nil
(the silent-failure case: a nil max-sentinel would have dropped it). A row at T+1 is
excluded. all-f is ≥ nil / mid / high-but-not-f at the same Instant, and < a SortKey at
T+1. `sort-key-hi(T)` equals `write(SortKey T all-f)`.

## The scorecard

| # | what | expected | **measured** |
|---|---|---|---|
| 1 | ★ the control, re-derived | `span_macros` 3 on mem AND 3 on sqlite (was 1/1) | ✅ floor `PASS (3552/5121) with_span_and_timed_emit_the_aggregated_metrics_on_close` — still asserts 3, now gets 3. sqlite of the same-ns sequence: `count=3;names=a,b,c` |
| 2 | the floor's known red goes green | `FLOOR=0` | ✅ `FLOOR=0`. 5121 passed, 0 failed |
| 3 | `Scope` gains ONE field | `wat/telemetry.wat:74`; splices to both | ✅ `event-id <- :wat::core::Uuid` after `time-ns`. Metric and Log ctor comments now 5 spliced fields. STOP-1 did not fire |
| 4 | the four span sites mint distinct ids | Log + three close Metrics must NOT share an id | ✅ Log: `eid` bound next to `now`. Three Metrics: each ctor has inline `(:wat::uuid::v4)` — they share `now`, they do not share an id |
| 5 | `SortKey` declares `time` FIRST | field order load-bearing | ✅ `[time <- Instant, event-id <- Uuid]` at `journal.wat:28–30` |
| 6 | `time-sk` is DELETED | `grep -c 'time-sk' wat/` → 0 | ✅ 0 (the token is gone from `wat/`, including comments) |
| 7 | ★ the boundary holds | a row at exactly `time-hi` is RETURNED | ✅ `hi=2` — both rows at T come back, one of them with event-id `aaaa…` |
| 8 | the max sentinel is really maximal | measured, not assumed | ✅ `nil<=max=1;mid<=max=1;high<=max=1;next>max=1;helper=1` |
| 9 | the new both-backends same-ns fixture | three events at one `time-ns`, all three survive, mem == sqlite | ✅ `PASS (3568/5121) same_ns_three_metrics_survive_on_mem_and_sqlite` — `count=3;names=a,b,c` |
| 10 | `by-uuid` GSI unchanged | `uuid-index-keys` untouched (STOP-3) | ✅ function body unchanged; still projects `Metric/uuid` / `Log/uuid` (span correlation), not `event-id` |
| 11 | no fixture's assertion weakened | `span_macros` still asserts 3 (STOP-4) | ✅ still `matches!(got, Value::i64(3))`. Two goldens *grew* a field (shape change, named below) |
| 12 | prior stones undisturbed | `probe_ex001_*`, inst arms, write-opts arms all PASS | ✅ delete / reput / write-opts arity; inst lexicographic arms; write-json 1-arg identity — all PASS in this floor |
| 13 | test count | 5119 + the new fixture's arms | ✅ **5121** = 5119 + same-ns + boundary |

## The census, re-derived

BRIEF's `grep -rn '(:wat::telemetry::Metric \|(:wat::telemetry::Log '` is the incomplete
one it warned about: it counts a construction whose head sits on the matched line, and
misses the Log at `span.wat:89` (continuation) plus every fixture that breaks the ctor
across lines.

**My number: 41 existing kwargs constructors of `Metric` or `Log`.**

| where | count | what |
|---|---|---|
| `wat/telemetry/span.wat` | 4 | 1 Log + 3 Metrics. Producer. Each mints `v4` |
| tests (16 files) | 36 | hand-built records. Mechanical `:event-id (:wat::uuid::nil)` |
| `wat-scripts/scratch-pad/probe-sift-body-direct.wat` | 1 | same, because `every_wat_scripts_file_loads` type-checks it |

Plus two new fixtures that construct with explicit ids from the start (not in the 41).

The 36+1 were rewritten by a wat-fix (`wat-scripts/fixes/add-event-id-to-metric-log-ctors.wat`):
insert `:event-id (:wat::uuid::nil)` after the `:time-ns` value inside a Metric/Log ctor.
Idempotent — `span.wat` already had `:event-id`, re-run was a no-op. Dry-run on a `/tmp`
copy of `journal_query.wat` and `sift_arena.wat` before applying; `span.wat` copy
byte-identical.

`(:wat::telemetry::log span :Info "…")` is unchanged. Users never construct a Log.

## STOP-2 — why `hi=2` is the row that would have been silent

Existing query fixtures write at t=1s / t=2s and query `[0, 3s]`. The Instant prefix
already puts those rows inside a 3s window, so a *too-small* event-id on the max
sentinel would not drop them. The collision the census named is same-ns; the silent
failure is **same-ns at the window's hi bound, with a non-nil event-id**.

`probe_ex001_sortkey_boundary` writes at T:

- event-id `aaaa…` (name `:boundary`)
- event-id all-f (name `:maxed`)
- event-id nil at T+1 (name `:after`)

`query-metrics [T, T]` → 2. If `sort-key-hi` had used nil, the `aaaa…` row would have
sorted after the sentinel and vanished, and every older fixture would still have
passed. It did not vanish.

The all-f uuid is constructable: `from-string` accepts the canonical 8-4-4-4-12
lowercase form with no RFC4122 version-nibble check. Measured by `helper=1`
(`sort-key-hi(T)` byte-equals `write(SortKey T all-f)`).

## STOP-4 — goldens that grew a field, not a weakened assertion

`span_macros` still asserts 3. Two tagged-EDN goldens gained `:event-id` because the
record grew a required spliced field (nil, matching the fixtures that write them):

- `tests/services/probe_arc278_metric_edn_write__metric.edn`
- `tests/services/probe_arc278_journal__stored_metric.edn`

That is shape-churn from C, not "change 3 to 1". Named, not patched around.

`journal_service_logs.rs` checks fields individually and did not need an edit.
`capacity_derive` asserts Log floor `>= 56`; a second Uuid only raises the floor.

## The same-ns fixture, and a handle I almost dropped

First run of `same_ns_three_metrics_survive_on_mem_and_sqlite` reded:
`recv': peer closed` at `same-ns-roundtrip`. Cause: a `connect-journal` helper started
the journal, returned the Peer, and **dropped the Handle** — the service died before
`query-metrics`. Not a backend disagreement. Handle now lives in the same `let` as
the write and the query, matching the differential exemplar. Second run: `count=3;names=a,b,c`.

Pre-SortKey this sequence stored 1. That is the census's trap-door 1, now a gate.

## Blast radius

- `wat/telemetry.wat` — one field on `Scope`
- `wat/telemetry/span.wat` — four mint sites
- `wat/telemetry/journal.wat` — `SortKey`, `sort-key-lo` / `sort-key-hi`, `metric->row` / `log->row`; `time-sk` deleted; four query/sift scans use the sentinels
- `uuid-index-keys` — untouched
- wat-fix + 16 test fixtures + 1 scratch-pad (mechanical nil)
- 2 goldens (new field)
- `probe_ex001_journal_same_ns.{wat,rs}`
- `probe_ex001_sortkey_boundary.{wat,rs}`

`:wat::query::Store` untouched. User-facing `log` / `incr` / `timed` / `with-span` untouched.

Nothing in `src/` or `crates/`.

---

# ORCHESTRATOR GRADING — re-run, not read

```
Summary [ 297.380s] 5121 tests run: 5121 passed (2 slow), 17 skipped     FLOOR=0
PASS (3553/5121) probe_arc278_span_macros…      ← the control, untouched, now green
PASS (3567/5121) probe_ex001_journal_same_ns…   ← the same-ns gate
PASS (3568/5121) probe_ex001_sortkey_boundary…  ← the boundary gate
```

**FIRST FULLY GREEN FLOOR IN THIS EXCURSUS. STRUCK.**

## The control was not moved — it was satisfied

`git diff --stat` on `probe_arc278_span_macros.{rs,wat}` is **empty**. The assertion is still
`Value::i64(3)`. It goes green because the defect is gone, which is the only way a control is
allowed to change colour. STOP-4 held.

Row 4 verified in the diff: **three separate `(:wat::uuid::v4)` calls** for the three Metrics
of one `close`, not one shared. That sharing was the bug's origin; sharing an id would have
reproduced it exactly while looking fixed.

## STOP-2 — demonstrated in both directions, which is more than was asked

```
hi=2  wide=3  nil<=max=1  mid<=max=1  high<=max=1  next>max=1  helper=1
```

`hi=2` — a `[T, T]` query returns **both** rows at `T`, one carrying a non-nil event-id: the
exact row a too-small sentinel drops in silence. `next>max=1` — a key at `T+1` sorts *above*
the sentinel, so the bound does not leak forward either. **The brief asked for inclusion; the
fixture proves inclusion AND non-over-inclusion.**

`time-sk` → **0 occurrences** in `wat/`. `uuid-index-keys` → **0 diff lines** (STOP-3).
The two goldens are **pure additions**, zero deletions — shape-churn, not weakened assertions.

## ★ Three things the executor did that the BRIEF did not ask for

1. **It used the recorded codemod.** `wat-scripts/fixes/add-event-id-to-metric-log-ctors.wat`
   swept the 36 fixtures. That is CLAUDE.md doctrine — *".wat corpus migrations → the
   self-hosted codemod, NEVER hand-edits"* — **and my BRIEF never mentioned it.** A standing
   rule correctly applied over a brief that forgot it, leaving the migration recorded and
   idempotent instead of 36 hand edits.
2. **It split `time-sk` into `sort-key-lo` / `sort-key-hi`** (`journal.wat:35`, `:44`), because
   the bounds are now asymmetric — minimal sentinel low, maximal high. My sketch showed one
   record and left the bounds as prose. The two-helper shape is right and is not mine.
3. **It re-derived the census.** My BRIEF said 3 production sites and marked its own grep
   **known-incomplete**. The real number: **41 constructors** — 4 in `span.wat`, 36 fixtures,
   1 scratch-pad. I was wrong by an order of magnitude on the fixture side, and it was caught
   because the brief said re-derive rather than trust.

That is the third consecutive stone where the census-command habit did real work.

## What actually closed here

The chain, in order:

```
2c    mem's put becomes a replace        → the floor goes RED
                                           (the oracle stopped hiding a real defect)
INST  #inst at constant width            → time-sk's hand-padding becomes redundant
CENSUS 13/15 agree — but only because    → the corpus barely exercises the shape
       they never drive the collision       production emits constantly
SORTKEY an event carries its identity    → the floor goes GREEN
```

**Stone 2c did not break anything.** It removed a blindfold, and every stone since has been
paying the debt that exposed. The bug it revealed — `journal` silently dropping two metrics in
three from every span close, on every conforming backend — is now fixed, with two permanent
fixtures standing where there were none.

★ And the thing worth carrying: **the fix was cheap because the substrate was made honest
first.** `SortKey.time` is a real `:wat::time::Instant` rather than a hand-padded string
*because* stone INST fixed the renderer. Had we patched journal first, `time-sk` would still
exist, the key would still be a hand-built string, and the ordering property would still be a
local trick nothing asserted.

## Owed — and it is now short

Nothing blocking. `docs/excursus/README.md`'s residue note stands. The excursus's own findings
(the record-accessor receiver type, filed to arc 109) remain open in their home arc.
