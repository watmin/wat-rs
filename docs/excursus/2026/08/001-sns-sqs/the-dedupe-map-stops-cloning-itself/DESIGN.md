# DESIGN — the dedupe map stops cloning itself

**`Seen`'s `claimed` becomes a `PersistentMap`.** `wat-scripts/fanout/circuit.wat` only.

The first stone in this run that makes something **faster** rather than measuring it. Its gate is
the curve.

## ⛔ WHY — `hashmap::assoc` clones the whole map, and `claimed` grows to 8000

`src/collection/eval.rs:367`, verbatim:

```rust
// Arc strategy: clone-then-new-Arc (functional; no aliased mutation; mirrors 216.5b).
let mut new_map: std::collections::HashMap<Value, Value> = (**m).clone();
new_map.insert(k.clone(), v.clone());
```

★★★ **Every `:wat::hashmap::assoc` copies the entire map.** And `Seen` marks one key per message
into a map that grows to `n × m` entries (`circuit.wat:178`, inside a fold):

```wat
(:wat::core::Tuple (:wat::hashmap::assoc claimed key true) (:wat::i64::+ recd 1) skip)
```

**O(N) per message ⟹ O(N²) per run:**

```
n=500    N=2000   avg map 1000  =   2,000,000 entry-copies
n=1000   N=4000   avg map 2000  =   8,000,000
n=2000   N=8000   avg map 4000  =  32,000,000        16× for 4× the work
```

★★ And `Seen` is **one service for all 12 workers** (`circuit.wat:2210`) — 3× the client count of a
queue — so this cost is paid through the narrowest actor on the hot path.

## ⛔ THE MEASUREMENT IT EXPLAINS

`is the queue saturated?` found ρ **falling** (0.574 → 0.516) while the biggest, fastest-growing
term was `not-in-queue` — time the queue is **idle** and the workers are elsewhere. Waiting on
`Seen` *is* "not-in-queue".

★★★★ A per-message cost linear in N fits the measured curve:

```
fit  not-in-queue/msg = a + b·N,  from n=500 and n=2000
     0.110 = a + 2000b ;  0.206 = a + 8000b   →   b = 1.6e-5, a = 0.078
predict n=1000:  0.078 + 4000·1.6e-5 = 0.142      observed 0.151
```

⚠ **A fit is not a proof** — nine mechanisms have died in this arc, several of which also fitted.
What makes this one different is that the mechanism is **read off the substrate source**, not
inferred from the shape of a curve, and the fix is a one-word type change whose effect is a
prediction the curve can refute.

## ⛔ THE ONE CONTRACT DECISION

`claimed` becomes `(:wat::core::PersistentMap :- [:wat::core::String :wat::core::bool])`, and its
three call sites move from `:wat::hashmap::` to `:wat::map::`.

`src/collection/eval.rs:594`:

```rust
// PMap::assoc returns a NEW map — Array copies the pair slice; Trie shares.
```

★ **Structural sharing instead of a clone.** The builder's own ruling on the two names
(`src/intrinsic/hashmap.rs:7-12`): *"the UNMARKED name goes to `PersistentMap`… the builder's move
to a persistent-backed default is 'probably a week or two' out, so `HashMap` takes the flavor-marked
home now"* — so `:wat::map::` is what the default will be. **This is opting in early, not working
around anything.**

⚠ `PersistentMap` carries the full API `Seen` needs — `get`, `assoc`, `contains-key?`, `length`
(`src/intrinsic/map.rs:60-183`) — so the change is a type and three prefixes.

## THE GATE — the curve, and it can refute the model

**`drain`/pair growth from n=500 to n=2000 must fall below the +65 % measured today.** If it does
not move, the clone was not the term and the model is dead — which is a result, not a failure.

⚠ **No claim that it goes flat.** The fit says the linear-in-N term is ~60 % of the growth at
n=2000; the rest is elsewhere. A partial win is the honest expectation.

## OUT OF SCOPE — REJECTED, and each is a real instance

The census found the same defect elsewhere. **None is fixed here**, so this stone's gate stays clean:

- **`circuit.wat:2075`** — the `distinct` fold, also growing to 8000, in the **`collect`** phase.
  Same defect, different phase; `collect` is already its own open item.
- **`circuit.wat:2082`** — the workers fold. Bounded by 12; harmless.
- **`wat/query/mem.wat:165 :182 :193 :202 :224 :237 :262 :275`** — the mem-store's indexes grow with
  every row. **Same O(N²) shape**, in the stdlib, in the differential oracle. Not this stone's
  blast, and `wat/` changes are the builder's call.
- **`wat/telemetry/span.wat`, `wat/rete/compile.wat`, `wat/deporder.wat`** — bounded by span names,
  rule count, file count. Correct as they are.
