# BRIEF — a dead runner names its orphan

**Read `DESIGN.md` beside this first.** It carries the one contract decision (name the orphan, do NOT
re-dispatch), the measured probe output, why the queue route is load-order blocked rather than merely
later, and five trap-doors — the first of which will send you to the wrong line if you ignore it.

⛔ **NO CONTRACT CHANGE.** `brackets/map` still fails on a dead runner. It stops failing anonymously.

## The work, in one paragraph

`collect-loop` hands item `cursor` to whichever runner just replied and advances the cursor *"regardless
of outcome"*, keeping no record of who holds what. So when a runner dies, the message can say *runner 0
crashed* and can never say **which item died with it**. Record `peer-pos → item-index` at dispatch, and
name the orphaned item in all three failure messages. The arms still raise.

## The rooms

| where | why you are going there |
|---|---|
| `wat/bracket.wat:594` `collect-loop` | the signature — `[peers items pairs-acc cursor collected m]`. **Nothing maps a runner to an item.** That is the whole defect. |
| `wat/bracket.wat:608`–`:625` the `Message` arm | the dispatch. `:611`'s comment says the cursor advances *"regardless of outcome"*. This is where the ledger entry is written, and `:624` is the tail recursion that carries it. |
| `wat/bracket.wat:626`/`:632`/`:642` | the three arms — `Closed` / `Lost` / `Malformed`. Raises at `:627`/`:633`/`:643`. ⛔ **Verified line numbers; see trap-door 1.** |
| `wat/bracket.wat:638`–`:641` | the Malformed arm's argument for being loud. **Do not weaken it** — you are adding the orphan's identity, not changing the disposition. |
| `wat-scripts/scratch-pad/probe-a-dead-runner-orphans-its-item.wat` | ⭐ THE FIXTURE. Today it prints `runner 0 crashed: …`. After this stone it must also name the item. |
| `the-waiter-folds-carry-a-named-aggregate/` | the struck ruling that closed nested-Tuple accumulators. Trap-door 2 — your ledger must not reopen it. |
| `wat/fix.wat` BOOTSTRAP / STASH-DANCE header | ⛔ `bracket.wat` is **stdlib, frozen into the binary at build time**. Read this before editing it. |

## Implementation sketch

```wat
;; collect-loop gains one binding: which item each runner position is holding.
;; A parallel vector indexed by peer-pos, or a named aggregate — NOT a fourth Tuple slot.
holding <- (:wat::core::Vector :- [:wat::core::i64])   ;; -1 = idle

;; at dispatch (the Message arm), record BEFORE advancing:
holding' = (assoc holding peer-pos cursor)

;; in each failure arm, the message gains the orphan:
"bracket collect-loop: runner {idx} crashed holding item {item}: {cause}"
```

⭑ **`-1` for idle is a sentinel, so say in the SCORE how an idle runner's death reads.** A runner can
die between finishing one item and receiving the next; the message must not claim it orphaned item 0.

## Blast radius

`wat/bracket.wat` only. **Expected 0 elsewhere** — confirm rather than inherit.

## STOP triggers

1. ⛔ **STOP-1 — if `peer-pos` is not stable for the pool's lifetime**, STOP and report. `select`
   returns a position into `peers`; a ledger keyed on a shifting index is worse than none. Verify
   `peers` is never compacted, or key on something stable.
2. ⛔ **STOP-2 — if the ledger cannot be added without a fourth nested-Tuple slot**, STOP.
   `the-waiter-folds-carry-a-named-aggregate` closed that shape deliberately.
3. **STOP-3 — do NOT re-dispatch, and do NOT silence a raise.** Both are contract changes the builder
   has not ruled, and the probe measured that the only alternatives to raising are a hang or a silent
   short answer.
4. **STOP-4 — if the stdlib freeze blocks the edit**, read `wat/fix.wat`'s BOOTSTRAP header; do not
   hand-edit your way out of a non-booting tool.

## Verify

- `cargo build --release` **and** `cargo nextest run --release --no-run`.
- `./scripts/floor.sh`, read the **Summary line**, never a piped exit code. ⚠ And never `$?` after a
  pipe — that is `tail`'s status; the orchestrator hit exactly that an hour ago.
- ⭑ **The fixture names the orphan:**
  `./target/release/wat wat-scripts/scratch-pad/probe-a-dead-runner-orphans-its-item.wat`
  → the control still prints `clean=[0 2 4 6 8 10 12 14]`, and the failure now identifies **which item**
  the dead runner held.
- ⭑ **The item named is the RIGHT one.** The probe kills item 3 specifically — the message must say 3,
  not "an item". A ledger that names the wrong index is worse than no ledger.
- ⭑ **A negative control:** a clean map must be byte-identical to today, and `wat-tests/bracket.wat`'s
  existing tests must pass unchanged.
- ⚠ Report how an **idle** runner's death reads (the `-1` sentinel).

## Shape to copy

`a-claim-remembers-its-owner/` — the struck stone that taught the queue's ledger to record **who** holds
a seq. Same idea, different layer, and the reason this one is small.
