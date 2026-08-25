# NEXT STRIKES — the theater hunt (all of rete)

> **Origin (2026-08-23).** Builder: *"whatever pattern matching you are doing…
> do it everywhere… there is not much left… you should be able to find them
> all… the hunt is on.. build a list… then we attack"* and *"the physics are
> our welcoming hand — there's cruft between us and physics, we are on a
> crusade to annihilate this cruft and find the physics boundary."*
>
> This file IS the work list. One strike at a time, in order. A strike that
> lands gets its own DESIGN-STONE + BRIEF and a `Weigh` block; then it is
> struck from here. Nothing is deferred out of this file — it ships or it is
> affirmatively cut with the reason written down.

## The ground this was hunted on

HEAD **`a58f9dda`**. Floor **GREEN** `.floor/2026-08-23T21-23-28Z` — 4927
passed, 19 skipped, 275.284s, no ARM. Clippy CI-identical
(`--release --workspace --all-targets -- -D warnings`) **silent, exit 0**.
Grid `GRID-native-vs-clara-2026-08-23T21-28-42Z.txt`
(`GRID_SKIP_ORACLE=1 GRID_RUNS=3`): **30/30 `:match`, 30/30 `:us`**.

### ⚠ THE INSTRUMENT'S NOISE FLOOR — measured, not assumed

`T21-28-42Z` and `T19-17-35Z` are **the same HEAD with no code between them**.
The difference between them is therefore the *instrument*, not the engine:

| cell | T19-17 | T21-28 | delta |
|---|---:|---:|---:|
| fanout `[40000]` | 24.81 | 24.72 | −0.10 |
| accum `[200 200]` | 13.65 | 13.44 | −0.21 |
| deep-cascade `[50 100]` | 10.00 | 10.13 | +0.13 |
| strat-neg `[6 2000]` | 13.21 | 13.60 | **+0.39** |
| strat-neg `[6 500]` | 3.91 | 3.21 | **−0.71** |
| min-finding `[500 3]` | 0.80 | 0.53 | **−0.26** |

**A grid delta under ~1% on a big cell, or under ~0.3 ms on a small one, is
noise and proves nothing.** This is why every stone in this chain weighs on
the same-session leftover `Instant` harness (`fanout_three_leftover_split`,
`accum_alpha_leftover_split`, `harvest_wrap_parts`, `a0_depth_cost_split_at_equal_work`)
before/after in ONE session, and cites the grid only for accuracy + rank.
Do not gate a sub-ms intern on the grid.

### Live leftover at this HEAD (measured this session, mean of 3)

`fanout_three_leftover_split` [100 20], instrument 152.1 ns per mark pair:

| lump | ms |
|---|---:|
| without-query FIRE | 23.96 |
| with-query FIRE | 30.50 |
| delta (A candidate) | 6.54 |
| **harvest:query** | **6.89** |
| compiled-rhs net (40000×) | 2.01 |
| out:production | 0.00 |

`harvest_wrap_parts`, 40k one-entry maps: C clone 2.91 · **R `Arc::from([pair])` 3.01**
· I fetch_add 0.21 · W from_pairs 7.26 · W−C 4.34.

---

## TIER 1 — confirmed theater, hot path, clear cut

### T1 — `merge_facts` rebuilds the whole `present` set every stratum — ✅ **LANDED 2026-08-23**

> Struck. `DESIGN-STONE-strat-merge-carried-set` + `BRIEF-strat-merge-carried-set`.
> strat-neg `[6 2000]` **13.1738 → 11.7679 ms (−1.406, −10.7%)**; `[6 1000]`
> −0.740; `[6 500]` −0.329. Probe `strat_merge_present_parts` named 2.23 ms of
> theater before the cut. 358/358 rete cohort, 7strat 3/3, `:accuracy :match`.

**Site:** `src/rete/kernel/fire/mod.rs:1183`, called from
`src/rete/kernel/fire/rules.rs:202` **inside the stratum loop**.

```rust
// mod.rs:1183 — inside merge_facts, called once PER STRATUM
let mut present: std::collections::HashSet<Value> = pv.iter().cloned().collect();
```

```rust
// rules.rs:202 — the loop
acc_facts = merge_facts(&acc_facts, &new_derived);
```

**Why it is theater.** `acc_facts` is the entire accumulated closure so far.
Every stratum re-hashes **and Arc-clones every fact accumulated to date**, to
re-learn a set the previous iteration already held and threw away. The cost is
O(S·N) deep structural hashes of `Value::Aggregate` where the honest cost is
O(N). The doc-comment above it correctly killed an O(n²) `.any()` scan; the
set it introduced is still rebuilt from scratch each call. This is the seed
`(0..n).collect()` enemy (`DESIGN-STONE-seed-d-alpha-range`) at a higher tier:
reconstructing knowledge already computed.

**The cut.** Hoist `present` out of the stratum loop; carry it alongside
`acc_facts`:

```
merge_facts_into(&mut pv, &mut present, derived)
```

**★ THE ONE CONTRACT DECISION.** *The membership set is carried across strata,
not rebuilt per stratum.* Same value-dedup semantics, same `push_back` order,
same output Value. Dual-impl WHAT unchanged — the oracle reads
`(:wat::rete::Session/facts fired)` and must see byte-identical facts.

**Gate.** (1) strat-neg drops by a named amount above the grid's noise floor;
(2) 7strat 3/3 including three-stratum; (3) `spec_equals_native_on_every_where_family`
green; (4) clippy `--lib -D warnings`.

**Predicted win (written first).** strat-neg `[6 2000]` is the only cell with
enough strata × facts to show it: **−1 to −3 ms**. Single-stratum axes
(fanout, accum, negation) unchanged — they call `merge_facts` once, where
rebuild == build.

**Blast radius.** `fire/mod.rs` `merge_facts`, `fire/rules.rs` stratum loop.
No `.wat`. No Session field. No `AlphaDelta`/`BetaMemory` type change.

**Out of scope = REJECTED.** Session-Vec. Skip freeze. Hashing facts by id
instead of value (changes dedup semantics — R18 is value-dedup, NOT concat).
Making `acc_facts` a native `Vec` in the frozen Session.

---

### T2 — the Exists leaf still memcpys occupancy — ✅ **LANDED 2026-08-24 (in `71d0e700e`)**

> Struck as part of the leading-filter correctness strike, because it was the SAME
> block. `.map(|v| v.as_ref().clone())` → `.cloned()`, an Arc bump — the contract
> already proven for catch-up, applied to its last unconverted sibling. The probe
> this entry demanded was never needed in the end: the change rode a CORRECTNESS
> gate (`probe_arc278_leading_filter_multiplicity`), not a perf claim, so no
> unmeasured win is asserted here. The memcpy is gone; no ms are claimed for it.

**Site:** `src/rete/kernel/fire/delta.rs:1250`.

```rust
let els: Vec<Element> = wm.alpha.get(&alpha_id)
    .map(|v| v.as_ref().clone())     // ← full Vec<Element> memcpy
    .unwrap_or_default();
```

**Why it is theater.** This is the *identical enemy*
`DESIGN-STONE-catchup-arc-occupancy` already killed one function over —
*"Catch-up holds occupancy by Arc, not by memcpy… asking occupancy who sits in
the leaf by copying the answer."* It was left standing at the Exists sibling
site. `els` is read-only; the clone exists solely to release the `wm.alpha`
borrow before `span_from_row(&mut wm.bind_pool, …)` mutably borrows `wm`.
An Arc bump releases the borrow without copying the bag.

**The cut.** `wm.alpha.get(&alpha_id).cloned()` (Arc clone), iterate
`els.iter()`. `Element` is `Copy`; the walk is WHAT.

**★ THE ONE CONTRACT DECISION.** *Exists holds occupancy by Arc, not by
memcpy* — the same contract already landed for catch-up.

> **⚠ CORRECTION 2026-08-23 (before the strike, not after).** The predicted
> win below named **neg-consumer**. Grounded against the disk: the branch is
> guarded by `pids.is_empty() && kind == NodeKind::Exists` — a **LEADING**
> `:exists`, one with no parent. Checked every axis: `negation`, `neg-consumer`
> and `strat-neg` use `:not` and contain **zero** `:exists`; the only axis with
> one is `accum` (`:acc::exists-rule`), where the `exists` is the **second**
> condition, so it HAS a parent and this branch never fires.
> **No grid axis exercises this site.** The memcpy is still real theater and the
> contract that kills it is already landed and proven, but its win **cannot be
> demonstrated by the grid**, and this project does not claim numbers it cannot
> measure. T2 therefore needs its OWN isolated probe (a leading `:exists` over a
> large alpha, in the shape of `strat_merge_present_parts`) before it is cut —
> and it is **re-ranked below T3 and T5**, which land on axes already proven
> sensitive. The original prediction is left below, struck, as the record of the
> error.

**Gate (revised).** (1) an isolated probe over a leading `:exists` with N
elements names the memcpy; (2) 7strat 3/3; (3) Clara `test-simple-exists`
distinct-inner-binds still matches; (4) clippy `--lib -D warnings`.
Do NOT gate on neg-consumer — it does not reach this code.

**~~Predicted win (written first)~~ — STRUCK, wrong on the disk.** ~~Sibling of
the catch-up Arc intern, which took fanout without-query FIRE −1.51 ms on a 40k
leaf. Exists leaves in the grid are far smaller, so expect −0.1 to −0.5 ms on
neg-consumer~~ — neg-consumer has no `:exists` at all. Revised: magnitude is
whatever a leading `:exists` sits over; unknown until the probe names it.

**Blast radius.** `fire/delta.rs` Exists-leaf branch only. No `.wat`. No
`AlphaMemory` type change.

**Out of scope = REJECTED.** Arc-wrap `wm.beta`. Occupancy as `Vec<u32>`.
Skip the walk.

---

### T3 — `harvest_class_scan_filter` builds a bag, then copies the bag — ✅ **LANDED 2026-08-23**

> Struck. `DESIGN-STONE-harvest-bag-in-place` + `BRIEF-harvest-bag-in-place`.
> `harvest:query` **7.60–7.99 → 5.15–5.48 ms (≈ −2.5)**; with-query FIRE
> 31.3 → 28.6; without-query FIRE and compiled-rhs unchanged; query-maps still
> 40000. Probe `harvest_bag_copy_parts` named 0.95 ms — **the prediction
> UNDERSHOT by 2.6×**; see the stone for why an allocation probe is a LOWER
> bound in-fire while a hashing probe (T1) is an UPPER bound.

**Site:** `src/rete/kernel/fire/mod.rs:990` (callee `harvest_class_scan`, `:966`).

```rust
let mut maps = Vec::new();                       // ← no capacity
maps.extend(harvest_class_scan(pv.iter().filter(..), pv.len(), &scan.var));
//          └─ callee already did Vec::with_capacity(cap) and filled it
```

**Why it is theater.** The callee allocates a full `Vec<PMap>`, fills it,
returns it; `extend` then copies every element into `maps` and drops the temp.
`PMap` is `Array(Arc<[(Value,Value)]>, u64)` — **56 B measured**, not the
24–32 B first estimated here — so at fanout 40k the intermediate is
**2.24 MB allocated, filled, memcpy'd and freed per fire**, with the
page-faults paid twice. It sits inside the named leftover: harvest:query
**6.89 ms** measured this session.

**The cut.** `harvest_class_scan(&mut maps, facts, &scan.var)` writes in place;
`maps.reserve(exact)` once up front. Two call sites in the same function plus
the `bag` branch.

**★ THE ONE CONTRACT DECISION.** *Harvest writes maps into the caller's vec;
no intermediate bag is materialized.* Same maps, same order.

**Gate.** (1) `fanout_three_leftover_split` still reports 40k query-maps AND
harvest:query drops (any named drop counts — this is below the grid's noise
floor by construction); (2) 7strat 3/3; (3) clippy `--lib -D warnings`.

**Predicted win (written first).** **harvest:query −0.2 to −0.4 ms**;
without-query FIRE unchanged. If it is a wash, revert — the remaining wrap is
the 40k heap maps, which `DESIGN-STONE-harvest-wrap-parts` already ruled
physics.

**Blast radius.** `fire/mod.rs` `harvest_class_scan` signature +
`harvest_class_scan_filter` + `harvest_query_memory` call site. No `.wat`.

**Out of scope = REJECTED.** `PMap::Array1`. Session-Vec. Skip freeze.
Dropping `next_intern` on one-entry maps. Columnar query-memory (one key, a
vector of values) — it changes the Session shape the oracle reads.

---

### T4 — `token_assoc` heap-allocates per call to walk the pool — ✅ **ALREADY LANDED; this entry was STALE**

> Found 2026-08-24 during `recolligere`: `extend_from_within` was already in
> `fire/mod.rs`, with a comment explaining it, while this list still carried T4 as
> open. The list was wrong, not the code. Recorded because a work list that claims
> open work already done is the same class of defect as one that hides work.

> **CORRECTION 2026-08-23.** This entry implied per-token volume. Grounded:
> `token_assoc` has exactly ONE caller (`delta.rs:1084`), the accumulate fold
> result — **once per group per fold, not per element**. At accum `[200 200]`
> that is ~200 calls, not 40k. The allocation is real; the volume is two orders
> of magnitude smaller than the entry implied.

**Site:** `src/rete/kernel/fire/mod.rs:510`.

```rust
let pairs: Vec<(u32, u32)> = pool_slice(intern.pool, tok.binds).to_vec();
let start = intern.pool.len();
for (ek, ev) in pairs { intern.pool.push(..); }
```

**Why it is theater.** The allocation exists only because `intern.pool` is
about to be borrowed mutably while the span is still being read. The bytes are
already in the pool; copying them out to copy them back in is the borrow
checker being paid in `malloc`.

**The cut.** `Vec::extend_from_within(start_idx..end_idx)` (stable) copies the
span inside the pool with no intermediate allocation; then patch the one slot
whose key matches, in place.

**★ THE ONE CONTRACT DECISION.** *The pool copies within itself; `token_assoc`
allocates nothing.* Same pairs, same order, same `found` semantics.

**Gate.** (1) accum leftover `Instant` (token_assoc is on the accumulator
binding path) does not regress, any drop counts; (2) 7strat 3/3;
(3) clippy `--lib -D warnings`.

**Predicted win (written first).** One allocation per `token_assoc` call
removed. Size depends on call volume — measure before claiming. If the
`Instant` shows nothing, revert and record it as physics.

**Blast radius.** `fire/mod.rs` `token_assoc` only.

---

## TIER 2 — real, smaller, or needs care

### T5 — two clones per derived fact per stratum — ✅ **LANDED 2026-08-23 (unmeasurable, taken on arithmetic)**

> Struck. `new_derived` is dead after the loop, so the vec push MOVES; only the
> dedup set clones. Probe `strat_acc_derived_clone_parts`: **0.103 ms** isolated
> at the `[6 2000]` ladder — a CPU probe, therefore an UPPER bound in-fire.
> In-fire `strat:acc` moved **0.628 → 0.600 ms**, which is noise. **Taken because
> it is strictly less work (one Arc bump per derived fact instead of two), NOT
> because it was measured in the fire.** Recorded that way so nobody later cites
> a win here that the instrument never showed.
`src/rete/kernel/fire/rules.rs:206-207`
```rust
if acc_derived_set.insert(d.clone()) { acc_derived.push(d.clone()); }
```
One clone feeds the dedup set, one feeds the vec. Both are Arc bumps of
`Value::Aggregate`. Kin to T1 — the same stratum loop. Consider folding into
T1's carried-set change rather than a separate strike.

### T6 — Exists candidates materialize a Vec purely to key a HashSet — ✅ **LANDED 2026-08-24 (in `71d0e700e`)**

> It DISSOLVED rather than being optimized. The `candidates` Vec existed only to
> escape the `wm.alpha` borrow that T2's memcpy was buying; convert the memcpy to an
> Arc bump and the second allocation has no reason to exist. T2 and T6 were one
> defect wearing two hats.
>
> NOT done by a u64 content hash, which this entry originally floated: a collision
> silently DROPS a distinct binding, trading a correctness lie for a malloc. The
> dedup is still exact, keyed on the interned pairs.
`src/rete/kernel/fire/delta.rs:1268`
```rust
(binds, pool_slice(&wm.bind_pool, binds).to_vec())
…
let mut seen_pairs: HashSet<Vec<(u32, u32)>> = HashSet::new();
```
A `Vec` per candidate exists only because the set needs an owned key. A content
hash (`u64`) of the span would key the set without materializing. Care: must
preserve the distinct-inner-binds semantics Clara's `test-simple-exists`
asserts. Do after T2 (same code region).

### T7 — `Op::Or` / `Op::Not` allocate a SlotFrame per branch, per element — ⚠ RECLASSIFIED 2026-08-24: it was never "cold", it was UNEXERCISED

> **"COLD" was the wrong word and it cost this entry its priority.** It meant "no perf
> axis measures this" and read as "we have looked at this." Grounded by arming a
> `panic!` in BOTH arms and rebuilding: the ENTIRE where-family differential passed,
> and a direct run of `where-boolean.wat` (whose rows include `not(or ...)`) never
> fired it. Nothing in the corpus reached `compiled_cond`'s `Op::Or`/`Op::Not` — not
> the perf grid, not the correctness corpus, not the oracle, not Clara.
>
> **They are LIVE, not dead** (`purgare` would be the wrong ward): the same panic
> fires instantly on an `:or` nested INSIDE one condition's constraint list. There
> are THREE different `or`s in this engine and the corpus had only two —
> top-level-across-conditions (network branches, 11 fixtures), a where-EXPRESSION
> (`where-boolean`), and INTRA-CONDITION, which is the only shape reaching these arms.
> Same surface syntax, three engines, one untested.
>
> **CLOSED as a coverage hole** by `where-or-inline.{wat,clj}` — native + oracle on
> every floor via `spec_equals_native_on_every_where_family`, Clara via
> `check-where-shapes.sh` (5/5 rows agree). Row 4 is the load-bearing one: `compile_one`
> compiles `or`/`not` branches against "a throwaway clone/scratch ... matching
> `eval_clause`'s discard of branch-local binds", an invariant that had no test; row 4
> nests the scopes two deep, where a clone bug would surface.
>
> **The allocation itself remains OPEN and is now honestly measurable** — the arms
> finally execute. It was always the least interesting thing about this site.

> **CORRECTION 2026-08-23.** This entry claimed the site is "hot in any `where`
> with `:or`/`:not` over a large alpha". The source already says otherwise —
> `compiled_cond.rs:558`: *"The one exception is `Op::Or`/`Op::Not`, which clone
> `scratch` into a fresh temporary — **not exercised by anything in the live grid
> corpus**."* Like T2, the allocation is real but **no grid axis reaches it**, so
> no win here is demonstrable without its own probe. Re-ranked with T2.
`src/rete/compiled_cond.rs:870,878`
```rust
let mut clone: SlotFrame = slots.to_vec();
```
The **copy is the semantics** (a failed branch must not leak bindings) — that
part is WHAT. The **malloc is not**: a reused scratch frame
(`clear()` + `extend_from_slice()`) carried through `exec_ops` removes the
allocation and keeps the copy. Hot in any `where` with `:or`/`:not` over a
large alpha.

### T8 — `d_beta_from_parents` materializes a capacity-less Vec — ❌ **CLEARED 2026-08-23, NOT theater**

> **Measured, not argued.** `dbeta_gather_volume` instruments the function
> across two workloads:
>
> | workload | calls | allocating | **MULTI-parent** | tokens/alloc |
> |---|---:|---:|---:|---:|
> | strat-neg `[6 2000]` | 12 | 6 (50%) | **0 (0.0%)** | 2000 |
> | accum `[200 200]` | 10 | 5 (50%) | **0 (0.0%)** | 200 |
>
> T8 claimed a capacity-less `Vec` that grows and reallocs. **It cannot.** An
> empty gather allocates nothing (`Vec::new`), and a single-parent gather
> `extend`s once from an `ExactSizeIterator`, which reserves the exact length
> in one shot. Growth requires a MULTI-parent call, and both fixtures measure
> **zero**. The proposed fix — reserve the total up front — would have done
> literally nothing.
>
> The entry also implied per-node-per-round volume. Real volume is **~10 calls
> per fire**. The remaining allocation is ~6 × 32 KB of `Token` (which is
> `Copy`), and both call sites document it as the deliberate price of the
> `d_beta` read/write borrow conflict.
>
> `dbeta_gather_volume` is left as a **TRIPWIRE**: it asserts 0 MULTI-parent
> calls on both fixtures. If one ever appears, T8's premise has changed and it
> is live again.
>
> One real but sub-trivial find, recorded and NOT struck: both call sites
> (`delta.rs:1004`, `delta.rs:1228`) already compute `pids` via
> `parents_of.get(node_id)`, and the function then performs the **same lookup a
> second time**. At ~10 calls per fire that is ~10 redundant HashMap lookups —
> real, and far below anything the instrument can see.
`src/rete/kernel/fire/mod.rs:769`
```rust
let mut out = Vec::new();
for pid in pids { if let Some(ts) = d_beta.get(pid) { out.extend(ts.iter().cloned()); } }
```
Callers iterate it once. Two rungs available: reserve the total first (cheap,
safe), or return an iterator (bigger, touches call sites). Start with reserve.

---

## TIER 3 — HUNTED AND CLEARED — do not re-hunt these

Recorded so the next pass does not spend the same hours. Each was read in
context this session and is **not** theater:

- **`fire/delta.rs:463`** `Arc::from(ids.iter().map(..).collect::<Vec<_>>())` —
  `Arc<Vec<T>>` from `Vec<T>` **moves** the Vec; it does not copy. The rune
  already names it *the occupancy-share door*. The `collect` reserves exactly
  (slice iter is `ExactSizeIterator`).
- **`fire/rules.rs:357`** `wm.derived_facts.extend(acc_derived.iter().cloned())`
  — `acc_derived` arrives as `&[Value]` (`rules.rs:346`); you cannot move out
  of a borrow. The clone is required.
- **`fire/delta.rs:357`** `leaf_aids.insert(class.to_string(), leaves.to_vec())`
  — once per class per fire, and classes are few. Not per-fact.
- **`kernel/session.rs:1118`** `session_with_fields` `a.fields.as_slice().to_vec()`
  — the Session is **8 fields**. Trivial.
- **`kernel/node.rs:182`** `a.fields.as_slice().to_vec()` — network mutation
  (child removal), arm-time, not fire-time.
- **`expr_ir.rs:1545-1549`** `args.iter().cloned().collect()` / `args.to_vec()`
  — `args: &[Value]`; producing an owned `PVec`/`Vec`/`List` requires the clone.
- **`fire/mod.rs:1183` the HashSet itself** — the set is NOT the theater; the
  linear `.any()` it replaced was worse (documented O(n²) hang at `[7,3000]`).
  **T1 is about WHERE it is built, not THAT it is built.**

---

## Rules of engagement

0. **⛔ THE 0.5 ms GATE IS RETIRED (2026-08-23, builder's direction):**
   *"that 0.5ms gate criteria... it may have outlived its purpose now.... we
   are basically at the physics boundary with a few bumps to smooth over... we
   are seeking those bumps to extract all remaining perf."* The standard is now
   **any named drop an `Instant` proves**, however small — the same standard the
   builder already set taking the 0.40 ms `from_one` cut
   (`DESIGN-STONE-harvest-wrap-parts`: *"chase everything that Instant-proves,
   including sub-gate construction on the existing arm"*). A size threshold at
   the boundary is a licence to leave cruft standing. T3/T4/T7/T8 are all
   sub-ms and are all in scope. What still disqualifies a strike is a **wash**
   (no drop the instrument can name) or a **correctness cost** — never smallness.
1. **One strike at a time**, in tier order. Each gets a DESIGN-STONE + BRIEF
   before the cut, with the predicted win **written first**.
2. **Weigh same-session**, before/after, on the leftover `Instant` harness —
   never across sessions, never on the grid for a sub-ms claim.
3. **Revert on a wash.** A strike that does not produce a named drop is
   reverted and recorded as physics, so it is never re-hunted.
4. **Floor + clippy green before each commit**; grid for accuracy + rank only.
   `scripts/floor.sh` — and on a red, DO NOT RE-RUN: capture the ARM.
5. Work stays on `grok-rete`. Never merge `origin/main` unless asked.

---

## Not perf — a guardrail hole found during the same sweep

**`wat-rs/CLAUDE.md` claims a delivery channel it does not have.** Its header
says the load-bearing subset — *the wat-fix codemod doctrine, the release
floor, the scratch-`.wat` convention* — *"is carried in `holon/CLAUDE.md` — the
only injected copy,"* marked **"Verified 2026-07-21."**

Grepped 2026-08-23: `holon/CLAUDE.md` contains **none** of them — no `wat-fix`,
no `codemod`, no `floor.sh`, no "known flake", no `scratch-pad`, and it never
mentions `wat-rs` at all. Its sections are the Python VSA library, holon-rs,
the DDoS lab, the website, and the trading lab.

**Consequence:** a fresh session or a spawned rider receives **zero** wat-rs
doctrine. It gets the floor discipline only if it independently opens
`wat-rs/CLAUDE.md`. A rider will not.

The assertion is self-certifying — it states its own delivery and dates it — so
nobody re-checks. Stem-patch: paste the subset into `holon/CLAUDE.md`.
Root-pull (`extirpare`): the `@wat-rs/CLAUDE.md` import that file already names
as a follow-up, which makes the two copies structurally one and the drift
unrepresentable. **Holon root git is FROZEN (`tmp/VIGILIA-LOOP.md`) — not
touched. Awaiting the builder's call.**

---

## Loop obligation carried forward

`tmp/VIGILIA-LOOP.md` records the last consecutive 0+0 at **`36802e7e`**
(recasts 33 + 34). Since then **`e21b7fba`** landed real code
(`fire/delta.rs`, +28/−7 — catch-up takes parent beta). Per the loop's own
step 1 that intern is **un-watched**. `a58f9dda` is grid text only.

---

## THE STRATIFIED LOOP WAS UNMEASURED — and the instrument changed the target

**2026-08-23.** About to spend a stone on T5's 0.103 ms, it became obvious that
`fire_rules_stratified` carried **no phase marks at all** except
`harvest:query`. 11.4 ms on strat-neg was entirely unapportioned. Grinding a
0.1 ms cut inside an unmeasured 11 ms is not engineering.

`fire/rules.rs` is now instrumented (`strat:slice` / `strat:session` /
`strat:collect` / `strat:merge` / `strat:acc`) and `strat_neg_stratum_split`
reads it. The marks are `#[cfg(test)]` — the release binary the grid measures
is byte-unchanged.

### Where strat-neg `[6 2000]` actually goes (no query, mean of 3)

| lump | ms | share |
|---|---:|---:|
| **FIRE (4 phases)** | **8.392** | 72% |
| `strat:merge` | 1.387 | 12% |
| `strat:acc` | 0.600 | 5% |
| `strat:slice` | 0.109 | 1% |
| `strat:collect` | 0.106 | 1% |
| `strat:session` | 0.006 | 0% |
| unaccounted (freeze/seed/outside marks) | ~1.04 | 9% |
| wall | 11.641 | |

**The hypothesis that sent me here was wrong.** I expected the per-stratum
network rebuild — `close_upstream` + `slice_active_network` +
`subset_rete_arm`, run once per stratum — to be the hidden cost. It is
`strat:slice` = **0.109 ms**. Not the target. Recorded because a wrong guess
that the instrument corrects is cheaper than a wrong guess that ships.

**The loop scaffolding is ~2.2 ms of 11.6, and most of it is persistent-
collection maintenance** (`merge` + `acc` = 1.99 ms), which is what a Session
holding a `PersistentVector` costs. The remaining lever on this axis is FIRE
itself at 8.4 ms, not the loop around it.

## A hypothesis the measurement killed — and the cliff it left behind

`push_back_mut` on `PVec::Array` is `Arc::make_mut(items).push(v)`, and
`merge_facts` clones the closure while the caller's copy is still alive. That
reads as a textbook copy-on-write bug: the first push of every stratum would
deep-copy the whole `Vec`, 2000+3000+…+7000 = **27000 Value clones per fire** —
the exact shape T1 cut on the hashing side.

Probe `strat_merge_cow_parts` confirmed the cost **for the Array arm**:

| lump | ms |
|---|---:|
| A SHARED receiver | 2.311 |
| B UNIQUE receiver | 0.292 |
| **A−B** | **2.019** (8×) |

**Then the fire said no.** `strat_merge_pv_owner_count` instruments
`merge_facts` directly: 6 calls, summed owner count **0**, and `array_owners`
returns 0 only for the **Tree** arm. The closure PVec is rpds `VectorSync`, so
`push_back_mut` path-copies O(log n) and **no whole-Vec deep copy happens**.
The probe measured a fixture built with `PVec::from_vec` (Array); the fire does
not use that arm. **No strike. Hypothesis falsified by measurement, not by
argument.**

> ⚠ **LATENT CLIFF — the 8× is real, it is just not armed today.** If the
> closure ever reaches `merge_facts` as `Array` while the caller's copy is
> alive, every stratum deep-copies the whole Vec.
> `strat_merge_pv_owner_count` is the **tripwire**: it prints the arm and
> asserts the call count. If `arm` ever reads `Array`, read
> `strat_merge_cow_parts` before anything else. `DESIGN-STONE-promoting-vector`
> is the other end of this thread — it is what decides which arm a vector is in.

### The probe-reading rule, now with a third case

- **CPU probe** (hashing, comparison) → **upper** bound in-fire. (T1)
- **ALLOCATION probe** → **lower** bound in-fire; a tight loop lets the
  allocator cache what the fire cannot. (T3)
- **A probe whose FIXTURE picks a representation** → may measure a code path
  the fire never takes. Build the fixture the way the fire builds it, or
  instrument the fire and check. (this section)

---

## The floor went RED on T8's tripwire — and the gate was right

**2026-08-23, `.floor/2026-08-23T23-18-25Z`, exit=100.**
`Summary [275.720s] 4934 tests run: 4933 passed, 1 failed, 19 skipped`

```
FAIL [0.093s] (44/4934) wat::lint no_loose_string_assert::tests_carry_no_loose_string_assert

🔥🔥🔥 LOOSE STRING ASSERTIONS — 2 site(s) assert a value with contains/starts_with/
ends_with where an exact `assert_eq!` belongs. A loose check passes on reordered fields,
malformed maps, and appended garbage.

    src/rete/kernel/tests.rs:9837
    src/rete/kernel/tests.rs:9846
```

**The red was mine and the lint was correct.** The T8 tripwire had been written as
a substring match on the harness's own rendered table:

```rust
assert!(
    out.contains("MULTI-parent      0") || out.contains("MULTI-parent          0"),
    ...
);
```

Two spacings ORed together — the author hedging against his own format string is
exactly the fragility the gate exists to catch. A column-width change would have
silently turned the tripwire into an assertion that always passes: a guard that
cannot fail is worse than no guard, because the file still reads as guarded.

**Fixed at the class, not the site** (`extirpare`): the harness no longer asserts
on text at all. `run` returns a `Gather { calls, tokens, alloc, multi }` and the
tripwire is `assert_eq!(strat.multi, 0)` / `assert_eq!(accum.multi, 0)` — exact,
structured, and immune to how the table is printed. Same measured numbers.

**Process note, recorded deliberately.** `scripts/floor.sh` says DO NOT RE-RUN on
a red, and it was not re-run. The ARM was captured by the script before anything
was read, the failing test's whole block was read verbatim, the exact arm was
named (`no_loose_string_assert`, 2 sites, with line numbers), the cause was
fixed, and only then was a NEW floor run. Re-running to see if a red goes away is
the forbidden act; re-running after fixing the named cause is the point.

---

## The concurrency audit — a hazard the perf hunt would never have found

**2026-08-23.** Told that the last unexplored perf axis was the allocator, the
builder redirected: *"we must have the rete subsystem be tolerant to highly
concurrent execution — imagine 512 threads all running their own rete — they
must never step on each other."*

That question found something this whole theater hunt had walked past, because
the hunt was looking for **wasted work** and this is **shared state**:
`next_intern` in `src/value/pmap.rs` was one process-global `AtomicU64`, taken
exclusively on every mint — and every one-entry `PMap` mints, 40k per fire on
the harvest path. `harvest_wrap_parts` had even *measured* it (`I fetch_add
0.21 ms`) and correctly ruled it not worth cutting **single-threaded**. It was
never the throughput that mattered.

Struck: `DESIGN-STONE-intern-lane-per-thread` + `BRIEF-intern-lane-per-thread`.
Ids are now laned per thread — uniqueness preserved, one atomic per THREAD
instead of per mint. A never improves with threads (16.03 → 15.87 ns/op),
B scales near-linearly (5.30 → 0.47), and B is faster single-threaded too.

**The audit found exactly one hazard in the whole fire path** — every other
shared site is a write-once `OnceLock`, a `thread_local`, or `#[cfg(test)]`.
The table is in the stone's Weigh.

### The lesson for this list

A hunt aimed at one defect class is blind to the others by construction. This
list hunts **theater** — work that produces nothing. It has no lens for
**contention**, and would have kept reporting "the remaining cost is physics"
while a single cache line serialised 512 engines. `circumspicere` is the ward
for exactly this: *steps back from the code the other spells look INTO and
surveys what surrounds it… finds what the guard walked past.* The guard walked
past this one, and a question found it.

---

## READ, not measured — is this code an exemplar? Not yet, and here is the shape

**2026-08-23.** Builder: *"how about we do a deep dive on the code… and 'read for'
theatre?.. not measure it?.. i want this code to be an exemplar - are we there
yet?"*

Measurement had hit a wall (a mark pair costs ~100–155 ns against sub-operations
of ~100–300 ns). Reading finds what the timer cannot.

### The one structural fact that explains most of the rest

```
fire_fixpoint_delta_armed   src/rete/kernel/fire/delta.rs:220-1994
    1774 lines  —  87% of the file
    12 levels of brace nesting at its deepest
    16 mutable locals at the top level
    8 passes braided into one body:
      alpha · root-join · hash-join · accumulate · filter ·
      join-after-filter · filter-after-join · production · terminate
```

Against the four questions this fails **Obvious** and **Simple** outright. No
reader holds 1774 lines and 12 levels; and "one function, eight passes" is not
one un-braided concept.

### It is also the ROOT of theater the hunt could not remove

Every clone the hunt found and could not cut is justified, in the source, by the
same sentence — the borrow checker:

| site | the source's own words |
|---|---|
| `delta.rs:1000` | *"clone to avoid the d_beta read/write borrow conflict"* |
| `delta.rs:1224` | *"to avoid a simultaneous borrow conflict (reading d_beta[parent] while writing d_beta[*node_id])"* |
| `delta.rs:1250` | releases the `wm.alpha` borrow before `span_from_row(&mut wm.bind_pool, …)` |
| `fire/mod.rs:510` | `to_vec` because `intern.pool` is about to be borrowed mutably |
| `DESIGN-STONE-catchup-take-left` | *"HashMap split-borrow needs the parent out of the map"* |

These are five workarounds for the borrow checker, and the arrangement that
forces them is one function holding one `&mut FireSession` across 1774 lines.

> **⚠ CORRECTED 2026-08-24, before the strike was briefed.** This paragraph
> first claimed narrowed per-pass borrows "would make several of these clones
> unnecessary". That is TOO STRONG, and probing it is what caught it. Only ONE
> of the five is a disjoint-FIELD conflict:
>
> | conflict | between | fixed by splitting? |
> |---|---|---|
> | `:1000`, `:1224` | `d_beta[parent]` read vs `d_beta[child]` write — the SAME HashMap, and a round-local, not a `wm` field | **no** |
> | `:1250` Exists | `wm.alpha` vs `wm.bind_pool` / `i64_by_fact` / `bind_only` / `cond_key_ids` — distinct fields | **yes** |
> | `mod.rs:510` | `intern.pool` read vs write — same container | **no** |
> | catch-up take-left | same beta map | **no** |
>
> So the split is justified on **craft** — Obvious and Simple — and removes one
> clone as a side effect. The other four are same-container conflicts needing
> their own techniques (disjoint-key access, a two-phase collect,
> `extend_from_within`, a restore guard). Claiming the refactor would dissolve
> them was the kind of tidy story that survives right up until someone does the
> work and finds it false.

This is also where the ~7–8 ms of unapportioned `production` time lives, and why
apportioning it costs more instrument than it measures.

### A hand-held invariant at 12 levels deep

`delta.rs:666-766` takes the parent beta out of the map (`wm.beta.remove`), walks
it, and puts it back — with **two** restore sites, one on the normal path
(`:765`) and one on an error path (`:740`) nested 12 levels in. Audited: exactly
one early return in that 100-line window, and it *is* handled. The invariant
holds today. But it is held by **convention**: any future `?` added in those 100
lines silently drops a beta memory, and nothing catches it. On `extirpare`'s
ladder that is the bottom rung where a guard/scope shape would be the top.

### Drift I introduced today, found by reading

`fire/mod.rs` `merge_facts` carried two adjacent doc paragraphs that contradicted
each other after T1: the P9 paragraph still stated the per-call cost as
`O(len(pv) + len(derived))` — true only while the set was rebuilt per call —
directly above the new paragraph saying the set is no longer rebuilt here. A
reader had to reconcile them. `cohaerere`, caught by reading, not by any gate:
clippy, the floor and 4942 tests were all green with the contradiction in place.
Corrected — the P9 note is now explicitly lineage, and names where the
`O(len(pv))` term is paid instead.

### The honest verdict

**Not an exemplar yet, and the gap is one function.** The parts around it read
well: names say what they are, the stones are unusually good, the census tree is
disciplined, `rg Mutex src/rete` is empty, and the dual-impl oracle keeps the
whole thing honest. `partire` would return LEAVE on most of this repo.

On `delta.rs` it returns the opposite, and the cut lines are already drawn — the
eight `// ── N. <pass>` section comments in that function are the seams, written
by the authors themselves. The file is one module wearing nine hats.

**Not proposed as a strike here.** Splitting a 1774-line fire loop is a
different kind of work from the intern chain: it is a refactor whose gate is the
differential and the floor, not a leftover `Instant`, and it is the builder's
call whether arc 278 takes it.


---

## READ THE GRID AGAINST A RANGE, NOT AGAINST THE LAST RUN (2026-08-24)

Five times this session a grid comparison showed a coherent move — every rung of
an axis in the same direction, the pattern that normally means signal — and five
times it failed to reproduce at `GRID_RUNS=5`:

| axis | apparent | re-measured | verdict |
|---|---|---|---|
| asym-join (T14) | +23/+2/+26% | 2.941 vs 2.92 pre | noise |
| strat-neg (T14) | +9.9/+1.6/+1.5% | 11.429 vs 11.77 post-T1 | noise |
| accum (T16) | +5.2/+14.5/+2.4% | 2.64 / 6.47 / 13.37, all mid-range | noise |
| fanout (T17, mid-partire) | +4.3% at 40k | 23.09, lowest ever | noise |
| fanout (T18→T19) | +3.2/+6.7/+3.0% | 23.64, dead centre | noise |

The last one is the clearest lesson. `fanout [40000]` across seven samples since
the harvest cut: **23.18 · 23.43 · 23.31 · 24.31 · 22.80 · 23.50 · 23.64** —
mean ≈ 23.45, range ±0.75. The partire-end run happened to read **22.80**, the
lowest of the seven. Comparing the next run to *that* produced a +3.0%
"regression" out of nothing.

**The rule, from now on:** a grid cell is a DISTRIBUTION, not a value. Compare a
new reading to the cell's recorded RANGE, not to whichever run happened to be
last. A move counts only if it lands outside the range, or if a same-session
before/after on a leftover `Instant` harness shows it. Both directions — a
flattering number picked from the low end of the range is the same error wearing
a nicer face, and this file has one of those in its own history.

`fanout [40000]` sits at **23.45 ± 0.75 ms**. Quote that, not a single run.


## AND CHECK THE LOAD BEFORE YOU RUN THE GRID (2026-08-24)

The range rule above catches a grid cell compared against one lucky run. It does
NOT catch a grid run taken on a busy box, and that produced the sixth false alarm
of this campaign — the first one with an identifiable cause rather than plain
variance.

The post-merge grid was started immediately after a floor and two release builds,
with `load average` still at **4.45** and stray processes from a killed run. It
reported `deep-cascade` up on all three rungs, +46.9% at `[10 100]`. That cell is
the TIGHTEST in the whole grid — 1.64, 1.69, 1.66, 1.67, 1.65, 1.64, 1.65 across
seven prior runs, a range of ±0.03 — so a reading of 2.42 was many multiples
outside it and looked unarguable.

Re-measured on a quieter box: **1.67 and 1.67.** The other flagged cells came back
the same way (strat-neg `[6 2000]` 12.45 → 11.13; fanout `[40000]` inside range).

Two things follow:

- **`uptime` before `run-all.sh`.** The floor's `nice -n 19` protects the
  *builder's keyboard*, not a later benchmark; the machine stays warm and loaded
  for minutes afterwards. A grid started in that window measures the weather.
- **A tight cell is not a safe cell.** `deep-cascade [10 100]`'s ±0.03 range made
  the load artefact look like a certainty — the tighter the history, the more
  convincing a contaminated reading appears. Tight history raises the value of a
  re-measure, it does not remove the need for one.

---

## THE HUNT AS OF 2026-08-24 — theater is done; the exemplar hunt is not

The theater list above is closed **except T7**: **T1/T3/T5 struck**, **T8 cleared
as not theater**, **T2/T6 struck 2026-08-24** (one defect, two hats — see their
entries), **T4 was already landed and this list was stale about it**. **T7 alone
remains, and it is COLD** — no grid axis reaches it, so it needs its own probe
before anyone claims a win. The perf campaign is at
the floor on every measured axis.

What replaced it is a different question — *is this code the exemplar the rest
of wat's subsystems should copy?* — and it is measured differently: **code
volume × nesting**, not milliseconds. Current state of rete, longest first:

| function | lines | comment | nesting | verdict |
|---|---:|---:|---:|---|
| `intrinsic_meta` (`purity.rs`) | 701 | **71%** | **2** | **NOT a defect.** ~200 code lines, flat cascade, 500 lines of justification. Line count is the wrong lens here. |
| `eval_axis_violation` (`purity.rs`) | 590 | 18% | **7** | **open** — ~480 code lines, never examined |
| `exec_compiled_rhs_at` (`compiled_rhs.rs`) | 451 | 10% | 4 | **open** — ~405 code lines, never examined |
| `fire_fixpoint_delta_armed` | 448 | 17% | 4 | done — was 1774 at nesting 12 |
| `hash_join_delta` | 361 | 18% | 9 | depth **explained** on `FireCtx`, not fixable without over-borrowing |
| `exec_dim` (`where_tree.rs`) | 388 | **0%** | 6 | **open** — zero comments in a codebase this documented is its own defect |

### Two findings that generalise beyond rete

They are mirrors, and only fixing the first made the second testable:

- **`AlphaNews::of` claimed a borrow it never took** — `alpha: &'a AlphaMemory`
  when `alpha` is read once for a `usize`. The false claim propagated into every
  caller and was the sole reason `hash_join_delta` sat at nesting 9. **A
  too-tight lifetime is a defect that shows up as DEPTH somewhere else.**
- **`FireCtx`'s thirteen-field literal cannot be collapsed** — a constructor must
  take `&mut wm` whole; the literal borrows eleven *named fields* and leaves
  `wm.alpha`/`wm.beta` free, which every call site needs. Tried, reverted within
  the hour, documented on the struct. **Verbosity that encodes field-level
  disjointness is data flow, not repetition.**

The same shape appeared a third time in the `d_beta` copies: what reads as a
borrow-checker workaround is often the data flow. **Before removing an apparent
workaround, check whether the type over-claims (fix it) or claims exactly what it
takes (leave it).**

### Next target

`exec_dim` — 388 lines, nesting 6, and the only place in rete where the house
style is simply absent. Then `eval_axis_violation`, the largest un-examined body
of code left in the subsystem.

---

## THE VIGILIA OF 2026-08-24 — rete answered "are we an exemplar?" with NO

Cast at HEAD `d55899373`, after `recolligere` against the disk. Seventeen inward
wards in parallel, then `circumspicere` last. Every ward fetched its own spell
from the signed datamancy channel — established by probe, not assumed: a
haiku-tier worker could NOT invoke the MCP tool, a sonnet-tier worker could, so
every ward was cast at sonnet.

**The verdict: 4 CONVERGED, 13 diverged.** `sequi`, `secare`, `cernere` and
`probare` came back clean. The rest did not, and the two most valuable findings
were things no amount of measuring would ever have surfaced — this campaign had
spent weeks on milliseconds and had a live correctness bug the whole time.

### ★ L1 — a leading `:not`/`:exists` emitted one token PER FIXPOINT ROUND

`temperare` found it; verified here by a repro built from scratch. The leading
arms of `filter_pass` are re-evaluated every round with no round gating, and
`wm.beta` is cumulative, so the token was appended again each round. Causation,
not correlation — varying ONLY the length of an inert chain that forces rounds:

    chain 2 -> 2 rows | chain 3 -> 3 | chain 4 -> 4 | chain 6 -> 6   (correct: 1)

Both arms. Leading `:not` over an empty class: 6 for 1.

**Why 5016 tests never saw it — TWO independent masking layers.**
1. `production_delta` dedups derived FACTS by value, so rule output stays correct
   and every oracle-differential passes regardless of token multiplicity.
2. `harvest_stratified_queries` (`rules.rs:361`) rebuilds query memory after a
   STRATIFIED fire from a fresh session with empty memories, replayed with
   `FireKind::Once` — a single round. (`complectens` found this one.)
Between them the duplication is observable only through a query on a
SINGLE-stratum leading filter — and `vocare` proved no such test existed:
`differential_exists_no_multiplicity` is NAMED for this contract but its fixture
puts `:exists` second, never leading, so it cannot reach the configuration it
claims to guard. It passed while the defect was live.

**The fix is one sentence: the dedup state lived at ROUND scope and belongs at
FIRE scope.** `LeadingEmitted` (session.rs) replaces two round-locals; the round
gate needs no counter because `leading_emitted.contains_key(node_id)` IS the
"have we run" test. Leading `:not` binds nothing, so its key is the empty vector
— one mechanism, no special case. Gate:
`tests/rete/probe_arc278_leading_filter_multiplicity`, which holds two namespaces
with identical queries over identical facts differing ONLY in round count, so a
fix that special-cases round 0 passes one and fails the other.

**It absorbed both open theater items, because they were one defect.** T2's
`Vec<Element>` memcpy was buying the borrow freedom that T6's per-candidate `Vec`
was also buying; convert the memcpy to an `Arc` bump and T6's allocation
dissolves. T4 was already landed and the list was stale about it.

### ★ L1 — the census told root-join twice, and the type allowed it

`purgare` (L1) and `struere` (L2) found it independently. `root_join.rs` called
`phase_end("root-join", __pt1)` twice against one mark: both the nanoseconds and
the PAIR COUNT — the calibration divisor — doubled, so every `root-join` figure
since `ae957b51a` is ~2x. That commit's message claims "MECHANICALLY VERIFIED...
identical... none of them a logic change." It added a line.

Deleting it is the stem. The root: `PhaseMark` was `Instant`, which is `Copy`, so
the second call compiled in silence. It is now a non-Copy newtype — a second
close is `E0382`, proven by reintroducing the defect. A whole-crate balance audit
confirms root_join was the only unbalanced site.

### THE MEASUREMENT DEFECT THAT STEERED THE HUNT FOR THREE SESSIONS

The exemplar hunt's target table was taken by hand and wrong the same way twice —
first `fn`-line-to-EOF (swallowing the test module), then missing the `///` block
ABOVE the `fn`. Recorded 388/451/590-line bodies are really 87/35/72, and
`compiled_rhs.rs` was not even at the recorded path. The table was not merely
inflating numbers, it was naming the WRONG functions: `wat-scripts/hunt/fn-census.py`
found `apply_core_kind` (267 lines, 0% comment) and `unpack_expr` (262, 0%),
which the broken table never saw. Both now documented; `eval_axis_violation` and
`exec_compiled_rhs_at` are CLEARED, the latter documented all along above the
line the measurement was not looking at.

**Do not re-derive these numbers by eye. The tool is the instrument.**

### WHAT REMAINS OPEN — the honest list

L1, unaddressed:
- `conformare` x9 — a real wat span was in scope and discarded for
  `rust_caller_span!()` (`eval_insert.rs:74,85,132,187`, `arm.rs:179,193,208,231,293`).
  A user's malformed `:then` points at wat-rs's own Rust source, not their file.
  `arm.rs:316` does it correctly in the same file — the pattern is known.
- `intueri` x3 — doc comments attached to the WRONG function (`purity.rs:808-939`
  describes `constructor_meta` but sits on `is_declaration_derived_construction`;
  `purity.rs:1128` describes `classify_expr` but sits on a non-recursive guard;
  `validate.rs:1089` promises source-form rendering and emits Rust `Debug`).
- `vocare` x6 — four join tests with a deliberately empty `:rhs` reading `wm.beta`
  with no `rune:vocare(vantage-bypass-test)` marker; one test that asserts a
  hand-written belief-array against itself and touches no production code;
  `differential_exists_no_multiplicity` misnamed for a contract it cannot detect.
- `exigere` x1 — a cache-stone brief promising "a later stone" that never came and
  is tracked nowhere.

L2, unaddressed (highlights):
- `solvere` — the `CallFallback` five-shape classification is written THREE times
  (`where_tree.rs:515`, `expr_ir.rs:1046`, `runtime.rs:10075`) and the third GUARDS
  on `matches!(op.ret, ParamType::F64)` while the two rete copies sniff the runtime
  value. `runtime.rs`'s own comment says why that is wrong. No current RETE_OPS row
  exercises the gap. **This makes the exec_dim doc committed in 8788601de a true
  description of a DIVERGENT copy — amend it, and unify to one classifier.**
- `solvere` — `where_tree.rs:331` bypasses `clause.rs`'s documented ONE DOOR
  `classify_constraint_head`, which has an anti-drift gate test for this exact
  pattern; enum-variant-constructor resolution hand-written at three sites.
- `conferre` — 4 of 10 grid axes (`fanout`, `min-finding`, `node-share`,
  `user-reduce`) compute `:oracle-derived` but NOTHING in CI compares it; only a
  standalone shell script does, and no test invokes it. Also the differential
  gate's header prose says 18 axes where the arrays hold 41.
- `excusare` x2 — `#[allow(clippy::too_many_arguments)]` at exactly 7 args in
  `join_after_filter.rs:26` and `production.rs:26`; the threshold is 8. Both
  carried over from sibling extractions that genuinely need it.
- `partire` x7 — split proposals for `fire/mod.rs` (3), `validate.rs` (2),
  `expr_ir.rs` (1), `arm.rs` (2). `purity.rs`, `export.rs`, `vocabulary.rs`,
  `session.rs`, `compiled_cond.rs` all LEAVE.
- `complectens` — the `harvest_stratified_queries` replay path is isolated by no
  test; my new probe is the contract's only gate and is end-to-end.
- MY OWN, verified: `RoundScratch` declared `bind_only` AND `cond_key_ids` as
  `&'a mut` while every consumer takes them shared — the `AlphaNews::of` class
  again. FIXED. And `matcher.rs:81` / `validate.rs:817` are byte-equivalent
  registry lookups while matcher's doc claims to be the sole reader — `solvere`
  RETRACTED this one; I verified it on the disk before the retraction and it
  stands. A ward's retraction is also just a report.

### circumspicere — cast LAST, against what the inward seventeen missed (0 L1, 3 L2)

- **The fixpoint has no cap.** `fire_fixpoint_delta_armed` (delta.rs) ends only when
  the delta empties — no round counter, no deadline, no memory ceiling — and the
  memories accumulate across rounds by design. A rule whose `:then` derives a
  structurally-novel fact each round hangs the calling thread and grows heap with no
  diagnostic. `DESIGN-STONE-4b-cascade-fixpoint` names this as a deliberate Datalog
  choice, but that reasoning lives ONLY there: README, USER-GUIDE, CLAUDE.md and
  `rete.wat` say nothing. Not hypothetical — the grid harness needed a cgroup blast
  door after an analogous run OOM'd the build machine. Nothing protects an embedder.
- **The arc's own closing condition is checked by no CI job.** `PERF-ARC` states it as
  "differential-tested bit-for-bit against the wat oracle AND benched at or past
  Clara." The parity scripts need a JDK+Clojure the runner lacks, so they never run
  there. A Clara-parity or throughput regression merges fully green. `run-all.sh`
  documents this already happening once — four axes sat dead for days.
- **The breadcrumb's census warranty went stale by its own rule** — it claims "read
  before citing ANY census number" and "this file wins", but predated the root-join
  double-count fix by 44 minutes. FIXED in this session's stamp, which now names both
  census defects.

**Cleared by circumspicere, worth knowing:** the purity/determinism/totality fence is
a genuinely closed default-deny system with a completeness ledger and no
foreign/extern/catch-all escape hatch. `build.rs`'s auto-generated module list reaches
every probe; migrate/fix tooling is one-shot and CI-uninvoked; the bench harness
honestly self-labels as non-gating.

### The corrected census — run the tool, do not read by eye

| function | file:lines | lines | comment | nesting |
|---|---|---:|---:|---:|
| `intrinsic_meta` | `src/rete/purity.rs:205-804` | 600 | 66% | 2 |
| `fire_fixpoint_delta_armed` | `src/rete/kernel/fire/delta.rs:217-662` | 446 | 17% | 4 |
| `hash_join_delta` | `src/rete/kernel/fire/pass/hash_join.rs:33-383` | 351 | 22% | 9 |
| `accum_alpha_seed_after_fold_split` | `src/rete/kernel/tests.rs:5730-6036` | 307 | 1% | 7 |
| `apply_core_kind` | `src/rete/expr_ir.rs:1337-1626` | 290 | 8% | 6 |
| `unpack_expr` | `src/rete/export.rs:532-808` | 277 | 5% | 4 |
| `node_share_where_cost_decomposition` | `src/rete/kernel/tests.rs:1018-1291` | 274 | 20% | 4 |
| `fire_rules_stratified` | `src/rete/kernel/fire/rules.rs:12-279` | 268 | 40% | 7 |

`intrinsic_meta` (600 lines, 66% comment, nesting 2) stays CLEARED — a flat table;
line count is the wrong lens. `apply_core_kind` and `unpack_expr` were found by the
tool and documented; `eval_axis_violation` (85) and `exec_compiled_rhs_at` (37) are
CLEARED, the latter documented all along above the line the old measurement missed.

### conformare's nine — CLOSED, and the block's priority was INVERTED (2026-08-24)

All nine `rust_caller_span!()` sites that had a real wat span in scope now carry it
(`arm.rs` x5, `eval_insert.rs` x4; zero left in either file). Three got MORE precise
than the ward asked: `eval_insert.rs:85` points at `other`, the offending head
itself; both operand loops point at `arg` rather than the enclosing form; `arm.rs`'s
alpha site points at `cond`.

**But grounding inverted the block, and this is the part worth keeping.** It reads as
nine user-facing diagnostic bugs. It is not:

- **`arm.rs` x5 — genuinely user-facing.** `compile_acc_fold` /
  `compile_alpha_conds_from_index` run at `compile-all`, so a malformed `accumulate`
  form now names the user's own line instead of `src/rete/kernel/arm.rs`.
- **`eval_insert.rs` x4 — the ORACLE path, not native fire.** The source says so in
  four places: "Native production runs `exec_compiled_rhs`" (`eval_insert.rs:44`,
  `:288`), "this file is the interpreter / differential" (`:4`), and "fire does not
  walk `build_insert_fact`" (`arm.rs:319`). Correct to fix, cheap, and it improves
  the interpreter's diagnostics — not the one a user hits.
- **THE GAP conformare DID NOT NAME.** A real user firing natively hits
  `unbound_operand` (`compiled_rhs.rs:250`), which takes only a debug STRING —
  `RhsOp::Bind` carries no span at all. So the native-fire diagnostic gap is
  STRUCTURAL. The ward audited the sites that HAD a span to discard; it did not
  audit the path that had already thrown one away.

**Deliberately NOT fixed, and why.** Widening `RhsOp::Bind` to carry a `Span` touches
`compile_rhs`, both exec paths, and `export.rs`'s pack/unpack — the SERIALIZED ABI,
guarded by a round-trip identity test and an ABI hash. That is a design change with
real blast radius, not a mechanical span swap, and it deserves its own decision
rather than riding along in a cleanup commit. **It is the highest-value remaining
diagnostic work in rete.**

Floor 5023/5023. Clippy silent.

### circumspicere's uncapped fixpoint — DEFERRED BY DECISION, bounded (2026-08-24)

`fire_fixpoint_delta_armed` ends only when the delta empties: no round counter, no
deadline, no memory ceiling, and the memories accumulate across rounds by design. A
rule deriving a structurally-novel fact each round hangs the calling thread with
unbounded heap and no diagnostic.

**Builder's ruling, 2026-08-24 — an affirmative cut, not an open item:** *"this one
does not concern me — if the caller dos's themselves that's their problem … when we
build rete-as-a-service is a problem for us to handle then, not now."*

So the boundary is named and it is not a date: **the cap becomes required when rete
is exposed as a SERVICE**, i.e. when the caller stops being the rule author. Until
then a self-inflicted hang is the author's own, exactly like an infinite loop in any
other language they write. `DESIGN-STONE-4b-cascade-fixpoint` already frames the
unbounded fixpoint as the deliberate Datalog-semantics choice; this ruling confirms
it and supplies the trigger the stone left as "let need reveal."

**What this does NOT excuse:** the reasoning still lives only in arc docs. README,
USER-GUIDE, CLAUDE.md and `wat/rete.wat` say nothing about fixpoint termination. A
rule author cannot read the decision that makes their hang their own fault. Documenting
the bound where a CALLER reads it is separate from capping it, and remains open.

### ⚠ A CLAIM I MADE AND RETRACT — the diagnostic goldens are NOT a vocare defect (2026-08-24)

While landing the `CallFallback` unification I twice described the five
`probe_diagnostic_value_snapshot_in_errors` goldens — which pin
`:location src/runtime.rs :line N` — as "`vocare`'s wrong-altitude class,
demonstrating itself." **Both halves of that were wrong.**

**It is not what vocare found.** vocare's six were: four join tests that hand-build
a `Rule` with an empty `:rhs` and read `wm.beta` (no production ever runs), one test
asserting a hand-written belief-array against itself, and
`differential_exists_no_multiplicity`. It explicitly CLEARED the rendered-output
family — "CLEAN, stated for contrast". I attributed to the ward a finding it never
made, which is precisely the phantom `examinare` forbids, committed about my own
ward's report.

**And it is wrong on the merits.** vocare measures VANTAGE: does the test stand
where the CALLER stands? The `:location` span is part of the error EDN a caller
receives, so pinning it asserts shipped output, not internals. The golden compares
the whole error structurally, so it cannot go green on a real break; and when it
reddened for me it reddened on a genuinely caller-visible change — the location
moved 25695→25722. That is signal: "the code raising this moved, confirm that is
intentional." Had the unification changed WHICH code raises `NotCallable`, the
recapture diff would have carried more than a `:line`.

**The distinguishing property**, worth keeping: a vocare defect asserts on something
NO CALLER CAN OBSERVE — it reddens on internal churn and stays green when the
behaviour breaks. The goldens are the opposite: they redden only on caller-visible
change and are maximally sensitive.

**Why it fooled me:** a Rust line number FEELS like an implementation detail. But a
`rust_caller_span!()` in an error is the honest answer to "where did this come from"
when there is no wat source — and this same session proved the converse mattered,
since `RhsOp::Bind`'s MISSING span was a real user-facing defect (`15064c9eb`). The
location cannot be load-bearing when absent and noise when pinned.

No action. Recorded so the retraction outlives the claim.

---

## CLOSING TALLY — the vigilia worked through, 2026-08-24 (HEAD `15dcca1df`)

**CLOSED.** `temperare` 1 L1 · `purgare` 1 L1 · `struere` 1 L2 · `intueri` 3 L1 ·
`conformare` 9 L1 + the gap it never named (`RhsOp::Bind` carried no span at all — the
audit examined sites that HELD a span to discard, not the path that had already thrown
one away) · `excusare` 2 L2 · `perspicere` 2 of 3 L2 · **`solvere` all 7 L2** ·
`vocare` 4 of 6 · `complectens` 1 of 2 · `conferre` 1 of 2 · `circumspicere` 1 of 3
(one DEFERRED by ruling, one correctly scoped as unrunnable in CI).

**STILL OPEN — nothing here can compute a wrong answer.**
- `conferre` 1 L2 — `wat_scripts_grid_axes_live.rs` header prose says "18 axes"; the
  arrays hold 41. Prose drift, load-bearing only for a reader reasoning about coverage.
- `exigere` 1 L1 + 1 L2 — a cache-stone "a later stone" that never came and is tracked
  nowhere; one "not a v1 blocker" row with no owner, arc, or gate.
- `perspicere` 1 L2 — `fire/mod.rs`'s `harvested` re-spells the existing `QueryMemory`
  alias, which is in scope and is literally what it is assigned into.
- `sequi` 2 L3 — `arm.rs`'s `ARM_TABLE` rune is categorised `host-idiom` where the
  structurally identical `EXEC_ARENA` two files over is `ambient-context`; and
  `bind_only`/`cond_key_ids` exist as two live copies (round-local + session) whose
  reason is nowhere stated, having already misled two scans.
- `vocare` 2 of 6 — `probe_arc278_49_one_core_covers_the_surfaces` (an honest DESIGN
  probe; NOT a vantage defect — see the retraction below) and
  `differential_exists_no_multiplicity`, named for a contract its fixture cannot reach.
- `complectens` 1 of 2 — the leading-filter contract has no base-layer unit test.
  DELIBERATELY not taken: `filter_pass` is unreachable from an integration test, and
  going in-crate means the empty-`:rhs` implementer vantage just marked as a runed
  exception.
- `circumspicere` 1 L2 — the grid's SPEED half runs in no CI job. Correctly scoped: it
  needs Clara and a JDK the runner lacks. The CORRECTNESS half never needed Clara and
  now runs on every floor.
- **T7** — `Op::Or`/`Op::Not` allocate a `SlotFrame` per branch per element. Last, and
  reclassified: it was never "cold", it was UNEXERCISED, and that half is now closed by
  `where-or-inline`. The allocation itself remains, on a rare shape, measurable only by
  building an axis for it — more work than the fix.
- **RECORD** — `wat-rs/CLAUDE.md`'s delivery claim (fix lives in the FROZEN holon root:
  builder's call) and `tmp/VIGILIA-LOOP.md`, stale and untracked.

## WHAT THIS ARC LEARNED THAT OUTLIVES IT

1. **When native disagrees with BOTH the `$oracle` and Clara, native is wrong** — and
   the real question is which fixture was missing. Three engine divergences, three times
   the two references agreed with each other.
2. **Three kinds of gap, hiding differently.** MISSING data (no fixture drove it),
   DISCARDED data (computed, correct, unread), MASKED data (a correct layer hides a
   broken one). The middle is cheapest to fix and easiest to walk past.
3. **A predicate can be right for a reason unrelated to why it is right.**
   `ends_with("::=")` held for `enum::=` only because that row's `core_name` is the
   generic spelling. Right answer, wrong reason — and the wrong reason is what drifts.
4. **Duplication is only a mumble until you check whether the copies disagree on a
   REACHABLE input.** Same ward, same grade, same file family: one triplication was a
   live L1, the next was benign.
5. **A ward's RETRACTION is also just a report.** `solvere` withdrew a true finding; the
   disk said otherwise. And a ward can only audit what is in front of it — `conformare`
   examined nine sites that held a span, and never the path that had already discarded
   one. The gap is usually one question further on.
6. **Arm a `panic!` to answer "does anything execute this?"** Cheaper and more decisive
   than reading every fixture. Paid three times.
7. **A gate that cannot go red is decoration** — mutation-test every one before landing
   it — and a gate that can silently compare a value to ITSELF is worse than none
   (`:derived` vs `:oracle-derived`; the `q-scan` control row).
