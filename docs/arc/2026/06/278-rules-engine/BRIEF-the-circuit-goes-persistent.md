# BRIEF — the circuit goes persistent

Two accumulators in the circuit are `:wat::core::Vector`, whose `conj` is O(n). Move both to
`:wat::core::PersistentVector`, whose `conj` is amortised O(1). Then run the circuit and report what
the drain actually does.

## Read in order

1. **`DESIGN-STONE-the-circuit-goes-persistent.md`** — the numbers, the scope ruling (container
   only, not the cursor), and the one contract decision.
2. **`wat-scripts/scratch-pad/probe-outbox-strategies.wat`** — the three strategies already measured.
   **Strategy B is exactly what you are building.** Copy its shapes; do not re-derive them.
3. **`wat-scripts/topic/sns-fanout.wat:86`** (`outbox <-` declaration) and **`:246`** (the rebuild
   `foldl`). Every `State` construction that carries `:outbox` changes type with it — find them all.
4. **`wat-scripts/fanout/circuit.wat:90`** and **`:172`** — `:fanout::worker`'s `outcomes`, and the
   `-tick` that conjes onto it.
5. **`wat/query/mem.wat:162`** — `rows <- (PersistentVector :- [StoredRow])`, a service that already
   does this correctly. The exemplar for how the type threads through `:durable`/`:ephemeral`.

## The sketch

Load-bearing: the container type and facing the `Option`. Illustrative: everything else.

```wat
;; declaration
outbox <- (:wat::core::PersistentVector :- [:wat::core::String])

;; the rebuild, unchanged in shape — only the verbs and the Option
rest (:wat::core::foldl
       (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::String])
                        i   <- :wat::core::i64]
         -> (:wat::core::PersistentVector :- [:wat::core::String])
         (:wat::vector::conj acc
           (:wat::core::Option/expect (:wat::vector::get box (:wat::i64::+ i 1))
             "topic -deliver: rebuild index in range by construction")))
       (:wat::core::PersistentVector :- [:wat::core::String])
       (:wat::core::range 0 (:wat::i64::- (:wat::vector::length box) 1)))
```

`:wat::vector::empty?` / `length` / `get` / `conj` replace the `core::` forms on that value.

## Blast radius

`wat-scripts/topic/sns-fanout.wat` and `wat-scripts/fanout/circuit.wat`. Type changes and verb
swaps only — no new fields, no new ops, no logic change. **`wat/`, `src/` and `sqs.wat` untouched.**

## STOP triggers

1. **If `total=8000; distinct=8000; dup=0` breaks — STOP.** A container swap must not be observable
   in behaviour. If it is, something else depended on `Vector` and that is the finding.
2. **If you need the cursor to make a row pass — STOP.** It is cut in the DESIGN with its number.
3. **If `wat/` or `src/` need to change — STOP and surface it.**
4. **If the drain does not improve — STOP and report it, do not chase it.** That would mean the
   isolated 30.7 s does not transfer, which is a finding worth more than the stone, and this
   campaign has had four of exactly that shape today.

## Shape to copy

`SCORE-the-wakeup-is-level-triggered.md` for a scale matrix with named drivers.

## Floor

`./scripts/floor.sh`. **Read the Summary line, never a piped exit code.** A red is a red — do not
re-run, name the arm, surface it. Check `ps` for a running `wat`/`cargo` before any timing.

Write `SCORE-the-circuit-goes-persistent.md` when done. It will be graded by re-running.
