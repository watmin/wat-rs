# NOTE — the measurement, and the three hypotheses it killed

**Found 2026-09-10**, mid dot-flip landing, at `02dc901a2`. The builder: *"you just found a very
deep bug… we do not leave here until this is resolved."*

## The defect, in one A/B

```
(:probe::E.Direct {:one (:probe::Plain :NOPE 1)})                              errs+=0  ty=:?726
(:probe::E.Direct {:one (:wat::core::kwargs-construct :probe::Plain :NOPE 1)}) errs+=1  ty=:probe::Plain
```

Same position, same checker, same field. The only difference is whether the kwargs **companion
macro** was expanded. `:NOPE` is not a field of `:probe::Plain`; the first form `--check`s **clean**.

**Macro expansion does not reach inside a variant map-ctor's field values.** At check time the head
`:probe::Plain` is the bare macro name — which carries no scheme, because arc 294 item 9a moved the
positional ctor scheme to the PRIME (`:probe::Plain'`) and left the bare name as the kwargs macro.
`infer` therefore returns a **fresh unification variable** with no error, and
`assignable(:?726, :probe::Plain)` passes trivially, because a var unifies with anything.

★ **The silent absorption is the second half, and it is the worse one.** An unresolvable call head
yielding a fresh var that satisfies every expectation is what converts a missed expansion into *no
diagnostic at all*. Either half alone would be visible; together they are silent.

## Blast radius — measured, five positions

```
variant map-ctor, DIRECT field    (:E.Direct {:one (:Plain :NOPE 1)})       MISSED  exit 0
variant map-ctor, VECTOR field    (:E.Vec {:many [(:Plain :NOPE 1)]})       MISSED  exit 0
RECORD kwargs, direct field       (:Holder :one (:Plain :NOPE 1))           caught  exit 1
RECORD kwargs, VECTOR field       (:Holder :many [(:Plain :NOPE 1)])        caught  exit 1
plain CALL argument               (:probe::take (:Plain :NOPE 1))           caught  exit 1
```

**Variant map-ctor field values only.** Records, vectors and calls are all fine.

## ⛔ THREE HYPOTHESES, EACH KILLED BY MEASUREMENT

Recorded because each was plausible, each would have sent a rider into the wrong module, and each
died to one command.

**① "The parametric machinery is defective."** The first repro used a parametric enum, so this was
the obvious read — and it is what I nearly told the builder. A four-row grid killed it: a
NON-parametric record element misses identically. Nothing in the type-parameter machinery is
involved.

**② "It is vector-specific."** The `Alarm` case lives in `arms <- (Vector :- [(Alarm :- [O])])`, so
the vector looked load-bearing. The five-position grid killed it: a DIRECT variant field misses too,
and a vector inside a RECORD is caught. The vector was incidental.

**③ "The `zip` at `infer_enum_map_ctor`'s value loop truncates."** A `zip` between two lists that
must be the same length silently drops the tail, so this looked like the mechanism. Instrumented:
`ordered=1 params=1` — the loop runs.

⚠ **And my own probe for ③ had the defect this whole campaign has been hunting**: it printed only on
MISMATCH, so `0 == 0` would have been silent. A null I would have read as "fine" could have meant
"never ran". Re-instrumented to print unconditionally.
`[[feedback_a_green_test_can_prove_nothing]]`

⚠⚠ **And one step from the end I nearly concluded from the wrong instrument.** I used
`:wat::core::macroexpand` to show a nested form stays unexpanded — but `macroexpand` is OUTER-ONLY
by design (`macroexpand-1` exists alongside it for exactly this distinction), so it could not have
shown otherwise. The conclusion happened to be right; the evidence did not support it. The A/B at
the top — hand-writing the expanded form — is what actually establishes it.

## Why nothing caught this

`alarm_op_internal_check` (`CheckErrorKind::PublicOpInAlarm`) is the one wall that lives in this
position: `:wat::service::Alarm` is constructed inside
`Outcome.ReplyAndArm`'s `arms` field. It has been **silent**.

★★ Its only live caller used the **positional** variant form, which takes a different inference path
— and positional variant construction was retired independently of all this. So the wall's sole test
was passing for a reason nobody intended, and when ruling ②-A rewrote that fixture to map form
(correctly), the wall's death became visible.

**A broken test hid a broken wall, and the test was broken in a way that made it look maintained.**

## Reproduction

`wat-scripts/scratch-pad/probe-parametric-vector-field-is-unchecked.wat` — kept under its original
name for the git trail, though hypothesis ① proves the name is wrong: it is neither parametric nor
vector-specific. The file's own header carries the corrected rows.
