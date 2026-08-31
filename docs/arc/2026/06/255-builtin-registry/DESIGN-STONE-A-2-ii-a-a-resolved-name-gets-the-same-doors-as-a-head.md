# DESIGN — STONE A-2-ii-a: a name resolved through the environment gets the SAME DOORS as a head

> ⛔ **THIS DESIGN'S MECHANISM ACCOUNT WAS WRONG, AND ITS UNBLOCKING CLAIM WAS FALSE.**
> Corrected in place 2026-08-30 after the rider refuted it and the orchestrator verified all four
> claims against the disk. **The stone still shipped** — the fix it describes is real and correct —
> **but it unblocks nothing, and the reason the accessor case fails is entirely different from what
> is written below.** Read `## ⛔ THE REFUTATION` before anything else on this page.

## ⛔ THE REFUTATION — what the disk actually says

**1. The accessor never reaches the arm this stone patches.** Every accessor `wat-rs` generates
(`register_aggregate_methods`, `runtime.rs`) is **`FunctionBody::Wat`**, not `Native` — verified:
`runtime.rs:884,1257,1695` all construct `FunctionBody::Wat`. Its body literally calls
`(:wat::core::Record/field-at …)`. So resolving `:probe::R/sk` through a binding takes
`classify_closure`'s **Wat arm** (a transitive body walk), never the Native arm.

**2. `FunctionBody::Native` has ZERO live constructors.** Verified: no construction site anywhere in
`src/` or `crates/`. And arc 278 recorded the identical finding independently, months earlier —
`tests/program/wat_arc278_sigma_fn_purity_gate.rs:25-30`: *"research proved (grepping every
`Function { .. }` construction site in the crate) that nothing anywhere constructs a `Function` with
`FunctionBody::Native`… There is no wat source a user can write that reaches the Native arm."*

**3. THE REAL CAUSE — and it is arc 255's own business.** The accessor's Wat body denies because the
verbs it calls are `KNOWN_UNREVIEWED`:
`:wat::core::Option/expect` (`purity.rs:2189`) · `:wat::core::Record/field-at` (`:2192`) ·
`:wat::core::struct-field` (`:2219`) · `:wat::core::type` (`:2224`).
**Four verbs need rulings.** That is a homing/ruling problem — the campaign's main thrust — not a
classifier-plumbing problem.

**4. Therefore the unblocking claim below is FALSE.** This stone does **not** unblock
`wat/query/mem.wat:136,163`. Those two sites stay refused until the four verbs above are ruled.

★ **How the orchestrator got it wrong:** I read `classify_closure`'s `Native` arm, saw it consulted
`intrinsic_meta` alone, and attributed the accessor's `false` to it — without checking which arm an
accessor actually takes. The adjacent implementation is not the subject.
`[[feedback_an_adjacent_implementation_is_not_the_subject]]`

## Why the stone shipped anyway

The fix is real: the Native arm *did* consult a narrower ladder than a head, and that inconsistency
is a defect whether or not anything reaches it today. It is a **removal of drift in existing code**
(31/−25 lines, one ladder instead of two), not new machinery — so this is not the `defined_in`/
`layer` "cheap to build is not worth building" case, which was about ADDING a field. Arc 278 already
treats this same defensive arm as worth having correct and Rust-exercised.

⚠ **But it is UNREACHABLE from any wat program today, and must not be read as load-bearing.** The
rider correctly triggered STOP-3 rather than widening scope to manufacture a witness, and the
brief's flagship `true`/`true` probe row **cannot be produced** — recorded honestly in the probe's
own header.

## THE INVARIANT THIS STONE ESTABLISHES — and it is the real deliverable

> **Reach-independence: the classifier's verdict on a name depends on the NAME, never on how the
> name was reached.**

A head and a binding that resolve to the same named callable must classify identically, on every
axis. This is a *correctness property*, testable as an invariant, not a feature request — and it is
the honest form of what A-2-i was reaching for.

## What ships

In `classify_closure`'s `FunctionBody::Native` arm: when the resolved `Function` has a `name`,
**route that name through the same door ladder a head takes** rather than consulting
`intrinsic_meta` alone. The one place that knows all four doors is `head_ok` itself, so delegating
to it keeps this as ONE mechanism that cannot drift, instead of a second ladder to maintain in
parallel.

⚠ **The recursion guards must carry across the delegation** — both the FQDN `seen` and the
`closure_seen` pointer set — or a named native reachable from its own body re-enters. A-2-i already
threads both; this stone must not drop them at the hand-off.

An **anonymous** native (`name: None`) keeps A-2-i's behaviour exactly: default-deny, because
nothing names it and nothing can prove it.

## ⛔ ~~Why this blocks A-2-ii-b~~ — STRUCK. IT DOES NOT. Here is what actually does.

~~This section claimed the stone unblocks `wat/query/mem.wat:136,163`.~~ **It does not, and never
would have** — see THE REFUTATION at the top. Kept struck rather than deleted so the error stays
legible.

**What A-2-ii-b is ACTUALLY blocked on.** Those two live sites pass a bare field accessor as
`sort-by`'s key function:

```clojure
sorted (:wat::core::sort-by :wat::query::Row/sk matches)
sorted (:wat::core::sort-by :wat::query::IndexRow/isk matches)
```

The accessor resolves to a **Wat-bodied** `Function` whose body calls four verbs that are all
`KNOWN_UNREVIEWED` — `Option/expect`, `Record/field-at`, `struct-field`, `type`. The walk denies on
those, not on anything about the accessor. **So the blocker is FOUR RULINGS**, which is arc 255's
own business and exactly the ratchet `KNOWN_UNREVIEWED` exists to force:

> *"a verb in this list that is no longer unreviewed ⇒ RED. Rule on it, delete its line."*

★ The imposition would still refuse those two sites today — but for the honest reason that four
verbs in the path are **unruled**, not because the instrument cannot see them. That is
default-deny working, not a defect the gate invented.

`wat/bracket.wat:783`'s inline-`fn` key function classifies `true` and is not at risk either way.

## Out of scope = REJECTED (not deferred)

- **The imposition at `sort$native`'s door, and homing the verb** — **A-2-ii-b**, the next stone.
  This one restores an invariant; nothing consumes it yet.
- **`freeze.rs:803` opting in** — unchanged, still `Static`.

## THE FOUR QUESTIONS — flat YES/NO

| option | Obvious? | Simple? | Honest? | Good UX? | verdict |
|---|:---:|:---:|:---:|:---:|---|
| **A-2-ii-a** delegate a resolved NAME to `head_ok`'s ladder | YES | YES | YES | YES | ✅ **ADMITTED** |
| copy the accessor/constructor doors into `classify_closure` | YES | **NO** | **NO** | — | ⛔ **DISQUALIFIED** |
| impose anyway; rune or special-case the two query sites | **NO** | YES | **NO** | — | ⛔ **DISQUALIFIED** |

- **copy-the-doors Simple? NO** — two ladders to keep in step. **Honest? NO** — they will drift, and
  the drift reappears as this same bug at the next door added to one and not the other.
- **impose-anyway Obvious? NO** — a reader meets two live call sites refused for being impure when
  they are not. **Honest? NO** — it reports a defect the instrument invented rather than found.

## Acceptance

| what | command | expected |
|---|---|---|
| ⛔ ~~the asymmetry is gone~~ | ~~probe: accessor as head vs binding~~ | ⛔ **UNPRODUCIBLE — this bar was written from a wrong mechanism.** The accessor takes the **Wat** arm, not the patched Native arm, so no edit in this stone's blast radius can make the rows agree. Measured `true` / `false`, and correctly so: the binding path denies on four `KNOWN_UNREVIEWED` verbs. Rider triggered STOP-3 rather than widen |
| still no widening | probe: effectful `keyfn` through a binding | `false` — **verified** |
| ⚠ anonymous native still denies | probe: unnamed native through a binding | **unconstructible** — `FunctionBody::Native` has zero live constructors (arc 278 proved the same); no wat program reaches it |
| A-2-i's rows hold | `255-probe-the-classifier-follows-a-capture.wat` | `true` / `false` |
| negative control holds | `255-probe-the-classifier-cannot-see-through-a-closure.wat` | `true` / `false` / `false` |
| additive | `scripts/floor.sh`, exit read UNPIPED | 5109/5109, 0 failed |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | 0 |
