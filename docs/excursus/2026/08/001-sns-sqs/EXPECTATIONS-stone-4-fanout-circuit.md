# EXPECTATIONS — excursus 001 stone 4: the fan-out circuit

**Written BEFORE the strike, 2026-08-31.** Blast radius derived from the BRIEF's own section.

## ⚠ Two outcomes are both valid deliveries

- **Zero duplicates at M×J concurrency** — the actor's serialization holds under real load, and
  a claim derived from reading a comment becomes a measured fact.
- **Duplicates observed** — ★ **that is a FINDING, and a successful stone.** It would mean the
  actor does not serialize the way `wat/query/mem.wat:22-24` implies, which is worth far more
  than a green demo. STOP-2 forbids fixing it.

The failure mode is neither: a summary produced by a circuit that never actually ran the
workers in parallel, or a property "proven" with a sleep.

## The scorecard

| # | what | expected |
|---|---|---|
| 1 | it is a circuit | `:user::main` is wiring only — no computation, per `docs/CIRCUIT.md` |
| 2 | ★ fan-out completeness | exactly **N × M** outcomes |
| 3 | no loss | a final `receive` on every queue returns empty |
| 4 | ★ parallelism proven | **all M×J worker ids appear** in the outcomes — no clock, no sleep |
| 5 | ★ duplicate count | **reported**, whatever it is. Zero is a result; non-zero is a finding |
| 6 | one queue service per queue | read the wiring — J workers dial ONE service each (STOP-1) |
| 7 | workers are processes | `:locus (:wat::spawn::process)`, not thread |
| 8 | standalone runs at weight | N=2000, M=4, J=3 → 8000 outcomes, or the number it broke at |
| 9 | floor fixture is scaled | fits the default budget, or an override is **proposed** not added |
| 10 | the floor fixture drives the SHIPPED program | `startup_from_file`, no second copy |
| 11 | blast radius | `wat-scripts/fanout/` + one `.rs` + SCORE. **topic/ and queue/ untouched** |
| 12 | floor | `FLOOR=0`, 5122 + the new arm |
| 13 | prior stones | all `probe_ex001_*` PASS; SNS still `"3 3"`; queue still `bound=x;…` |

## Runtime prediction

**3–5 hours.** The largest stone in the excursus. The circuit itself is wiring, but 18
processes with grant-before-dial at M queues is fiddly, and row 5 needs a way to *detect* a
duplicate — which means every outcome carries its message id and the summary counts distinct
vs total.

## Trap-doors

1. **★ Row 4 can be faked by accident.** If work is handed out round-robin by the wiring rather
   than pulled by the workers, all ids appear whether or not anything ran in parallel. The ids
   must appear because workers *pulled* from a shared queue, not because the parent dealt them.
2. **★ Row 5 needs the instrument to exist first.** You cannot count duplicates unless each
   outcome carries its message id and the summary reports `total` vs `distinct`. Build that
   before the assertion, or a duplicate is invisible and row 5 silently reports zero.
3. **Grant-before-dial at scale.** The SNS demo grants 3 subscribers in a post-spawn hook; here
   it is M queues and M×J workers. If the grant ordering does not compose at this size, that is
   a finding about `wat-topic`, not a workaround to improvise.
4. **The visibility window vs worker latency.** A worker that takes longer than the window to
   ack will see its message redelivered — a *correct* at-least-once behaviour that will look
   like a duplicate. **Distinguish the two in the summary**, or row 5's number is meaningless.
   This is the subtlest thing in the stone.
5. **18 processes on a 12-core box** is oversubscribed by design. That is fine for correctness
   and bad for wall-clock; do not read slowness as a defect, and do not assert on timing.

## Not in this stone

- Any change to `wat-topic` or `wat-queue` — if one is needed, that is a finding.
- Promotion of either to `wat/`.
- Dead-letter queues, retry limits, FIFO ordering.
