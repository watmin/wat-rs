# HANDOFF → grok — excursus 001 stone 7: the fan-out proof, re-attempted

Branch `sns-sqs`. Read in full:

- `docs/excursus/2026/08/001-sns-sqs/BRIEF-stone-7-fanout-proof.md`
- `docs/excursus/2026/08/001-sns-sqs/EXPECTATIONS-stone-7-fanout-proof.md`
- `docs/excursus/2026/08/001-sns-sqs/SCORE-stone-4-fanout-circuit.md` — your own strike, and why
  `dup=0` was correctly called vacuous

**This is the stone the whole detour was for.** Stone 4 was sent the wrong way by a guard that
could not see a parametric field type. That guard now sees it, the queue is fixed, and the
circuit's own instance of the same defect has surfaced. Three jobs:

**1. Lift `:fanout::Outcome` into `:fanout::Worker`'s `:messages`** — `circuit.wat:30` beside
the surface at `:69`, referenced as `(Vector :- [:fanout::Outcome])`. **`Envelope`'s exact twin**,
and the same move you made in stone 6. A message may keep a non-prefixed name, so it is a pure
lift: no rename, no call-site churn. **The floor's one red goes green here.**

**2. Delete the foreign-read workaround** (`~:127-140`) — `read-foreign (write e)` +
`ForeignRecord/get :id/:body`, written when `Envelope` could not cross. It now can; use
`(:queue::Envelope/id e)` and `(:queue::Envelope/body e)`.
⚠ **Do not assume the workaround's silent-zero path (`None → ""` → ack nothing) caused stone 4's
`total=0`.** You flagged it as a candidate and said it was unmeasured — that judgement stands.
Delete it because it is obsolete, then **measure what the circuit actually does.**

**3. Run the proof.** The four properties, unchanged: fan-out completeness (`N × M`), no loss,
parallelism by **worker ids not a clock**, and a **reported** duplicate count.

★ **ROW 4 IS THE HONESTY GATE.** Stone 4 produced a zero summary and you correctly refused to
bank it. **That standard holds** — if the circuit still returns zeros, that is the deliverable,
and `dup` stays vacuous. A green-looking number from a circuit that did not run is the one
outcome worse than a red one.

★ **AND THE CURRENT RED MAY NOT BE THE LAST.** Two instances of this defect were found one stone
apart because a type-check halts at the first error. If a **third** appears after `Outcome` is
lifted, that is expected — fix it and say so plainly.

Weight: keep `n=12, m=2, j=2` for the floor fixture (30s default kill), then run **standalone at
weight** — 12 cores / 28 GB free, target N=2000 M=4 J=3, 8000 outcomes, or the number it broke at.

Zero changes to `wat/`, `src/`, `crates/`, `topic/`, `queue/`. Verify in the FOREGROUND; read the
Summary line. Floor is **5126 with 1 red**; this stone must take it to **FLOOR=0**.
