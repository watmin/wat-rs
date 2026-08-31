# `wat-scripts/fanout/` — the circuit that composes wat-topic and wat-queue

N messages → 1 topic → M queues → J workers per queue → N×M outcomes.

`:user::compute` is the floor weight (`run 12 2 2`). `:user::main` is the
standalone weight (`run 2000 4 3`). Same wiring.

```bash
./target/release/wat --check wat-scripts/fanout/circuit.wat
./target/release/wat wat-scripts/fanout/circuit.wat
```
