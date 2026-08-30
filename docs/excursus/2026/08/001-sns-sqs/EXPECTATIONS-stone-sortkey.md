# EXPECTATIONS — excursus 001 stone SORTKEY

**Written BEFORE the strike, 2026-08-30.** Blast radius derived from the BRIEF's own section.

## ★ This is the first stone expected to produce a FULLY GREEN floor

Every prior stone here ran against a known red. **This one fixes it.** `FLOOR=0` is the target,
and a remaining `probe_arc278_span_macros` failure means the stone did not land.

## The scorecard

| # | what | expected |
|---|---|---|
| 1 | ★ the control, re-derived this session | `span_macros` returns **3 on mem AND 3 on sqlite** (it is `1`/`1` today) |
| 2 | the floor's known red goes green | `FLOOR=0` — no failures |
| 3 | `Scope` gains ONE field | `wat/telemetry.wat:74`; splices to both `Metric` and `Log` |
| 4 | the four span sites mint distinct ids | `span.wat:89` (Log) + `:118`/`:141`/`:144` (Metrics) — the three Metrics of one close must NOT share an id |
| 5 | `SortKey` declares `time` FIRST | field order is load-bearing; `edn::write` emits declaration order |
| 6 | `time-sk` is DELETED | `grep -c 'time-sk' wat/` → 0 |
| 7 | ★ the boundary holds | a row at exactly `time-hi` is RETURNED by `query-metrics` — demonstrated, not argued (STOP-2) |
| 8 | the max sentinel is really maximal | measured, not assumed |
| 9 | the new both-backends same-ns fixture exists and passes | three events at one `time-ns`, all three survive, mem == sqlite |
| 10 | `by-uuid` GSI unchanged | `uuid-index-keys` untouched (STOP-3) |
| 11 | no fixture's assertion weakened | `span_macros` still asserts 3 — it goes green because the bug is fixed (STOP-4) |
| 12 | prior stones undisturbed | `probe_ex001_*`, inst arms, write-opts arms all PASS |
| 13 | test count | 5119 + the new fixture's arms; any other delta needs explaining |

## Runtime prediction

**2–4 hours.** The largest stone in this excursus. `Scope` + four span sites + two `->row` fns
is mechanical, but **row 7 is real work** — the range bounds must be re-derived and
demonstrated, and every fixture that constructs a `Metric`/`Log` by hand needs the new field.

## Trap-doors

1. **★ Row 7 fails silently.** If the maximal sentinel is not actually maximal, a row at
   exactly `time-hi` is excluded — `query-metrics` drops the newest data and **every existing
   fixture still passes**, because none of them queries a boundary. A test that demonstrates
   the boundary is required; reasoning about lexicographic order is not sufficient. This is the
   same class as the `#inst` width bug: an ordering property nothing asserted.
2. **The three Metrics of one `close` share `now`.** That is the bug's origin. If they end up
   sharing an event id too, the fix does nothing and `span_macros` still returns 1. Row 4
   exists for this.
3. **`Log` has no `name`.** This is why the `(time, uuid, name)` shape from an earlier sketch
   was abandoned — it could never have worked for logs. If the implementation reaches for a
   content-derived discriminator, it has drifted back to the rejected design.
4. **Hydration reads rows back** (`journal.wat:262`, `:327` build `Log`s from stored rows). A
   new `Scope` field means the stored EDN changes shape — check the read path, not just the
   write path.
5. **The census in the BRIEF is known-incomplete** and says so. Do not treat its number as the
   answer; re-derive it and report yours.

## Not in this stone

- The `by-uuid` GSI / `uuid-index-keys` — the span correlation index, a separate concern.
- `:wat::query::Store` — untouched. This is a `journal` fix.
- Anything in excursus 001's earlier stones.
