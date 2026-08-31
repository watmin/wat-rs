# HANDOFF → grok — excursus 001 stone 4: the fan-out circuit

Branch `sns-sqs`. Read in full:

- `docs/excursus/2026/08/001-sns-sqs/BRIEF-stone-4-fanout-circuit.md`
- `docs/excursus/2026/08/001-sns-sqs/EXPECTATIONS-stone-4-fanout-circuit.md`
- `docs/CIRCUIT.md` — the wiring axiom this app must obey

**Build the app that proves wat-topic and wat-queue compose.** N messages → 1 topic → M queues
→ J workers per queue → N×M outcomes, processed in parallel across real processes.

★ **THE TOPOLOGY IS THE SAFETY ARGUMENT.** `receive` is `scan-index` then `put` — two Store
calls. What makes that safe is not the put; it is that **a defservice is a serializing actor**
(`wat/query/mem.wat:22-24`). So: **ONE queue service instance per queue, J workers dialing it.**
J queue services over one store would each serialize internally and not against each other,
which reintroduces the race. STOP-1.

★ **PARALLELISM IS PROVEN BY IDS, NOT BY A CLOCK.** `mora`: *"sleep is a guess; guesses race."*
Give each worker an id at spawn; assert all M×J ids appear in the outcomes. `:locus
(:wat::spawn::process)` already guarantees a distinct process each, so ids-did-work is
processes-did-work. Do not reach for a self-pid — `:wat::kernel::peer-pid` answers *the pid at
the other end*, and self-pid is unverified and unneeded.

★ **IF YOU OBSERVE A DUPLICATE DELIVERY, STOP AND REPORT IT.** Do not fix it, retry past it, or
dedupe it away. **That is the finding this stone exists to produce** — the actor's serialization
under load is currently a claim derived from reading a comment, and this is the first thing
that measures it. Report the count and the concurrency it appeared at.

⚠ **But distinguish a duplicate from a legitimate redelivery.** A worker slower than the
visibility window will correctly see its message again. If the summary cannot tell those apart,
its duplicate count means nothing. That is the subtlest thing here.

**Weight:** full standalone (host measured at 12 cores / 28 GB free — target N=2000, M=4, J=3,
8000 outcomes), and a **scaled** version on the floor. The floor kills at 30s by default;
precedent for the split is `wat-scripts/perf/grid/` + `wat_scripts_grid_axes_live`. If the
scaled version will not fit, propose an override — do not add one silently.

⛔ **Placement:** `wat-scripts/fanout/` is a suggestion. Say so in the SCORE if you disagree —
and **do not move `wat-scripts/topic/` or `wat-scripts/queue/`.** Zero changes to `wat/`,
`src/`, `crates/`. If wat-topic or wat-queue needs a change, that is a finding: name it and stop.

Verify in the FOREGROUND; read the Summary line. Floor is **5122, FULLY GREEN** — no known-red
to hide behind. `.contains(` trips `no_loose_string_assert`; use `assert_eq!` from the start.
