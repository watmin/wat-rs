# NOTE (arc 296) — 24 of 39 error kinds hand a wat program an EDN blob where the other 15 hand it a sentence

**Filed 2026-08-28 during arc 255 Stone O-iv-a. A POINTER, not a decision — and the decision may
already belong to this arc.** Surfaced by the rider that struck O-iv-a, which stayed inside its
measured blast radius and reported the tension instead of widening scope. It was right to; the
orchestrator ruled the one instance in-scope (it was that stone's whole deliverable) and filed the
CLASS here.

## The measurement

`runtime_error_to_eval_error_value` (`src/runtime.rs:~22460–22530`) builds the `:wat::core::EvalError`
a wat program actually reads. It matches `RuntimeErrorKind` with an explicit arm per variant and ends:

```rust
        // Fallback for variants that don't deserve a dedicated kind.
        _ => ("runtime-error", format!("{}", err)),
```

**`err` is a `&RuntimeError`, and `RuntimeError`'s `Display` renders the full EDN WIRE FORM** — not
the human-readable prose that `RuntimeErrorKind`'s own `Display` (`src/value/signal.rs:584+`) so
carefully writes. So a variant with an explicit arm gives the caller a sentence; a variant without
one gives it a nested blob:

```
armed   (:wat::core::EvalError/message …)  ->  "unknown function: :wat::f64::max-of"
unarmed (:wat::core::EvalError/message …)  ->  "#wat.runtime/NotValueDispatchable {:message \"…\"
                                                 :location #wat.core/Span {:file \"…\" :line 43 …}
                                                 :causes [] :name \"…\"}"
```

**Measured at `00146f9bc`+O-iv-a: 15 of 39 variants are armed. 24 fall through.** The 24:

```
AssertionFailed · DeclarationInExpressionPosition · DottedName · DuplicateDefine · EdnCoerceMismatch
IntegerOverflow · MacroAbort · MacroExpansionFailed · NoEncodingCtx · NoMacroRegistry
NoMatchingClause · NoSourceLoader · ParamShadowsBuiltin · PostconditionFailed · ReservedPrefix
ReteDefnAxisViolation · ReteDefnRecursive · SandboxScopeLeak · ServiceNotRunning · UnknownField
UnnamespacedName · UnreachableClause · UserMainMissing · WriteStopped
```

Reproduce:
```bash
awk '/pub enum RuntimeErrorKind/,/^}/' src/value/signal.rs | grep -oP '^\s{4}\K[A-Z][A-Za-z]+' | sort > /tmp/all
awk 'NR>=22460 && NR<=22530' src/runtime.rs | grep -oP 'RuntimeErrorKind::\K[A-Z][A-Za-z]+' | sort -u > /tmp/armed
comm -23 /tmp/all /tmp/armed        # the unarmed 24
```
⚠ The second command's line window is hand-pinned and will drift. Re-derive it from
`grep -n '_ => ("runtime-error"' src/runtime.rs` before quoting the count.
`[[feedback_an_instrument_must_outlive_the_number_it_produced]]`

## Why this is NOT obviously a defect — and why it is this arc's call, not a passing stone's

**Arc 296 is named `diagnostics-fully-edn`.** An EDN-rendered message may be the *direction*, not a
wart. Three readings, and this note deliberately does not choose:

1. **The blob is the goal.** Diagnostics become fully structured; the 15 armed variants are the
   legacy prose path and should eventually be *removed*, not extended. Then the 24 are ahead, not
   behind — and O-iv-a's new arm is a step backwards that should be reverted with the other 15.
2. **The sentence is the goal, on THIS surface.** `EvalError/message` is a `:String` a program
   prints; the structure already lives in `EvalError`'s other fields and in the wire form. Then the
   24 are a gap and the fallback should be retired variant by variant.
3. **Both, split by surface.** `message` stays prose; the wire form stays EDN. Then the defect is
   narrower and purely mechanical: the wildcard should not reach for `RuntimeError`'s `Display` at
   all — `format!("{}", err.kind)` would give prose for every variant, armed or not, in ONE line.

★ Reading 3 is worth weighing first because it costs one token change and closes the whole class
without deciding the philosophy: `RuntimeErrorKind`'s `Display` is already written for all 39
variants — the fallback simply reaches past it to the wrapper. **This has not been tried and this
note does not claim it works**; `RuntimeError::Display` may add the span prefix the prose expects.
Measure before believing it.

## What the O-iv-a stone did, and why it stopped there

Stone O-iv-a minted `NotValueDispatchable` so `apply` would stop calling 331 registered verbs
"unknown function". It landed with an explicit arm, because a stone whose entire deliverable is an
honest, readable diagnostic cannot ship that diagnostic as a nested blob when every neighbouring
error on the same path is a sentence. **That is one instance handled, not the class** — and this
note exists so the class is not mistaken for handled.

## Refs

- `src/runtime.rs` — `runtime_error_to_eval_error_value`, the match and its wildcard.
- `src/value/signal.rs:584+` — `RuntimeErrorKind`'s `Display`, written for all 39, reached for 15.
- `src/value/signal.rs:189` — `#[derive(wat_edn::ToEdn)]`; the wire form is generated and is fine.
- `docs/arc/2026/06/255-builtin-registry/BRIEF-STONE-O-iv-a-the-honest-word.md` — the stone whose
  blast radius the rider correctly refused to widen.
