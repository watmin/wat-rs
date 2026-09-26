# SCORE — STONE 255.43: the telemetry stdlib faces the outcomes it swallows

Struck against draw `dae176769`. `journal'` `:init` matches a failed
`ensure-schema` and fails start-up with that reason. `span.wat` is
unchanged: `Span::LogResponse` has no arm that can carry a journal write
failure.

## Span is a stop

`Span::LogResponse` (`wat/telemetry.wat` 257–260) is `:Ok`,
`:RequestTooLarge`, and `:RequestMalformed`. The last two are a wire
breach of the span's own request. `Span::CloseResponse` (263–269) already
passes the sink outcome through (`:Done`, `:Constraint`, `:Transient`,
`:Fatal`, plus the two wire-breach variants). `Journal::WriteLogsResponse`
mirrors `Store::PutResponse` (`Success`, `Constraint`, `Transient`,
`Fatal`, `RequestTooLarge`, `RequestMalformed`), wrapped in `RecvOutcome`.

A failed journal write has no honest `LogResponse` arm. Mapping `Fatal`
onto `RequestMalformed` would call a store failure a malformed request.
STOP 1: a new response variant is a vocabulary ruling. `wat/telemetry/span.wat`
line 98 still binds `_w` and line 100 still replies `LogResponse.Ok`.

## Journal start

`EnsureSchemaResponse` (`wat/query.wat` 543–548) is `Success`,
`Constraint`, `Fatal`, `RequestTooLarge`, `RequestMalformed`. `:init`
already fails start on every `ConnectOutcome` arm except `Connected`, via
`assertion-failed!`. Arc 278 startup-crash parity is that same path: `/start`
re-raises the assertion to the owner as `AssertionPayload`.

`wat/telemetry/journal.wat` 98–129 is the `ensure-schema` match, in place
of the `_es` binding. `Success` returns `journal::State`. Every other arm
`assertion-failed!`s. The `Fatal` message is `journal ensure-schema fatal: `
concatenated with `Fault/message` of `Fatal/reason`.

The probe is a `Store` satisfier that replies `EnsureSchemaResponse.Fatal`
with fault message `SCHEMA-SETUP-FAILED`. The store handle is bound in an
outer `let`. A tail-position `journal/start` drops that handle first, and
connect then fails with the listener-dropped sentence before `ensure-schema`
runs.

## The row

Pre-stone, with the nested let and `_es` still swallowing the outcome, the
same test panicked:

```
thread 'probe_arc255_43_journal_ensure_schema::a_failed_ensure_schema_fails_journal_start' (515122) panicked at /home/john/work/holon/wat-rs/tests/services/probe_arc255_43_journal_ensure_schema.rs:25:26:
journal start swallowed the schema failure and returned Aggregate(AggregateValue { class: "wat::telemetry::journal::Handle", names: ["handle", "addr"], fields: [RustOpaque(RustOpaqueInner { type_path: ":wat::kernel::Thread" }), RustOpaque(RustOpaqueInner { type_path: ":wat::kernel::Address" })], nature: Struct, holon: Empty })
```

After the match, the test passed. `AssertionPayload.message` is exactly
`journal ensure-schema fatal: SCHEMA-SETUP-FAILED`. The panic hook prints
that assertion before `catch_unwind`; the test exit is 0.

## Named-underscore census

| site | what the binding holds | disposition |
|---|---|---|
| `wat/telemetry/journal.wat` (was ~95, `_es`) | `RecvOutcome` of `ensure-schema` | matched; start fails |
| `wat/telemetry/span.wat` 98, `_w` | `RecvOutcome` of `write-logs` | STOP; `LogResponse` cannot say it |
| `wat/kernel/services/stdio.wat` 52 and 78, `_bytes` | `IOWriter/write-string` → `:i64` byte count | left; a count, and the service replies `WriteResponse.Ok` |
| `wat/rete/compile.wat` 838, `_fact-ty` | `field-names-of`; raises unless the head is a record | left; the raise is the fence |
| `wat/rete/compile.wat` 1120, `_rhs-fence` | `foldl` of `then-item-fence` from 0 | left; the raise is the fence |

`stdio-write-out` / `stdio-write-err` (`_ack`, lines 213 and 242) already
match every `RecvOutcome` and `WriteResponse` arm and raise on failure.
Those two were outside the F5 six. The discard gate is unchanged.

## Gates

Clippy `--all-targets --workspace -- -D warnings` exited 0.

Census `.census/2026-09-26T00-11-32Z.txt` against
`.census/2026-09-26T00-01-31Z.txt`: `census-diff: no STOP-8`. 0 rc flips
either way. 2281 files, 215 nonzero.

Delta `.delta/2026-09-26T00-12-30Z`: NEW 2 / RECOVERY 0. The two NEW files
are `wat-scripts/probes/arc-170/probe-c1-clean-surface.wat` and
`wat/holon/Ngram.wat`.

Ledger 198. `the_heresy_ledger_matches_its_frozen_census` passed.

`.floor/2026-09-26T00-13-57Z`: `6137 tests run: 6137 passed (10 slow), 22 skipped`.
