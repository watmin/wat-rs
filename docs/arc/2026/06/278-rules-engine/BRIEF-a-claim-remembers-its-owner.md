# BRIEF — a claim remembers its owner

Make the claim ledger record **who** holds a seq, and let a worker's own retry be answered
`DupSelf` so it emits. `wat-scripts/fanout/circuit.wat` only.

Read `DESIGN-a-claim-remembers-its-owner.md` first — it carries the evidence and the one
contract decision. `wat-scripts/scratch-pad/probe-a-claim-remembers-its-owner.wat` is the worked
mechanism: a three-arm claim on a miniature service, `discriminates=yes`. **Copy its shape.**

⚠ Its client-side match is the shape that cost a compile: the generated method returns the
**Response** directly (`ClaimResponse::First`), *not* wrapped in `Reply::Claim`. Match the
response arms.

## READ IN ORDER

| room | why you are there |
|---|---|
| `circuit.wat:45-66` | the `:fanout::Seen` surface — `ClaimRequest` gains `owner <- String`; `ClaimResponse` `:Dup []` becomes `:DupSelf []` + `:DupOther []` |
| `circuit.wat:82` | **the defect** — `claimed <- HashMap [String bool]` becomes `[String String]`, keyed seq → owner |
| `circuit.wat:86-125` | the `claim` impl — `already?` becomes a three-way on the stored owner; `dups'` counts **DupOther only** |
| `circuit.wat:368` | `wid` — the owner to send, already in scope in the worker's `let` |
| `circuit.wat:410-430` | where the worker builds and sends its `ClaimRequest` — add `:owner wid` |
| `circuit.wat:475-493` | **the emit rule.** `first?` becomes "First or DupSelf". ⚠ `:479` has a `_` arm that assertion-fails `"claim not First/Dup"` — it must learn the new arms or it will fire on the correct path |
| `circuit.wat:555-565` | `held-worker` — `:peers [:queue::Queue]`, does **not** claim. Confirm it needs nothing |

## SKETCH

In the service:

```wat
prior (:wat::hashmap::get claimed key)
resp  (:wat::core::match prior
        ((:wat::core::Some who)
          (:wat::core::if (:wat::core::= who owner)
            (:fanout::Seen::ClaimResponse::DupSelf)
            (:fanout::Seen::ClaimResponse::DupOther)))
        (:wat::core::None (:fanout::Seen::ClaimResponse::First)))
```

The ledger is written **only** on `First`; `DupSelf` must be idempotent — a second retry must
still answer `DupSelf`, never flip to `First`.

In the worker: `first?` → `emit?`, true for `First` and `DupSelf`.

## BLAST RADIUS

`wat-scripts/fanout/circuit.wat` only. No `wat/`, no `sqs.wat`, no `src/`, no codemod, no
nextest config.

## STOP TRIGGERS

- **STOP-1** — if the drop arm's interaction with `write?` / `drop-after?` means a dropped
  `DupSelf` cannot be distinguished from a dropped `First` on the *next* retry, STOP and report.
  That is a real hole in the contract, not something to paper over.
- **STOP-2** — if `owner` cannot be threaded to the claim site without touching a second file,
  STOP. `wid` is at `:368` and the send is in the same `fn`; a spill means the seam moved.
- **STOP-3** — if making `DupSelf` emit pushes `dup` above 0 in **any** run, STOP. Double-emission
  is a worse defect than the stranding and must not be traded for it.
- **STOP-4** — do not touch the send-path scan count or anything else perf-shaped. Correct first.

## PRIOR RESULT TO COPY FOR SHAPE

`SCORE-depth-is-read-not-counted.md` — same campaign, immediately prior. Note especially how its
red floor was reported rather than re-run, and how its fixture was corrected rather than weakened.
