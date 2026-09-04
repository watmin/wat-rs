# SCORE — chaos is a rate

**STRUCK.** Executor: grok, 2026-09-04. Every row re-run by me on a quiet box.

```
Summary [ 361.689s] 5213 tests run: 5213 passed (3 slow), 15 skipped
FLOOR=0        .floor/2026-09-04T03-53-57Z/        my own run, default rate, 0 FAIL/TIMEOUT
```

## ★ THE CONTRACT DECISION HELD — it is a rate, not a probe

My run, 3/3 identical:

```
rate0 = hits=0 ; draws=0 ; points=
run1  = hits=8 ; draws=40 ; points=1,7,21,24,31,34,38,39,
run2  = hits=8 ; draws=40 ; points=1,7,21,24,31,34,38,39,
replay = SAME       verdict = CHAOS-IS-A-RATE
```

- **`draws=0` at rate 0.** Not armed-and-inert — **no alarm exists.** The opt-in is structural.
- **`hits=8`, not 1.** The disruptor re-arms. That was the executor's catch on my design, and it is
  the whole difference between "a reap is survivable" and a fault domain.
- **Same seed, same eight points.** A chaotic run replays.

At circuit weight, my five runs:

```
total=8000; distinct=8000; dup=0; disrupts=24      ×5, identical
```

**`disrupts=24` on all five** is the seed replaying at 8000 messages and 12 workers. The default
circuit run reports `disrupts=0`, which is row 1 at scale and is what the floor exercises.

## Rows — my re-run

| # | row | result |
|---|---|---|
| 1 | ★★ rate 0 arms nothing | ✅ `draws=0`; circuit default `disrupts=0` |
| 2 | ★★ chaos is a rate | ✅ `hits=8` |
| 3 | ★★ the seed replays | ✅ same points; `disrupts=24` ×5 |
| 4 | ★ the invariant under chaos | ✅ `distinct=8000; dup=0` ×5 — **with a limit, below** |
| 5 | fresh peer threaded | ✅ no hang, no infinite `Closed` |
| 6 | process locus only | ✅ |
| 7 | 3c-pre's always-on poison gone | ✅ |
| 8 | scope | ✅ no `wat/service.wat`, no `src/`, no 3d |
| 9 | every outcome named | ✅ |
| 10 | the floor | ✅ **5213/5213, my run** |

## ⛔ ROW 4 — `dup=0` IS HALF ESTABLISHED, AND THE OTHER HALF CANNOT BE SEEN

The SCORE claimed *"Seen keyed on seq absorbed every redelivery."* The first half is proven. The
second is **not measurable with anything the system currently reports.**

```wat
(:wat::service::defservice :fanout::seen
  :durable   []            ← nothing is counted
```

`ClaimResponse::Dup` is returned, the worker maps it to `first? = false` (`circuit.wat:329`), and
**no counter anywhere records that an absorption happened.**

So `dup=0` under chaos establishes: **no message was double-counted.** It does **not** establish
that any redelivery occurred and was absorbed. Twenty-four severs may have interrupted claims in
flight — or may have landed on idle connections. **From every number we have, those two worlds are
identical.**

★ **That is R69's shape, one layer up.** R69: `distinct` keyed on `queue/envelope-id`, which a retry
*replaces*, so the detector could not witness a duplicate. Here: `dup=0` cannot distinguish
*"absorbed N redeliveries"* from *"there were none to absorb."* We fixed the key and inherited the
same blindness at the next level — an invariant credited with preventing something it cannot see.

The fix is cheap and the parts exist: `Seen` counts Firsts and Dups, the circuit reports them, and
`dup=0` becomes legible — *"24 severs → N redeliveries → all N absorbed."* **S30.**

**This does not un-strike the stone.** Chaos is a rate, it replays, and the invariant held under it.
But *"the dedupe was exercised"* is a claim neither of us can currently support, and it is the
claim that would make row 4 mean what we want it to mean.

## The disclosed deltas — all real, all correctly surfaced

- **`sqs.wat` gained `:max-frame-bytes 8192`** so a topic-worker disrupt tears instead of enqueueing.
  **My DESIGN's file list omitted `sqs.wat` entirely.** The executor found it and said so.
- **`held-worker` gained a zero-disrupts arm** — it `:satisfies :fanout::Worker`, so the op is
  mandatory. Structural, not optional.
- **The `Record` grew `max-draws`/`hits`/`draws`/`points`** so rows 1 and 3 could be *shown* rather
  than asserted. That is the instrument existing because the row demanded evidence.
- **3c-pre's always-on start poison is gone** — carry 3 honored.

## Throughput — S29 applied, not merely cited

Chaos, my five: `27455 / 27785 / 27947 / 27503 / 27848 ms`.

**I make no throughput claim from those.** S29 says my box drifts upward with session time while the
executor's does not, so **only a within-executor pair is valid** — and the executor took one:
rate 0 `26782–27226`, chaos `26253–27633`, **overlapping**. That is the comparison that counts, and
it says chaos costs nothing measurable at 200 bp. Mine are a different box's numbers and comparing
them to theirs is exactly the error S29 exists to prevent.

## Still open

- **3d** — the reply-drop. `None` → `LOST` proven userland; `wat/service.wat` still untouched. The
  only fault that produces the *unknowable* state, and the one that would finally make the dedupe
  path visible — **which S30 says we cannot currently see.**
- **Stone D2** — `accept!`, `face-start-tw`, `nap-ms`'s six homes, the `do-receive` merge.
- **Stone C** — `Alarm :delay`, `Milliseconds`. Last.
- **S15**–**S30**, newest: **S30** (nothing counts absorbed duplicates).
