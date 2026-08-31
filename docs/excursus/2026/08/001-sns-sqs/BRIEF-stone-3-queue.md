# BRIEF — excursus 001 stone 3: SQS in userland

**The thing this excursus was opened to build.** SNS shipped at stone 1
(`wat-scripts/demos/sns/sns-fanout.wat`, `"3 3"`). Everything since has been substrate debt
that drawing *this* stone uncovered — see the ⛔ section below.

## ⛔ ONE DECISION IS NOT MINE AND IS NOT MADE HERE

`wat/queue.wat` (stdlib) or `wat-scripts/demos/sqs/` (userland demo)?

- **SNS shipped as a demo.** `journal` and `query` are stdlib (`src/load/stdlib.rs:444`).
- Adding to `wat/` means `include_str!` into the binary, frozen at build time, and the
  BOOTSTRAP/stash-dance doctrine applies from then on.
- **This is an excursus.** Promoting an experiment into the stdlib is at least as consequential
  as minting an arc number, and that was ruled to be the builder's call.

**Build it as `wat-scripts/demos/sqs/sqs-queue.wat`, matching SNS.** If it earns promotion to
`wat/queue.wat`, that is a separate act and the builder's to make. **If you think it belongs in
the stdlib, say so and STOP — do not put it there.**

## The design — every primitive already exists and is proven

```
pk  = the queue name
sk  = a STABLE message id                      (never changes; `ack` names it forever)
GSI "by-visible-at":  ipk = queue name,  isk = when the message becomes visible

send     → put   a StoredRow, index-keys{by-visible-at → (queue, now)}
receive  → scan-index by-visible-at where isk <= now, take N;
           then RE-PUT each row with isk = now + visibility-timeout
ack      → delete by (pk, sk)
```

★ **The visibility timeout is a re-put that moves the index key into the future.** No lock, no
timer, no side state. Redelivery is simply what happens when nobody moved it again.

**Why a stable `sk` and not `sk = visible-at`:** `ack` names the same key forever — no
receipt-handle drift — and making a message invisible is ONE atomic `put` rather than
put-at-new-key + delete-at-old-key, which has a crash window that **duplicates the message**.

## Why this was blocked until now — the debt this stone uncovered

Drawing this stone is what found everything else in this excursus:

| | |
|---|---|
| `receive` needs a **re-put** | mem appended, sqlite replaced → **stone 2c** |
| `ack` needs a **delete** | `Store` had none → **stone 2** (+ 2b's differential) |
| 2c made mem honest | which exposed `journal` losing metrics → **INST, census, SORTKEY** |

All of it has landed. Floor is **5121, FLOOR=0**. Nothing blocks this stone.

## Read in order — exact sites

1. **`wat-scripts/demos/sns/sns-fanout.wat`** — the demo shape: `defsurface :nature
   :wat::kernel::Peer`, a `defservice` satisfying it, `:user::main` running BOTH loci and
   printing both results so the differential is the artifact. **Copy this structure.**
   ⚠ It carries a `bijection-anchor` wart and explains why in its own header — you will need
   the same if the queue holds a peer.
2. **`wat/telemetry/journal.wat:62`** — the canonical "service holding a `Store` peer":
   `:ephemeral [store <- (Peer :- [Store::Op Store::Reply])]`, `:peers [:wat::query::Store]`,
   and an `:init` that connects then calls `ensure-schema` **declaring its GSI**. The queue
   does exactly this with `by-visible-at` instead of `by-uuid`.
3. **`wat/query.wat:497`** — the `Store` surface. Five features now: `ensure-schema`, `put`,
   `delete`, `scan`, `scan-index`.
4. **`wat/query.wat:46`** — `IndexRow` carries `pk sk ipk isk data`. **That is everything
   `receive` needs to re-put a row** — no base-table read required.
5. **`tests/services/probe_ex001_journal_same_ns.wat`** — the newest both-backends fixture;
   copy its shape for the gate.
6. **`wat/telemetry/journal.wat:35`/`:44`** — `sort-key-lo`/`sort-key-hi`. **Do not reuse
   them** (they are telemetry's `SortKey`), but read them: the queue's `isk` bounds have the
   same asymmetric-sentinel problem, and stone SORTKEY's STOP-2 is the precedent for proving
   a boundary rather than arguing it.

## The gate

A both-backends fixture (`mem-store` and `sqlite-store(":memory:")`) driving the full lifecycle:

- `send` 3 messages → `receive` 2 → the two received become invisible → a second `receive`
  returns only the third
- `ack` one of the received → it is gone; the unacked one **reappears** once its visibility
  window passes
- both backends produce an identical rendered summary

★ **The redelivery row is the one that matters.** A queue that never redelivers is a queue that
loses messages, and a fixture that only tests send/receive/ack would pass without it.

## STOP triggers

1. **If you conclude this belongs in `wat/` — STOP and say so.** See the ⛔ section. Do not
   place it there.
2. **If the visibility re-put needs anything beyond `put`** — a lock, a timer, a second write,
   a base-table read — **STOP and report.** The whole design rests on it being one atomic
   `put`, and if it is not, the design is wrong rather than the implementation.
3. **If the `isk` range bounds cannot be demonstrated** the way stone SORTKEY's boundary
   fixture demonstrates its `sk` bounds — STOP. Same silent-failure class: a too-small
   sentinel drops the newest messages and every other row still passes.
4. **If a `.wat` corpus sweep is needed** — use the recorded codemod (`wat/fix.wat`,
   `wat-scripts/fixes/*.wat`), never hand edits. This brief should not need one; if it does,
   that is worth saying.
5. **If the floor reds at all — STOP**, capture whole, do NOT re-run. **Floor is 5121 and
   FULLY GREEN.** There is no known-red to hide behind any more.

## Blast radius

`wat-scripts/demos/sqs/` (new) · the gate fixture (`.wat` + `.rs`) · this excursus's SCORE.

**Zero changes to `wat/`, `src/`, or `crates/`.** Every primitive this needs already ships.
If that turns out to be false, it is a finding — say which primitive is missing.

## Verify — never through a pipe

```bash
./scripts/floor.sh; echo "FLOOR=$?"
```

**`FLOOR=0` is now the baseline, not the target.** 5121 passing. Any failure is yours.
