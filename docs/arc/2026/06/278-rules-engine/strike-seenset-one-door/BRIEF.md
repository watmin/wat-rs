# BRIEF — A4: give the fixpoint's dedup set one door

Read `DESIGN.md` first, including the severity note: **the hazard is not live** — `from_parts` is the
sole stamping site, so equal aggregates cannot land in different halves. This is structural hygiene
plus one guard, and the SCORE must say so.

## Read in order

1. `src/rete/kernel/fire/delta.rs:186-197` — `seen_insert`. The whole behaviour moves into
   `SeenSet::insert` unchanged.
2. `src/rete/kernel/fire/delta.rs:279-281` — where the two sets are created, and `:491`, `:513`,
   `:547`, `:567`, `:587`, `:618`, `:637`, `:656` — where the pair is threaded. These collapse to one
   handle.
3. `src/rete/kernel/fire/pass/mod.rs:155-156` — the ctx struct's two fields. One field after.
4. `src/rete/kernel/fire/pass/production.rs:31-32`, `:88`, `:122` — a caller that takes both
   handles, **reserves one half alone**, and inserts through the door. `:88` becomes
   `SeenSet::reserve`.
5. `src/rete/kernel/fire/pass/round_census.rs:33-34`, `:133` — `seen_ids.len() + seen_rest.len()`,
   the union's cardinality computed at the call site. Becomes `seen.len()`.
6. `src/rete/kernel/fire/pass/alpha.rs:133-134`, `:182` — the other insert site.
7. `docs/arc/2026/04/109-kill-std/NOTE-value-hash-has-two-functions-selected-by-a-flag.md` — the
   `src/value/` invariant this strike makes rete robust against rather than dependent on. **Do not
   fix that note here.**

Test-side users (`accum_cost.rs:1710`, `gather_probe_cost.rs:262`/`:271`,
`accum_alpha_cost.rs:481`…`:610`) construct their own pairs for timing arms — they move to
`SeenSet` too, and their arms must keep measuring the same work.

## Sketch

```rust
/// The fixpoint's dedup set. Stamped aggregates key on their construction
/// fingerprint; everything else stores the whole value. ONE set, two
/// representations — `insert` is the only door.
pub(crate) struct SeenSet { ids: FxHashSet<u64>, rest: FxHashSet<Value> }

impl SeenSet {
    pub(crate) fn insert(&mut self, v: &Value) -> bool { /* today's seen_insert body */ }
    pub(crate) fn len(&self) -> usize { self.ids.len() + self.rest.len() }
    pub(crate) fn reserve(&mut self, n: usize) { self.ids.reserve(n) }
}
```

## The guard

Inside `insert`, on the `rest` arm **when the value is an `Aggregate`** — the only suspicious case —
`debug_assert` that the same logical value is not already present as a stamp in `ids`. Release cost
zero. It turns a silent dedup failure into a panic if the `src/value/` invariant is ever broken.

**STOP-2 if that needs a hash re-derivation on the hot path.** Report the cost; a cheaper expression,
or none with the reason written down, both beat a slow assert.

## Mutation proof — from OUTSIDE the module

After the change, reaching `ids` or `rest` directly must not compile. Write the probe in a file that
does **not** own the type and quote the `error[E0616]`/`E0609`. A probe placed inside the owning
module reaches a private field legitimately and proves nothing — that mistake was made earlier this
session on the `SlotZip` bypass.

## Blast radius

`src/rete/kernel/fire/delta.rs` (the type + creation + 8 threading sites) ·
`fire/pass/{mod,production,round_census,alpha}.rs` · four test files' timing arms.
**No `src/value/`. No behaviour. No change to what is deduped.**

## STOP triggers

1. Any measured value changes → STOP; dedup behaviour is identical.
2. The `debug_assert` needs a hot-path hash → STOP and report the cost.
3. A signature outside `src/rete/` needs changing → STOP.
4. `prod:derivations` or `seen_facts` move on any axis → STOP; dedup changed.

## Prior result to copy for shape

`../strike-factbag-one-owner/SCORE.md` — the same cure on the fact base: one owner, doors, the
honest load-order delta, and the behaviour-neutrality claim checked against pre-change values.
