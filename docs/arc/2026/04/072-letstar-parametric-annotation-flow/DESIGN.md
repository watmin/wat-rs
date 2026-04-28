# Arc 072 — `let*` annotations don't propagate parametric type args to match arms

**Status:** shipped 2026-04-27. Pre-implementation reasoning artifact.

**Note on diagnosis:** the DESIGN named "let* annotation propagation
to match arms" as the flaw. The infra session's reproduction (with
the exact probe text) found a different root cause one layer
earlier: the LEXER tracked `()` depth but ignored `<>` depth, so
`:Result<(i64,i64), i64>` (whitespace after the comma) tokenized as
`:Result<(i64,i64),` (truncated at the space) plus a separate
`i64>` symbol. The type parser saw a malformed Result with one
arg; downstream the type checker showed a fresh-var `:?N`
unsolved at the pattern-arm site — *that's* what the DESIGN was
diagnosing as a propagation gap.

The fix is in the lexer: track `<>` depth alongside `()`; whitespace
inside an unclosed bracket raises `LexError::UnclosedBracketInKeyword`
at the lex layer (clean diagnostic pointing at the byte offset)
instead of silently truncating into a downstream type-check
mystery. Operator `<` / `>` in keyword paths (`:wat::core::<`,
`:wat::core::>=`) are disambiguated by the preceding char — only
`<` after an alphanumeric counts toward depth.

The substrate's whitespace rule for type keywords stays strict
(`:Result<i64,String>` — no space). The arc fixes the diagnostic,
not the rule. With correct whitespace, the let*-bind-then-match
flow already worked — the type checker's propagation was fine.

See INSCRIPTION.md for the actual mechanism + the lab probe path.
**Predecessors:** arc 028 (Result wrap), arc 048 (tagged-enum constructors + match), arc 055 (recursive patterns), arc 071 (lab-harness enum-method parity — closed first; this arc surfaced after).
**Surfaced by:** holon-lab-trading proof 018, mid-walker-rewrite, after arc 071 closed the harness-parity gap.

Builder direction (2026-04-27, after the second flaw revealed itself behind the first):

> "this is an arc - yes - incredible.. how long as this been here?... what test didn't we write?...."

The flaw is real, narrow, and *probably ancient*.

---

## The reproduction

Minimal probe at
`holon-lab-trading/wat-tests-integ/experiment/099-walkstep-probe/probe.wat`:

```scheme
;; Probe B — Result<(i64,i64), i64> with let*-bound annotation,
;; matched in a subsequent expression. Pair binding's type is :?
;; despite the annotation pinning T = (i64, i64).
(:deftest :probe::b-result-with-tuple-payload
  (:wat::core::let*
    (((wrapped :Result<(i64,i64), i64>)
      (Ok (:wat::core::tuple 7 11)))
     ((extracted :i64)
      (:wat::core::match wrapped -> :i64
        ((Ok pair) (:wat::core::second pair))    ;; pair = :?79
        ((Err _) -1))))
    (:wat::test::assert-eq extracted 11)))
```

```
:wat::core::second: parameter #1 expects tuple or Vec<T>; got :?79
```

The let*-binding's `:Result<(i64,i64), i64>` annotation does not
propagate `T = (i64, i64)` to the match arm's `pair` binding.

---

## What works, what doesn't, what we tried

| Probe | Pattern | Result |
|---|---|---|
| **A** | `let* ((pair :(i64,i64)) ← tuple)` then `(second pair)` | ✓ passes — non-parametric tuple binding works |
| **B** | `let* ((wrapped :Result<(i64,i64),i64>) ← Ok ...)`, then `match wrapped ((Ok pair) (second pair))` | ✗ pair = `:?79` |
| **B1** | Same B, annotate inside: `((Ok (pair :(i64,i64))) ...)` | ✗ "list sub-pattern in `:?23` position" |
| **B2** | Same B, destructure inline: `((Ok (a b)) b)` | ✗ "list sub-pattern in `:?3` position" |
| **C** | Match walk's return INLINE: `(match (walk ...) ((Ok _) 1) ((Err _) -1))` | ✓ passes — wildcard, no binding to type |
| **D** | Match walk's return INLINE: `(match (walk ...) ((Ok pair) (second pair)) ((Err _) -1))` | ✓ passes — call-site context resolves the return type |

The narrowest claim: **the type annotation on a let*-bound
parametric-enum value is inert with respect to the type of pattern
variables in subsequent match arms.** When the same value is
matched inline (probes C, D), the type checker has full call-site
context — visitor signature, function return type, etc. — and
resolves. When the value flows through let* with an annotation,
the annotation is recorded for the binding's name but doesn't seed
match-pattern type inference.

The two attempted workarounds (B1: annotate inside the constructor;
B2: destructure inline) both fail at a *prior* step — the type
checker can't decide the constructor's payload position is a tuple
because the parametric `T` is unsolved at that point. Annotating
deeper doesn't help because the deeper position has the same
unresolved variable.

---

## How long has this been here

**Probably since `let*` and `match` were both wired to parametric
enums.** Result was added in arc 028 (Result wrap). Match-on-tagged-
enum-with-binding works (proof 016 / 017 / 020 use it everywhere
INLINE). The let*-bind-then-match flow with an annotation must have
been a path the type-checker walked but a path no test exercised.

This arc's commit history does not need a precise blame — the
diagnosis is: *the test plane never crossed the failure mode.*
Years, possibly. Until proof 018's walker rewrite happened to want
exactly this flow.

---

## What test didn't we write

The substrate has tests for:
- `match` on bare Option, Result, user enum — INLINE matches.
- `let*` annotation on primitive types (i64, f64, bool, String).
- `let*` annotation on user struct types.
- `match` arm binding on parametric variants (when the value is
  call-site-typed by the surrounding expression).

The substrate does NOT have a test for:
- **`let*`-bind a parametric-typed value with explicit type
  annotation, then match on the binding in a separate expression
  where the match arm USES the parametric structure.**

The gap is specifically the *flow*: the test corpus exercises each
component (let* annotations work; match arm bindings work) but
never the chain *let* annotation → match arm binding → consumer
of the binding's parametric structure*.

This is the test category that needs to be added. One test per
parametric built-in that the substrate ships:

- `Option<T>` with let*-bound annotation, matched, payload destructured.
- `Result<T, E>` with let*-bound annotation, matched, Ok payload destructured.
- `Result<T, E>` with let*-bound annotation, matched, Err payload destructured.
- `:wat::eval::WalkStep<A>` (arc 070) with let*-bound annotation, matched, Continue payload destructured.
- `:wat::eval::StepResult` (arc 070) with let*-bound annotation, matched, each variant's payload destructured. (Monomorphic — but worth covering for parity.)
- User parametric enum with let*-bound annotation, matched, payload destructured.

The reason this matters as a substrate-level test category, not a
lab-test category: parametric inference flow IS a substrate
behavior; consumers WILL try this pattern; the substrate must
either support it or refuse it loudly.

---

## What this arc ships

| Op / change | What it does |
|----|----|
| Type-checker change: when a let*-binding has an explicit type annotation, that annotation must seed the binding's type so subsequent match arms on the bound name can infer the parameter args. | Closes the propagation gap. Whatever path the inline-match code uses to resolve the parametric type from call-site context, that path must run when the match's scrutinee is a let*-bound name with annotation. |
| Six new substrate tests (one per parametric built-in + one user-enum) | The test category that didn't exist. Each test does the full chain: let* annotation → match → bind payload → destructure / call-with-parametric-arg / pass-to-typed-function. |
| Improved error message when this case is hit | Today's error is `parameter #1 expects tuple or Vec<T>; got :?79` — opaque. Better: when the type checker hits an unsolved variable inside a match arm whose scrutinee was let*-bound with an annotation, the message should name what's happening: *"let*-bound annotation didn't propagate to this match arm. This is a known substrate gap (arc 072 / arc 072-fix). Workaround: match the value inline."* |
| USER-GUIDE entry on parametric Result handling — once the fix lands, REMOVE any "match inline" workaround language. | Documents the invariant: let*-binding a parametric value with annotation works the same as inline matching. |

Four pieces. The fix is a type-flow change in the checker; the
tests close the gap that hid it; the error message ensures the
*next* failure mode (whatever it is) doesn't take this arc's full
session to diagnose.

---

## Open questions for the substrate session

1. **Is this checker code path or constraint-solver code path?** The
   visible symptom is "type variable :?79 unsolved" at the
   `:wat::core::second` call site. The root cause is upstream — the
   binding's annotation didn't generate the constraint `pair_type =
   T` where `T` was bound from `Result<(i64,i64), i64>`. Find which
   pass should have generated that constraint; explain why it
   didn't.
2. **Does this affect Option<T> too?** Probe B is Result-specific.
   The type system likely treats Option and Result the same way
   (both arc 048 tagged enums). Spot-check before declaring.
3. **Does the same flaw affect non-let*-binding contexts?** What
   about a function parameter `(x :Result<T, E>) -> ...` then
   matching x in the body? Function parameters are also annotated
   bindings.
4. **Do user-defined parametric enums have this flaw?** If they do,
   the fix has to be in the generic match-pattern inference path,
   not a built-in-Result special case.

---

## What this arc deliberately does NOT do

- **Does not redesign the type system.** The fix is a propagation
  rule, not a rewrite. Find the missing constraint; emit it.
- **Does not reach into wat-side surface.** No new forms; no new
  syntax.
- **Does not pre-emptively annotate.** A `:Result<T, E>` value
  whose `T` *cannot* be solved from let* + match (e.g., truly
  underdetermined) should still error — but with the better
  message, not the cryptic one.

---

## Test strategy

The probe at `wat-tests-integ/experiment/099-walkstep-probe/probe.wat`
becomes the lab-side regression test (currently 3/6 passing — should
be 6/6 after this arc). Six new substrate-side tests in
`runtime.rs`'s test module per the table above. After this arc:

- Probe B passes.
- Probe B1 passes.
- Probe B2 passes.
- Proof 018's walker rewrite collapses cleanly.
- Lab umbrella 059 slice 1's L1+L2 cache uses `walk` without
  workarounds.
- Future consumers can let*-bind any parametric value and match it
  freely.

---

## The thread

- **Arc 028** — Result wrap. The first parametric tagged enum.
- **Arc 048** — tagged-enum constructors + match. Parametric match
  patterns landed.
- **Arc 055** — recursive patterns. Sub-patterns work.
- **Arc 070** — `:wat::eval::walk` + `WalkStep<A>`. First
  consumer of parametric variant constructors at scale.
- **Arc 071** — lab-harness enum-method parity. Closed the *first*
  flaw blocking proof 018.
- **Arc 072 (this)** — let*-annotation propagation. The *second*
  flaw, surfaced behind the first.
- **Next** — proof 018's walker rewrite ships. The trader's
  enterprise consumes `walk` directly.

PERSEVERARE.
