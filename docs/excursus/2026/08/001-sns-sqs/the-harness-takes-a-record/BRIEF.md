# BRIEF — the harness takes a record

**Read `DESIGN.md` beside this first.** It carries the one contract decision (`None` = default,
`(Some 0)` = zero), the four overloaded-zero sites this exists to fix, and five trap-doors — including
why the SCOREs' invocation strings must NOT be rewritten.

⛔ **Sequenced after `the-topic-can-be-slow`.** If that stone has not landed, its two new knobs
(`delay-bp`, `delay-ms`) are not yet in the tree; include them if they are, and say which you saw.

## The work, in one paragraph

Replace `circuit.wat`'s 15-slot positional argv with one EDN record read from stdin. Declare
`:fanout::Input` with required fields for what a run cannot mean without (`n`, `m`, `j`, `sub-cap`,
`fill-first?`) and `(Option i64)` for every knob that today has a default. `main` reads it with
`(:wat::kernel::readln)` and passes accessors to `run-with`. The point is not brevity: it is that
`None` and `(Some 0)` become different, so `vis-ms`, `inbox-vis-ms` and `inbox-cap` stop having an
unreachable value.

## The rooms

| where | why you are going there |
|---|---|
| `wat-scripts/fixes/angle-brackets-to-binder.wat:282`–`:300` | ⭑ THE PROVEN HARNESS — *"identical shape to every recorded migration"*. `(:wat::kernel::readln)` with all three `ReadlnOutcome` arms faced. Copy it; it reads a vector, you read a record. |
| `wat-scripts/fanout/circuit.wat:3635` | the usage string — it becomes the record's shape, printed. |
| `wat-scripts/fanout/circuit.wat:3650`–`:3700` | the argv→`run-with` wiring you are replacing. Read every `opt-i64` and `Option/expect` here: each encodes a default you must carry into the record. |
| `wat-scripts/fanout/circuit.wat:2920`, `:2929`, `:3666`, `:3699` | ⛔ THE FOUR OVERLOADED ZEROS. Read each comment before converting it — trap-door 1. `:3666` is the sharpest: *"0 means the default, not 'no redelivery'"*. |
| `wat-scripts/fanout/circuit.wat:2877`–`:2893` | `run-with`'s **18** parameters. Whether it keeps them or takes the record is yours; say which and why. |
| `tests/services/probe_ex001_fanout.rs` · `probe_chaos_gate_has_teeth.rs` · `probe_arc278_sane_circuit.rs` · `probe_async_publish.rs` | the four live callers. Each must pipe the record. |
| `scripts/capped.sh` | verify whether it is arg-agnostic; it wraps, it may not care. |
| `wat-scripts/scratch-pad/probe-a-slow-peer-desyncs-the-next-call.wat:80` | an `(:wat::edn::read "#ns/Rec {…}")` in anger, for the record-literal spelling. |

## Implementation sketch

```wat
(:wat::core::defrecord :fanout::Input
  [n <- :wat::core::i64  m <- :wat::core::i64  j <- :wat::core::i64
   sub-cap <- :wat::core::i64  fill-first? <- :wat::core::bool
   vis-ms <- (:wat::core::Option :- [:wat::core::i64])
   …one field per today's argv slot, Option iff it has a default…])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [inp (:wat::core::match (:wat::kernel::readln)
           ((:wat::kernel::ReadlnOutcome::Datum d) d)
           (:wat::kernel::ReadlnOutcome::Eof    <decide deliberately — STOP-3>)
           (:wat::kernel::ReadlnOutcome::Stopped <decide deliberately>))]
    …(:fanout::run-with (:fanout::Input/n inp) …)…))
```

```
printf '#fanout/Input {:n 2000 :m 4 :j 3 :sub-cap 8192 :fill-first? true :vis-ms 1000}\n' \
  | ./target/release/wat wat-scripts/fanout/circuit.wat
```

## Blast radius

`circuit.wat` + 4 `.rs` test files + possibly `scripts/capped.sh`. **`docs/**` is 0** — see STOP-5.

## STOP triggers

1. **STOP-1 — if any of the four overloaded zeros cannot be converted without guessing the author's
   intent**, STOP and report which. A blanket `0 → None` is the collapse-failures defect wearing a new
   costume; each of the four has a comment stating what zero meant there.
2. **STOP-2 — if converting a caller changes a REPORTED COUNTER**, STOP. A green test measuring
   something different is the worst outcome here. Diff the counters, not just pass/fail.
3. **STOP-3 — decide `Eof` and `Stopped` deliberately and say what you chose.** The migrations raise on
   both, which is right for a codemod invoked by a human. ⚠ `readln` blocks: a caller that forgets
   stdin **hangs**, and the floor's terminate wall is 30 s.
4. **STOP-4 — if `run-with`'s 18 parameters cannot take the record directly**, say so rather than
   building a second unpacking layer. Two shapes for one input is the thing this stone removes.
5. **STOP-5 — do NOT rewrite invocation strings in `docs/**`.** They are evidence of what was run at
   the time. Editing them to match a later interface is map-drift, and `curare` forbids it explicitly.

## Verify

- `cargo build --release` **and** `cargo nextest run --release --no-run`.
- `./scripts/floor.sh`, read the **Summary line**, never a piped exit code.
- ⭑ **The equivalence proof (the real gate):** the record form and today's positional form must produce
  **identical reported counters**, not merely both-green. Run the happy path both ways if the old path
  still exists at that moment; otherwise diff against the baseline below.
- ⭑ **The defect is fixed:** `(Some 0)` for `vis-ms` must reach the service as **0**, not as the
  default. That value is unreachable today and is the reason this stone exists — demonstrate it.
- ⭑ **A negative control:** a record missing a required field must be **refused**, not silently
  defaulted. If a missing `:n` runs with some fallback, the type is not doing the work.

## Shape to copy

`wat-scripts/fixes/angle-brackets-to-binder.wat` for the stdin harness, and
`docs/excursus/2026/08/001-sns-sqs/vis-is-swept-not-chosen/` for what a swept knob's call sites look
like — that stone swept `vis` and could not have expressed 0.
