# BRIEF — depth is read, not counted

Delete the queue's two hand-maintained depth counters and answer every depth question by
counting the index the queue already maintains. `wat-scripts/queue/sqs.wat` only.

Read `DESIGN-depth-is-read-not-counted.md` first — it carries the why and the one contract
decision. `wat-scripts/scratch-pad/probe-depth-derived-from-the-index.wat` is the worked
mechanism; copy its `count-in-range` shape rather than inventing one.

## ⛔ THE SHAPE — a CLOSURE, not a top-level defn

**The circuit starts every queue at PROCESS locus** (`circuit.wat:1198, 1208, 1512, 1558,
1591` — all `:wat::spawn::process/post-spawn`), and **a process child does not see sibling
top-level `defn`s.** `sqs.wat:152-153` says so in its own words and is the reason `take` exists
as a closure at all:

> *"Closed over `store`. The one receive path — process children do not see sibling defns, so
> the body lives here, called via `State/take`."*

So the depth helper lives **inside the `:init` closure beside `take`**, is carried as an
`:ephemeral` field, and is called through `State/`. A top-level `(:wat::core::defn :queue::depth …)`
type-checks, passes at thread locus, and dies at process locus — which is every circuit queue.

⚠ **The committed probe counts from the DRIVER at THREAD locus.** It proves the two range scans
return the right numbers; it does **not** prove the position. What proves the position is `take`
itself: it already calls `Store/scan-index` from inside an arm at process locus, in production,
today. Copy that, not the probe's placement.

## READ IN ORDER

| room | why you are there |
|---|---|
| `sqs.wat:111-130` | the `defservice`. `visible` and `unacked` are `:ephemeral` (`:127-128`) — **delete both**; add the depth closure field beside `take` |
| `sqs.wat:152-200` | `take` — the closure pattern to copy, and the scan-index call shape at `:161-166` |
| `sqs.wat:243-247` | `:init` — drop `:visible 0` / `:unacked 0`, build the new closure |
| `sqs.wat:253-262` | **the cap gate.** `depth` is the sum of the two counters; it becomes a derived count |
| `sqs.wat:431-441`, `:460-470` | the two `Full` responses — both report `visible + unacked` as depth |
| `sqs.wat:483-501` | `take`'s accounting — `p1`/`f1`, the `+n` that never balances |
| `sqs.wat:600-616` | the ack path — `f1 = if f0 <= 0 then 0 else f0 - 1`, the clamp that admits the counter goes negative |
| `sqs.wat:701-711` | `stats` — the surface the drain reads |
| `sqs.wat:296-320`, `:378-411`, `:645-680`, `:773-780` | pure carry-forward of the two fields through `State` reconstruction; they vanish with the fields |

31 `State/visible` / `State/unacked` sites total. Most are carry-forward.

## SKETCH

Inside `:init`, beside `take`:

```wat
depth (:wat::core::fn
        [st <- (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])
         q <- :wat::core::String  now-ns <- :wat::core::i64  lim <- :wat::core::i64]
        -> (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])
        ;; (visible unacked): count [0, now], count [0, +inf), subtract.
        ;; lim is cap+1 at every call site — see the contract decision.
        …)
```

Every former read becomes a call through `(:queue::queue::State/depth s)`.

## BLAST RADIUS

`wat-scripts/queue/sqs.wat` only. No `wat/`, no `src/`, no `circuit.wat`, no codemod, no new
Store surface verb.

## STOP TRIGGERS

- **STOP-1** — if the depth helper cannot be reached as a closure through `State/` and you find
  yourself reaching for a top-level `defn`, STOP. That is the process-locus asymmetry above and
  it will pass every thread-locus test and die in the circuit. Surface it; do not work around it.
- **STOP-2** — if counting requires cursor pagination (a scan at `limit = cap + 1` returning a
  `Some` cursor in normal operation), STOP. The contract assumes depth is cap-bounded; if it is
  not, that is a finding about the cap gate and changes the stone.
- **STOP-3** — if any call site cannot supply `now-ns`, STOP and report which. Depth is a
  question about a moment; a site with no moment is a site whose semantics we have not settled.
- **STOP-4** — if removing the fields forces a change in `circuit.wat` or `wat/`, STOP. The
  blast radius is one file, and a spill means the seam is not where this brief assumed.

## PRIOR RESULT TO COPY FOR SHAPE

`SCORE-the-reactor-grows-a-seam-v3.md` — same campaign, same discipline: a mechanical change
whose whole burden of proof is "nothing else moved."
