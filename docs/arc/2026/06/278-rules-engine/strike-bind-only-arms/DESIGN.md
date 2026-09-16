# DESIGN — the arm labelled THE PRODUCTION PATH disables the branch production takes

## Why

Work-list **C4**. Two timed arms in `accum_alpha_cost.rs` call the real
`super::alpha_activate_fact` and hand it `let bind_only = HashMap::new()`. `fire/delta.rs:71`
computes the branch from exactly that map:

```rust
let fields = cx.bind_only.get(&aid).map(Vec::as_slice);
let skip_span = compiled.fact_bind().is_none()
    && match fields { Some([]) => true, Some(_) => row.is_some(), None => false };
```

An empty map returns `None` for every id, so `skip_span` is **false unconditionally** and the arm
always runs `exec_compiled_with_key_ids`. Production builds the map at `fire/delta.rs:339-346`.

**Driven, and this is why the row is not cosmetic** (`f90d4c126`, the committed probe):

```
C4 PROBE: bind-only conds 3/3 · built pool=0 vals=1000 · empty pool=120200 vals=1000
```

**All three** conds in the accum axis are bind-only. Production interns **nothing** here; the two
arms intern **120,200 pool entries**. They do not mis-measure the production path — they measure
its opposite branch, on 100% of conds, and print it as `A alpha_activate_fact`.

## ⛔ THE CUT THAT IS NOT OBVIOUS: BUILDING THE MAP BREAKS BOTH TABLES

Both tables are **cumulative ladders** with A at the top, and both derive a row by subtracting a
lower rung. Measured at HEAD:

```
table 1 (leftover_split)  M + exec_compiled 12.09 → A alpha_activate_fact 14.18 → A−M push      2.09
table 3 (push_split)      M exec_compiled   11.61 → A alpha_activate_fact 13.90 → A−M push lump 2.28
```

`A−M` is "push" **only while A ⊇ M**. A contains M today *because* `skip_span` is forced off. Build
the map and A skips that exec entirely (probe: `pool=0`, no interning at all) — A drops **below** M,
`A−M` prints **negative**, and `A−D`, `H−M`, `D−V` are anchored on the same A.

So "just build the map" is not the fix. It trades a dishonest label for a nonsense subtraction.

## The contract decision, pinned

**Every timed arm declares which `skip_span` branch it takes, and the table says so in the label.**

- The existing arm is KEPT and RELABELLED — it is `alpha_activate_fact` with `skip_span` forced
  OFF, which is what makes the ladder nest. That is a legitimate decomposition of the exec path and
  the arc uses it; what was illegitimate was calling it the production path.
- A NEW arm is added with the map built exactly as `fire/delta.rs:339-346` builds it — the
  production-faithful cost — reported as its own row, NOT folded into the ladder.

Rejected: *build the map in place and cut the derived rows.* It fails **Good UX** — it destroys a
decomposition that is genuinely informative — and the four questions do not license discarding a
working instrument to fix a label.

Rejected: *leave it and fix only the label.* Fails **Honest** at the table level: the file would
then report no production number at all for the function it is named after.

## Files

- `src/rete/kernel/tests/accum_alpha_cost.rs` — the two arms, the two label blocks, one new arm each.
- Nothing under `src/rete/kernel/fire/`. **The engine is not the defect here.**

## Out of scope = REJECTED

- C3 (`accum_cost.rs:1630`, the phase mark), C5, C6. Same class, separate rows, separate strikes.
- The `compiled:calls` union finding (below). It is real and it is C3-adjacent; it is **not** this
  strike, and this strike must not quietly widen into it.

## ⚠ A SECOND FINDING THE PROBE TURNED UP — NOT THIS STRIKE'S TO FIX

`compiled:calls` cannot see this branch. `fire/delta.rs:78` bumps it in the skip arm and
`compiled_cond.rs:928` bumps it inside `exec_compiled_with_key_ids` — the else arm — deliberately.
It reads **80,200 either way** (driven; my first probe used it and was refuted). `accum_cost.rs:52`
pins that number as a correctness assertion and is therefore blind to which branch produced it.
Recorded on the work list, not fixed here.
