# Arc 071 — Lab harness parity for parametric built-in enums

**Status:** shipped 2026-04-27. Pre-implementation reasoning artifact.

**Note on diagnosis:** the DESIGN's diagnosis named the flaw as
"register_enum_methods isn't called on the lab harness path." The
infra session's reproduction with the probe surfaced a different
mechanism. Both `freeze.rs` and `stdlib_loaded()` already called
`register_enum_methods` (since arc 070); the actual bug was inside
`register_enum_methods` itself: synthesized variant constructors
used a bare-path return type, dropping `<A>` for parametric enums.
Same observable symptom (lab harness fails type-check on
`WalkStep<A>` use), different root cause. The fix is in the
substrate's enum-method synthesis, not in adding a missing call site.

See INSCRIPTION.md for the actual mechanism + the test-coverage
discipline change that prevents the failure mode from recurring.
**Predecessors:** arc 070 (`:wat::eval::walk` + `WalkStep<A>`).
**Surfaced by:** holon-lab-trading proof 018, attempting to consume `:wat::eval::walk` from the lab `wat::test! {}` harness.

Builder direction (2026-04-27, after proof 018's walker rewrite couldn't type-check):

> "if you find a true flaw - we'll construct an arc for it and we'll
>  get infra to fix it"

The flaw is real. It is reproducible at minimum scale.

---

## The reproduction

Minimal probe at
`holon-lab-trading/wat-tests-integ/experiment/099-walkstep-probe/probe.wat`:

```scheme
(:wat::test::make-deftest :deftest
  ((:wat::core::define
     (:my::test::count-visit
       (acc :i64)
       (form :wat::WatAST)
       (step :wat::eval::StepResult)
       -> :wat::eval::WalkStep<i64>)
     (:wat::eval::WalkStep::Continue (:wat::core::i64::+ acc 1)))))

(:deftest :probe::walk-fires-once-on-already-terminal
  (:wat::core::let*
    (((form :wat::WatAST)
      (:wat::core::quote
        (:wat::holon::Bind
          (:wat::holon::Atom "k")
          (:wat::holon::Atom "v"))))
     ((walk-result :Result<(wat::holon::HolonAST,i64), wat::core::EvalError>)
      (:wat::eval::walk form 0 :my::test::count-visit)))
    (:wat::core::match walk-result -> :()
      ((Ok pair)
        (:wat::core::let*
          (((count :i64) (:wat::core::second pair)))
          (:wat::test::assert-eq count 1)))
      ((Err _e) (:wat::test::assert-eq :walk-ok :walk-err)))))
```

This is **byte-for-byte the substrate's own `walk_w1`/`walk_w2`
prelude** (`runtime.rs::walk_count_prelude`) wrapped in the lab's
deftest harness.

```bash
cargo test --release --features experiment-099 --test experiment_099 -- --nocapture
```

```
test probe.wat :: probe::walk-fires-once-on-already-terminal
  failure: startup: check:
  2 type-check error(s):
    - :my::test::count-visit: body produces :wat::eval::WalkStep;
                              signature declares :wat::eval::WalkStep<i64>
    - :wat::core::second: parameter #1 expects tuple or Vec<T>; got :?143
```

The first error is load-bearing. The second cascades from it.

The same code, run through `cargo test --lib` (substrate unit
tests), passes as `walk_w1_chain_to_terminal` and
`walk_w2_already_terminal_input`. **730 substrate unit tests
pass; 0 lab tests pass.**

---

## What the substrate KNOWS but doesn't surface here

Arc 070's INSCRIPTION (lines 102–109) names the gap directly:

> *"register_enum_methods now runs in unit-test setup. Pre-arc-070,
> the unit-test SymbolTable only ran register_struct_methods —
> synthesized struct accessors but no tagged-variant constructors.
> Arc 070's tests need (:wat::eval::WalkStep::Continue acc) and
> (:wat::eval::WalkStep::Skip terminal acc) to resolve at the test
> boundary. **One-line addition to stdlib_loaded().**"*

The one-line addition is in the substrate's *unit-test*
`stdlib_loaded()`. The lab `wat::test! {}` macro takes a different
path through the harness, and that path does not run
`register_enum_methods`. So `:wat::eval::WalkStep::Continue` (and
any other parametric built-in variant constructor) resolves as a
non-parameterized `:wat::eval::WalkStep` value — the type checker
sees the signature ask for `:wat::eval::WalkStep<i64>` and the
body produce `:wat::eval::WalkStep`, and refuses.

The substrate already has the synthesized constructors. They just
aren't installed on the lab harness's frozen world.

---

## Why this matters now

Proof 018 (`fuzzy-on-both-stores`) was the trigger for arc 070;
its rewrite to `:wat::eval::walk` is the natural next step. With
this gap open:

- **Proof 018 cannot collapse to the documented walk pattern.**
  The visitor refuses to type-check; the proof has to keep its
  handwritten 50-line walker (which arc 070 was designed to
  replace).
- **Lab umbrella 059's L1+L2 cache cannot consume `walk`.** Slice
  1 of the trader rebuild was scheduled to use `walk` directly.
- **Every future lab proof that wants the walker pattern hits the
  same wall.** The substrate ships the primitive but the lab
  harness can't consume it.

The cost of leaving this open is that arc 070's surface lives in
unit tests only; the broader system can't use it. Closing the gap
brings the surfaces to parity.

---

## What's broken (the precise mechanism)

Two divergent SymbolTable initialization paths:

### Path A — substrate unit tests

```
cargo test --lib
  → wat-rs/src/runtime.rs::run / run_with_ctx
    → stdlib_loaded()
      → register_struct_methods    (already there)
      → register_enum_methods      (added by arc 070)
```

`register_enum_methods` walks the substrate's enum registry and
synthesizes constructor functions for each variant
(`:wat::eval::WalkStep::Continue`, `:wat::eval::WalkStep::Skip`,
etc.) — installing them as callable forms on the SymbolTable so
the type checker can resolve them.

### Path B — lab `wat::test! {}` harness

```
cargo test --test experiment_NNN
  → wat::test! {} macro expansion
    → wat::compose_and_run_with_loader (or sibling)
      → some SymbolTable build path that does NOT call
        register_enum_methods
```

The lab path shares most of the substrate's frozen-world build
but skips `register_enum_methods`. The unit-test addition was
local; the lab path was not updated.

(Confirmation requires reading the substrate; the user's preference
in this session was to demonstrate the gap from wat alone, which
the probe above does.)

---

## What this arc ships

| Op / change | What it does |
|----|----|
| `register_enum_methods` invocation in the lab harness's SymbolTable build path (whichever path the `wat::test! {}` macro reaches — `compose_and_run_with_loader` or sibling) | Synthesizes `:wat::eval::WalkStep::Continue`, `:wat::eval::WalkStep::Skip`, and any other built-in parametric variant constructor on the frozen world the lab harness gives to test files. |
| Same change to the **`wat::main! {}`** path if it shares the gap | Prevents the same flaw in production lab applications (not just tests). |
| Regression test in the lab harness | Mirror `walk_w1`/`walk_w2` as a lab-side test (the probe in `experiment/099-walkstep-probe/` becomes the regression suite — promote it to a real proof or fold it into proof 018's walker rewrite). |

Three pieces. The fix itself is likely the same one-line
`register_enum_methods` invocation, copied to a second call site.
The investigation cost is locating that second site. The
regression test exists; it just has to start passing.

---

## Open questions for the substrate session

1. **Is `wat::main! {}` affected too?** The probe was lab-test
   only. If `wat::main!` also skips `register_enum_methods`, every
   production wat application that wants `:wat::eval::walk` is
   blocked. Recommend: check both paths; fix both if both need
   fixing.
2. **Are there other parametric built-in enums beyond `WalkStep<A>`?**
   `Option<T>`, `Result<T, E>`, `HashMap<K, V>`'s carrier — these
   may already work via different code paths. The fix should
   *uniformly* propagate enum methods to whichever harness path
   needs them, not patch `WalkStep<A>` specifically.
3. **Should this become a substrate invariant?** If the unit-test
   harness and the lab harness diverge on any registration step,
   that's a class of bug. Recommend: factor the SymbolTable build
   into one canonical fn called by both paths, so future
   registrations cannot drift between them.

---

## What this arc deliberately does NOT do

- **Does not redesign the harness.** The fix is harness parity,
  not a rewrite. One missing call site to add.
- **Does not modify the substrate's unit-test path.** That path
  works; arc 070 fixed it. The fix is downstream.
- **Does not require new wat surface.** Existing `:wat::eval::walk`,
  `WalkStep<A>`, `StepResult` stay as-is. After this arc, the
  USER-GUIDE example becomes runnable; today it's documentation
  for an unreachable surface in the lab.

---

## Test strategy

The probe at
`wat-tests-integ/experiment/099-walkstep-probe/probe.wat` becomes
the regression test. Today:

```
0 passed, 1 failed
```

After arc 071:

```
1 passed, 0 failed
```

Once green, the probe folds into proof 018's walker rewrite (the
real consumer of arc 070). The probe itself can be deleted or
kept as a sanity check on harness parity for future arcs that add
parametric built-in variants.

---

## The thread

- **Arc 070** — shipped `:wat::eval::walk` + `WalkStep<A>` +
  `AlreadyTerminal`. Unit tests pass.
- **Proof 018 walker rewrite (2026-04-27)** — first lab consumer
  of arc 070. Surfaces this gap.
- **Arc 071 (this)** — lab harness parity. Closes the gap.
- **Next** — proof 018's walker collapses to the documented
  pattern. Lab umbrella 059 slice 1 starts using `walk`.

PERSEVERARE.
