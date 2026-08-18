# DESIGN-STONE — the keyed gather: Accumulate / Negation / Exists get the index the joins already have

> **2026-08-17 — this is cut 2, not started.** Live breadcrumb:
> **`CURRENT-STATE-annihilate-interpretation.md`**. Cut 1 (dirty,
> uncommitted) already moved fact-shaped `:exists` / `:not` onto
> **alpha** on both mouths. This stone keys that same bag.
>
> The **"no `.wat` changes"** blast radius below applies to the
> **key**, not the bag. Cut 1 already edited `wat/rete.wat` so the
> oracle reviews the right bag. Cut 2 is native-only over that
> alpha; the oracle stays a linear fold (`OCVLI NOVI, ORACVLVM
> IMMOTVM`). Line numbers in this file are stale; the algorithm
> and the three contract clauses (order, empty-bucket, empty
> `join_keys` → cartesian) are not.
>
> After cut 1, accum 200×200 native fire is **215 ms / filter 47%**
> (was 1.83 s / 94%). That 47% is ExistsNode over `|alpha|`.
> Accumulate `:from` gets the same index but is **not** the current
> wall (`accum:fold` 9 ms). The 2026-07-31 numbers below are
> pre-cut-1.

> **Origin (2026-07-31):** the Clara grid re-measure inverted three axes — `accum` is a decisive LOSS
> (Clara ~19×), `min-finding` a loss (~6×), `negation` a coin flip — all three recorded as wins. The
> root is one defect class, measured this session: **the Accumulate and Negation/Exists gathers scan
> the full cumulative alpha memory once per token, while the join nodes are keyed.** P6 keyed the
> joins and never came back for the gathers.

## The measurement that grounds this (probe on disk, re-runnable)

`wat-scripts/scratch-pad/probe-accumulate-gather-cost.wat` — the `accum` axis copied byte-for-byte
modulo namespace, with five instants (build / compile / seed / fire / derive). Run under the memory
guard, ladder upward. The load-bearing sweep holds the fact count CONSTANT (8000 Readings exactly)
and varies only the group count, so "the gather is quadratic" is separated from "there are more facts":

```
 G    W     seed-ms    fire-ms   derive-ms   count
 50  160     117.77     149.28       6.71    250 ✓
100   80     118.42     275.05      15.39    500 ✓
200   40     119.04     551.25      30.90   1000 ✓
400   20     117.54    1256.56      63.48   2000 ✓
```

- **`seed` is flat to within 1.3%** — the control validates itself; the workload really is constant-size.
- **`fire` rises 8.42× across an 8× rise in G.** Predicted for an un-keyed per-token scan: ~8×.
  Predicted for anything keyed: flat. It is not flat.
- **Non-vacuity holds at every rung** (`derived-count == expected-count == 5·G`).
- The cost model — 4 AccumulateNodes × G tokens × 8000 elements — predicts the whole curve with ONE
  constant, **~86–98 ns per compatibility check** (±15%; the model omits the ExistsNode, whose
  `.any()` short-circuits, so the true per-check cost is somewhat lower). This is not merely "grows
  with G"; the mechanism accounts for essentially all of `fire`.
- Scale sweep (W fixed at 20): seed ~2× per doubling (linear), derive ~2× (linear), **fire ~4× per
  doubling (O(G²))**.

**The instrument's boundary, stated:** at G=400 `fire` is 1256ms of ~1438ms measured — **87%**. So
unlike node-share (where fire was 1.4% and the grid's verdict was about a sliver), the grid's
fire-only window is HONEST for this axis. The `accum` loss is a genuine engine loss.

## The defect, grounded to file:line

**Accumulate pass — `src/rete/kernel.rs:1939-1952`:**
```rust
let from_elements: Vec<Value> = wm.alpha.get(&from_alpha_id).cloned().unwrap_or_default();
for tok in new_tokens {
    let gathered: Vec<&Value> = from_elements.iter()
        .filter(|el| token_element_compatible(&tok.bindings, element_fact_bindings(el).1))
        .collect();
```
O(|new_tokens| × |from_elements|) per node per round, no index.

**Filter pass — the Negation/Exists twin (same file, the `filter_elements` snapshot + a per-token
`.any()`):** identical shape. The `.any()` short-circuits on the first compatible element, so the
worst case is exactly the case each node exists to detect: a **Negation token that PASSES** has no
compatible element and therefore scans the whole memory. Exists short-circuits on a hit but still
averages a large fraction of the memory when matches are not front-loaded.

**The asymmetry:** `keyed_join` (`:779-831`) indexes the right side by a join-key tuple —
`HashMap<Vec<Value>, Vec<usize>>`, `join_keys` = the sorted intersection of token/element binding
keys (`:785-801`) — and P6 persists per-node `left_idx`/`right_idx` (`:1542-1544`) with `key_of`
(`:1203`). The gathers never got it. This is R24's `merge_facts` shape again: a linear scan where a
hash lookup belongs.

## The algorithm

Per AccumulateNode / NegationNode / ExistsNode, **once per round** (not per token): build a
`HashMap<Vec<Value>, Vec<usize>>` over the node's cumulative `:from` / tested elements, keyed by the
shared-variable tuple, exactly as `keyed_join` step 2 does. Then each token probes with `key_of` in
O(1) and folds/tests only its own bucket.

`O(G·N)` → `O(N + G)`. At G=400/W=20: **12.8M compatibility checks → ~32K index inserts + 400 probes.**

**Why the key is exactly right (not merely close):** `token_element_compatible` (`:730-743`) iterates
the ELEMENT's bindings and requires equality on any key the token also has — i.e. agreement on the
INTERSECTION of the two key-sets. Key-tuple equality over `join_keys` (that same intersection) checks
exactly the same keys. The predicates are equivalent, not approximately equal.

## ★ THE ONE CONTRACT DECISION (pin this exactly; three clauses, all load-bearing)

**A keyed gather returns the same elements, in the same order, including when the bucket is absent.**

1. **Order is preserved.** Buckets hold element *indices* pushed in alpha-memory iteration order, and
   the gather reads them in that order. The current code gathers "in alpha-memory insertion order
   (matches the wat foldl over from-els)" (`:1944-1945`) and the native kernel is differential-tested
   bit-for-bit against the wat oracle. A reordered gather is a silent oracle divergence, not a perf win.
2. **A missing bucket yields an EMPTY gather — never a skipped token.** `accumulate_value` on an empty
   gather is meaningful: `count`/`sum` emit their identity, `min`/`max` return `None` and the token is
   dropped. Writing `if let Some(bucket) = index.get(&key)` and continuing on `None` would silently
   drop every `count`/`sum` token whose group has no readings. The probe on this axis would NOT catch
   it (W≥1 always). Use `index.get(&key).map_or(&[][..], |v| v.as_slice())`.
3. **Empty `join_keys` degenerates to the current behaviour, correctly.** With no shared variables
   every element keys to `[]` and every token probes `[]` — one bucket, all elements, i.e. the
   cartesian case. No special-casing; it is the same answer at the same cost as today.

## The RED gate — a census, not a wall

**Do not gate on wall-clock.** A wall is timing-flaky, and yesterday a wall drawn over the cheap
container passed *before the fix existed*. Gate on the mechanism itself.

Extend the existing `a8_node_share_fire_census` pattern (`src/rete/kernel.rs`, `#[cfg(test)]`,
thread-local, records nothing unless armed) with a **gather-visit counter**: increment once per
element examined in the accumulate/filter gathers. Then:

> Run the constant-N workload at **G=50/W=160** and **G=400/W=20** (identical element counts, 8×
> apart in group count) with the census armed. Assert the two visit counts are within a small
> constant factor of each other.

Today that assertion fails by ~8×. It can only pass if the gather is genuinely keyed — a scan cannot
fake it. Deterministic, timing-free, and it names exactly what would have to break to turn it red.

**Plus the standing anchor:** the native==oracle differentials must stay green. The wat oracle does
not move (R22 `OCVLI NOVI, ORACVLVM IMMOTVM`), so any semantic drift — order, empty-bucket, key
derivation — surfaces there. That net is why this rewrite can be done aggressively.

## Blast radius

- **`src/rete/kernel.rs` only.** The accumulate pass (`:1904-1961`), the Negation/Exists filter-pass
  gather, one shared index helper, and the `#[cfg(test)]` census counter.
- **No `.wat` changes. No corpus migration. No codemod.** The wat oracle is untouched by ruling.
- The `join_keys`-from-samples derivation is **inherited** from `keyed_join`, not newly introduced:
  it assumes all elements at one alpha node share a binding key-set (true — one alpha memory holds
  one condition's elements) and likewise for tokens at one beta node. The joins already rest on this
  guarantee; the gather rests on the same one. No new risk is taken.

## Out of scope = REJECTED (affirmative cuts, not deferrals)

- **Persisting the index across rounds** (P6-style `left_idx`/`right_idx` for gathers). Rejected for
  this stone: a per-round index is already O(N) and delivers the ~400× at G=400; persistence adds
  invalidation complexity for a second-order win. If a later measurement demands it, that is its own
  stone with its own number.
- **Touching `wat/rete.wat`.** The oracle stays unmoved — it is the anchor this is checked against.
- **Changing `token_element_compatible`'s semantics.** The predicate is correct; only how often it is
  called is wrong.
- **Strike 2b (the seq-traversal verbs).** Four real quadratics in the substrate, but `wat/rete.wat`
  and the grid axes call **zero** of them — it cannot move the Clara number. Separate work,
  deliberately not bundled here. *(It was tracked in `SEAM-2026-07-31.md`, a dated seam since pruned
  — the breadcrumb is now the single `SEAM.md`, and the text is preserved in git history. The claim
  above stands on its own: the "zero callers" count is checkable against `wat/rete.wat` and
  `wat-scripts/perf/grid/` today, and does not depend on the deleted note.)*
- **Re-measuring the whole grid.** Its own step, after this lands, and it needs a phase-split or
  CPU-time column first or the verdict will mean as little as the last one did.

## Sequencing

1. Extend the census with the gather-visit counter; land the RED gate (fails ~8×).
2. Key the Accumulate gather; the gate's accumulate half goes green.
3. Key the Negation/Exists gather (same helper); `negation` and `min-finding` follow.
4. Weigh: the census gate, the full native==oracle differentials, and the `--release` floor — by my
   own re-run, never a rider's report.
5. Re-run this probe's constant-N sweep and record the before/after in the seam.
