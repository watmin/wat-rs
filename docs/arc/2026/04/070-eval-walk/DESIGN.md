# Arc 070 — `:wat::eval::walk` and `StepResult::AlreadyTerminal`

**Status:** shipped 2026-04-27 (2 phases). Pre-implementation reasoning artifact.
**Predecessors:** arc 057 (HolonAST `Hash + Eq`), arc 058 (`HashMap<HolonAST, V>`), arc 066 (`eval-ast!` Result wrap), arc 068 (`eval-step!`), arc 069 (`coincident-explain`).
**Downstream consumer:** holon-lab-trading proof 018 (the *fuzzy-on-both-stores* proof) — its handwritten walker reimplements the chain-traversal driver every fuzzy/exact cache proof has had to reinvent. Also: every future proof that walks an expansion chain.

Builder direction (2026-04-27, mid-proof-018 review, after the
walker silently swallowed the case where `eval-step!` returned `Err`
on a trader-shape thought):

> "this realization you just had... we need this as a thing wat
> provides... the terminal-ness - what's missing from wat-rs for
> this?...."

> [proposed: bigger — promote the walker pattern itself]
>
> "that sounds like the thing - that sounds /very/ core"

This arc closes two gaps surfaced by proof 018:

1. **`eval-step!` conflates three meanings of `Err`.** "I am a value already" is bundled with "I am malformed" and "no rule applies for this head". Callers can't tell which. Proof 018's walker silently dropped the cache write because it couldn't distinguish.
2. **The walker pattern is reimplemented in every chain proof.** Proofs 015 / 016 / 017 / 018 each rewrote the same `eval-step! → match → record → recurse` shape. This is a substrate concern, not user code.

After this arc, the proof's walker collapses to a substrate call and a per-coordinate fold function — ~10 lines instead of ~50.

---

## Two changes, one arc

### Change 1 — `StepResult::AlreadyTerminal`

Today:

```scheme
(:wat::core::enum :wat::eval::StepResult
  (StepNext     (form  :wat::WatAST))
  (StepTerminal (value :wat::holon::HolonAST)))
```

`Err(EvalError)` covers value-as-built, malformed, and no-rule-found alike. Callers infer terminal-ness from the absence of an error, which is the wrong polarity.

Proposal — add a third variant:

```scheme
(:wat::core::enum :wat::eval::StepResult
  (StepNext        (form  :wat::WatAST))             ;; one rewrite happened
  (StepTerminal    (value :wat::holon::HolonAST))    ;; reduced to value this step
  (AlreadyTerminal (value :wat::holon::HolonAST)))   ;; input was already a value; no work
```

`Ok(AlreadyTerminal h)` fires when `eval-step!` is given a WatAST that lifts back to a HolonAST with no β-redex — e.g., `to-watast(Bind(Atom, Thermometer))` from a trader-shape thought. The substrate KNOWS this; encode it.

`Err(EvalError)` retains its narrow meaning: malformed, type-clashed, effectful-in-step, or `NoStepRule`. Genuine failures.

The existing two variants keep their current semantics. The conversion of "Err because already a value" → `Ok(AlreadyTerminal _)` is an internal classification change in `eval_form_step`.

### Change 2 — `:wat::eval::walk`

Today every chain proof writes the same shape:

```scheme
(walk form acc):
  match eval-step!(form):
    Ok(StepTerminal t):  record (form, t); return (t, acc')
    Ok(StepNext next):   record (form, next); recurse on next; backprop; return
    Ok(AlreadyTerminal t): record (form, t); return (t, acc')
    Err e:               (genuine error; bail or panic)
```

Proposal — pull this into the substrate as a fold:

```
:wat::eval::walk
  (form    :wat::WatAST)
  (init    :A)
  (visit   :fn(:A, :wat::WatAST, :wat::eval::StepResult) -> :wat::eval::WalkStep<A>)
  -> :Result<(:wat::holon::HolonAST, :A), :wat::core::EvalError>
```

Where `WalkStep<A>` controls the walker's loop:

```scheme
(:wat::core::enum :wat::eval::WalkStep<A>
  (Continue (acc      :A))                            ;; keep walking; here is the new acc
  (Skip     (terminal :wat::holon::HolonAST) (acc :A))) ;; short-circuit; here is a known terminal
```

**Semantics:**

- The walker calls `visit(acc, current-form, step-result)` once per coordinate. `step-result` is what `eval-step!` returned at this coordinate.
- `visit` chooses what to do:
  - `Continue(acc')` — keep walking. If `step-result` was `StepNext`, walker recurses on the next form; if `StepTerminal` or `AlreadyTerminal`, the walk terminates with that value as the chain's terminal.
  - `Skip(terminal, acc')` — caller has its own answer (e.g., a fuzzy cache hit on `from-watast(current-form)`); walker stops, returns `terminal`.
- If `eval-step!` returns `Err`, walker stops and propagates the error as `Result::Err`.
- The walker's return: `(final-terminal-HolonAST, final-acc)`.

**Critical invariant:** `visit` sees every coordinate exactly once, in order. The caller's fold is over the coordinate sequence; the walker handles the eval-step!/recurse/backprop plumbing.

---

## What proof 018's walker collapses to

Before — ~50 lines of recursion, manual cache record on the way down and the way up, silent-swallow Err branch:

```scheme
(:wat::core::define
  (:exp::walk-and-record (form-h :wat::holon::HolonAST) (tier :exp::L1Tier)
                         -> :(wat::holon::HolonAST,exp::L1Tier))
  (:wat::core::let*
    (((form :wat::WatAST) (:wat::holon::to-watast form-h)))
    (:wat::core::match (:wat::eval-step! form)
      ((Ok r)
        (:wat::core::match r
          ((:wat::eval::StepResult::StepTerminal t) ...)
          ((:wat::eval::StepResult::StepNext next) ... recurse ...)))
      ((Err _e) ... silent fallback ...))))
```

After — one call, one fold function:

```scheme
(:wat::core::define
  (:exp::record-coordinate
    (tier :exp::L1Tier)
    (form-w :wat::WatAST)
    (step :wat::eval::StepResult)
    -> :wat::eval::WalkStep<exp::L1Tier>)
  (:wat::core::let*
    (((form-h :wat::holon::HolonAST) (:wat::holon::from-watast form-w)))
    (:wat::core::match step -> :wat::eval::WalkStep<exp::L1Tier>
      ((:wat::eval::StepResult::StepNext next-w)
        (:wat::core::let*
          (((next-h :wat::holon::HolonAST) (:wat::holon::from-watast next-w)))
          (Continue
            (:exp::L1Tier/new
              (:exp::cache-record (:exp::L1Tier/next-cache tier) form-h next-h)
              (:exp::L1Tier/terminal-cache tier)))))
      ((:wat::eval::StepResult::StepTerminal t)
        (Continue
          (:exp::L1Tier/new
            (:exp::L1Tier/next-cache tier)
            (:exp::cache-record (:exp::L1Tier/terminal-cache tier) form-h t))))
      ((:wat::eval::StepResult::AlreadyTerminal t)
        (Continue
          (:exp::L1Tier/new
            (:exp::L1Tier/next-cache tier)
            (:exp::cache-record (:exp::L1Tier/terminal-cache tier) form-h t)))))))

;; usage:
(:wat::eval::walk (:wat::holon::to-watast thought-h)
                  (:exp::tier-empty)
                  :exp::record-coordinate)
```

The visit-fn handles the THREE step-result variants distinctly (no silent fallback). The walker handles iteration. Backprop-on-the-way-up disappears because `walk` is forward-only — the cache lookup short-circuits via `Skip`, which is what backprop was approximating.

---

## Why this shape, not others

**Why a fold, not a custom recursion?** Every consumer wants a different accumulator (cache, trace, counter, tier). A fold takes the accumulator type as a parameter; recursion bakes it in. Folds compose; recursive walkers don't.

**Why `WalkStep<A>` instead of just returning `A`?** Early termination. Fuzzy cache hits are the central pattern (proof 017, proof 018, BOOK Chapter 59). Returning `A` forces the walker to step every coordinate; `Skip(terminal, A)` lets the caller short-circuit on a hit.

**Why pass step-result to visit, not just the form?** The visitor needs to know the chain shape — was this an intermediate `StepNext` (record `(form → next)`), a `StepTerminal` (record `(form → terminal)`), or an `AlreadyTerminal` (record `(form → form)`). The substrate already computed this; pass it through.

**Why not split into separate `on-next` / `on-terminal` / `on-already-terminal` callbacks?** Three separate function arguments per call site is hostile. One visitor with a `match` is cleaner — and the consumer often shares record-cache logic across the variants.

**Why surface `AlreadyTerminal` separately?** Because it's not the same thing as `StepTerminal`. `StepTerminal` says "this step reduced a redex to a value". `AlreadyTerminal` says "no work was done; the input was the value". For a cache, both result in the same record. For a tracer, the distinction matters (one is a chain length of 1, the other is 0). Don't conflate.

---

## What's already there (no change)

| Surface | Status |
|---------|--------|
| `:wat::eval-step!` | shipped (arc 068) |
| `:wat::eval::StepResult` (StepNext, StepTerminal) | shipped (arc 068) |
| `:wat::core::EvalError` | shipped (arc 028 + arc 068 extensions) |
| `:wat::holon::to-watast` / `from-watast` round-trip | shipped |
| HolonAST `Hash + Eq` | shipped (arc 057) |
| `EvalError::EffectfulInStep`, `EvalError::NoStepRule` | shipped (arc 068) |

Five pieces in place. The new surface attaches at one new public function plus one enum variant plus one new built-in enum.

## What's missing (this arc)

| Op / change | What it does |
|----|----|
| `:wat::eval::StepResult::AlreadyTerminal` (new variant) | Distinguishes "input was already a value" from genuine `Err`. |
| Internal `eval_form_step` classification | Detect post-`from-watast(to-watast)` round-trip: if input lifts to HolonAST with no β-redex anywhere in the tree, return `Ok(AlreadyTerminal h)` instead of `Err(NoStepRule)`. |
| `:wat::eval::WalkStep<A>` (new built-in enum) | Generic over A; two variants (`Continue` / `Skip`). |
| `:wat::eval::walk` (new primitive) | Fold over the chain; visit fires per coordinate. Returns `Result<(terminal, acc), EvalError>`. |
| USER-GUIDE rows + a chain-cache example | Shows the visit-fn pattern, the Skip-on-cache-hit pattern, the AlreadyTerminal handling. |

Five pieces. The expensive part is the AlreadyTerminal classification in `eval_form_step` — needs to recognize HolonAST-shaped WatAST inputs without false positives.

---

## Open questions for the substrate session

1. **AlreadyTerminal classification — at what depth?** A trader thought is `Bind(Atom, Thermometer)` — all three are HolonAST constructors. But `Bind(Atom, (some-fn 5))` has a redex inside. Does `eval-step!` walk down and step the inner `(some-fn 5)`, returning `StepNext`? Or does it return `AlreadyTerminal` because the OUTER constructor is a value-shape? Recommend: walk down to the leftmost-outermost redex; if NO redex exists anywhere, then `AlreadyTerminal`. Same as today's CBV order, just labeled correctly.
2. **`walk` recursion limit.** A pathological infinite chain shouldn't hang the runtime. Recommend: reuse `eval-ast!`'s recursion cap (whatever today's substrate enforces).
3. **`WalkStep::Skip(terminal, acc)` — is `terminal` validated?** If the caller short-circuits with `Skip(arbitrary-holon, acc)`, the walker trusts them. Recommend: trust. It's the caller's contract; they're saying "I know this is the answer." Mirrors how cache-hit short-circuits work in user code today.
4. **Should `walk` accept a `max-depth` parameter?** Same shape as a recursion cap, but per-call. Recommend: yes, optional. Default = no cap; explicit cap when caller needs it. Phase-2 if it complicates the signature.
5. **Per-iteration short-circuit on `eval-step!` Err.** If `eval-step!` returns `Err(NoStepRule)` mid-chain, walker propagates as `Result::Err`. The visitor never sees the error. Recommend: propagate as-is. If a consumer wants to recover, they wrap `walk` and `match` on the Result.

---

## Test strategy

Reuse proof 018 — once this arc lands, rewrite the proof's walker to use `:wat::eval::walk`. The 8 existing assertions (T0, T1, T2, T3, T3a, T4, T5, T6) become the substrate's regression suite for the walker primitive. Specifically:

- **T0** asserts `AlreadyTerminal` fires on a holon-constructor thought. Substrate-level invariant. **Was previously a probe test that revealed the silent `Err` swallowing.**
- **T1** asserts the walker fills the terminal cache via the visit-fn. Smoke test for `walk` itself.
- **T2** asserts byte-identical thoughts hit the cache (degenerate fuzzy). Tests `from-watast` identity inside the visit-fn.
- **T3** asserts near-equivalent thoughts hit fuzzily — load-bearing test, depends on Thermometer locality. `assert-coincident` renders diagnostic on failure (arc 069).
- **T3a** is the substrate's `coincident?` probe; unchanged after this arc.
- **T4** asserts distant values miss; locality is bounded.
- **T5** asserts cache fills without neighborhood interference at √d entries.
- **T6** asserts the fuzzy mechanism is the same primitive used for next-cache. Architectural completeness.

Add new tests for the substrate primitive itself:

- **W1** — `walk` on a fully-reducible chain (`(:my::indicator 1.95)` from proof 017): visit fires once per intermediate; final return is the terminal HolonAST. Fold-acc reflects the chain.
- **W2** — `walk` on an `AlreadyTerminal` form: visit fires once with `step-result = AlreadyTerminal`; final return is the form itself.
- **W3** — `walk` with `Skip(known-terminal, acc)` from the visit-fn: walker stops at the first coordinate; returned terminal is `known-terminal`, not the chain's natural terminal. Fuzzy-cache-hit shape.
- **W4** — `walk` propagates `Err(NoStepRule)`: malformed mid-chain form returns `Result::Err`; visit never fires for that coordinate.

The tests exist in proof 018's wat file plus a new `wat-tests/eval/walk.wat` for the substrate-level cases.

---

## Why this is the right next arc

The walker pattern has shown up in proofs 015, 016, 017, 018. Each proof reimplemented it. Each time, the consumer was a thinker (the user) re-debating the substrate's semantics in user code. The substrate already KNOWS what `StepResult` means — but consumers can't ask it cleanly.

After this arc, the question "how does a thought walk to its terminal?" is answered by `:wat::eval::walk`. The trader's enterprise can use it. The MTG enterprise will use it. Any future domain that wants chain-of-rewrites caching uses it.

The terminal-ness becomes a substrate fact, not a per-consumer inference. That is what "very core" means.
