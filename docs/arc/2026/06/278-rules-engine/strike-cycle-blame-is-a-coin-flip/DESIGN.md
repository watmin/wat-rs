# DESIGN — the code names the principle one paragraph above the line that breaks it

## Why

Work-list **C20**: *a compiler diagnostic names a different function depending on the run.* Opened by
C19's twice-run gate, quarantined with evidence rather than half-fixed, because traversal determinism
was explicitly cut from that strike's radius.

**Re-driven at HEAD `04abe37fc`, twelve runs of the same binary on the same file:**

```
6 runs → :probe::b at :line 5
6 runs → :probe::a at :line 8
```

A clean coin flip. `tests/rete/probe_arc278_rete_defn_recurse_mutual.wat.bad` declares a mutual
rete-defn cycle `a↔b`; the refusal is correct either way, but **a user following the caret is sent to
a different function next time.**

## The root, located — and the file convicts itself

`src/rete/purity.rs:1711`:

```rust
pub(crate) fn apply_rete_defn_contracts(
    sym: &mut SymbolTable,
    declared: &std::collections::HashSet<String>,
) -> ReteDefnCheckOutcome {
    for name in declared {
```

Twenty lines below, at `:1731-1735`, the same function states the principle in its own words:

> *"`declared` is a HashSet, so `for name in declared` runs in ARBITRARY, run-varying order. Seeding
> only `name` leaves a MUTUAL reference order-dependent … **A check that answers differently
> depending on hash iteration order is not a check.**"*

**That comment is the fix for the four AXES, and it worked** — `seen` is seeded with every declared
name, so the pass/fail verdict no longer depends on order. **But `rete_defn_cycle` runs in the same
arbitrary-order loop and returns on the FIRST failure.** The author cured the order-dependence of the
*verdict* and left it in the *identity of the blamed name*, having written the governing sentence one
paragraph above the surviving instance.

Both reports are truthful — walking from `a` closes the cycle at the call to `b` (line 5); walking
from `b` closes it at the call to `a` (line 8). **The entry point is what is arbitrary**, and the
entry point is a hash order.

## The contract decision, pinned

**Make the arbitrary order unrepresentable, not merely sorted.**

`declared_rete_defns` becomes a **`BTreeSet<String>`** at every site it exists —
`freeze.rs:460,519`, `freeze/env.rs:56,385,386`, `purity.rs:1709` (~6). Iteration is then ordered by
construction and no future hand can reintroduce the flip by dropping a `.sorted()` at one call site.

Sorting *at the loop* was considered and **rejected**: it is a convention living at one of several
call sites, and this defect is already a case of a convention being stated and then not held. The
`seen` set inside the loop stays a `HashSet` — it is a membership probe, order-irrelevant.

This is the extirpare ladder's top rung reached honestly: not "remember to sort" (convention), not "a
test that catches the flip" (check), but **a type in which the wrong order cannot be written down.**

## ⛔ THE OTHER TWO QUARANTINED FILES ARE A DIFFERENT ROOT — driven, not assumed

`probe_arc170_w2a_kwargs_check_mint_swap.wat.bad`, 8 runs: the error **set** hashes two ways (5/3),
while the first two error kinds are **stable** (`CheckErrors`, `TypeMismatch`) every run. That is
check-phase error *ordering*, not the rete purity loop, and `check.rs` shows no `HashMap`/`HashSet`
iteration feeding error order — so the root is not located and finding it is real work.

**This strike cures ONE of the three and says so.** `QUARANTINE_LEN` goes 3 → 2 and the remaining two
keep their captured evidence. **Bundling them would be the claim that one fix covered three defects,
which is exactly the shape this arc keeps finding.**

## Out of scope = REJECTED

- **The two check-phase files.** Different root, driven above. They stay quarantined with evidence
  and are re-rowed as their own work — not silently absorbed.
- **Reporting the full cycle path** (*"a → b → a"*) instead of one offender. It is better UX and it is
  a diagnostic-content change, not a determinism fix. Named, and cut.
- **Any other `HashSet`/`HashMap` iteration in the tree.** This strike fixes the one that is driven
  and located. A sweep is a different, larger strike and must be measured before it is drawn.
