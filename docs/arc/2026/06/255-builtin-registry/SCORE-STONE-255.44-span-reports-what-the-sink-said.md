# SCORE — STONE 255.44: the span reports what the sink said

Struck against draw `624ab533b`. `Span::LogResponse` mirrors
`CloseResponse`. `span'` matches the journal write and replies that arm.

## The vocabulary

`wat/telemetry.wat` 258–264. `LogResponse` is `:Ok`, `:Constraint [err]`,
`:Transient [err]`, `:Fatal [err]`, `:RequestTooLarge`, `:RequestMalformed`.
The three failure arms use the same field name and type as
`CloseResponse` (267–272). `:Ok` stays where `CloseResponse` has `:Done`.

## The write

`wat/telemetry/span.wat` 100–131. `_w` is gone. The match is the close
match with `WriteLogsResponse` and `LogResponse`:

| journal | span replies |
|---|---|
| `Success` | `Ok` |
| `Constraint` / `Transient` / `Fatal` | the same arm, carrying `err` |
| `RequestTooLarge` / `RequestMalformed` | the same arm, carrying the wire fields |
| `RecvOutcome.Lost` / `Stopped` / `Closed` | `Fatal`, with the sentences close already uses |

Close had a precedent for every arm. No stop.

## The rows

A `Store` whose `put` is `Success`, `Fatal`, `Constraint`, or `Transient`.
`journal'` maps that onto `WriteLogsResponse`. `Span/log` returns the
`LogResponse`. The fault message is `SPAN-SINK-FATAL`,
`SPAN-SINK-CONSTRAINT`, or `SPAN-SINK-TRANSIENT`.

Pre-stone, with `_w` still dropping the write, the three failure rows
failed and the success row passed:

```
assertion `left == right` failed
  left: "Ok"
 right: "Fatal"
```

```
assertion `left == right` failed
  left: "Ok"
 right: "Constraint"
```

```
assertion `left == right` failed
  left: "Ok"
 right: "Transient"
```

After the match, all four rows passed. The failure rows carry that fault
message on `err`.

## Consumers

| site | what it does |
|---|---|
| `wat/telemetry/span.wat` `log` | the producer. Matches the write and replies the arm |
| `tests/services/probe_arc278_span_surface.wat` | toy satisfier. Still replies `LogResponse.Ok`. Does not match the enum |
| `tests/services/probe_arc278_log_captures_call_line.wat` | matches `RecvOutcome` and binds `_resp`. The payload is unread. The probe checks the captured call site |
| `:wat::telemetry::log` | the macro returns the `Span/log` call. The caller receives the `RecvOutcome` |

No exhaustive `LogResponse` match existed outside the new producer, so no
consumer body needed a new variant.

## Gates

Clippy `--all-targets --workspace -- -D warnings` exited 0.

Census `.census/2026-09-26T00-48-51Z.txt` against
`.census/2026-09-26T00-46-16Z.txt`: `census-diff: no STOP-8`. 0 rc flips
either way. 2282 files, 215 nonzero.

Delta `.delta/2026-09-26T00-49-41Z`: NEW 2 / RECOVERY 0. The two NEW files
are `wat-scripts/probes/arc-170/probe-c1-clean-surface.wat` and
`wat/holon/Ngram.wat`.

Ledger 198. `the_heresy_ledger_matches_its_frozen_census` passed.

`.floor/2026-09-26T00-51-04Z`: `6141 tests run: 6141 passed (10 slow), 22 skipped`.
