# ward `temperare` — vigilia 2026-09-05

> Cast at HEAD `21530efab`, branch `grok-rete`. Read-only unless the ward says otherwise.
> **Verbatim ward return, preserved unedited.** The 2026-08-30 cast lost its 19 returns
> because they lived only as subagent messages; this file exists so that cannot recur.
> Nothing here is adjudicated — see `../WORK-LIST.md` for what the orchestrator credited.

---

I hold `temperare`. Read-only sweep of `src/`, `tests/`, `wat/` at `21530efab`. Every citation below is a line I opened this session.

---

# L1 — defects

### 1. `join_extend` does three SipHash `HashMap<i64,_>` lookups per join emission, all keyed on a value constant for the whole join node

`src/rete/kernel/fire/mod.rs:686-724` — the innermost function of the engine, called once per emitted `(token, element)` pair.

- **`mod.rs:692`** — `rematch_compiled(ctx.compiled_conds, alpha_id)?` → `compiled_conds.get(&alpha_id)` (`mod.rs:352`).
- **`mod.rs:693`** — `compiled.has_seed_cmp()`, a function of `compiled` and therefore of `alpha_id` alone.
- **`mod.rs:709`** — `span_from_row(…, alpha_id, ctx.i64_by_fact, ctx.bind_only, ctx.cond_key_ids)`, whose prologue (`mod.rs:599-607`) is `bind_only.get(&alpha_id)?`, `cond_key_ids.get(&alpha_id)?`, `kids.len().saturating_sub(fields.len())` — two more lookups plus a length subtraction, all of `alpha_id` alone.

`alpha_id` is a parameter, fixed for both enclosing loops at every one of the four call sites: `hash_join.rs:240` (catch-up cross-join), `hash_join.rs:444` (step 4 term2), `hash_join.rs:509` (step 3 term1), `mod.rs:852` (`keyed_join_persistent` probe). Each is `for tok in … { for el in bucket { join_extend(tok, el, alpha_id, ctx) } }`.

**Trip count, cited:** the code's own arithmetic sizes it — `hash_join.rs:220-224` computes `n_join = all_left.len() * (n_right / idx.len())` and `hash_join.rs:225` reserves against it with *"Reserve the 40k appends."* The sibling hoist note at **`hash_join.rs:286-288`** states the emit path handled **80,000 map lookups = 2 per token on the fanout cell**, i.e. 40,000 tokens through one emit; `join_extend` runs at least once per such token. On the accum axis `accum_alpha_cost.rs:41` pins `alpha_elements == 80_200`.

**Aggravating:** `compiled_conds`, `bind_only` and `cond_key_ids` are all `std::collections::HashMap` (`arm.rs:665`, `session.rs:175`, `session.rs:178`; `use std::collections::HashMap` at `session.rs:3`, `arm.rs:35`) — SipHash-1-3 on an `i64`. The neighbouring `AlphaMemory` is `FxHashMap` (`session.rs:154`). So this is the expensive hasher, three times, per pair.

**Tempered direction:** resolve the per-alpha triple once per join node and carry it in `FireCtx` — which exists for exactly this ("the split borrow that lets a join hold eleven session fields", `mod.rs:57`). Removes 3N lookups → 3 per node.

---

### 2. `root_join_delta`'s inner loop repeats five map operations per element on keys fixed by the two enclosing loops — and the batched form is 20 lines away

`src/rete/kernel/fire/pass/root_join.rs:59-79`. Loop nest is `for node_id in &arm.kind_ids.alpha` → `for child_id in child_ids` → `for ei in news.iter()`.

- **`root_join.rs:65`** — `span_from_row(&mut wm.bind_pool, &el, *node_id, &wm.i64_by_fact, &wm.bind_only, &wm.cond_key_ids)`. Two `HashMap<i64,_>` lookups (`mod.rs:599`, `mod.rs:602`) keyed on `*node_id`, the **outermost** loop variable. Only the `pool.push` at `mod.rs:609` is real per-element work.
- **`root_join.rs:78`** — `record_token(&mut wm.beta, d_beta, &arm.beta_readers, *child_id, tok)` → `beta_readers.contains(&node_id)` + `beta.entry(node_id).or_default()` + `d_beta.entry(node_id).or_default()` (`pass/mod.rs:45-49`), three hash operations on `*child_id`, the middle loop variable.

That is five hash operations per element where five per `(alpha, child)` pair would do. Every batched leaf element carries `binds.len == 0` (`alpha.rs:208`, `make_element(idx, 0, 0)`), so **all** of them take the `span_from_row` branch, not the cheap `seed_token_binds` one.

**Trip count, cited:** `news` is the whole `0..len` range for a packed seed (`mod.rs:1732-1734`, `DESIGN-STONE-seed-d-alpha-range`); `accum_alpha_cost.rs:41` pins `last.alpha_elements == 80_200` on the accum axis, and this loop visits every one of them once per RootJoin child.

**Tempered direction:** `record_tokens` — the batched twin — is at `pass/mod.rs:59-75`, already does one `entry` + one `reserve` + `extend_from_slice`, and its doc says outright *"growing a `Vec` one token at a time inside the fire loop is the cost that reserve exists to avoid."* `hash_join.rs:428` and `hash_join.rs:389` use it. Root-join, the most element-dense caller in the engine, does not. Hoist the `span_from_row` prologue to the `node_id` scope and buffer the tokens into one `record_tokens`.

---

### 3. `production_delta` pays a `HashMap` `entry` per derived fact — the identical hoist is documented 100 lines away

`src/rete/kernel/fire/pass/production.rs:124-127`:

```rust
wm.production
    .entry(*node_id)
    .or_default()
    .push(derived.clone());
```

This sits inside `for pid in pids { … for tok in ts { for (compiled, slots) in … } }` (lines 70, 87, 88). `*node_id` is the **outermost** loop's variable — constant for the entire nest. `ProductionMemory = HashMap<i64, Vec<Value>>` (`session.rs:156`), std hasher.

**Trip count, cited and pinned by an existing gate:** the `entry` runs once per *new* derived fact; `census_count("prod:derivations")` at **`production.rs:112`** sits nine lines above it and `fanout_cost.rs:236-237` asserts that counter is **exactly 40,000** on the fanout cell (`keys=100 × fanout=20`). So ≤40,000 `entry` lookups where 1 per `(node, pid)` suffices.

**This is a known, solved problem in this codebase.** `hash_join.rs:286-288` carries the fix verbatim: *"`entry()` HOISTED out of the per-token loop: the key is constant, so the old form paid two map lookups per token (80,000 on the fanout cell) where two total will do."* Production was not swept.

**Tempered direction:** buffer new facts into a local `Vec<Value>` per `pid` and `extend` once, mirroring `record_tokens`. Note the borrow shape: `wm.derived_facts` (line 129) and `wm.production` are disjoint fields; only the opt-in explain arm at line 117-122 (`encode_view(wm)`) takes `&wm` whole, so it is the one thing that must stay outside a held `&mut wm.production`.

---

### 4. `key_of_el`'s `col_field_of` is hoisted in two places in `mod.rs` and hoisted in none of the three `hash_join.rs` per-element loops

`col_field_of` (`mod.rs:1580-1591`) is a pure function of `(intern.alpha_id, join_key)` — no element input at all. Its body is two `HashMap<i64,_>` lookups, then **a linear `position` scan over the whole interned `bind_keys` table with `Value` equality** (`mod.rs:1584-1587`), then a second `position` scan over `kids`.

`key_of_el` (`mod.rs:1637`) calls it per element: `mod.rs:1645` in the **unary** arm and `mod.rs:1655` per join key in the n-ary arm.

The hoist is already written, twice:
- `build_gather_index`, unary arm — `mod.rs:1861-1865` lifts `col_field_of` and `key_id` above `for (i, el) in elements.iter().enumerate()`.
- `GatherIndex::append`, unary arm — `mod.rs:1701-1706`, same lift.

It is missing at every per-element `key_of_el` site:
- **`hash_join.rs:188`** — `for &el in right` over `all_right`, the whole cumulative `wm.alpha[alpha_id]`; the census immediately below (`hash_join.rs:213-216`) records `n_right` and the comment at `:210` states *"`n_right` is the loop's exact trip count."*
- **`hash_join.rs:318`** — `for ei in dr.iter()`, step 2's Δright append.
- **`hash_join.rs:433`** — `for ei in dr.iter()`, step 4's term2.
- and inside `mod.rs` itself, the n-ary arms at **`mod.rs:1715`** and **`mod.rs:1879`**, which the unary arms beside them deliberately avoid.

Each of those also rebuilds a fresh `GatherIntern::from_wm(wm, alpha_id)` (8 borrows, `mod.rs:1532-1546`) inside the loop body.

**Trip count, cited:** `accum_alpha_cost.rs:41`, `alpha_elements == 80_200` for the catch-up walk over `all_right` on that axis; the Δright walks are round-scoped subsets of the same population.

**Tempered direction:** give `key_of_el` a precomputed `&[Option<u8>]` fields-per-join-key slice, resolved once where `jk` is bound (`hash_join.rs:148`) — the same shape the two unary arms already use.

---

### 5. `ensure_gather` re-derives its own cache key on every call — two heap allocations per token, thrown away on a hit

`src/rete/kernel/fire/mod.rs:1889-1908`:

```rust
let join_keys: Arc<[Value]> =
    gather_join_keys(sample, els, GatherIntern::from_wm(wm, alpha_id)).into();
let index = cache.entry((alpha_id, Arc::clone(&join_keys))).or_insert_with(|| …);
```

`gather_join_keys` (`mod.rs:1461-1500`) filters the sample's binding keys through `col_field_of` (`mod.rs:1471`, the same linear-scan function as §4), `.cloned().collect()`s into a `Vec<Value>`, and `sort_by`s it (`mod.rs:1487-1499`). `.into()` then allocates an `Arc<[Value]>` and memcpys. On a **cache hit** — which is the designed common case — all of that is discarded; only the `entry` probe's hash of `(i64, Arc<[Value]>)` (hashing every `Value::String`'s bytes) is consumed.

The join keys are a function of `(alpha_id, the sample's key set)`, and every token reaching one Negation/Exists node carries the same key set.

**Where the loop is:** `filter.rs:236-245`, `for tok in new_tokens { token_exists_under(driver, …, gather_cache) }` → the `CondDriver::Leaf` arm (`mod.rs:439-449`) → `any_seeded_keyed` (`mod.rs:1912`) → `ensure_gather`. `driver` is already correctly hoisted at `filter.rs:235`; the key derivation is not.

**Trip count:** `new_tokens` = `d_beta_from_parents(…)` per filter node per round. On the accum world `gather_probe_cost.rs:9-16` enumerates the readers: `exists -> Reading-?g (filter pass)`, one of five readers over two distinct `(alpha_id, join_keys)` pairs. `gather_probe_cost.rs:53-62` already gates the *build* at ≤2 and ≤80,000 elements — the key derivation runs on every call underneath that green gate and no counter sees it.

**Tempered direction:** cache the `Arc<[Value]>` per `(node_id, alpha_id)` for the node's token loop, or resolve it once at `filter.rs:235` beside `driver`. Note that a hoist here would sit *behind* an already-green gate — `[[a_cache_can_make_a_gate_unfalsifiable]]` cuts the other way: nothing today can go red if this regresses.

---

# L2 — weaknesses

**a. The fire path hashes `i64` keys with SipHash while its sibling map uses FxHash.** `AlphaMemory = FxHashMap<i64, …>` (`session.rs:154`), but `BetaMemory` (`:155`), `ProductionMemory` (`:156`), `ParentsOf` (`:160`), `JoinKeysCache` (`:168`), `JoinLeftIndex` (`:208`), `CondKeyIds` (`:175`), `BindOnlyFields` (`:178`), and the arm's `feeding_alpha_of` / `parents_of` / `beta_readers` (`arm.rs:572-575`, `arm.rs:673-688`) are all `std::collections::HashMap`/`HashSet` (`session.rs:3`, `arm.rs:35`). Every finding above is multiplied by that choice, and nothing in the tree states it as a decision. Remedy: either switch the fire-scoped maps to `FxHashMap` (the file already imports it — `session.rs:6`, `arm.rs:41`) or write the rune saying why SipHash is wanted on a node id.

**b. `lookup_form` rebuilds the entire builtin type and check environment on every call.** `src/runtime.rs:13833-13835`:
```rust
let _builtin_types = crate::types::TypeEnv::with_builtins();
let env = crate::check::CheckEnv::with_builtins_and_types(&_builtin_types);
if let Some(scheme) = env.get(name) { … }
```
`register_builtin_types` spans `src/types.rs:908`→`2830`; `register_builtins` spans `src/check.rs:16470`→`21664` — roughly 7,000 lines of registrations, plus `types.build_unit_variant_map()` (`check/env.rs:255`), constructed and dropped to answer one `get(name)`. Both inputs are constant. Growth dimension is *n reflection calls* (`runtime.rs:13930`, `:14033`, `:14303`, `intrinsic/reflect.rs:247`), and the corpus calls those rarely — so this is a weakness, not a defect. Remedy: `OnceLock` the pair; the comment at `:13832` says "on-demand CheckEnv", which names a laziness the code does not have.

**c. `query_class_scans` is computed twice with identical arguments on the fire-rules query path.** `rules.rs:507` computes `let scans = query_class_scans(&arm, network)` and returns early at `:509` when `class_scans_cover_queries` holds; the fall-through reaches `harvest_stratified_queries` at `:579`, which recomputes `query_class_scans(full_arm, network)` at `rules.rs:376` from the same `session` and re-tests the same predicate at `:377` — a branch already proven false. Network-sized, not fact-sized, but it is two full walks of `arm.kind_ids.join_parent` and `arm.kind_ids.alpha` with a `get_node` PMap probe each (`mod.rs:1088-1105`, `:1109-1143`). Remedy: thread the already-computed `scans` in as a parameter; the `:257` caller passes `None` and computes its own.

**d. `dispatch_where_tests` builds two `HashSet<i64>` per token** — `mod.rs:2058-2059`, from `cands.proven`/`cands.maybe`, which `where_tree.candidates` (`where_tree.rs:187-196`) has just allocated as two fresh `Vec`s. Four heap allocations per token before a single probe. **This is adjacent to the item you told me is rowed but is a different site** — the rowed item is the *probes* per `(token, tid)`; this is the *construction* per token, which today's hoist does not touch. Remedy: reuse two scratch sets across the token loop (`clear()` + `extend`), or have `candidates` write into caller-owned buffers.

**e. `alpha_seed` allocates `input_facts.len()` capacity for every leaf class.** `pass/alpha.rs:86-88`: `class_ids.insert(class.clone(), (Vec::with_capacity(input_facts.len()), true))` runs once per key of `leaf_aids`. With K leaf classes and N input facts that is K×N×4 bytes reserved where the union of all `ids` is at most N. On the accum axis N = 40,200 (`accum_alpha_cost.rs:34`). Remedy: `Vec::new()` and let it grow, or divide the hint by K.

**f. `alpha_activate_fact` re-derives per-`aid` facts on every fact.** `fire/delta.rs:69-78`: `rematch_compiled(cx.compiled_conds, aid)`, `cx.cond_key_ids.get(&aid)`, `cx.bind_only.get(&aid)` — three SipHash lookups per `(fact, candidate aid)` pair, plus a `skip_span` whose only fact-dependent term is `row.is_some()` (`:76`). The arm is immutable for the whole fire, so all four could be an `aid`-indexed side table built once beside `cond_key_ids` (`delta.rs:346-355`). Growth is facts × candidates per fact.

**g. `fold_bucket` re-resolves its operand column per token.** `fire/acc.rs:339` and `:373` call `packed_operand_field(var, view, sample)` → `operand_field` → `view.col_keys.iter().position(|k| k == var)` (`acc.rs:182`) — a linear `Value`-comparison scan invariant across the accumulate node's whole token loop (`pass/accumulate.rs:184`). The *inner* bucket fold is correctly hoisted; the enclosing per-token call is not. Same for the `!operand_keys.iter().any(|o| o == *k)` half of the `group_keys` filter at `pass/accumulate.rs:198`, which depends on nothing the token varies.

---

# L3 — judgement

- **The engine already knows this discipline; it is applied unevenly.** `hash_join.rs:286`, `mod.rs:1701`, `mod.rs:1861`, `pass/mod.rs:59`, `pass/accumulate.rs:153` are all correct hoists with the reasoning written down. Findings 2, 3 and 4 are the same hoist *missing* at its most element-dense sites. A sweep that grepped for the pattern rather than fixing the site it was noticed at would have caught all three — `[[a_finding_names_one_site_enumerate_the_rest]]`.

- **`span_from_row`'s shape invites the defect.** It takes `alpha_id` plus three tables and re-derives `(fields, kids, skip)` on every call (`mod.rs:591-620`). Three of its five call sites are inside per-element loops (`root_join.rs:65`, `mod.rs:709`, `filter.rs:123`). Splitting it into a resolve-once `SpanPlan { fields, kids, skip }` and a `plan.write(pool, el, row)` would make the invariant half *unrepresentable* inside a loop, rather than merely discouraged.

- **The census is honest about cost and blind to this class.** `census.rs:300-345` compiles every instrument out of release and documents its own tax — that discipline is sound and none of the above is instrumentation. But every counter here measures *occurrences of an operation the design names* (`accum:index-builds`, `prod:derivations`, `right_idx_appended`). None measures *lookups performed*, so §1–§5 are all invisible to the 265 gates by construction. If any of these is cured, gate the ratio — lookups per emission — not the millisecond.

---

# What I could not check, and why

- **I did not run anything.** No build, no floor, no bench, no `cargo wat`. Every number above is either a constant asserted by a test I read (`accum_alpha_cost.rs:41` = 80,200; `fanout_cost.rs:237` = 40,000) or a trip count stated by the code's own comment (`hash_join.rs:210`, `:225`, `:286`). **I have measured no speedup and I claim none.** Six samples or no number — I have zero samples.

- **I could not fan out.** I tried to launch two Explore agents over `check.rs`/`types.rs` and `runtime.rs`/`value/`/`collection/`; both were refused — twenty subagents already running for this vigilia. So `src/check.rs` (22,509 lines), `src/runtime.rs` (40,883), `src/collection/`, `src/macros/`, `src/edn_shim.rs`, `src/io.rs`, `src/load.rs` and `src/freeze.rs` got only targeted greps, not a read. **`runtime.rs` in particular is the interpreter loop and is almost certainly where the next findings of this class live; I sampled one function of it.** Treat the non-`rete` surface as unswept.

- **I read no `.wat`.** The target named `wat/` and I did not open it. `wat/rete/oracle/pass.wat` is the reference implementation the whole fire path mirrors, and I cannot say whether the oracle carries the same invariant work.

- **Relative cost is unranked.** I can say §1 sits in the innermost function and §3 has a pinned 40,000, but I cannot say which of the five is worth the most, because that requires driving them and I am read-only. The `phase_end` marks already in place (`"root-join"`, `"  ├ prod:dedup-store"`, `"  ├ hj:catchup:right-idx"`) would answer it — but per the standing ruling, do not add a mark to a hot path to find out.

- **§5's remedy is the one I trust least.** `GatherCache` is keyed `(i64, Arc<[Value]>)` precisely so two readers of one alpha with different parent key-sets do not collide — `gather_probe_cost.rs:22-25` names that as failure mode (b) and says the gate cannot catch it. Any hoist of the key derivation must preserve that, and I have not traced every `ensure_gather` caller's key-set stability. Do not act on §5 without driving the differentials that stone points at.
