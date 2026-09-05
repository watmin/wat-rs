# DESIGN — a claim remembers its owner

**The stranding.** `wat-scripts/fanout/circuit.wat` only. Correctness; no perf work.

## WHY — the information loss is one word

`circuit.wat:82`:

```wat
:ephemeral [claimed <- (:wat::core::HashMap :- [:wat::core::String :wat::core::bool])]
```

**The ledger records THAT a seq was claimed and discards WHO claimed it.** So *"someone else
owns this"* and *"I own this and never heard back"* collapse to one answer — `Dup` — and they
are different facts the caller must act on differently.

`circuit.wat:491` is the caller acting on it:

```wat
outs1 (:wat::core::if first? (:wat::vector::conj outs0 (:fanout::Outcome …)) outs0)
```

A `Dup` emits nothing, **and the message is acked either way.**

## THE EVIDENCE — six runs of mine, drop-after, `n=50 m=2 j=2` at 10 %

```
seen-firsts=100  every run          total ∈ {89, 90, 90, 91, 89}   no worker died in the completing runs
```

Every message was claimed `First` **exactly once**, and ~10 of those First-claimers never
emitted. Worker A claims → `First` → ledger written → **reply dropped** → A times out at 200 ms,
redials, retries → the ledger already holds the seq → A is told `Dup` → A emits nothing. **A was
the only claimant.** At-least-once delivery became at-most-once processing, silently.

⚠ **The path is INFERRED, not observed** — from `seen-firsts=100` ∧ no worker died ∧ `total<100`.
The row that converts it to observed is `total` returning to **100**. If it does not, this
mechanism is wrong and the stone is refuted. Said here so the refutation is cheap.

## ⛔ THE ONE CONTRACT DECISION

**The server returns the ANSWER, not the data to compute it.** `ClaimRequest` gains `owner`;
`ClaimResponse` becomes `First` / `DupSelf` / `DupOther`.

Not `Dup [owner <- String]` with the caller comparing. A caller that can forget to compare is a
caller that will, and we are back to today. The server holds both the stored owner and the
requesting owner; it is the only place the question can be answered once. The caller's rule
becomes `emit if First or DupSelf` — and `held <- HashMap [String String]`, where the value **is**
the fact.

Proven expressible, `probe-a-claim-remembers-its-owner.wat`:

```
A-first=First;A-again=DupSelf;B-same-seq=DupOther;B-new-seq=First;discriminates=yes
```

## ONE SENTENCE, TWO CONSEQUENCES

**A retry by the owner is not a duplicate.** That fixes the emit rule *and* the count:

1. `DupSelf` must **emit** — the work is this worker's and nobody else will report it.
2. `DupSelf` must **not** increment `dups` — it is a retry, not a second delivery.

★ Consequence 2 predicts something checkable: **rate-0 `seen-dups` should return to 0**
(currently `7 7 10 7 7`, mine). At rate 0 there are no drops, so those dups are T1's deadline
firing and the worker retrying — `DupSelf` by construction. If they do not vanish, they have a
second source and consequence 2's reasoning is wrong.

## FILES

`wat-scripts/fanout/circuit.wat` only — the `:fanout::Seen` surface, the `:fanout::seen` service,
and the worker's claim arm. `held-worker` declares `:peers [:queue::Queue]` and does not claim.

## OUT OF SCOPE = REJECTED

- **All perf work**, including the send-path double scan I found grading the last stone.
  Correct first. That is a separate stone and it is not this one.
- **The `claim deadline exhausted` crash** (1/6 runs). Report the count; do not repair. Killing
  a worker on retry exhaustion instead of releasing the message is its own defect with its own
  design question.
- **Emitting on `DupOther`.** That double-counts and breaks `distinct == total`.
- **`wat/`, `sqs.wat`, the reactor.** The claim ledger is entirely in `circuit.wat`.
