# HANDOFF → grok — stone INST: `#inst` renders at constant nanosecond width

Same branch, `sns-sqs`. Read in full:

- `docs/excursus/2026/08/001-sns-sqs/BRIEF-stone-inst-renders-at-constant-width.md`
- `docs/excursus/2026/08/001-sns-sqs/EXPECTATIONS-stone-inst-renders-at-constant-width.md`

**One token in the substrate.** `crates/wat-edn/src/writer.rs:227` uses
`SecondsFormat::AutoSi` — chrono's "shortest representation that is a multiple of 3 digits" —
so `1.200000000s` prints `.200Z` and `1.200000100s` prints `.200000100Z`. `'Z'` (0x5A) sorts
after `'0'` (0x30), so **the earlier instant compares greater**, and every range `scan` over a
timestamp sort key is unsound. `SecondsFormat::Nanos` always emits 9.

`crates/wat-edn/src/json.rs:170` has the same call. **That one is a DECISION, not a
copy-paste** — EDN's `#inst` is a sort key in this system, JSON's is an interchange value.
Either answer is fine; an unstated one is not. Say what you did in the SCORE.

**The gate** is `PROBE-inst-lexicographic-order-is-not-chronological.wat`, already committed.
At HEAD its comparisons give
`9-digit=false whole-second=false 6-digit=false 3-digit=false control=true widths=32/38/28`.
Done when all six deftests pass **with no edit to the probe**, and it is promoted into
`wat-tests/` so the property stays on the floor.

⚠ **THE FLOOR IS ALREADY RED AND THAT RED IS NOT YOURS.**
`probe_arc278_span_macros::with_span_and_timed_emit_the_aggregated_metrics_on_close` fails at
HEAD — the journal key-collision bug stone 2c exposed. **Expected: exactly ONE failure, that
one.** Two means you added one. Do not fix the span arm; it is drawn separately.

Measured golden churn is **zero** — that is a claim in the BRIEF, and STOP-1 says if a golden
does churn, report it, because a wrong census matters more than the golden.

Verify in the FOREGROUND; read the Summary line, never a piped exit code. On a NEW red: do NOT
re-run, capture the arm whole, name the exact assertion.
