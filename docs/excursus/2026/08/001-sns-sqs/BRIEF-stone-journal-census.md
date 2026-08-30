# BRIEF — excursus 001 stone JOURNAL-CENSUS: run every journal fixture against sqlite

**This stone changes NO production code and fixes NOTHING. It measures.** Its output is a
table, and a RED result is the deliverable.

## Why it comes before the `SortKey` fix

`journal`'s base key is `(namespace + kind, time-ns)` — nothing in it identifies *which*
metric. Stone 2c's oracle fix exposed one consequence:
`probe_arc278_span_macros` asserts a span's close emits 3 Metrics, and on `sqlite-store` it
gets **1** (`NOTE-journal-loses-metrics-on-sqlite-because-sk-is-time-only.md`).

**That is one fixture. Fourteen more have never run against the real backend.**

Census, command shown so its scope is auditable:

```
grep -rlE ':wat::telemetry::journal/start|:wat::telemetry::Journal/' tests/ wat-tests/ --include=*.wat
  → 17 real callers
  → 2 touch sqlite-store   (journal_backend_differential, journal_service_sqlite_on_process)
  → 15 are MEM-ONLY
```

The `SortKey` fix changes a **stored key layout**. Doing that with one fixture's worth of
evidence about the blast radius is guessing. This stone buys the number first.

## The work

For each of the 15 mem-only fixtures: run its scenario against `sqlite-store(":memory:")` as
well as `mem-store`, and report whether the two agree.

**Do not fix anything.** A fixture that disagrees is a finding, not a chore. A fixture that
cannot be run against sqlite for a structural reason is also a finding — say which and why.

## The exemplar — copy it, do not invent

**`tests/services/probe_arc278_journal_backend_differential.wat`** already does exactly this
swap for one path. Its shape:

- a helper parameterized on `store-addr <- (:wat::kernel::Address :- [Store::Op Store::Reply])`
- `mem-store/start` and
  `sqlite-store/start :record (… :path ":memory:" :index-names ["by-uuid"])`
- run both, compare, return a sentinel string on mismatch that the `.rs` catches

`journal`'s `:init` calls `ensure-schema` declaring the `by-uuid` GSI, so `:index-names` must
carry `"by-uuid"` — that is where the differential gets it.

## The 15

```
tests/services/probe_arc278_journal_logs_on_process.wat
tests/services/probe_arc278_journal_query_logs.wat
tests/services/probe_arc278_journal_query_metrics_on_process.wat
tests/services/probe_arc278_journal_query.wat
tests/services/probe_arc278_journal_service_logs.wat
tests/services/probe_arc278_journal_service_on_process.wat
tests/services/probe_arc278_journal_surface.wat
tests/services/probe_arc278_log_captures_call_line.wat
tests/services/probe_arc278_sift_arena.wat
tests/services/probe_arc278_sift_logs.wat
tests/services/probe_arc278_sift_rules_arena.wat
tests/services/probe_arc278_sift_rules.wat
tests/services/probe_arc278_span_macros.wat      ← KNOWN: 3 on mem, 1 on sqlite
tests/services/probe_arc278_span_nested.wat
tests/services/probe_arc278_span_service.wat
```

`span_macros` is the control: it **must** show the disagreement. If your instrument says it
agrees, the instrument is wrong — that is STOP-1.

## The deliverable

A markdown table in the SCORE, one row per fixture:

| fixture | mem | sqlite | agree? | if not — what differs |
|---|---|---|---|---|

Plus a one-line verdict: **how many of 15 lose data on the real backend.**

## How to run them without disturbing the floor

These are `.wat` fixtures driven by `.rs` harnesses. **Do not edit the fixtures in place** —
that would change what the floor asserts, and the floor's one known red is load-bearing
evidence. Work on copies (the session scratchpad is correct for non-`.wat` output; for `.wat`,
`wat-scripts/scratch-pad/` per CLAUDE.md — but see STOP-3, these are throwaway measurement
copies, not programs to keep).

If a cleaner instrument exists — a harness that parameterizes the backend without copying —
prefer it and say so.

## STOP triggers

1. **If `span_macros` shows AGREEMENT — STOP.** It is the control; it is known to disagree
   (3 vs 1, measured 2026-08-30). Agreement means the instrument is not actually swapping the
   backend, and every other row is worthless.
2. **If a fixture cannot run against sqlite structurally** (needs a real path, a fork the
   in-memory store cannot serve, etc.) — that is a **finding**, not a skip. Name the fixture
   and the reason.
3. **If you find yourself fixing `journal`, `metric->row`, `log->row`, or a fixture's
   assertion — STOP.** That is the next stone and it is drawn from THIS one's numbers.
4. **If the floor changes at all — STOP.** This stone should not touch it. Floor is **5119
   with ONE known failure**; the same one, before and after.

## Blast radius

**Zero production files. Zero committed test files.** The deliverable is a SCORE document plus
whatever throwaway measurement copies you needed, which are deleted before the commit.

If you conclude a permanent both-backends fixture is worth keeping, **say so and stop** — that
is a real addition and it belongs to the next stone.

## Verify

```bash
./scripts/floor.sh; echo "FLOOR=$?"
```

Unchanged: 5119, one known failure. If this stone moved the floor, it did something it should
not have.
