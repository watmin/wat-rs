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

### T2 — the Exists leaf still memcpys occupancy — ⚠ RE-RANKED, needs its own probe

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

### T4 — `token_assoc` heap-allocates per call to walk the pool

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

### T5 — two clones per derived fact per stratum
`src/rete/kernel/fire/rules.rs:206-207`
```rust
if acc_derived_set.insert(d.clone()) { acc_derived.push(d.clone()); }
```
One clone feeds the dedup set, one feeds the vec. Both are Arc bumps of
`Value::Aggregate`. Kin to T1 — the same stratum loop. Consider folding into
T1's carried-set change rather than a separate strike.

### T6 — Exists candidates materialize a Vec purely to key a HashSet
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

### T7 — `Op::Or` / `Op::Not` allocate a SlotFrame per branch, per element
`src/rete/compiled_cond.rs:870,878`
```rust
let mut clone: SlotFrame = slots.to_vec();
```
The **copy is the semantics** (a failed branch must not leak bindings) — that
part is WHAT. The **malloc is not**: a reused scratch frame
(`clear()` + `extend_from_slice()`) carried through `exec_ops` removes the
allocation and keeps the copy. Hot in any `where` with `:or`/`:not` over a
large alpha.

### T8 — `d_beta_from_parents` materializes a capacity-less Vec
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
