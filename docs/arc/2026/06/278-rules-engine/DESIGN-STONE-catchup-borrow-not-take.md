# DESIGN-STONE — catch-up borrows the parent beta, it does not take it

> **Origin (2026-08-24).** `DESIGN-STONE-partire-fire-loop` closed leaving
> four named follow-ups. This is the one that was a latent CORRECTNESS
> hazard rather than a performance question, so it goes first — rete is
> the cornerstone the rest of wat's subsystems are meant to copy, and a
> cornerstone does not ship an invariant held by hand.

## The enemy

```rust
let (all_left, restore_parent) = match wm.beta.remove(node_id) {
    Some(v) => (v, true),
    None => (Vec::new(), false),
};
…                                            // ~100 lines, 12 levels deep
    Err(e) => { if restore_parent { wm.beta.insert(*node_id, all_left); }
                return Err(e); }             // restore site 1 — error path
…
if restore_parent { wm.beta.insert(*node_id, all_left); }   // restore site 2
```

`DESIGN-STONE-catchup-take-left` introduced this, and it was a real
improvement: it replaced a full `.cloned()` of the parent's token vector with
a move. What it left behind is an invariant held by **convention** — one take,
two restores, one of them nested twelve levels inside an error arm. Audited at
extraction: exactly one early exit in the window and it does restore. It is
correct today. A future `?` anywhere in those 100 lines silently drops a beta
memory, and no test asserts it.

## The probe, and what it found

The obvious strike was a guard: an RAII shape or a `with_parent_beta_taken`
scope that restores however the body exits. Before designing one,
`extirpare`'s deeper rung — *do not eliminate the failure, eliminate the
situation that produces it* — asks whether the take is needed at all.

The stone's stated reason is *"HashMap split-borrow needs the parent out of
the map while `entry(child)` mutates."* Checked against the window as it
stands: **every mutable touch of the session inside it is `wm.bind_pool` or
`wm.match_pool`**, and those are disjoint fields from `wm.beta`. Rust splits
disjoint field paths, so a shared borrow of the parent coexists with them. The
`entry(child)` emit the take was protecting now runs *after* the window.

So the take was tried as a borrow, and it compiles.

**The workaround outlived its cause.** That is not a criticism of the stone
that introduced it — taking beat cloning, and the surrounding code has moved
since.

## The algorithm

```rust
let all_left: &[Token] = wm.beta.get(node_id).map(|v| v.as_slice()).unwrap_or(&[]);
```

Both restore sites delete. The error arm becomes `Err(e) => return Err(e)`.
`&all_left` iterations become `all_left`.

## ★ THE ONE CONTRACT DECISION

**The parent's beta is read where it lives; nothing is removed from the map,
so nothing has to be put back.** The "empty vs missing restored exactly"
guarantee the previous stone had to state is not preserved — it is
*unnecessary*, because the map is never disturbed. No guard is added: there is
nothing left to guard.

## The gate

1. Oracle differential `spec_equals_native_on_every_where_family` green — the
   load-bearing one, since a wrong beta would change derived facts.
2. Rete cohort 363/363. `differential_three_stratum_negation` 3/3.
3. `probe_arc278_concurrent_retes` 5/5.
4. Floor GREEN. Clippy `-D warnings` silent.
5. Leftover `Instant`, same session, before/after.

## Predicted win

Written before measuring: **correctness first, perf incidental.** Removing a
`HashMap::remove` plus re-`insert` per catch-up node per round should show as a
small drop on the fanout cell — call it **−0.2 to −0.7 ms** — but a wash would
still land this stone, because the invariant is the reason for it.

## Weigh (2026-08-24) — LANDED

Clippy silent. Rete cohort **363/363** including the oracle differential.
`differential_three_stratum_negation` **3/3**.
`probe_arc278_concurrent_retes` **5/5**.

`fanout_three_leftover_split` `[100 20]`, same session, each figure already a
mean of 3:

| | before (take/restore) | after (borrow) |
|---|---:|---:|
| without-query FIRE | 24.31 | **23.63 · 23.20 · 23.59** |
| with-query FIRE | 29.74 | **28.67 · 28.32 · 28.64** |

**without-query FIRE ≈ −0.8 ms, with-query ≈ −1.2 ms**, stable across three
after-runs. Above the top of the predicted range, and the mechanism is plain:
a `HashMap` remove hashes, relocates and may leave a tombstone, and the
re-insert hashes again — per catch-up node, per round.

`harvest:query` 6.01 → 6.19 is unrelated to this window and inside its noise.

**What this does NOT claim:** not that `DESIGN-STONE-catchup-take-left` was
wrong — it beat the clone it replaced, and this only became available as the
code around it moved. Not that other take/restore shapes in the engine are
likewise unnecessary; this was checked for this window only.
