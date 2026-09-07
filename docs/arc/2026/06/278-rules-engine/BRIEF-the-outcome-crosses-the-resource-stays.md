# BRIEF — the outcome crosses, the resource stays

Give the worker's three retry ladders a **nullary-payload closed enum** whose `Exhausted` variant
every match must name, with the peer and reply riding **beside** it in a Tuple. Give the two ladders
that still stop after three flat tries the vis-bounded backoff the ack path already has.
`wat-scripts/fanout/circuit.wat` only.

**You already built the enum half of this and it ran green at n=12** on the previous strike. This
brief adopts that construction; it does not ask you to re-derive it.

## Read in order

1. **`SCORE-exhaustion-is-a-named-variant.md`** — **your own SCORE, and the GRADING under it.** The
   three walls, and the sentence that explains all of them: a `Peer` *"cannot be reconstructed from
   EDN bytes across an address-space boundary."* Do not re-attempt a payload-carrying variant.
2. **`wat/service.wat:207`** — `:ephemeral … resources + peer clients; **never crosses**`. The rule
   from the other side.
3. **`circuit.wat:494-518`** — the **check** ladder. `once` → `(peer, Option<reply>, retry?)`;
   `a1`/`a2`/`a3`; `:518` returns `(Tuple 1 0)` on exhaustion. **The terminal event at depth.**
4. **`circuit.wat:560-576`** — the **mark** ladder. `mm1`/`mm2`/`mm3`; **`second mm3` discarded.**
5. **`circuit.wat:606-692`** — the **ack** path: `ack-limit-ms` `:606`, inlined backoff `:675-676`,
   `ar-tick` `:691`. **The loop shape to copy.**
6. **`circuit.wat:698-718`** — `tick-pair` → `gb-tick`/`ar-tick` → `worker::Record`. The counter groove.
7. **`circuit.wat:2153` / `:2188` / `:2201`** — failure-path string, summary, phases.

## The work

**1. Two nullary closed enums**, `:fanout::SeenRetry` and `:fanout::QueueRetry`, each
`:wat::enum::Pure` with `:Got []` and `:Exhausted [attempts <- :wat::core::i64]`. Declared where your
n=12 build put them (the Worker surface, as you measured).

**2. Each retry helper returns `(Tuple outcome peer reply)`** — outcome is the enum; peer stays a
local binding and **never enters a crossing value**.

**3. All three ladders use them.** Delete `a1/a2/a3`, `mm1/mm2/mm3`, every surviving `bool` retry
flag. Each call site **matches** `Got` / `Exhausted` by keyword variant.

**4. Check and mark get the ack's loop.** On `DeadlineFired`, draw the inlined backoff and retry
until answered or `elapsed >= vis-ns / 1000000`. `Lost` / `Closed` redial as today.

**5. Honest counters.** One per site, named for what it measures. **Rename `gave-back`.** Surface in
summary, phases, **and the failure string** (`:2153`).

## Blast radius

`wat-scripts/fanout/circuit.wat` **only**. No `wat/`, no `sqs.wat`, no `sns-fanout.wat`. The receive
path (`:452`), `:1531`, and the per-call 200 ms deadlines are untouched.

## STOP triggers

- **STOP-1** — if a nullary `:Got []` / `:Exhausted [i64]` enum cannot be keyword-matched in the
  child, **STOP and quote the checker.** Your n=12 run says it can; a contradiction outranks the stone.
- **STOP-2** — if any drop-run assertion changes (the 38 `drop` floor tests, `drop-after`,
  `drop-before`, `drop-recv-tiny`, `drop-ack-tiny`), **STOP and report which.** The vis-derived bound
  is designed to preserve them; if it does not, the derivation is wrong and must be re-thought,
  **not patched with a flag.**
- **STOP-3** — if the no-args run differs in any **pre-existing** summary field, **STOP.**
- **STOP-4** — if `2000 4 3 8192 true` still fails, **STOP and report the terminal sweep with every
  counter.** That is the diagnosis; do not raise a bound to force a pass.
- **STOP-5** — floor red on any arm: **STOP; do not re-run.** Name the exact arm.
- **STOP-6** — **do not promote anything to `wat/`.** The builder's ruling; this stone earns it.
- **STOP-7** — **do not put a `Peer`, a `Reply`, or any live resource inside a variant.** Three
  drafts died there. If the design seems to need it, the design is wrong — STOP and say so.

## Shape to copy

`SCORE-the-ack-retries-like-the-publisher.md` — its ack loop is the exact loop check and mark need.
