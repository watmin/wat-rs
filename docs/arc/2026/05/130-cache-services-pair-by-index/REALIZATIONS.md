# Arc 130 — Realizations

**The discipline named here: TEST-FILE COMPOSITION as a failure-
engineering primitive.**

A failure engineer must structure tests as a top-down dependency
graph IN ONE FILE. Each function does ONE thing. Each subsequent
function composes only from functions defined above it. Each
function gets its OWN deftest proving it in isolation. The final
deftest at the bottom references only the topmost layer.

A monolithic deftest body — the kind that crashes with
`expected "hit", actual "<missing>"` and gives no further
structure — is a discipline violation. It cannot answer YES to
all four questions. It cannot point at the broken unit when it
fails.

## Provenance

Coined 2026-05-02 after the arc 130 slice 1 sonnet sweep was
killed mid-run.

The user's framing (transcript verbatim):

> we have been repeatably burned by trying to one shot tests..
> these continue to violate our questions...
>
> is this obvious?
> is this simple?
> is this honest?
> is this a good ux?
>
> the latest test failure is absolutely not answering all yes...

And the additional layer:

> each of our composable functions.. we can have a deftest for
> each one.. we prove they work as we define them...

The discipline is failure engineering applied at the test-file
level: every primitive is a first-class proof; every layer
composes from below; every layer carries its own deftest; the
final test is just a composition.

## What the discipline IS

Three rules.

### Rule 1 — One file, top-down dependency graph

The test file reads top-to-bottom. Layer 0 at the top — primitives
that do ONE thing. Each subsequent layer composes from layers
above it. The deftest at the bottom references only the topmost
layer.

```scheme
;; ─── Layer 0 ─── primitives. Each does ONE thing.

(:wat::core::define (:test::spawn-and-join ...) ...)
(:wat::core::define (:test::pop-handle-finish-pool ...) ...)

;; ─── Layer 1 ─── single-verb actions, composed from Layer 0.

(:wat::core::define (:test::send-one-put (handle k v) ...) ...)
(:wat::core::define (:test::send-one-get (handle k) ...) ...)

;; ─── Layer 2 ─── scenarios composed from Layer 1.

(:wat::core::define (:test::put-then-get (handle k v) ...) ...)

;; ─── The deftest ─── 3-7 lines. Body is short BECAUSE the layers exist.

(:wat::test::deftest :test::cache-service-round-trip ()
  (:wat::core::let*
    ((handle (...)))
    (:wat::test::assert-eq
      (:test::put-then-get handle "k" 42)
      (:wat::core::vec :wat::core::Option<...> (:wat::core::Some 42)))))
```

### Rule 2 — Each helper has its own deftest

For each named function defined above the final test, a deftest
proves it in isolation:

```scheme
(:wat::test::deftest :test::test-spawn-and-join ()
  ;; Exercises just :test::spawn-and-join.
  ...)

(:wat::test::deftest :test::test-send-one-put ()
  ;; Exercises just :test::send-one-put.
  ...)

(:wat::test::deftest :test::test-put-then-get ()
  ;; Exercises just :test::put-then-get.
  ...)

(:wat::test::deftest :test::cache-service-round-trip ()
  ;; The actual scenario — composed from the proven layers.
  ...)
```

`cargo test --list` then shows the proof tree:

```
test deftest_test_spawn_and_join                  ... ok
test deftest_test_send_one_put                    ... ok
test deftest_test_send_one_get                    ... ok
test deftest_test_put_then_get                    ... ok
test deftest_test_cache_service_round_trip        ... ok
```

When ANY level fails, the failing deftest's name localizes the
bug. If `test_send_one_put` passes but `test_put_then_get` fails,
the bug is in the COMPOSITION, not in put. If `test_send_one_put`
fails, the bug is in put itself, before any composition is even
attempted.

### Rule 3 — Inherent vs accidental complexity

A scenario has irreducible inherent complexity: must spawn, pop,
send, recv, drop, join. That can't be wished away.

What CAN be wished away is the let* archaeology. Anonymous
sequential bindings in a 30-line let* are accidental complexity:
every binding is a name that lies about what it represents.

```scheme
;; ❌ Accidental complexity — anonymous bindings, no failure surface.
(:wat::test::deftest :round-trip ()
  (:wat::core::let*
    (((spawn ...) (...))
     ((pool ...) (...))
     ((driver ...) (...))
     ((handle ...) (...))
     ((reply-pair ...) (...))
     ((reply-tx ...) (...))
     ((reply-rx ...) (...))
     ((ack-pair ...) (...))
     ((ack-tx ...) (...))
     ((ack-rx ...) (...))
     ((_put ...) (...))
     ((results ...) (...))
     ((_join ...) (...)))
    (:wat::test::assert-eq ...)))
```

When the assertion fails: which binding broke? Could be ANY of
them. The let* tells you nothing about WHICH unit of work
failed. The accidental complexity has erased the diagnostic
surface.

```scheme
;; ✓ Inherent complexity preserved; accidental complexity NAMED.
(:wat::test::deftest :round-trip ()
  (:wat::core::let*
    ((handle (:test::pop-handle (...))))
    (:wat::test::assert-eq
      (:test::put-then-get handle "k" 42)
      (...))))
```

When the assertion fails: `:test::put-then-get` broke. We have a
deftest for that. We have deftests for its inputs. The failure
trace IS the dependency graph.

## The worked example — arc 130's failed sweep

Arc 130 slice 1 sonnet sweep ran 2026-05-02. The sonnet agent was
asked to reshape the LRU cache service substrate AND rewrite its
single test in one slice. The rewritten test was a single
monolithic let* with ~30 bindings.

**Failure 1:** parse error from a missed `PutAck` keyword vs
symbol. The parser reported `unexpected ')' at ...:374:41`. The
agent chased parens for ~10 minutes before recognising the issue
was elsewhere. The substrate's `MalformedVariant` error carried
no span / enum_name / hint, so the diagnostic stream was useless.
(Substrate fix shipped in commit `db4ecc7` adding span +
offending text + migration hint to `TypeError::MalformedVariant`.)

**Failure 2:** runtime assertion failed:
`expected "hit", actual "<missing>"`. The 30-binding let* gave
no clue WHICH unit of work was broken. Could be:
- The driver loop indexing wrong DriverPair?
- Reply variant dispatch mismatch?
- Put not actually persisting?
- Get's match arm wrong?
- A typo in any of 30 bindings?

The agent had no narrowing surface. It iterated blindly. The
user pulled the kill switch.

**The lesson:** the rewrite was a one-shot of a hard problem.
This violates all four questions:
- **Obvious?** No — 30 bindings hide structure.
- **Simple?** No — irreducible complexity is conflated with
  accidental.
- **Honest?** No — names like `state`, `pair`, `_finish` lie
  about what's being constructed.
- **Good UX?** No — failure narrows nothing; localizes nothing.

A compositional rewrite would have surfaced WHICH layer failed.
If `test_send_one_put` passed but `test_put_then_get` failed,
the bug is in the composition, not in put. The diagnostic
narrows by construction.

## Why this is a failure-engineering primitive

Failure engineering says: the failure isn't recovered from; it
is read.

For test-file composition, "reading the failure" means knowing
EXACTLY which layer broke. That requires named layers AND
per-layer proofs. A monolithic deftest gives you a panic; a
compositional one gives you a diagnostic tree.

The pre-handoff EXPECTATIONS document tells the agent what
failure means at SCORECARD level. Test-file composition tells
the agent what failure means at SOURCE level. Both are
calibration mechanisms; both need to be in place for the
failure-engineering discipline to function.

## Principles

### 1. The deftest body is short BECAUSE the layers exist

If the deftest body is long, the layers don't exist yet. Stop
writing the deftest. Define the layers. Come back.

Empirical bound: a deftest body with more than ~10 let* bindings
is a violation. Push the complexity into named helpers.

### 2. Layer N tests Layer N

Don't write a final composition test before the layer-N tests
that prove the pieces. Bottom-up proofs THEN top-down composition.
The proof tree is the graph; you don't get to skip nodes.

### 3. Failure trace IS the dependency graph

When `test_put_then_get` fails, look at its dependents. Do
`test_send_one_put` and `test_send_one_get` pass? If yes, the
bug is in COMPOSITION. If no, the bug is in the failing primitive.

The cargo-test output is a leveled diagnostic. The discipline
makes it possible.

### 4. After arc 130 ships, the deftests are unchanged

Most arc-130-style substrate reshapes change helper-verb
SIGNATURES. Per-layer helpers absorb the signature change in
their bodies. The deftests at every layer don't move. The final
deftest doesn't move. The diff is small and bounded.

This is the structural payoff: the test file is decoupled from
the substrate's internal evolution. Every helper is an interface
contract; every deftest is a regression test for that contract.
The substrate can churn beneath the contracts without breaking
the test file.

### 5. Inline helpers are fine for self-contained tests

Hermetic tests need helpers visible inside the spawned-program
body. Two paths:

- **Inline**: each test's hermetic program defines its helpers
  inline. Some duplication; but each helper is small and the
  shape is the lesson.
- **Prelude via `make-deftest`**: a `make-deftest :alias` factory
  with helpers in its prelude. All deftests using that alias
  inherit the helpers. One-file constraint is satisfied if the
  prelude is in the same file.

Cross-file split (proof_004's stepping-stones-as-files shape) is
NOT the right answer. The user explicitly rejected it: "i
dislike that proof 004 does this across many files... it should
be one files with incremental complexity."

### 6. The four questions ARE the test-file structure check

Every test file passes through obvious / simple / honest / good
UX before merge. The pre-flight check lives in the author's
hands; cargo test catches what got through. A monolithic deftest
fails obvious + simple + good-UX before the test even runs.

## How this connects to existing disciplines

| Discipline | Where it touches test-file composition |
|---|---|
| Failure engineering | Composition surfaces WHICH layer broke; the failure IS read, not recovered from |
| Substrate-as-teacher | Each helper's name + signature IS a teaching about what it does |
| Four questions | A monolithic deftest cannot answer YES to all four |
| Iterative complexity | "Build small funcs. Prove each stepping stone. Don't one-shot multi-piece changes." Same lesson, applied to test files |
| One let* per function | Cap a function at ONE outer let*; offload to named helpers — same principle, scoped narrower |
| Test first | Write the test BEFORE the implementation; the test's COMPOSITIONAL shape is part of the design |
| No broken commits | Per-layer deftests catch breakage at the layer that broke; the bisect window is the helper, not the file |

## When this discipline applies

- Any deftest with > ~10 bindings (the empirical threshold).
- Any test that touches multi-thread coordination, channel
  lifecycles, or service driver loops (the deadlock-prone
  scenarios where failure localization is critical).
- Any test that's part of a substrate-reshape arc (the helper
  signatures are the moving target; per-layer deftests provide
  the stable contracts).

## When it's overhead

- Trivial single-step tests: `assert-eq (1 + 1) 2`. No helpers
  needed.
- Pure-function unit tests where the function under test IS the
  primitive: `assert-eq (factorial 5) 120`. The function under
  test is itself the layer.
- One-off exploratory tests where we're feeling out the API
  shape and intend to throw the test away.

The discipline scales with the test's structural complexity.
Don't pay for layering you don't need. But for tests that touch
services, drivers, or substrate boundaries — composition isn't
optional, it's the only way to keep the test file diagnosable.

## What was preserved on disk to read this lesson from

The arc 130 slice 1 sonnet sweep is preserved in the working
tree (uncommitted) as the calibration set:

- `crates/wat-lru/wat/lru/CacheService.wat` — substrate WIP
  showing the `PutAck` keyword/symbol confusion + the
  monolithic-let* pattern in the test it expected.
- `crates/wat-lru/wat-tests/lru/CacheService.wat` — test WIP
  with the 30-binding monolithic shape.

These stay on disk as historical artifacts of the failed sweep.
The user's instruction (verbatim): *"do not revert - we need to
know what bad looks like to make good - keep it here."*

The next sweep restarts with this REALIZATIONS doc + the
compositional rewrite (this arc's demonstration commit) as the
two-document handoff: the discipline + the worked example.

## Cross-references

- `DESIGN.md` — the substrate reshape this arc proposed.
- `BRIEF-SLICE-1.md` + `EXPECTATIONS-SLICE-1.md` — the original
  one-shot brief that violated this discipline.
- `docs/arc/2026/05/126-channel-pair-deadlock-prevention/REALIZATIONS.md`
  — the parent realization that named "failure engineering" +
  "artifacts-as-teaching"; this arc's REALIZATIONS extends the
  discipline to test-file structure.
- `holon-lab-trading/wat-tests-integ/proof/004-cache-telemetry/`
  — the historical multi-file stepping-stone version. The user
  noted: "i dislike that proof 004 does this across many files."
  The lesson: same layering, ONE file.
- Memory: `feedback_test_file_composition.md` — the durable
  user feedback memory for this discipline.
- Memory: `feedback_iterative_complexity.md` — the parent rule
  about not one-shotting multi-piece changes.
- Memory: `feedback_simple_forms_per_func.md` — the
  one-let*-per-function rule, narrower scope.

## What this realization adds to the substrate

This document IS the realization. It names test-file composition
as a failure-engineering primitive. Future test files written
under this discipline — and future agent dispatch briefs that
include test-rewrite work — reference this REALIZATIONS doc as
the canonical playbook.

The next arc 130 sweep brief MUST include this REALIZATIONS as a
read-in-order anchor. The agent's deliverable MUST be measurable
against the three rules. A monolithic deftest in the deliverable
is a hard-row failure regardless of any other criteria.
