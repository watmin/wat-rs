# BRIEF — STONE I: no EDN representation means tagged nil

Read `DESIGN-stone-I-no-repr-means-nil.md` first — it quotes the rulings and names the three calls
this reverses.

## The work

In `src/edn/render.rs`, route the three arms through `opaque_nil_or_refuse`:
`Value::Vector` (`~:4770`), `wat__kernel__HandlePool` (`~:4788`), `wat__stream__Stream` (`~:4882`,
EVERY state — `Empty`, `Cons`, `Thunk`, `NativeThunk`). Replace each arm's "preserve the data"
comment with one citing the 2026-09-24 ruling and arc 294's `BRIEF-294.i:15`.

## Before changing

- Measured by the orchestrator: zero goldens/tests pin `#wat.kernel/HandlePool`,
  `#wat.stream/Stream` or `#wat.holon/Vector`. Re-derive it.
- ⚠ Find anything that CONSUMES these bodies — a HandlePool's name, a Vector's `:dim`, a Stream's
  head read back out of EDN. Report each before removing its body.

## STOP triggers

1. A consumer relies on one of these bodies for behaviour (not just display) — report it.
2. Any gate reddens that you did not add — capture whole, name the arm. ⛔ Do not re-run first.

## Prove it

Extend the stone H probe: over a process wire, each of the four cases (Vector, HandlePool, forced
Stream, **empty Stream**) now REFUSES at the sender with stone G's message; a stream materialized to a
vector first crosses whole. Outside the wire, `:wat::edn::write` of each prints the tagged nil.
**Mutation:** restore the empty-Stream `()` arm — the empty case goes back to arriving as a List.

## Mechanics — ⛔ read

- Floor: `nohup scripts/floor.sh > <file> 2>&1 &`, then repeated foreground
  `until grep -qE '^ *Summary' <that file>; do sleep 30; done` blocks. Do not end your turn while it
  runs. Never `pgrep -f 'cargo …'`. Read `Summary` lines, never piped exit codes.
- `cargo fmt` reformats the whole workspace — `rustfmt <file>` or neither. Stage explicit paths.
