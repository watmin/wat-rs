# BRIEF — the alpha network becomes a discrimination tree

## The work

Today every fact entering the rete is match-tested against **every alpha node of its own type**, one
at a time. In a depth-D cascade that is D tests per fact where exactly one can succeed, and it is
**79% of the measured depth cost**. Replace that linear scan with a **discrimination tree**: a fact
walks root-to-leaf, one field per level, and arrives at only the alphas it could possibly satisfy.

The reference implementation is ours. We shipped this shape twice at line rate six months ago, in
the kernel. You are bringing it home to `src/rete/`.

All edits land in `/home/watmin/work/holon/wat-rs/`. Verify with `pwd` first; if the reported path
contains `.claude/worktrees/`, re-anchor to that absolute path and use
`git -C /home/watmin/work/holon/wat-rs` for git.

## Read in order

1. **`docs/arc/2026/06/278-rules-engine/DESIGN-STONE-alpha-discrimination-tree.md`** — the measurement,
   the contract, and the affirmative cuts. The contract decision is the whole stone; read it twice.

2. **`src/rete/kernel.rs:2009-2050`** — the loop you are replacing. Note `alpha_by_type.get(fact_class)`
   then `for aid in alphas`, and that `alpha_match_inner` is called per (fact × alpha).

3. **`src/rete/kernel.rs:1916-1930`** — where `alpha_by_type` and `alpha_cond` are built at setup
   (P8), from the immutable network. **This is where the tree gets built**, once, in the same place.

4. **`src/rete/matcher.rs:285` and `:365`** — `ReteClauseShape` and `classify_rete_clause`. This is
   the shared classifier your analyzer **consumes**; it already recognises `Bind`, `Constraint`,
   `And`, `Or`, `Not`, `Where`. It exists to be the single source of "what shape is this form."

5. **`src/rete/matcher.rs:406-446`** — the `Bind` and `Constraint` arms, so you can see how a field
   reaches a variable and how a constant reaches a comparison. The cascade's discriminator is
   *bind-then-constrain*: `(?l <- :level)` followed by `(:wat::core::= ?l 7)`.

6. **`/home/watmin/work/holon/holon-lab-ddos/veth-lab/filter/src/tree.rs:75`** — `ShadowNode`, the
   reference node shape (read-only reference; nothing in that repo changes). Three edge kinds in one
   node: equality fan-out, wildcard, guarded ranges. `Rc` for subtree sharing.

7. **`src/rete/kernel.rs`, test `a0_depth_cost_split_at_equal_work`** — the committed instrument. It
   fires the cascade at equal work and prints the per-phase split. It is your gate for row 3.

## ★ THE ONE CONTRACT DECISION

**The tree may OVER-approximate. It may never UNDER-approximate.**

`alpha_match_inner` remains the sole authority on whether a condition holds and the sole producer of
bindings. The walk returns a **candidate set**; the matcher then runs on the survivors exactly as it
does today. For every fact: `walk(fact) ⊇ { alphas that actually match }`.

Anything your analyzer cannot prove a discriminator for — `not=`, `or`, `not`, a computed operand,
an unfamiliar shape — goes on the **wildcard edge** and is therefore always walked. Being unable to
understand a condition costs a wasted `alpha_match_inner` call and can never drop a derivation. Lean
on this: **when in doubt, wildcard.** A conservative tree is a correct tree.

## Implementation sketch

A new module, `src/rete/alpha_tree.rs`:

```rust
pub(crate) struct AlphaTree { roots: HashMap<String, Rc<Node>> }   // fact class → root

pub(crate) struct Node {
    dim: usize,                                  // index into this class's declared field order
    children: HashMap<Value, Rc<Node>>,          // field value → subtree (equality fan-out)
    wildcard: Option<Rc<Node>>,                  // alphas that do not constrain `dim`
    range_children: Vec<(RangeEdge, Rc<Node>)>,  // present in the type, UNPOPULATED this stone
    leaves: Vec<i64>,                            // alpha ids terminating here
}

impl AlphaTree {
    // Built ONCE at setup, from (class → alpha ids) + `alpha_cond`, partitioning by dimension.
    pub(crate) fn build(...) -> Self;
    // Pre-extract the fact's field values once, then descend: at each node take the `children`
    // edge for fields[dim] if present, AND the wildcard edge if present; union the leaves.
    pub(crate) fn candidates(&self, class: &str, fields: &[Value]) -> Vec<i64>;
}
```

At the call site, `for aid in alphas` becomes `for aid in tree.candidates(fact_class, fact_fields)`.
Nothing inside the loop changes.

The analyzer's job, per alpha condition: produce `Option<(field_index, Value)>` per dimension where
an equality discriminator is **provable** — resolve the `Bind` var→field chain, then a `Constraint`
with op `:wat::core::=` whose other operand is a literal. Everything else: no discriminator, wildcard.

## Blast radius

`src/rete/kernel.rs` (the setup-time build and step 1's inner loop) plus the new
`src/rete/alpha_tree.rs`. `matcher.rs` is consumed, not edited. Nothing under `wat/` changes — the
oracle does not move.

## STOP triggers (each is a rejection: ship nothing, report the gap)

1. **STOP-1** — if `classify_rete_clause` is not reachable from your new module (visibility,
   module boundary), STOP and report it. A second private parser for condition shapes re-opens the
   exact drift hole arc 294 extracted that classifier to close.
2. **STOP-2** — if the superset invariant fails for any fact, STOP and report the fact, the walk's
   candidate set, and the matcher's true set. Do not relax the invariant to make it pass.
3. **STOP-3** — if mean candidates per fact at `[50 100]` does not fall to ~1, STOP and report the
   actual distribution. A correct tree that discriminates nothing is not this stone.
4. **STOP-4** — if the work appears to require an edit under `wat/`, or a semantic change inside
   `matcher.rs`, STOP and report what forced it.

## Definition of done

- A test asserting the **superset invariant** over every fact of the `[50 100]` cascade.
- A test asserting **mean candidates per fact ≈ 1** at `[50 100]`, and that it is `~D` with the tree
  bypassed (so the assertion cannot pass vacuously).
- `a0_depth_cost_split_at_equal_work` re-run: report the full table before and after.
- Tree construction does not push `SETUP: indexes` past ~2 ms at `[50 100]`.
- `cargo nextest run --release` — the floor, read from the Summary line, never a piped exit code.
- Report: both tables, the two new tests' output, the candidate distribution, and `git diff --stat`.

Leave the tree dirty and uncommitted. Do not commit, push, or stash, and leave alone any file you
did not write (there are untracked A0/A1 grid files in the tree that are not part of this work).

## A prior result to copy for shape

`ShadowNode` in the kernel tree is the node. For how a substrate stone lands in this repo — a new
module consumed by `kernel.rs` with the oracle untouched and a differential as the gate — the
closest recent shape is the native-Element stone (`32142f8a`) and the array-bindings stone
(`41c59cde`).
