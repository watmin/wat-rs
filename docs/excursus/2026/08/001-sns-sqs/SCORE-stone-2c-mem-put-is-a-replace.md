# SCORE — excursus 001 stone 2c: mem's `put` becomes a replace

**STRUCK on (a)(b)(c). STOP-1 FIRED. Floor is RED — not re-run.** Executor: grok, 2026-08-30.

```
Summary [ 300.319s] 5097 tests run: 5096 passed (3 slow), 1 failed, 17 skipped
FAIL [   0.682s] (3531/5097) wat::services probe_arc278_span_macros::with_span_and_timed_emit_the_aggregated_metrics_on_close
FLOOR=100
```

ARM captured whole at `.floor/2026-08-30T11-37-18Z/ARM.txt`. Do not re-run this stamp.

The agreed re-put summary, produced identically by `mem-store` and `sqlite-store(:memory:)`:

```
base=1:a;gsi=1:v9
```

Pre-fix: `MEM[base=2:a,a;gsi=2:v1,v9]  SQLITE[base=1:a;gsi=1:v9]`.

## The scorecard

| # | what | expected | **measured** |
|---|---|---|---|
| 1 | re-put differential exists and runs | present, PASS | ✅ `PASS [0.701s] (457/5097) wat::rete probe_ex001_reput_differential::reput_differential_mem_and_sqlite_agree` — same red floor |
| 2 | both backends agree on re-put | `base=1:a;gsi=1:v9` | ✅ pinned `AGREED_SUMMARY`; mismatch would be `DIFFERENTIAL-MISMATCH mem=… sqlite=…` |
| 3 | mem drops the superseded row | reuse `key-hits-row?` | ✅ reused, not rewritten. `put` inner-fold calls the stone-2 predicate; `row-in-delete-batch?` untouched |
| 4 | the GSI follows | `gsi=1:v9`, not `v1,v9` | ✅ `gsi=1:v9`. Mem derives projections from surviving rows; the old `v1` dies with the dropped StoredRow |
| 5 | blast radius | mem + query comment + DESIGN:150 + two tests + SCORE; sqlite untouched | ✅ porcelain: `M wat/query/mem.wat`, `M wat/query.wat`, `M DESIGN-store-contract.md`, `?? tests/rete/probe_ex001_reput_differential.{rs,wat}`, this SCORE. `git diff -- wat/query/sqlite-store.wat` empty |
| 6 | `put`'s signature unchanged | comment lines only | ✅ `wat/query.wat \| 7 +++++++` — seven comment lines, zero type/arity change |
| 7 | the surface states the rule | `put`'s doc-comment names replace-by-`(pk,sk)` | ✅ `wat/query.wat:576–581` — REPLACE-BY-(pk,sk) / PutItem / later-wins / sqlite DELETE+clear+INSERT vs mem drop-StoredRow |
| 8 | the design table is corrected | `:150` no longer plain INSERT | ✅ `replace-by-(pk,sk): DELETE+INSERT in BEGIN/COMMIT (PutItem; a duplicate key is unrepresentable)` / mysql `REPLACE` / mongo `replaceOne` upsert |
| 9 | floor | `FLOOR=0` | ❌ `FLOOR=100` — STOP-1. See below. Did not re-run |
| 10 | test count | 5096 → **5097** | ✅ 5097 started = 5096 + the reput arm |
| 11 | stones 2 and 2b undisturbed | still PASS | ✅ same floor: `PASS (456/5097) delete_differential_mem_and_sqlite_agree`; `PASS (458/5097) store_delete_removes_exactly_the_named_row` |

## STOP-1 — the named test, the ARM, the mechanism

**The test:** `wat::services probe_arc278_span_macros::with_span_and_timed_emit_the_aggregated_metrics_on_close`

**The assertion, verbatim** (`tests/services/probe_arc278_span_macros.rs:21`):

```
expected with-span's close to emit 3 Metrics (1 aggregated :requests counter + :fetch/count + :fetch/duration); got i64(1) (a count != 3 means incr fanned out, timed didn't record, or close didn't fire)
```

**The ARM is at** `.floor/2026-08-30T11-37-18Z/ARM.txt`. Not re-run.

**Why it reds.** The fixture (`tests/services/probe_arc278_span_macros.wat`) scans mem-store at `pk = PartitionKey {namespace "probe-ns" kind Metric}`, `sk-lo "#" … sk-hi "#z"`, and returns `count rows`. It expects 3 because with-span close emits three Metrics in one `Journal/write-metrics` → one `Store/put`.

Journal keys (`wat/telemetry/journal.wat:36–46`, `metric->row`):

- `pk` = EDN of `(PartitionKey namespace kind=Metric)` — same for every metric in the namespace
- `sk` = `time-sk(Metric/time-ns)` — **time only**. No metric name, no uuid in the base sort key
- `write-metrics` (`journal.wat:102–113`) maps the whole batch to rows and puts them in **one** `Store/put`

Three Metrics emitted in one close can share a nanosecond → three incoming rows with one `(pk,sk)` → last-wins replace → **one stored row**. Append used to keep three unrepresentable duplicates; the test counted that.

This is exactly trap-door 1 / STOP-1: a test asserting a state DynamoDB cannot represent. The fix is probably the test (or the journal key — `sk` uniqueness is time-only, which is the deeper contract hole). **Not patched as part of 2c.** Brief: capture, name, report.

Sibling span arms in the same floor still PASS:

- `probe_arc278_span_nested::nested_with_span_emits_into_each_namespace_independently`
- `probe_arc278_span_service::span_accumulates_a_counter_and_emits_it_as_a_metric_on_close`
- `probe_arc278_span_surface::span_surface_freezes_and_all_four_ops_reply`

So it is this one fixture's row-count-of-a-same-ns-batch, not span close itself.

## (a) the fold

One outer fold over incoming rows, starting from existing `Record/rows`. For each incoming `r`:

1. build `Key` from `StoredRow/pk` + `StoredRow/sk`
2. inner-fold drop via **`key-hits-row?`** (stone 2's delete predicate — not a second one)
3. `conj` the incoming row onto what remains

Last-wins in-batch is automatic: a later duplicate drops the earlier conj'd row then conj's itself. sqlite's `put-rows` recursion already last-wins; STOP-2 did not fire.

Intra-batch extra cost is O(batch²) on top of O(rows × batch). Finding 3 (O(rows × keys)) is still open and not this stone's. The N term is not worse.

Header comment no longer says "`put` conj's the batch".

## (b) the surface

`put`'s doc-comment was "write a batch ATOMICALLY". The governing model (DynamoDB PutItem) lived as a prose aside on `wat/query.wat:7`. That is how this was misread once already. Same edit corrected `DESIGN-store-contract.md:150`, which mapped `put` to a plain INSERT that would error against `PRIMARY KEY(pk,sk)` rather than replace.

## (c) the differential

Promoted `docs/excursus/2026/08/001-sns-sqs/PROBE-reput-divergence.wat` → `tests/rete/probe_ex001_reput_differential.{wat,rs}`. Standalone `:user::main` dropped. Unused delete helpers from the 2b-copy dropped. `compute` returns the shared summary IFF equal, else `DIFFERENTIAL-MISMATCH mem=… sqlite=…`.

## STOP triggers 2–4

- **STOP-2** (batch last-wins vs sqlite): did not fire. Sequential replace matches sqlite's `put-rows` recursion.
- **STOP-3** (`sqlite-store.wat`): did not fire. Empty diff.
- **STOP-4** (shape worse than O(rows × batch)): did not fire. Inner fold is O(acc) per incoming row; acc starts at existing rows and stays ≈ that size. Same N term as delete. Finding 3 still open.

## Not done here (per brief)

- The span_macros test is **not** patched. Naming it is the finding.
- Journal `sk = time-sk(time-ns)` uniqueness is a contract hole **downstream of 2c**, not 2c's to fix.
- Finding 3 (O(rows × keys)) still open.
- `wat/queue.wat` is stone 3, and it starts after this reports.

## Porcelain at report time

```
 M docs/arc/2026/06/278-rules-engine/DESIGN-store-contract.md
 M wat/query.wat
 M wat/query/mem.wat
?? tests/rete/probe_ex001_reput_differential.rs
?? tests/rete/probe_ex001_reput_differential.wat
?? docs/excursus/2026/08/001-sns-sqs/SCORE-stone-2c-mem-put-is-a-replace.md
```

Uncommitted. Not pushed. sqlite-store empty.

---

# ORCHESTRATOR GRADING — re-run, not read

Everything above is the executor's report. Everything below I measured myself.

## Rows confirmed on my own floor

```
Summary [ 295.091s] 5097 tests run: 5096 passed (2 slow), 1 failed, 17 skipped     FLOOR=100
FAIL (3534/5097) wat::services probe_arc278_span_macros::with_span_and_timed_emit_the_aggregated_metrics_on_close
PASS (456/5097) delete_differential…  PASS (457/5097) reput_differential…  PASS (460/5097) store_delete…
```

**Same single arm, same assertion.** Independently reproduced, so the red is real and not a
stamp artifact. 5096 → 5097, exactly +1. Stones 2 and 2b undisturbed.

(A second floor run here is not a forbidden re-run: the executor's evidence is captured whole
at `.floor/2026-08-30T11-37-18Z/ARM.txt`, and a confirming measurement destroys nothing. The
rule exists to stop a green re-run erasing the only record — not to stop verification.)

## ⛔ THE FINDING IS BIGGER THAN THE REPORT SAYS

The executor called the failing test *"exactly 'asserting a state DynamoDB cannot represent'"*
and left it. **That framing is too generous to the fix**, and I only caught it by asking which
backend that fixture runs on. It runs on `mem-store`. So I ran the identical scenario against
sqlite, changing nothing else:

```
mem-store    → 3      ← what the test asserts
sqlite-store → 1      ← two metrics silently gone
```

**`journal` loses metrics on the production backend, and has since before this arc.**
`put-one-row` is untouched by every commit here (stone 2's 54 lines are pure additions; 2c did
not open the file).

The mechanism, shown as data — three metrics from one span's `close`:

```
:requests        pk=#wat.telemetry/PartitionKey{:namespace "probe-ns" :kind Kind/Metric}  sk=#inst "…01.234567890Z"
:fetch/count     pk=  ⟨identical⟩                                                          sk=  ⟨identical⟩
:fetch/duration  pk=  ⟨identical⟩                                                          sk=  ⟨identical⟩
```

The metric's **name** — the only thing distinguishing them — is in neither key. The `(pk, sk)`
is modelled at the wrong granularity: it identifies *a namespace at an instant*, but the item
is *a metric*, and one namespace emits many metrics at one instant. That is the normal case for
a span, not an edge case.

**So the test's assertion is CORRECT and the key is wrong.** 2c did not cause a regression — it
removed a blindfold. Full write-up:
`NOTE-journal-loses-metrics-on-sqlite-because-sk-is-time-only.md`.

★ This is the concrete value of fixing an oracle: the oracle was wrong *in a direction that made
a real defect invisible*, and what it hid was data loss in the telemetry path on the only
backend anyone would ship.

## Verdict

**2c is STRUCK.** (a)(b)(c) all confirmed; blast radius held with `sqlite-store.wat` untouched;
`key-hits-row?` reused rather than rewritten; the surface now states the rule; STOP-1 fired
exactly as drawn and was handled correctly — captured, named, not re-run, not patched past.

The remaining red is **expected and correct**: it is the substrate reporting a bug it previously
could not see.

## Owed, in order

1. **Stone INST** — `#inst` renders at constant nanosecond width. Drawn; it is upstream of the
   journal key and deletes the need for `time-sk`'s hand-padding.
2. **The journal `SortKey`** — downstream of INST, drawn separately.
3. **The both-backends census** over every `journal` fixture. `probe_arc278_span_macros` is one
   of many that only ever see `mem-store`. One red fixture is our entire evidence for how far
   this reaches, and reading them is not the instrument — running them on sqlite is.
