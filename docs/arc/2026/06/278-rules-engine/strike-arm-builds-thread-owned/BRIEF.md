# BRIEF — make `ARM_BUILDS` thread-owned, like the table it counts

## The work

`ARM_BUILDS` (`src/rete/kernel/arm.rs:728`) is a process-global `AtomicUsize` counting builds into
`ARM_TABLE`, which sits thirteen lines above it and is **`thread_local!`** by an explicit ZERO-MUTEX
contract. Move the counter into a `thread_local! { … Cell<usize> }` beside it. Update the 22 read
sites' spelling. Recategorise its rune to `ambient-context`. Add one probe that proves the
thread-ownership.

**No signature changes. No production behaviour change** — the counter is `#[cfg(test)]`.

## Read in order

1. `.../strike-arm-builds-thread-owned/DESIGN.md` — why thread-ownership rather than the bool
   `sequi` proposed, and the pinned `Cell`-not-`Atomic` decision.
2. `src/rete/kernel/arm.rs:703-730` — **`ARM_TABLE`'s `thread_local!` and its rune.** This is the
   shape to copy and the argument to reuse; your new rune should read like its neighbour.
3. `src/rete/kernel/arm.rs:903-909` — `build_rete_arm`, the single write site, `#[cfg(test)]`.
4. `src/rete/kernel/tests/arm_lease.rs` — the five consuming tests (reads at `:17,30,43`, `:110,113,124`,
   `:151,157`, `:181,184,192`, `:263,284`).
5. `src/rete/kernel/tests/cascade_cost.rs` — 3 more reads. ⚠ `sequi` established these only **print**
   into a report table and never gate pass/fail; they must keep working, but they are not the risk.
6. `docs/CONVENTIONS.md`, "The `rune:sequi` vocabulary" — the four categories and their decisive
   questions. `ambient-context` is *"real DOMAIN state, reached globally or per-thread instead of
   threaded"*, and `ARM_TABLE` is its cited example.

## Implementation sketch

```rust
thread_local! {
    #[cfg(test)]
    static ARM_BUILDS: Cell<usize> = const { Cell::new(0) };
}
```

with the smallest accessor pair the call sites need (`arm_builds()` / the `+= 1` at the write site).
Keep the reads' *shape* — snapshot, act, snapshot, compare — so the five tests stay recognisable.

**The new probe** is the deliverable that makes this real: **spawn two threads, build an arm in
each, and assert each thread observes exactly its own count.** Under the old process-global that
assertion is false; under thread-ownership it holds. `arm_lease.rs:53-91`
(`intern_index_thread_owned_workers_do_not_collide`) already spawns threads for exactly this
subsystem — copy its shape, and put your probe beside it.

## STOP triggers

1. **If any of the 22 sites needs more than a spelling change** — STOP and report which. The design
   claims the read/compare shape is preserved; if it isn't, the design is wrong.
2. **If the two-thread probe passes BEFORE your change** — STOP. It would mean the probe cannot tell
   thread-owned from process-global, and it is the only positive proof this strike has.
3. **If `cascade_cost.rs`'s three reads turn out to gate a verdict** rather than print — STOP.
   `sequi` said they only print; if that is wrong, the blast radius is larger than the brief.
4. Do not touch `ARM_TABLE`, `.config/nextest.toml`, or any production (non-`cfg(test)`) path.

## Blast radius

`src/rete/kernel/arm.rs` (the declaration, the write site, the rune), the 22 read sites in
`arm_lease.rs` + `cascade_cost.rs`, one new probe. **Nothing else.**

## ⛔ Write `SCORE.md` AS YOU GO, not at the end

Append each measurement and each row's result to `SCORE.md` **when you get it**. A previous executor
held 54 measurements in context, backgrounded a floor, and stopped — the numbers existed nowhere on
disk and were nearly lost. Land them as you take them.
