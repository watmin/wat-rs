# DESIGN — `ARM_BUILDS` is process-global while the table it counts is thread-owned

**Status:** drawn 2026-09-08. Collapses vigilia rows **`4D1`** (secare, L1) and **`4Q2`** (sequi, L1)
— one defect seen through two lenses — into one cure.

## Why

`src/rete/kernel/arm.rs:728`:

```rust
#[cfg(test)]
// rune:sequi(performance-counter) — test-only intern-miss count; not fire domain.
pub(crate) static ARM_BUILDS: std::sync::atomic::AtomicUsize = …
```

Incremented at exactly one site — `build_rete_arm` (`:908`), under `#[cfg(test)]`. Read at **19
sites across 5 tests** in `src/rete/kernel/tests/arm_lease.rs` and **3** in `cascade_cost.rs`.

**`4D1`:** those five tests snapshot it before and after a call and assert on the delta
(`assert_eq!(ARM_BUILDS.load(…), builds, "fire HIT must not rebuild")`). That is race-free **only
because nextest forks a fresh OS process per test** — a fact stated in a *comment* at
`.config/nextest.toml:3-4`, not a config key. ⛔ **The builder has already ruled on this exact
shape, in this arc:** *"the tests must never have a race, period… **Not 'green under our runner.'
Race-free by construction.**"*

**`4Q2`:** `docs/CONVENTIONS.md`'s closed-set table defines `performance-counter` as *"instrumentation,
off by default — **nothing about the answer; arming it cannot change a result, only a
measurement**"* — and cites **`ARM_BUILDS` itself** as the example. In these five tests it is the
**sole oracle**: if it stopped counting, all five assertions pass vacuously (`0 == 0`).

## ⭐⭐ The structural error, and the file already names it

Thirteen lines above the counter, `ARM_TABLE` — **the very table `ARM_BUILDS` counts builds into** —
is `thread_local!`, under a rune that spells out the contract:

> *"ZERO-MUTEX intern index is thread-owned RefCell … Connection-thread affinity is the ZERO-MUTEX
> contract (`DESIGN-STONE-intern-zero-mutex`): fire/release on another thread **miss the arming
> thread's row**."*

**A build is a per-thread event by the file's own design. The counter counting those events is
process-global.** That mismatch is the whole defect: it is why the tests need process isolation, and
it is why a counter the convention calls harmless decides five verdicts.

## What this delivers

**`ARM_BUILDS` becomes `thread_local! { static ARM_BUILDS: Cell<usize> }`, matching `ARM_TABLE`
one line up.**

- **Race-free by construction**, not by runner — the builder's ruling, satisfied structurally.
- **No signature changes and no caller ripple.** The read/compare shape in all 22 sites is preserved;
  only the accessor spelling changes.
- The counter finally **agrees with what it counts**.
- ⭐ **`4Q2` closes as a consequence, not a separate edit**: once thread-owned, the honest category is
  `ambient-context` — *"real DOMAIN state, reached globally or per-thread instead of threaded"* —
  which is **the same category its neighbour `ARM_TABLE` already carries**, for the same reason. The
  `performance-counter` claim ("arming it cannot change a result") stops being asserted about a value
  that decides five tests.

## The one contract decision, pinned

⛔ **`Cell<usize>` in a `thread_local!`, NOT an `AtomicUsize` in a `thread_local!`.** An atomic inside
a thread-local is a contradiction wearing a safety belt: it pays for synchronisation that, by
construction, can no longer be contended, and it leaves the next reader believing cross-thread
sharing is still possible. `ARM_TABLE` uses a bare `RefCell` for exactly this reason.

## Out of scope = rejected

- **Threading a rebuilt-or-not bool out of `fire_fixpoint_delta`.** `sequi` proposed it; it changes a
  hot production signature to serve a test observation, and thread-ownership gets the same guarantee
  for nothing. Rejected, not deferred.
- **Removing the counter.** It is the only observation of arm reuse there is.
- **Touching `ARM_TABLE`, the ZERO-MUTEX contract, or `.config/nextest.toml`.**
