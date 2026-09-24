# FINDING — S5: no failure needs the checker to know a transport slot

**Measured 2026-09-24** in an isolated worktree at `main` @ `4e95fbd8f`. Nothing landed. Patches are in
the session scratchpad: `s5.patch`, `s5-m2.patch`, `s5-m3.patch`. The orchestrator re-read every
Summary line from the worktree's `.floor/`.

## The question

C-b has to say *which parameter is the transport slot*. Two options survived the four questions: S1, a
new bounded-parameter syntax, and S5, no slot at all (the transport is an ordinary type parameter, and
the special cases are deleted). The measurement: delete the special cases and classify everything that
breaks.

## The code had more special cases than the prompt listed

The missing-slot case is **three** arms, not one: two in `unify` (the head-named `n_fixed` arm, with
Address/Bound = 2 and Launched = 4, and the `is_transport_slot` n/n+1 arms) and one in `assignable`.
A sixth arm was left alone: "uninstantiated aggregate `Handle` accepts any instantiation". It is not
letter-based.

## Results (Summary lines verbatim)

| run | what was removed | Summary |
|---|---|---|
| baseline | nothing | `6027 tests run: 6027 passed (9 slow), 22 skipped` |
| M1 | everything | `6027 tests run: 2862 passed (1 slow), 3147 failed, 18 timed out, 22 skipped` |
| M2 | all but the missing-slot arms | `6027 tests run: 2863 passed (1 slow), 3164 failed, 22 skipped` |
| M3 | only the letter-rewritten keys | `6027 tests run: 6015 passed (8 slow), 12 failed, 22 skipped` |

The failures nest: M3 ⊂ M2 ⊂ M1.

| class | M1 | M2 | M3 |
|---|---|---|---|
| **(a) needs to know a slot** | **0** | **0** | **0** |
| (b) the missing-slot shorthand | 3164 | 0 | 0 |
| (c) a free, undeclared transport letter | hidden behind (b) | 3164 | 12 |
| (d) the purity wall | 0 | 0 | 0 (known from 255.17a: 57) |
| (e) other | 1 (the heresy ledger shrank 220 → 219, the good direction) | 0 | 0 |

In M1 and M2 the stdlib itself stops type-checking, so almost every test fails at startup.

## What each class is: a declaration the language does not make

- **(b), at 3 declarations, not uses:** `wat/spawn.wat:417` (the `Locus/launch` surface method returns a
  4-argument `(Launched :- [S R Sh Lu])`) and `wat/capability.wat:46/67` (`Dialable/coord`,
  `TypedCapability/coord` return a 2-argument `(Address :- [S R])`). All 119 stdlib errors in M1 trace
  to these three.
- **(c), defservice's free `T`/`Xt`:** it appears in `launch-tp-ann` (`service.wat:2376`), `status-ty`
  (`:1107`) and `handle-bare-name`, inside emitted defns that have no `:- [T]` binder. That accounts
  for 36 stdlib errors in M2, 4 per service across 9 stdlib services.
- **M3's 12 (c), plus a missing general mechanism:** defservice registers `(Handle :- [… T])`
  extend-type edges to `Dialable`/`TypedCapability`/`Capability`, keyed by the **exact string**. With
  the letter rewrite gone, `(Handle :- [Wire])` cannot match its generic edge. What is missing is an
  ordinary-generics mechanism, not a transport one: **match a generic `extend-type` edge by unification,
  not by string.**
- **(d):** `Shared`/`Wire` are `defstruct`s (255.17a).

## Verdict

- **Measured:** S5 holds for the stdlib and M3's surface. **No failure needs a transport slot.**
  S1 now fails **Simple**: it would add a language feature (bounded parameters) that no measured case
  needs.
- **Limit, measured:** M1 and M2 fail at stdlib startup, so any test-local reliance is hidden. (a) = 0
  is proven only for the stdlib and M3's surface. The final strike re-floors to measure the rest.

## The strike sequence this implies (C-b, re-decomposed)

Each strike is **additive while the special cases stand**: the floor stays green, and the special
cases become dead code before they are deleted.

1. **C-b1:** the three declarations spell full arity, or declare `T` (`spawn.wat:417`,
   `capability.wat:46/67`). This closes (b).
2. **C-b2:** defservice declares its transport `T` in the binder of every emitted generic defn
   (`launch-tp-ann`, `status-ty`, `handle-bare-name`). This closes (c).
3. **C-b3:** a generic `extend-type` edge is matched by unification. This closes M3's class.
4. **C-b4:** `Transport` becomes a `Pure` enum with `Shared`/`Wire` as variants (the names are the
   builder's; 23 lines in 14 files), and the child main says `Wire`. This closes (d).
5. **C-b5:** delete the special cases, land the parked 255.17 patch (D2), and re-floor to measure any
   test-local reliance.
