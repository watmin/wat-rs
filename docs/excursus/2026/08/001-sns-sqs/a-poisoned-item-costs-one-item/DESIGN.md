# A poisoned item costs one item

**The surface wall, on its fifth encounter, with a ruling.** Excursus `001-sns-sqs`.

Read first — the four stones that hit this wall and deferred it:

1. `../the-bracket-runs-on-the-queue/SCORE.md` — the queue path had no slot for a report
2. `../a-dead-runner-loses-one-item-not-the-run/` — the `Malformed` arm, same
3. `collect-gave-up!`'s own text — *"per-item causes are not carried — the surface has no slot"*
4. `../an-oversized-reply-is-not-a-death/SCORE.md` — `FrameTooLarge` is REPORT-FINAL **by
   construction** and cannot be reported

## The ruling

Builder, after the four-questions: **option (c)** — `map` and `each` keep their signatures; a
**sibling verb** returns per-item outcomes.

> ⭐ **This is the codebase's own idiom.** `recv` / `recv-by-deadline`. `send` / `try-send`.
> `select` / `select-by-deadline`. **The system never widens a verb; it adds a qualified sibling.**

And it satisfies the standing doctrine directly — *"the wat surface must remain unchanged"* —
because nothing existing moves.

## What it buys, measured

With 3 runners and **one** oversized item, the whole fleet is consumed: runner 0 wedges, the item is
re-queued to runner 1, that wedges, and so on to `REPORT-GONE` (measured, `1e5c16866`). The map fails
either way — but only because there is nowhere to record *"item 3 was refused"* and carry on.

> **With a per-item slot, one poisoned item costs one item. Without one, it costs the item AND the
> pool.**

⭐ **And the causes are already in hand.** `collect-loop` binds them at every failure site today:
`Lost … cause`, `Rejected … cause`, `Malformed … _cause` — the last one discarded with an
underscore. This stone does not *discover* failures. It stops **throwing away what the coordinator
already knows**.

## The work

### 1. The sibling verb

⛔ **Not `try-`.** In this tree `try-` means **non-blocking** (`try-send`), so `try-map` would
mislead on the axis that matters. Follow `-by-deadline`'s shape — a qualifier naming the thing
gained. Choose the name and say why.

`map`/`each` signatures **byte-identical**; all **60** existing call sites (51 `map`, 9 `each`)
untouched. ⚠ Verify that count yourself before relying on it.

### 2. The per-item outcome type

⚠ **There is no precedent in this corpus for a `Vector` of outcomes** — measured, zero. So the shape
is a real decision, not a lookup. What it must carry, from the causes already bound:

| fate | today | carries |
|---|---|---|
| success | the `O` | the `O` |
| `Rejected` (oversize) | the runner is dropped, the item re-queued, eventually `REPORT-GONE` | ⭐ REPORT-FINAL **by construction** — the clearest case for a per-item report |
| `Malformed` | re-queued; `_cause` discarded | the cause |
| `Lost` / `Closed` | re-queued to a survivor | only reportable if no survivor took it |
| `GaveUp` | one raise for the whole map | ⚠ which items were outstanding |

⛔ **Do not invent a fate the coordinator cannot distinguish.** `collect-gave-up!` already says it
*"could not distinguish death from garbling"* was removed as false — do not reintroduce a
distinction the code does not have.

### 3. ⭐ Retry stops being futile for a deterministic fault

Once an item's failure is reportable, `FrameTooLarge` can be **REPORT-FINAL in fact** rather than in
theory: record it, drop it, **keep the runner**, continue with the rest. That is the taxonomy's own
answer, and it is the first time the surface allows it.

⚠ State explicitly what changes for the existing arms and what does not. `Lost`/`Closed` re-dispatch
is still right — a dead runner's item is not a poisoned item.

### 4. The control

A map where **one** item is poisoned and the rest are good must return the good results **and** name
the bad one, with the fleet intact. ⛔ Judged by MUTATION: remove the per-item recording → the run
loses the pool again (measurable as `REPORT-GONE` instead of a result set). Show both.

⚠ Non-vacuity: prove the poisoned item really failed, and that the **other** runners are still alive
at the end.

## Scope wall

⛔ `map`/`each` unchanged. Thread tier. Suppression. The lineage-`Admin` delay. Queue knobs.
`RecvOutcome::Rejected`. The 7 remaining blocking `send` sites.

## The four questions

- **Obvious** — ✅ ruled, and it is the idiom the tree already uses three times over.
- **Simple** — ✅ for the surface (nothing moves); ⚠ the outcome *shape* has no precedent, so row 2
  is the real work.
- **Honest** — ✅ it stops discarding causes that are already bound, and it lets `FrameTooLarge` be
  what the taxonomy says it is.
- **Good UX** — ✅ the happy path stays `(Vector :- [O])`; the fallible path is explicit at the call
  site.
