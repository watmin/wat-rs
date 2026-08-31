# `wat-scripts/fanout/` — the circuit that composes wat-topic and wat-queue

N messages → 1 topic → M queues → J workers per queue → N×M outcomes.

**Struck as STOP-5.** A process-tier worker cannot consume wat-queue: `:queue::Envelope`
is a top-level defrecord, not a member of `Queue`'s `:messages`, so a forked child
does not receive `Envelope/id` (or a decode of `ReceiveResponse`). See
`docs/excursus/2026/08/001-sns-sqs/SCORE-stone-4-fanout-circuit.md`.

`circuit.wat` is the wiring that got far enough to measure that. It load-file!s the
shipped topic and queue programs (`set-redef!` so this file's `:user::main` wins).
Running it prints the diagnostic summary (`total=0;empty=0` — messages reach the
queues, process workers cannot pull them).

```bash
./target/release/wat wat-scripts/fanout/circuit.wat
```
