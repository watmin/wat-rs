# NOTE — the bare-symbol constructors are RETIRED at the door and LIVE behind it

> **Builder, 2026-08-30:** *"what is `(Some 1)` .... we killed the short hand names... like.... a
> long time ago?....."*
>
> Measured that day, in three probes. **No row, nothing drawn.**

## The answer: you killed the SURFACE, not the MECHANISM

**Check time — closed, with a remedy.** Arc 109 slice 1h did the retirement properly:

```
Some: parameter (retired bare-symbol exception) expects :wat::core::Some; got Some
Remedy { form: ":wat::core::Some", kind: :retirement,
         note: "rename `(Some x)` -> `(:wat::core::Some x)` at constructor sites;
                rename `((Some v) ...)` -> `((:wat::core::Some v) ...)` at match-pattern
                sites (arc 109 slice 1h)" }
```

**Runtime — open.** The dispatch arms survive at `src/runtime.rs:5183,5186,5189`:

```rust
WatAST::Symbol(ident, _) if ident.as_str() == "Some" => …
WatAST::Symbol(ident, _) if ident.as_str() == "Ok"   => …
WatAST::Symbol(ident, _) if ident.as_str() == "Err"  => …
```

and any path that reaches evaluation without the static checker still takes them. Measured:

```clojure
(:wat::core::Result/expect (:wat::eval-ast! (:wat::core::quote (Some 99))) "…")
;; => #wat.core.Option/Some [99]     exit 0
```

★ **This is FM 14 exactly** — *"surface retirement leaving internal identifiers as leftovers"* — and
its own worked example is arc 155's `lambda`, which cost arc 162 to clean up six months later.

## Why it matters more now than it did yesterday

`:wat::core::Some` was homed and ruled this session (Stone A-2-ii-b-1: Pure · Deterministic ·
Total). The bare form was not, and **the two now disagree on a reachable path**:

```
(:wat::core::Some 1)   pure? -> true     registered, ruled
(Some 1)               pure? -> false    unregistered, unruled
```

`[[feedback_a_slot_with_two_implementations_is_two_slots]]`. One spelling of one constructor,
answering two ways depending on which door it came through.

⚠ **My first read of this was wrong and is worth recording.** Seeing the check-time refusal, I
concluded the arms were *"unreachable from any checked program, so not a live two-slots defect."*
Half true: unreachable from a **checked** program, reachable from an **unchecked eval path** — and
`eval-ast!` is a supported, deliberate one. A door is not closed because the *main* corridor to it
is. `[[feedback_a_pass_answers_only_the_question_the_instrument_asks]]`

## What is NOT wrong here

- The retirement itself is exemplary: named, remedied, and pointing at the exact replacement.
- The rider that homed the FQDN forms was **right to leave these alone** — they were outside the
  stone's blast radius, and it flagged the second dispatch path in its report rather than quietly
  widening scope.
- The arms are not *unused* code in the compiler's sense; they are reachable, which is why no lint
  ever complained.

## The disposition, when someone draws it

Three arms in `eval_list`. Deleting them makes the runtime agree with the checker, and makes
`(Some x)` fail the same way whichever door it arrives through. The check-time refusal, the remedy,
and the `RETIREMENT_TABLE` machinery all already exist — **only the runtime arms have to go.**

⚠ Confirm first that nothing in the substrate's own unchecked paths relies on them: the `.wat`
corpus is statically checked, but macro expansion, `eval-ast!` call sites, and any freeze-time
evaluation are not the same population. **Grep is not a census here** — the probe above is the
instrument, one per path.

★ And note the ROAD's step 4 — *"every call head a symbol"*. A future arc will make bare-symbol
heads the NORMAL form. These arms must be deleted for the right reason (a retired shorthand the
checker refuses), not kept for the wrong one (they look like the coming syntax).
