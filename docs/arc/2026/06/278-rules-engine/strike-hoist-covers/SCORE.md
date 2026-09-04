# SCORE — −32.4% of the filter phase, and the instrument I named could not see it

> **Written after the orchestrator's own weighing.** The ★ was that all four artifacts pointed the
> measurement at code structurally incapable of observing the change.

## The result

**Live `filter` phase, node-share [50 200], six samples per side:**

| | samples (ms) | median | range | spread |
|---|---|---|---|---|
| before | 0.394 0.406 0.394 0.389 0.411 0.391 | **0.394** | 0.389–0.411 | 5.6% |
| after | 0.262 0.279 0.277 0.266 0.262 0.267 | **0.267** | 0.262–0.279 | 6.4% |

**−32.4%, ranges fully disjoint** (before-min 0.389 > after-max 0.279).

The loop itself, measured by mirroring the hoist into the arms' replica (restored by hash afterwards,
not in the delivered tree): **J−I 281.7 → 148.7 µs, −47.2%, disjoint.** G/H/I/L unchanged.

**No wall-clock claim.** Fire is 0.18% of wall on this axis.

## ⛔⛔ THE ★ — ALL FOUR ARTIFACTS NAMED AN INSTRUMENT THAT CANNOT SEE THIS CHANGE

BRIEF item 4, the DESIGN, EXPECTATIONS row 5 **and the rider's prompt** all direct the before/after at
`node_share_cost.rs` arms G–L.

**Arms J and K are a hand-written REPLICA of the loop.** They call `arm.where_tree.covers(tid)`
directly (`:332`). `dispatch_where_tests` is private to `fire/mod.rs` and appears in that test file
**only in comments** — and one of them says it outright at `:299`:

> *"Arms G..K **reconstruct** the REUSE branch of `dispatch_where_tests` only."*

Measured with **only** the engine edit applied: `J−I` **281.7 → 282.4 µs**. Zero movement.

**Had the rider followed the brief, it would have reported "unmeasurable, STOP-2" for a change worth
−32.4%.** The word *reconstruct* was in the file the brief cited.

The right instrument was in the same test, unnamed by any artifact: the **live `filter` phase read**
(`node_share_phase_census`, 7 real fires), printed as `vs a LIVE 'filter' of X.XXX ms`. That is engine
code.

### And the two instruments cross-validate each other

- the live phase moved with the **engine** edit and **not** with the arms-mirror (0.267 both times);
- the arms moved with the **mirror** and **not** with the engine edit.

**Each is proven to see only what it measures** — the evidence my artifacts assumed without checking.

## Mutations

**★ 1 — the gate still bites on the NEW code.** Drop `&& !maybe.contains(&tid)` from the hoisted
loop → **RED**, `where-join-order row 5`, `DROPPED` non-empty and `INVENTED: []`. **Row 2 earned on
the new code, which is the whole reason this strike waited behind the differential.**

**2 — my mutation as written does not prove what I said it proves.** `covered := !covers` REDs, but it
collapses the **reference** arm: the differential builds that arm with `WhereTree::empty()` precisely
so `covers` is false everywhere, so inverting it makes the empty tree report full coverage and the
reference arm takes the *tree* branch. The panic text even mislabels 35 correct facts as *"INVENTED"*.
**It proves the vector feeds `use_tree`, not that the loop reads it.**

**2b — the rider's replacement, which is the mutation I meant.** Invert only the loop read
(`!covered[i]`), leaving `use_tree` on true coverage → **RED** on the anti-vacuity arm:
`skips 13,982 → 0`, tree-evals 17,945 → 31,912, **derived facts identical at 9,576**. Skipping is only
an optimisation, so the *sets* stayed correct and the gate caught it anyway. **The differential is
stronger than a set comparison.**

**3 — unprovable on this corpus, and proven so rather than inferred.** `covered[0]` instead of
`covered[i]` → GREEN. The rider shipped a probe that panics on any mixed-coverage `tids` slice; across
115 fixtures / 34,368 pairs it never fired. **Positive control:** inverting the probe to fire on
*uniform* coverage REDs immediately (`MIXED-COVERAGE tids SLICE FOUND`), proving the probe was
compiled and live. **Corpus finding: no dispatch ever sees mixed coverage**, structurally — an
uncovered dispatched tid would fall through to `exec_stashed_where` and raise
*"TestNode N has no compiled where"*.

That is the difference between *"no hits"* and *"my probe was broken"*.

## Gates

- **Differential GREEN on the new code**, byte-identical totals to HEAD: 115 fixtures, **9,576 derived
  facts**, 17,945 tree-evals, 2,441 reuse, **13,982 skips**, 34,368 pairs.
- Floor: **`5408 tests run: 5408 passed (1 slow), 21 skipped`**, rc=0, **0 FAIL rows**.
- `binary_id(wat::lint)` **258 passed**; clippy `--release --all-targets` rc=0, zero warnings.

**STOP-3 discharged**: `covers` is `self.ids.contains(&id)` over `ids: HashSet<i64>`, and
`WhereSink.where_tree` is `&'a WhereTree` — shared, no interior mutability, no `&mut` path.
Token-independent structurally, confirmed empirically by identical differential totals.

## The SLOW row — the rider flagged it without a baseline; the orchestrator has thirty

`SLOW [> 15.000s] reachability_shard_0_of_6` appears across floors at **many** HEADs this session,
back to one at 5203 tests — long before this strike. The count oscillates **0 → 6** across runs of
different trees, sibling shards flipping in and out; the immediately preceding floor had 0, two
before it had 2. **Not new, not attributable to this change, and a timing annotation on a PASS rather
than a red.** The rider was right to refuse to characterise it without a baseline.

## Honest deltas

- **"the Clara ratio" is unreachable in scope.** The prompt asked for it as part of the honest claim;
  Clara is the JVM grid, which the DESIGN and the prompt both forbid this strike from running. Fire
  time only.
- **My `~414 µs` phase figure sits above the entire six-sample HEAD range** (389–411, median 394). It
  is the denominator anyone will compare against.
- **Mutation 2 must be recorded beside 2b**, or the recorded proof overstates itself.
- The BRIEF's edit snippet was applied verbatim and held up: compiles clean, clippy-silent.
