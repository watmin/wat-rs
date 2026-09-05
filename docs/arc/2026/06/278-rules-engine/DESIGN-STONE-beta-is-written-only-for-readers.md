# DESIGN-STONE — a beta memory is written only if something reads it

> **Origin (builder, 2026-08-01):** *"wm.beta first - measure whether terminal beta is ever read."*
> Measured before designed, because the identical shape of claim — *"surely this store is
> redundant"* — was proposed for production-memory's freeze one session ago and **died on the disk**
> (derived facts live only there; the freeze IS the output). This stone exists only because the
> measurement came back the other way.

## What was measured

`beta_write_read_traffic` (`src/rete/kernel/tests/fanout_cost.rs`) counts, per node, tokens written into `wm.beta` and
tokens read back out. All six write sites in `fire_fixpoint_delta` are instrumented by a script that
**asserts it found exactly six**, so a seventh write path added later fails the edit rather than
silently under-counting. Both real read sites are instrumented; the two remaining `wm.beta` readers
are `#[cfg(test)]` census blocks.

```
fanout [100 x 20] — two conditions        node  written   read
                                             1     2000   2001   read      (root-join, parents J)
                                             3    40000      0   NEVER     (J, terminal)
                                                                  95.2% of writes never read

deep-cascade [10 x 100] — chained by       1,6,11…    100    101   read
DERIVED FACTS, not by beta                 3,8,13…    100      0   NEVER
                                                                  50.0% of writes never read

tri [10 x 5] — THREE conditions              1       50     51   read      (root-join, parents J1)
                                             3      250    251   read      ← J1, MIDDLE join
                                             5     1250      0   NEVER     (J2, terminal)
                                                                  80.6% of writes never read
```

The `+1` on every read count is the `.first()` join-key sample; the rest is the catch-up's
`all_left`. Both readers fire, on every reader.

## ★ THE ONE CONTRACT DECISION

> **A node's beta memory is needed iff that node is the PARENT of a `HashJoinNode`.**

Not "terminal joins don't need beta" — that phrasing is a workload observation. This is the
mechanism: `wm.beta` has exactly two readers, both in the hash-join first-keying path, and both read
**the parent** of the join being keyed (`.first()` for one sample token to derive the join keys,
`all_left` for the catch-up cross-join). A node no hash-join names as parent can never be reached by
either.

That makes the guard a **static property of the network**, derivable once at setup from the
`parent_of` map that already exists — not a heuristic, not a workload constant, and not something
that varies per fire.

The rule predicts all **16 measured nodes across three shapes** with no exception, including the one
case that could have refuted it.

## Why the third world exists (and why the first two were not enough)

Fanout and deep-cascade are both **two-condition** rules, so every hash-join in them is a leaf. A
rule about hash-join betas drawn from those alone would be generalising from a corpus containing no
counter-example — the exact class of claim this arc keeps retracting
(`feedback_ground_each_case_before_the_verdict`).

`TRI_CENSUS_WORLD` (three conditions → `root-join → J1 → J2`) produces a **middle** join. It is
wired as a hard assertion, not a print:

```rust
assert!(tri_reads >= 2,
  "a three-condition rule showed only {tri_reads} node(s) reading beta. Either the middle join
   J1 is NOT read — which kills the parent-of-a-HashJoinNode guard — or the network is not the
   shape this world intends. Do not draw the stone on this.");
```

J1 came back **read 251**. Had it come back 0, this document would not exist.

## The change

At setup, alongside `parent_of` / `feeding_alpha_of`, derive:

```rust
// A node needs its beta iff some HashJoinNode names it as parent — the only two readers of
// wm.beta both read the PARENT of a join being keyed for the first time.
let beta_readers: HashSet<i64> = /* parents of every HashJoinNode in the network */;
```

Guard the six write sites on `beta_readers.contains(&id)`. A guarded-out write skips a **Token
clone**, a map `entry()` lookup, and a `Vec` push — and leaves that many fewer tokens for
`round:drop-memories` to drop.

**Riding along, needing no hypothesis at all:** `wm.beta.entry(child_id)` and
`d_beta.entry(child_id)` are both called *inside* the per-token loop on an unchanging key — 80,000
map lookups on the fanout cell where 4 would do. Hoist both out of the loop. That is correct
regardless of how the guard lands.

## Expected, honestly

`hj:catchup:emit` is **7.2 ms** (40,000 Token clones + two map lookups each) and
`round:drop-memories` **4.6 ms**. The guard removes the clones and the pushes for write-only nodes,
and shrinks what drop-memories drops. It does **not** remove `d_beta` — production consumes that.

I am not putting a single figure on it. The last three predictions of this kind were wrong by 10×,
right, and right — and the two that were right were mechanisms, not extrapolations. Measure it.

## The gate

1. **The differential is the real gate.** `native == oracle` on every rete axis. If the guard drops
   a beta something actually reads, joins silently lose tokens — the failure is *wrong answers*, not
   a crash, so a green floor is the load-bearing check and not a formality.
2. **`beta_write_read_traffic` re-run**: every node still showing `read > 0` must still be written.
   The probe becomes the guard's own regression test — if a future node kind starts reading beta,
   it goes red instead of silently losing tokens.
3. **A three-condition rule must still fire correctly** — `tri` is in the floor now, not just the
   probe.
4. `:accuracy :match` on every grid axis; release floor unchanged; clippy 0.

## Out of scope = REJECTED (affirmative cuts)

- **Deleting `wm.beta` entirely.** It has two genuine readers. This narrows *who* is written, not
  whether the memory exists.
- **Touching `d_beta`.** Different store, consumed by production every round. Untouched.
- **The batch path** (`root_join_pass` / `hash_join_pass`, the P4a reference impl). It is the
  documentation-grade twin, not the hot path, and it is not what was measured.
- **`Token` as `Arc<Token>` to make the remaining clones cheap.** A real idea, a different stone,
  and one that only pays *after* this one establishes how many clones are left.
- **The `TokenBindings` promoting representation** (`NOTE-token-bindings-stays-a-trie.md`). Ranked
  below this deliberately: ~3 ms and a new type, versus ~12 ms and a subtraction.
