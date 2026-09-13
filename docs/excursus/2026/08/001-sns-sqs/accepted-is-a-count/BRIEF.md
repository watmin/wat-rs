# BRIEF — accepted is a count

**Read `DESIGN.md` beside this first.** It carries why the defect is an **overloaded field** rather than a
wrong number, the sibling question (the ack side is **out**, with its reason), and the residual when even the
probe fails.

## The work, in one paragraph

A store `put` whose reply is lost leaves the queue unable to say how many rows landed, so it returns
`Accepted 0` as a **retry signal** — and its two callers both read `n` as something else, one under-reporting
a row that *is* in the store and one raising. Make the queue **ask**: after the existing redial, scan the base
table over the batch's `sk` range, count how many of those rows are present, and report **that**. `Accepted n`
becomes a count and only a count.

## The rooms

| where | why you are going there |
|---|---|
| `wat-scripts/queue/sqs.wat:565–580` | the put site. ⭑ `rows` is built as `Vector<StoredRow>` over `(range 0 take)`, each `:pk q :sk sk`. **`rows`, `sk` and `take` are all in scope at the arms below** — that is what makes the query possible without threading anything new. |
| `wat-scripts/queue/sqs.wat:780`, `:814`, `:845` | ⭑ **THE THREE ROOMS** — the put's `Lost` / `Closed` / `TimedOut` arms. Each already **redials** (`:782`) and rebuilds state with `:store fresh`. Read `:806`'s comment — *"Accepted 0 is the caller's retry"* — that is the intent you are replacing. |
| `wat-scripts/queue/sqs.wat:785` | the redial's own raise, *"peer is dead, not a broken pipe"*. ⭑ **This stays**, and it is the precedent for what to do if the probe also fails: a dead store raises. |
| `wat/query.wat` `Store::ScanRequest` | `[pk sk-lo sk-hi limit cursor]` — a **base-table page** request. ⚠ It is **paged**; `cursor` is `None` for the first page. |
| `wat-scripts/queue/sqs.wat:279` | an existing `Store/scan-index` call — the worked shape for issuing a scan and facing its response in this file. |
| `wat-scripts/query/faulting-store.wat` | ⭑ **YOUR TEST RIG, already built.** `drop-reply-bp` forwards the put to the real store and then destroys its own reply — the write lands, the caller gets nothing. That is precisely this stone's condition. |
| `wat-scripts/scratch-pad/probe-store-can-fail.wat` | how that proxy is wired and driven; copy it. It already prints `real-rows` from the real store, which is the number your `n` must match. |
| `wat-scripts/topic/sns-fanout.wat:118`, `:847` | the two callers. ⛔ **Do not touch either.** They become correct because `n` becomes a count; changing them would hide whether the fix worked. |

## Implementation sketch

```
;; inside each of the three arms, AFTER the existing redial to `fresh`:
sk-lo   = minimum sk across rows          ;; they are consecutive, built from (range 0 take)
sk-hi   = maximum sk across rows
scanned = (:wat::query::Store/scan fresh
            (:wat::query::Store::ScanRequest :pk q :sk-lo sk-lo :sk-hi sk-hi
                                             :limit <at least take> :cursor None))
landed  = | { r ∈ scanned : (StoredRow/sk r) ∈ rows.sk } |     ;; ⛔ INTERSECT — not (length scanned)
→ (:queue::Queue::SendResponse::Accepted landed)
```

⚠ **The sketch is a convenience; the disk is the contract.** Check `ScanResponse`'s variants and what its
`Success` carries (`rows` + `next-cursor`), and confirm `Store/scan`'s exact name and arity. **My sketches
have been wrong five times in this campaign and the disk was right every time** — including once where my
named API (`try_wait`) would have passed every row while breaking the contract. Take the shape, verify every
name.

## ⛔ Three ways to get this silently wrong

1. **Counting `length scanned` instead of the intersection.** The `sk` range can hold rows from earlier
   batches. This produces a plausible, wrong, *larger* count.
2. **Ignoring the cursor.** `ScanRequest` is paged; a truncated first page is a silently *smaller* count.
   Either follow the cursor or prove one page covers `take`.
3. **Letting the scan run on the happy path.** It belongs inside the three failure arms only.

## ⭑⭑ The proof — the injector makes this measurable

Using the faulting proxy with `drop-reply-bp` live:

| assert | expected |
|---|---|
| ⭑⭑ `n` equals the true count | `Accepted n` where **n == real-rows** in the store — **not 0**, and not `take` unless all landed |
| the write really landed | `real-rows > 0` while the reply was destroyed |
| a relation, not a threshold | prefer `n == real-rows` (an **equality**) over `n > 0`; D2 proved a threshold's flake rate depends on parameters another stone may move |
| no happy-path cost | `store-calls` on the unperturbed `2000 4 3 8192 true 1000` run **unchanged** |

## Verify

- `./scripts/floor.sh`, read the **Summary line** → green at **5239** (D2 added two tests; do not report 5237).
- ⭑ **`probe_chaos_gate_has_teeth` must stay green** — D2's gate watches this path deliberately.
- ⚠ **State any new test's runtime against the floor's WALL, not just in seconds.** Headroom is ~1.2× and a
  19–25 s test already timed out at 40 s under `nice -n 19` contention (`.floor/2026-09-13T04-00-06Z/`).
- ⛔ **Never a piped exit code** (a type-error run this session reported `$?` = 0 through `| head`; true exit 3).
- `cargo nextest run --release --no-run` — the build does NOT compile tests.
- `cargo clippy --release --workspace --all-targets -- -D warnings` → 0.
- Happy path `2000 4 3 8192 true 1000` → `distinct=8000;dup=0`; shipped chaos `… 0 0 7 0 0 0 0 0 500` → exit 0.
- ⚠ **`rt-store` must NOT improve** — the −17.1 % patch is unblocked by this stone, not banked by it.
- Everything heavy through `./scripts/capped.sh --limit 8g`.

## STOP triggers

1. **STOP-1 — if the batch's rows cannot be identified in a scan** (e.g. `sk` is not derivable at the arm),
   STOP and say what is missing. Do **not** substitute a count you cannot justify.
2. **STOP-2 — if one page cannot cover `take`** and following the cursor is awkward here, STOP and report it.
   A truncated page is a wrong count, and a wrong count is worse than the `0` we are replacing.
3. **STOP-3 — do not touch the ack side** (`:1215`/`:1248`/`:1279`) **or either caller**
   (`sns-fanout.wat:118`, `:847`). All three are out of scope with reasons in DESIGN.
4. **STOP-4 — if the probe scan can fail in a way that is NOT a dead store**, STOP and describe it. The
   contract says probe-also-fails ⇒ dead store ⇒ raise; if there is a third case, the contract is incomplete
   and I want to know before it ships.
5. **STOP-5 — do not add `:Unknown` or any new `SendResponse` variant.** D1-c was ruled over D1-b for exactly
   that reason.

## Shape to copy

`the-store-can-fail/SCORE.md` for driving the proxy and reading `real-rows`, and
`the-store-says-what-it-deleted/SCORE.md` for a stone that made a store answer honestly and proved it with a
probe the floor could not give.
