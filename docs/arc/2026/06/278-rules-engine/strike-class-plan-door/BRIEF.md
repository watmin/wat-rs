# BRIEF — A8: make the class plan a type with one door

Cure **and** prove it. **Floor GREEN when you are done.**

## Read in order

1. **`DESIGN.md`** — the contract is one door plus a DERIVED `has_mixed`, and the hot-path
   constraint is non-negotiable.
2. **`src/rete/kernel/fire/pass/alpha.rs:60-96`** — the header (it explains the original
   double-write defect and calls the bool the cure) and the two declarations.
3. **`alpha.rs:126-150`** — the two arms, and **`:131-137`**, which argues the packed-arm-first
   ordering. Read that before you restructure anything.
4. **`alpha.rs:212-222`** — `any_mixed` gating `activate_deferred_mixed_classes`.
5. **`src/rete/kernel/session.rs`** — `JoinRightIndex` / `JoinLeftIndex` / `BetaStore`: the shape
   this arc has landed three times. Private state, one door, no `&mut` escape.

## Implementation sketch

```rust
// private map; `has_mixed` derived, never stored
impl ClassPlan {
    #[inline]
    fn observe(&mut self, class: &str, i: u32, packed: bool) -> bool {
        if packed {
            if let Some(e) = self.map.get_mut(class) { e.ids.push(i); return true; }
            false
        } else if let Some(e) = self.map.get_mut(class) { e.uniform = false; true }
        else { false }
    }
    fn has_mixed(&self) -> bool { self.map.values().any(|e| !e.uniform) }
}
```

Return value replaces the `continue`/fall-through: `true` = handled (deferred), `false` = fall
through to `alpha_activate_fact`. **Branch on `packed` before the lookup**, exactly as today.

## The proof

**A compiler error**, the shape this arc has produced three times:

```
error[E0616]: field `map` of struct `…::ClassPlan` is private
```

Re-introduce a demote-without-gating (set `uniform = false` reaching past the door) and quote what
the compiler says. And show `has_mixed` cannot drift: there is no field to set.

## Blast radius

`src/rete/kernel/fire/pass/alpha.rs` (+ `session.rs` if the type lands there). No wat corpus change.

## STOP triggers

1. **If ANY `*_cost` gate moves, STOP and report the number.** `accum_alpha_cost` pins
   `alpha_elements == 80_200`; the seed pass is the most element-dense loop in the engine. A moved
   cost number means the cure taxed the batch path — the thing the site's own header forbids.
2. **If the borrow checker forces `observe` to take the whole `wm`, STOP.** The doors in this arc
   deliberately take narrow `&mut`s; `pass/mod.rs` records why (the `FireCtx` lesson).
3. **If you cannot derive `has_mixed` without a second walk that shows up in a cost gate, STOP and
   report both options** — a cached flag with a private setter is a fallback, not the plan.
4. **On any RED: DO NOT RE-RUN.** Capture whole, name the arm, surface it.

## Prior result to copy for shape

`../strike-beta-write-door/` — private state, one door, proof is a compiler error, and the SCORE
names the one `cfg(test)` hatch and why it is safe.
