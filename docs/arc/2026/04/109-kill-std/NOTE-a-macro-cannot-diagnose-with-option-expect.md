# NOTE (arc 109) — a wat macro CANNOT diagnose with `Option/expect`: it panics, it does not raise

**Filed 2026-08-20 during binder strike β-i. MEASURED, both directions.**

## The finding

`:wat::core::Option/expect` in a **macro body** produces a `#wat.kernel/AssertionFailure` that
**aborts the thread**. It does not produce a macro error VALUE. So a caller that expects to inspect
a structured failure — `expect_startup_err`, any consumer reading `#wat.macro/ProgramBodyEvalFailed`
— gets nothing to read.

Measured, on the same one-arg `defrecord` fixture, two macro bodies apart:

```
Option/expect guard   → #wat.kernel/AssertionFailure {:message "defrecord: missing field-vector — …"}
                        …and the test harness fails with NO error value to match against.

primitive failure     → #wat.macro/ProgramBodyEvalFailed {:macro-name ":wat::core::defrecord"
                          :cause #wat.macro/MacroEvalRuntimeFailed
                            {:cause #wat.runtime/MalformedForm
                              {:head ":wat::core::rest" :reason "cannot take rest of empty Vec"}}}
```

**The second is worse to READ and better to HANDLE.** The first names the actual fault in the user's
own vocabulary; the second names `:wat::core::rest`, an internal primitive, and points at
`wat/Record.wat`. But only the second is a value.

## Why this bit, and the mistake it produced

β-i made `defrecord` variadic so an optional `:- [T…]` binder can sit between the name and the field
vector. That retired the fixed-arity macro signature — and with it the diagnostic
*"macro :wat::core::defrecord expects 2 arguments; got 1"*, which named the macro and the fault.

The orchestrator saw the replacement message leak `:wat::core::rest` at the user, judged it a net
loss of diagnosis (correctly — `EXPECTATIONS` row 6 says the message must be *replaced*, not
deleted), and "fixed" it with `Option/expect`. **That traded a bad message for a broken error
class**, which is strictly worse: a structured error with poor prose can be improved in place; a
panic cannot be caught by the consumer at all.

★ The general shape: **optimizing the STRING while destroying the SHAPE.** The test that caught it
was not asserting on the message — it was asserting that an error VALUE comes back.

## The dead-code corollary, which explains the precedent

`:wat::core::defstruct` (`wat/core.wat:1830`) — the worked example β-i copied — carries
`(Option/expect (last args) "defstruct: missing field-vector")`. **That string is unreachable.**
`(first args)` is bound earlier in the same `let` and throws on an empty vector, so the `None` case
`Option/expect` guards can never be reached. Verified: `(:wat::core::defstruct :usr::BadStruct)`
answers *"malformed structtype declaration: … got 2 args"* — from the RUST declaration parser, never
from that string.

So the precedent's friendly message was never doing anything, and the pattern was copied forward
in good faith. Both `defrecord` macros now carry the same latent shape, with a comment saying so.

## What a macro SHOULD use — open, not answered here

The honest diagnostic needs a wat-level way to raise a **structured** error from a macro body, the
way the Rust declaration parsers raise `MalformedDecl`. `Option/expect` is not it. Candidates worth
weighing when someone closes this:

- a `raise`-family verb that produces a macro-error value rather than an assertion;
- letting the macro EMIT a deliberately-malformed declaration and relying on the Rust parser's own
  diagnostic — which is, measured above, exactly what `defstruct` already does by accident.

The second is the cheaper answer and it already has a working precedent in the tree.

## Status

`defrecord` / `holon defrecord` currently diagnose a missing field-vector by leaking
`:wat::core::rest`. The golden
(`tests/types/probe_arc227_stone2_defrecord__probe_two_arg_form_only_one_arg_errors.edn`) records
that as the shipped behaviour, and the test's comment says why the class changed. **This is a known
loss of diagnosis quality, recorded rather than papered over** — not deferred prose: the work is
bounded to the two macro bodies in `wat/Record.wat` and blocked only on choosing one of the two
mechanisms above, which is a builder call about wat's error vocabulary.
