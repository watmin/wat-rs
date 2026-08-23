# ⛔ CLOSED — the read-failure path was dead substrate-wide, and it is fixed

**Filed 2026-08-23** while measuring the codemod against the arc 109 lexer wall. **Closed the same
day**, one commit later, after the builder asked the right question: *"we just committed a known
bug?"*

## What was wrong

`:wat::core::ReadOutcome::Malformed`'s cause field is **declared** `:wat::core::Error`
(`src/types.rs`). `read_outcome_malformed` (`src/edn_shim.rs`) built it with a two-rung decode
ladder — STRICT first, FOREIGN fallback — and the FOREIGN rung yields a `Value::ForeignRecord`, a
self-describing dynamic bag that satisfies the `Error` surface **nowhere**. It was returned directly.

**The declared type was a lie at the boundary.** Every consumer of the house idiom

```clojure
((:wat::core::ReadOutcome::Malformed __cause) … (:wat::core::Error/message __cause) …)
```

died with `UnknownFunction: :wat::edn::ForeignRecord does not implement surface method message`
instead of reporting the read failure.

```
75 call sites · 57 files
   wat/fix.wat (9) · wat/lint.wat · wat/service.wat · wat/deporder.wat
   wat/core.wat — string::interpolate · wat/telemetry/journal.wat
   32 of the 66 recorded migrations in wat-scripts/fixes/
```

Written 75 times. **Invoked zero times** — because until arc 109's lexer walls landed, `read-string`
never failed on corpus text. `[[feedback_a_green_test_can_prove_nothing]]` at substrate scale.

⚠ **And it was never about arc 109.** Measured: `(unclosed` — an unbalanced paren, nothing to do with
angle brackets — crashed identically. Every read failure, from any cause, was unreportable.

## The eighth instance of the shape, and the worst

`check_failed_cause` (`src/runtime.rs`) runs the **identical ladder** and disposed of it **correctly**,
and its own comment states the rule:

> *"The nested diagnostic rides as a CAUSE under a real `Fault`, rather than BEING the returned
> value, so `:CheckFailed`'s declared `:wat::core::Error` is always satisfied by a genuinely typed
> record — the dynamic part is contained in the causes chain, which is exactly what a causes chain
> is for."*

`read_outcome_malformed`'s comment says *"Identical ladder to `check_failed_cause` in `runtime.rs`,
and for the identical reason."* The ladders were identical. **The disposal was not, and that is where
the entire defect lived.** `[[feedback_a_slot_with_two_implementations_is_two_slots]]` — and here the
confidence transferred by an explicit comment asserting the two were the same.

★ A third site had it too: **`read_json_outcome_malformed`** — the `wat --mcp` JSON path, whose whole
declared purpose (`src/types.rs`) is that *"a malformed byte must not be able to raise"* because its
input arrives from an untrusted remote harness. Two defective, one correct, and **both defective ones
cited the correct one by comment.** A grep for the function name finds one; a grep for the BEHAVIOUR
(`edn_to_value_foreign`) finds all three.

## The fix

`fault_with_cause` is now `pub(crate)`, takes a real location, and is **the one door** all three
ladders pass through. The decoded diagnostic rides as a cause under a genuine `:wat::core::Fault`;
the declared type stops lying; the dynamic part stays navigable in the causes chain.

Measured after:

```
(unclosed                 →  "unclosed '('"
:wat::core::Vector<i64>   →  the full arc 109 refusal message
```

## What it cost, and the lesson that is actually mine

Three tests in `tests/resolve/` had been re-pointed to assert the crash — `err.contains("ForeignRecord")`
— as their expected behaviour, with honest comments explaining why. They were committed that way.

**"It predates the strike" is a justification wearing a fact's clothes.** Our own floor rule says
"pre-existing" is not a disposition; it describes the search, not the failure. The same reasoning
that is forbidden for a red floor is forbidden here.

And the real failure was upstream of that: **I filed this instead of surfacing it.** A NOTE in an arc
directory is not a report. It was mentioned to the builder once, in a subordinate clause, as a
footnote to a sequencing point — and then a rider built three tests on top of it and it shipped. The
builder learning of a substrate-wide dead error path by asking *"we just committed a known bug?"* is
the measurement of that failure.

★ **A finding that changes what a rider should build is not a NOTE. It is a report, made before the
rider is released.** `[[feedback_state_what_the_instrument_can_see_before_quoting_it]]`

All three tests now assert the REFUSAL, through a fixture that RETURNS the cause's message rather
than diverging — so each one exercises the very `Error/message` path that was dead.

floor 4881/4881, clippy 0.
