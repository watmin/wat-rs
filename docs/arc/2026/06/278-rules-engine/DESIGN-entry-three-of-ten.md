# DESIGN — entry three of ten

**Step 1 of the ordered plan: provoke per-entry batch failure and measure what the stack does.**
`wat-scripts/scratch-pad/` only. **No production change.**

## WHY — every batch surface is vector-in, one-outcome-out

```wat
Store::DeleteRequest  [keys <- Vector[Key]]        →  :Success [] | :Transient | :Fatal | …
Store::PutRequest     [rows <- Vector[StoredRow]]  →  :Success [] | :Transient | …
Queue::AckRequest     [queue  ids  <- Vector]      →  :Ok []
Seen::MarkRequest     [queue  seqs <- Vector]      →  :Ok []
```

`:Transient` on ten keys says *something went wrong* and cannot say **which**. A caller that
retries the whole batch is the duplicate risk.

★ **I argued last stone that a `Failed[]` on `ack`/`mark` would be "a lying surface, because our
impl provably cannot partially fail."** That was wrong. It cannot partially fail **because
nothing has ever made it** — the same shape as a crash reading 0/6 while the injector looks
elsewhere. *Provably cannot* was *never injected*.

★★ **Every fault this arc has injected drops a WHOLE reply.** Entry 3 of 10 failing while 1, 2
and 4–10 succeed is a fault domain the circuit has never seen.

## ⛔ THE ONE CONTRACT DECISION

**Provoke it in a wrapper, not in production.**

A `:fs::failing-store` service satisfying `:wat::query::Store`, holding a real store peer and
forwarding everything — except that on `put`/`delete`, at a seeded rate, it **applies k of n and
then returns the only thing the surface allows.**

★ Why not a knob on the real stores: `sqlite-store::Record` has **26** construction sites across
14 files and `mem-store::Record` has **102** across 46. **128 sites for a provocation** is a
corpus migration spent on a question we have not yet answered. If the answer says production
needs the knob, it is earned then.

## ⚠ THE PREDICTION, WITH ITS MECHANISM

The arc's record is that every prediction naming only a number has died and every one naming a
mechanism has held. So:

- **Partial `put` DUPLICATES.** The store writes k of n and reports `:Transient`. The queue's
  `send` cannot know which landed, so it retries the whole batch — and the k already written are
  written again. **`distinct` should hold, `total` should exceed it.**
- **Partial `delete` is SAFE.** The store deletes k of n and reports `:Transient`. The retry
  re-deletes rows that are already gone — and we **measured** that a second delete of a missing
  row is a no-op returning `Success` (`SCORE-the-queue-can-drop-too`, row 4). **Nothing is lost
  and nothing is duplicated.**

★ If that asymmetry holds, it says exactly where per-entry outcomes are load-bearing: **`put`
needs them; `delete` may not.** That is what earns step 2 and scopes it.

★★ If **both** are safe, the collapsed surfaces are honest and step 2 shrinks to nothing —
which would be a finding worth the stone on its own.

## WHAT THIS STONE MUST REPORT

1. What the store **can say** when it partially fails — the literal response value.
2. What the layer above can **infer** from it.
3. What the system **does** — `total`, `distinct`, and whether a message is lost.

★ (3) is the one the types cannot answer, and it is why this is chaos and not a code review.

## FILES

`wat-scripts/scratch-pad/` — a failing-store wrapper and a probe. Nothing else.

## OUT OF SCOPE = REJECTED

- **Fixing anything.** This stone makes the defect visible; step 2 repairs it.
- **Knobs on the real stores** — 128 sites, and unearned until this answers.
- **The topic batch** (step 3) — deliberately after step 2, so it is not built on a collapsed
  surface.
- **`src/`, `wat/`, compiled wat.**
