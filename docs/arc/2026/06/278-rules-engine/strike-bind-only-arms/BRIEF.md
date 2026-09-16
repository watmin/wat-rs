# BRIEF — give each alpha arm a `bind_only` that says which branch it takes

Two timed arms in one benchmark file call the real `alpha_activate_fact` with an empty `bind_only`,
which forces the `skip_span` branch OFF and makes them time the opposite path from production. Keep
those arms (their ladders nest *because* the branch is off), relabel them honestly, and add one
production-faithful arm to each table beside them.

## Read in order

1. `src/rete/kernel/fire/delta.rs:67-80` — **the mechanism.** `cx.bind_only.get(&aid)` decides
   `skip_span`; `None` (what an empty map always returns) forces the `else` arm. Read this first;
   everything below follows from it.
2. `src/rete/kernel/fire/delta.rs:339-346` — **how production builds the map.** This is the shape to
   copy, literally.
3. `src/rete/kernel/tests/accum_alpha_cost.rs:530-534` — **the arm in this same file that already
   does it right.** Copy this, not the prose above it.
4. `src/rete/kernel/tests/accum_alpha_cost.rs:219-258` — table 1's broken arm; the empty map is
   `:231`, the call is `:233`.
5. `src/rete/kernel/tests/accum_alpha_cost.rs:262-300` — table 1's label block. `A` is `:269`,
   `A−M push` is `:274`.
6. `src/rete/kernel/tests/accum_alpha_cost.rs:1088-1120` — table 3's broken arm; empty map `:1100`,
   call `:1102`.
7. `src/rete/kernel/tests/accum_alpha_cost.rs:1124-1148` — table 3's label block. `A` is `:1130`,
   `A−D` `:1135`, `A−M` `:1136`.
8. `src/rete/kernel/tests/accum_alpha_cost.rs` tail — `c4_probe_bind_only_decides_skip_span_for_the_accum_axis`,
   the committed probe. It builds the map correctly and states the numbers; use it as the worked
   reference for the new arms.

## Sketch

In each of the two tables, beside the existing accumulator, add one more:

```rust
let mut a_prod = f64::INFINITY;          // beside `let mut a = f64::INFINITY;`
...
// after the existing `a = a.min(...)` arm, same run loop:
let mut bind_only_prod: HashMap<i64, Vec<u8>> = HashMap::new();
for (&id, c) in &arm.compiled_conds {
    if let Some(fields) = crate::rete::compiled_cond::bind_only_fields(c) {
        bind_only_prod.insert(id, fields);
    }
}
a_prod = a_prod.min(elapsed_ns(|| { /* the SAME body, `bind_only: &bind_only_prod` */ }));
```

Then in each label block: relabel the existing row to say the branch is forced off, and add the new
row. The existing derived rows keep pointing at the existing `a` — they nest, and that is the point.

## Blast radius

`src/rete/kernel/tests/accum_alpha_cost.rs` **only**. No `src/rete/kernel/fire/` edits, no new types,
no change to `bind_only_fields` or to `skip_span`. The engine is correct; the instrument is not.

## STOP triggers

1. **If `a_prod` comes out ABOVE the existing `a`**, stop and report. The whole strike rests on
   skip_span being cheaper; a production arm that is *slower* than the forced-exec arm means the
   mechanism is not what `delta.rs:71` reads like, and the fix is wrong, not the number.
2. **If you find yourself editing anything under `src/rete/kernel/fire/`**, stop. That is a
   different strike and this one has said so.
3. **If a derived row would print a negative number**, stop and report which. Do not "fix" it by
   reordering the ladder or clamping — a negative there is a finding about what nests.
4. **If `bind_only_fields` returns `None` for all three conds** in your run, stop: that contradicts
   the committed probe and one of the two is wrong.

## Report

Both tables, before and after, verbatim. Say for each new row whether it landed above or below its
neighbours and by how much. Name any derived row whose meaning changed.
