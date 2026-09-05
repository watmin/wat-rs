# DESIGN — A1: a `:where` query loses rows, and D2's cure fixed only one side

> Drawn 2026-09-05 at HEAD `8bca0f7fe`. Source: vigilia 2026-09-05 CLASS A1. Found independently
> by `solvere` and `sequi`; **both named the wrong writer**; `experiri` drove it and corrected the
> mechanism. **Orchestrator re-drove it at THIS HEAD before drawing** — it reproduces exactly:
>
> ```
> native=[OutW=1,OutP=2,C=2,OutN=2]   oracle=[OutW=2,OutP=2,C=2,OutN=2]
> ```

## The defect: a user query returns 1 row where the spec says 2

Shape: `[A ?k] :where [B ?k] [C ?k ?v]` — a guard followed by TWO fact conditions, with a `C`
**derived one round later** for a key already seen. The native engine drops the join.

## The mechanism, every line verified on disk this session

`join_keys_cache` (`session.rs:168`) is documented as a memo of a join's shared-variable list.
`hash_join.rs:120` gives it a **second, undeclared job**:

```rust
let first_keying = if !join_keys_cache.contains_key(child_id) {
```

Membership IS the latch for "the one-time catch-up on J has run". That catch-up (`hash_join.rs:160`)
is the **only bulk builder of `left_idx`** — `hash_join.rs:270`, whose own comment reads
*"Build left_idx[J] from ALL cumulative left tokens."*

The second writer is **`left_activate_join`** (`fire/pass/mod.rs:107`), reached from
`filter_after_join.rs:75` — a HashJoin child of a *frontier* HashJoin, so it IS in
`kind_ids.join_parent` and pass 3 does visit it. It goes through `keyed_join_persistent`, which
writes `join_keys_cache.entry(join_id)` at `fire/mod.rs:802` — and its `FilterJoinIdx`
(`fire/mod.rs:784-787`) has exactly two fields, `right_idx` and `join_keys_cache`. **No `left_idx`.**

Round 2: `contains_key(C)` is already true → `first_keying` false → catch-up skipped forever →
`left_idx.get(child_id)` at `hash_join.rs:429` is a **silent `None`** → `term2 = old_left ⋈ Δright`
never runs. Rows vanish. No error, no diagnostic, exit 0.

⚠ **`solvere` and `sequi` both cited `keyed_join_persistent` reached from `join_after_filter`
(pass 3.6). That is WRONG** — those joins have a *filter* parent and `kind_ids.join_parent` is
`RootJoin | HashJoin` only (`arm.rs:542`), so pass 3 never visits them. Two independent wards
converged on a real defect through a false citation. Do not inherit it.

## ★ D2's cure fixed the right side and left this hole open — in the same struct

`FilterJoinIdx`'s own doc at `fire/mod.rs:782-783` says why it exists:

> *"a sibling `&mut HashMap<i64, usize>` that only THIS struct's user maintained, while two sites
> in `hash_join_delta` appended to the same buckets without it."*

That is D2, described by its own cure. **The same struct now carries the left-side version of the
identical defect.** `session.rs:224-231` already ruled on the general case: *"⛔ THE CURE IS
STRUCTURAL, NOT CONVENTIONAL."*

## The one contract decision, pinned

**The cure is STRUCTURAL, not a patch to `first_keying`.** Fold the join's left buckets and its key
list into ONE type whose only door builds both — the shape `JoinRightIndex` (`session.rs:242`)
already has on the right side, and `ClassIntern::intern` (`export.rs:1671`) has elsewhere. A writer
that sets the keys without indexing the left side must not be **writable**.

Patching `first_keying` alone is rejected on two grounds: it cures today's two writers and leaves a
third free to appear, and `sequi` L2-a found the conflation is currently **load-bearing as D2's
guard** — the catch-up's right-index walk pushes the whole alpha memory rather than the tail, and
what protects it from double-counting is precisely that `keyed_join_persistent` sets the same key
that gates `first_keying`. Break the conflation naively and you reopen D2.

## Scope

**IN:** the structural cure + the gate, **in this one strike**, floor GREEN at the end.

**OUT, affirmatively cut:** the other nine CLASS A instances; `sequi` L2-a's tail-only right walk
(name it in the SCORE if the cure makes it safe, do not land it here); Ω3, Ω4, Ω5.

## ⛔ Why cure and gate ship together

The previous strike (`../strike-mode-parity/`) shipped a gate WITHOUT its cure and the orchestrator
pushed a red floor, then argued in the commit message that the red was principled. It was not —
`CLAUDE.md`: *"No test is pre-blessed — not by name, not by category."* **A strike ends with a green
floor or it does not end.**
