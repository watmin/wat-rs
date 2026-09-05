# DESIGN-STONE — conditions compile once; the matcher stops re-deriving a static program

> **Origin (2026-07-31).** Grounding why Clara pays less per alpha test than we do. Their
> `activation` is a fn compiled once at session-build (`compiler.clj:1907`, `engine.cljc:529`); ours
> re-derives the program from a `WatAST` on **every fact**. Measured at constant work, our per-unit-
> depth cost grows ~2.3× theirs — and reading the call, it is not "interpretation overhead" in the
> abstract. It is that **everything statically known is recomputed dynamically, with allocations, in
> the innermost loop.**
>
> **Queued behind `DESIGN-STONE-alpha-discrimination-tree.md`.** The tree cuts the *number* of calls;
> this cuts the *cost* of each one. They are multiplicative — `D × C` → `1 × c` — and they edit
> adjacent code, so they do not fly together.

## What one `alpha_match_inner` call actually pays

For the cascade's `(?l <- :level)` + `(:wat::core::= ?l 7)` — grounded, `src/rete/matcher.rs`:

| line | cost, per fact × per alpha | why it is waste |
|---|---|---|
| `:223-228` | `cond_head.trim_start_matches(':') != fact_class` | **redundant** — `alpha_by_type` already proved the type before the call |
| `:365` | `classify_rete_clause(clause)` per clause | re-derives the shape of a **static** AST every call |
| `:715` | `field_names.iter().position(\|n\| n == field_name)` | linear scan + string compares for an index fixed at compile time |
| `:411` | `Value::String(Arc::new(var.to_string()))` | **heap allocation** for the bind key — a constant |
| `:509` | `Value::String(Arc::new(name.to_string()))` | **second heap allocation**, same constant, in `resolve_operand` |
| `:412` | `bindings.iter().find_map(\|(k,v)\| *k == key …)` | linear scan comparing those freshly-built Strings |

**Two heap allocations and three-plus string operations per fact per alpha**, to rebuild the constant
`"?l"`. At `facts × D` = 500,000 calls at `[50 100]`, that is ~1M allocations for a key known at
network-compile time.

## The change — a pre-resolved program, built where the tree is built

Not a JIT, not codegen. A small instruction sequence produced **once** at network-compile time, from
the immutable network, at the same setup site the discrimination tree uses:

```rust
enum Op {
    BindField   { field_idx: usize, slot: usize },             // no name lookup, no key alloc
    CmpFieldLit { field_idx: usize, op: CmpOp, lit: Value },
    CmpSlotLit  { slot: usize,      op: CmpOp, lit: Value },
    CmpFieldSlot{ field_idx: usize, op: CmpOp, slot: usize },
}
struct CompiledCond { ops: Vec<Op>, slot_keys: Arc<[Value]>, n_slots: usize }
```

Every dynamic lookup above resolves at build time: the head check **disappears** (the index proved
it), clause shapes become enum variants, field names become `usize`, `?var` references become slot
indices, and the binding-key `Value`s are built once into `slot_keys`. Execution is a loop over `ops`
with array indexing.

## ★ THE ONE CONTRACT DECISION

**Slots internally; the public `Arc<[(Value, Value)]>` is materialized ONCE, on SUCCESS ONLY.**

The intermediate accumulator becomes a `Vec<Value>` indexed by slot. Only when every op has held does
the executor zip `slot_keys` with the slot values into the canonical bindings array that
`DESIGN-STONE-element-bindings-array.md` made the Element's shape.

The reason is the failure path, and it is the whole point: **most calls fail** — that is what a
matcher is for. Today a failing call still allocates both key Strings before it discovers the
mismatch. Under this contract **a failed match allocates nothing at all**; the successful one pays a
single array build it was going to pay anyway.

The corollary that keeps it honest: the public binding representation **does not change**. `Element`
and `Token` see exactly the `Arc<[(Value, Value)]>` they see today, with the same keys, in the same
order. This stone is invisible above the matcher.

**`alpha_match_inner` is NOT deleted.** It stays as the reference implementation and the other half of
the differential (§ the gate). Whether it survives once it has no production caller is a separate
ruling and not this stone's to make (`feedback_no_consumers_does_not_mean_dead`).

## Blast radius

`src/rete/matcher.rs` (the compiler and the executor) and `src/rete/kernel/` (build the compiled
forms alongside the tree at setup; call the executor from step 1). **Nothing under `wat/`** — the
oracle does not move, and stays naive by ruling.

## ⚠ AMENDED 2026-08-01 — the perf case COLLAPSED; the stone stands on correctness

Everything above about the MECHANISM is unchanged and still grounded. The SIZING is not: this
document was written before the alpha discrimination tree landed, and its origin cites an
`alpha:match` of 117 ms that no longer exists.

Post-tree, measured (`fanout_per_call_alpha_census`, R4's 40,000-pair cell, `D=1` — the shape where
the tree buys nothing and every millisecond is per-CALL cost):

```
  ROUND LOOP          82.751 ms
    production        37.407 ms     32% of the fire
    hash-join         31.585 ms     27%
    alpha              4.057 ms
      alpha:match      1.211 ms      1.1%   <-- everything this stone targets
  OUT: to_persistent  31.525 ms     27%
  fire             ~ 115.19 ms
```

I had extrapolated ~11 ms from the cascade's per-fact rate. **Actual: 1.211 ms — wrong by ~10x, in
the direction that favoured building it.** The census also says WHY: fanout derives 40,000 `Pair`
facts, `Pair` appears in no condition, so `alpha_by_type.get` returns `None` and those facts
`continue` before any per-call cost. The TYPE TIER already discards ~91% of the facts for free; my
extrapolation assumed every fact walks the match path.

**So there is no workload on the nine-axis board where this is worth a strike on perf grounds.**
Eliminating the target entirely buys ~1% fact-heavy, ~5% on the cascade.

### Why it is built anyway (the builder's ruling, 2026-08-01)

> *"i say we build the thing we speced out - prove the shape - we will need this shape of solution
> later... i do not wish to abandon a solution we have repeatedly asserted is resolving a problem
> that is described as WRONG, NOT SLOW."*

Three reasons, none of them a percentage:

1. **It is wrong, not slow.** The condition AST is immutable from compile onward; re-deriving the
   program from it on every fact is a defect whose cost share happens to have dropped. A class does
   not become correct because something else got faster
   (`feedback_choose_correct_not_cheap_difficulty_is_not_a_design_axis`).
2. **Allocation pressure is a threat to a property we CLAIM.** R2 sells "no GC, so the tail is
   jitter-free by construction" as a structural edge over Clara, and the tail is what the DDoS case
   actually needs. ~1M heap allocations per fire rebuilding the constant `"?l"` is the most obvious
   remaining threat to a measured CV of 3.5%. That argument never depended on CPU share.
3. **The shape is needed for the regime we are building toward.** The workload where this pays is
   many facts each reaching many conditions — every packet against thousands of rules, i.e. R25's
   chaos engine. That axis does not exist yet. Proving partial evaluation on a small, understood
   surface now is cheaper and safer than discovering it at 1M rules under load.

### What the amendment CHANGES in this document

- **The gate below is re-based on COUNTERS, not timing.** A move from 1.211 ms to ~0.4 ms on a
  115 ms fire is inside the noise; a scorecard row reading "alpha:match falls materially" would be
  unfalsifiable. The mechanism is provable exactly (allocations on the failure path go to zero) and
  that is what gets gated.
- **The `TOTAL (nested phases)` figure in any table quoted here is suspect.** The census probe's
  first draft summed indented rows as its denominator and printed shares totalling 209.3%; fixed to
  divide by the top-level phases. Millisecond figures were always sound; the percentages were not.

## The gate

1. **The differential, on bindings — not on a boolean.** Over a corpus of (condition, fact) pairs
   drawn from the live grid axes, `compiled(cond, fact)` must equal `alpha_match_inner(cond, fact)`
   **including the exact bindings array**: same pairs, same order, same values. A both-matched
   comparison that ignores the bindings would pass while producing wrong joins downstream.
2. **Zero allocation on the failure path**, asserted via the existing `census_count` counters — not
   inferred from a timing improvement.
3. **`alpha:match` does not REGRESS in either column** of `a0_depth_cost_split_at_equal_work`, and
   does not regress in `fanout_per_call_alpha_census`. Per the amendment this is a no-harm row, NOT
   a win row: the target is 1.211 ms of a 115 ms fire, so any improvement is inside the noise and a
   scorecard that demanded one would be demanding an unfalsifiable claim.
4. **Setup cost stays bounded** — compilation does not push `SETUP: indexes` past the budget the tree
   stone set.
5. **`:accuracy :match` on every grid axis, and the release floor unchanged.**

## Out of scope = REJECTED (affirmative cuts)

- **The discrimination tree.** Its own stone; lands first; this one assumes it.
- **Compiling the RHS, `eval_test_core`, and the accumulate fold.** The same recompute-a-static-
  program shape plausibly appears there — *I have not read those hot paths, so this stone does not
  claim it.* Tracked as `DESIGN-STONE-compiled-rhs` in this arc, to be drawn only after the same
  grounding this one got.
- **Any JIT or native codegen.** A resolved instruction vector is the whole idea; a code generator is
  a different project.
- **Changing the public bindings representation.** Pinned above as the contract's corollary.
- **Keyword interning.** `109-kill-std/NOTE-keyword-storage-must-intern.md` is real and it finally
  *pays* here — that note was filed with an honest "NOT the rete lever" caveat because binding-map
  *lookup* measured 1.0–1.2×, and that verdict is correct for lookup and wrong for **construction**,
  which is what this path does a million times. It remains a language-level change with its own arc;
  smuggling it into this diff would destroy the attribution.
- **`wat/rete.wat`.** The oracle is never optimized.
