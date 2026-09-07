# SCORE — the queue reports time inside its store

**SCORED. Parse not green.** Executor: grok, 2026-09-07. Tree dirty.
The instrument is sketched in `sqs.wat` + `circuit.wat` + six ripple
files. It does not load.

## WHAT LANDED IN THE TREE

- `store-ns` last on `StatsResponse::Ok` and on `:ephemeral`, init 0.
- Clock pair around `take`'s scan-index and put; `take` returns
  `(Tuple peer (Tuple envs ns))`.
- Clock pair around `depth`/`count-hi` (two count-index) and `total`.
- Clock pair around send `Store/put`, ack `Store/delete`,
  `retry-put` / `retry-delete` (those two now return `(Tuple count ns)`).
- `sum-store-ns` + phases `store-ms=` (divide by 1e6 at format).
- Eight ripple `_` last on the six extra files.
- Receive applies `take-ns` onto `store-ns`.

## THE WALL — waiter folds

`take` returning ns requires the three waiter folds (send, `-tick`,
`send-after-put`) to accumulate a second i64. Tuple has no fourth
accessor, so the existing third slot (`taken`, a store-call count)
becomes `(Tuple sc ns)`.

That rewrite is **not paren-stable across the three copies**. The send
fold was forced to parse by reducing closes on the surrounding send
match (`Continue` 6→4, `RequestMalformed` 4→2, TimedOut one-liner
8→2). Those reductions are **send-arm local**. The same 8+2 recipe on
`-tick` and `send-after-put` does not transfer: their enclosing depth
is different (`SelfOutcome` / a top-level `defn`).

Last parse errors, iterating:

- `-tick` foldl: `UnclosedParen` at `:985` with Continue=4;
  `UnexpectedRParen` at Continue with Continue=5. No integer works.
- `send-after-put`: `UnclosedBracket [acc` at `:1288` with Continue=4;
  `UnexpectedRParen` at Continue with Continue=5/6.

`send-after-put`'s fold was reverted to i64 `taken` so that arm would
not block; **its waiter takes would not add `store-ns`**. `-tick` still
has the Tuple acc and does not parse.

n=12 did not run. No curve, no floor.

## WHAT THE REDRAW MUST CARRY

Do **not** nest `(Tuple sc ns)` into the waiter-fold third slot.

Options that keep the clock pair *inside* `take` (trap-door: do not
time the helper from the outside):

1. **A second fold accumulator** returned beside the existing 3-tuple,
   nested as `(Tuple peer (Tuple keep (Tuple box (Tuple sc ns))))` —
   one extra nest, designed as a paren-counted rewrite of **one** fold
   then copied, not three independent edits.
2. **Keep third as `sc`.** Thread `ns` by changing the fold's *outer*
   acc to `(Tuple old-acc ns)` so the existing inner 3-tuple is
   untouched. One extra `first`/`second` at the fold boundary.

`Topic::StatsResponse` stays `[n ticks]`. No `wat/`. `store-ns` still
last. `store-ms=` still nanos-on-the-wire, millis at format.

STOP-6 (store-ms > drain) was not reached. It is still expected: the
counter is whole-run (fill puts + drain). Report both; do not "fix".

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ★ `store-ms=` on phases | not reached. does not load |
| 2–7 | bounded / monotonic / 8 of 8 / curve / no-args / n=2000 | not reached |
| 8 | extra files `_` only | ✅ `git diff -- topic scratch-pad` is trailing `_` |
| 9–11 | drop / scripts / floor | not run |

**STOP-1 did not fire** (15 Ok sites). STOP-2–6 not reached.

No-args baseline captured *before* edits (HEAD, no instrument):

```
total=8000;distinct=8000;dup=0;seen-skipped=0;check-exhausted=0;mark-exhausted=0;ack-retries=0
queue-receive-calls=4937 store-calls=37634 drain=102
```

---

# GRADING — claude, 2026-09-07

**NOT STRUCK — the strike failed to PARSE, not to design.** I restored the tree myself (it was left
dirty and non-loading); `sqs.wat` now parses and runs. Only this SCORE is untracked.

## The root cause, confirmed independently — and it is not the file the builder named

```
:wat::core::third    used at wat/sqlite.wat:82, wat/fix.wat:376        — exists
:wat::core::fourth   ZERO occurrences anywhere in wat/ or wat-scripts/  — does not
sqs.wat:1030         145 characters wide; the tree's UnexpectedRParen landed at col 145
```

★★★ **`Tuple` tops out at three.** Threading a fourth value through sqs.wat's three waiter folds
forced `(Tuple sc ns)` into the third slot, which shifts paren depth at every use site. grok's own
words: *"UnclosedParen with Continue=4; UnexpectedRParen with Continue=5. **No integer works.**"*

⚠ **The parse wall was in `sqs.wat`, not `circuit.wat`.** `:985` and `:1288` are sqs.wat lines. My own
corpus measurement had ranked `circuit.wat` first (204 lines ≥50 cols vs sqs.wat's 33) — **the metric
was wrong.** Sustained indentation is not what breaks an edit; a **145-column line whose paren depth
must change** is. sqs.wat's max indent is 100 vs circuit's 92, and it holds the folds.

## ⛔ THE DEFECT CLASS — threading one more value through a fold requires paren surgery

That is the thing to remove, and it is removable. Probed this session
(`probe-a-fold-accumulator-can-be-a-struct.wat`, committed, green):

| aggregate | holds a `Peer`? | why |
|---|---|---|
| `Tuple` | yes | but **arity 3**, so a 4th value must nest — the paren surgery |
| `defrecord` | **NO** | `ImpureFieldInPureAggregate` (arc 293.W): *"a record or holon holding a struct field could never cross — it must not exist"* |
| **`defstruct`** | **YES** | the impure-capable aggregate; **stays in shared memory and never crosses** — exactly what a local fold accumulator does |

★★★★ **A `defstruct` accumulator with named fields removes the class.** Adding a fifth field changes
**nothing at any use site** — no nesting, no paren depth change, no re-balancing by trial.

★ And it is the same lesson as `the outcome crosses, the resource stays`, arriving from the other
side: the aggregate that may hold a live resource is precisely the one that never crosses.

## What the builder observed, and what it actually was

> *"grok is making closing bracket mistakes often now… we need to refactor the circuit file to have
> less massive exprs."*

**The instinct is right and the target was one file off.** Measured across the corpus:

```
204 lines ≥50 cols, max  92   circuit.wat
200 lines ≥50 cols, max 105   wat/service.wat   (the defservice macro — metaprogramming)
 33 lines ≥50 cols, max 100   sqs.wat           ← where the strike actually died
 ≤28                          everything else
```

⚠ Deep nesting is the **amplifier**. `Tuple` arity is the **cause**. Refactoring `circuit.wat` would
not have saved this strike.

## Rows

| # | how I checked it | result |
|---|---|---|
| 1–7, 9–11 | tree does not load | — not reached |
| 8 | `git diff -- topic scratch-pad` was trailing `_` only | ✅ (now reverted with the rest) |

**STOP-1 did not fire** — 15 `Ok` sites, as briefed. The wall was elsewhere.

## ⛔ WHAT I OWE THIS STRIKE

My BRIEF said *"`store-ns` rides every arm that already carries `store-calls`"* and listed the eight
`Store/*` sites. **It never asked whether the accumulators could carry another value.** `store-calls`
fitted because it replaced an existing i64 slot; `store-ns` needed a fourth. I briefed the edit
without checking the shape the edit had to fit into — the same class as the blast-radius miss two
stones ago, one level down: **I checked what I was touching, not what had to hold it.**

## THE REDRAW

`DESIGN/BRIEF/EXPECTATIONS-the-fold-accumulator-is-a-struct.md`. It carries the instrument unchanged
and adds the enabling change first: **the three waiter-fold accumulators become a `defstruct`.** That
is also, directly, the "less massive exprs" the builder asked for — applied where the massiveness
actually bites.

---

# CORRECTION — the builder, 2026-09-07

> *"make a struct or record — **this is on purpose, wide tuples are unwieldy**. structs may hold
> impure bindings, records may only hold pure data bindings (must be edn expressible, a file handle
> or socket is not edn expressable)"*

**My GRADING above called this a "defect class." It is not.** `Tuple`'s three-element limit is a
deliberate constraint, and the aggregate split is the design:

| | holds | why |
|---|---|---|
| `Tuple` | ≤3 of anything | wide tuples are unwieldy — the limit **is** the push toward a named aggregate |
| `defrecord` | pure, **EDN-expressible** fields only | it must be reconstructible across a comms boundary |
| `defstruct` | impure bindings too (a `Peer`, a socket, a file handle) | it never crosses |

★ So the language was **telling** sqs.wat to use a struct, and the code fought it with nested
Tuples. The failure was ours, not the substrate's. Every one of my three probes this session
re-derived a rule that already had a reason.

## THE CENSUS THAT FOLLOWS FROM IT

Where does the corpus fight the three-element limit?

```
nested-Tuple ACCESS chains   (first/second of a first/second)
    36   wat-scripts/fanout/circuit.wat
     2   wat/service.wat
     0   everything else

lines CONSTRUCTING 2+ Tuples
    33   wat-scripts/fanout/circuit.wat
     5   wat-scripts/queue/sqs.wat
    ≤4   everything else
```

★★★★ **69 sites in `circuit.wat`, and the rest of the corpus is clean.** This is a far better
metric than the sustained-indentation one I used earlier — which ranked files by a symptom and put
`sqs.wat` third in the very strike it killed.

★★★★★ **And it merges the builder's two observations into one problem.** The mega-expressions are
not arbitrary sprawl: they are the *shape* of packing four-plus values into three-slot Tuples.
Replace the packing with named aggregates and the nesting collapses with it — and the "less massive
exprs" ask and the `store-ns` blocker have the same fix.

⚠ The metric is also the acceptance test: **count nested-Tuple sites before and after.**
