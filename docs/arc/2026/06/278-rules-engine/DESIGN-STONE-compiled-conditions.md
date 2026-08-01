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

`src/rete/matcher.rs` (the compiler and the executor) and `src/rete/kernel.rs` (build the compiled
forms alongside the tree at setup; call the executor from step 1). **Nothing under `wat/`** — the
oracle does not move, and stays naive by ruling.

## The gate

1. **The differential, on bindings — not on a boolean.** Over a corpus of (condition, fact) pairs
   drawn from the live grid axes, `compiled(cond, fact)` must equal `alpha_match_inner(cond, fact)`
   **including the exact bindings array**: same pairs, same order, same values. A both-matched
   comparison that ignores the bindings would pass while producing wrong joins downstream.
2. **Zero allocation on the failure path**, asserted via the existing `census_count` counters — not
   inferred from a timing improvement.
3. **`alpha:match` falls in BOTH columns** of `a0_depth_cost_split_at_equal_work`. This stone is
   per-call, so unlike the tree it must improve the shallow column too. If only the deep column
   moves, something depth-shaped is being measured instead.
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
