# NAMING TARGET — the CheckErrorKind for "a public op was armed on an alarm"

> **Materialized for an intueri cast (R17 self-prompt-injection: give the ward a real artifact to
> gaze at, not a description of one).** The builder's verdict on the shipped placeholder:
> *"`AlarmArmsPublicOp` — pretty awful name."* The cast was already marked OWED at ship; this is it.

## What the error MEANS — the promise the name must keep

A `defservice` handler may schedule a future invocation of one of its own ops by constructing an
`:wat::service::Alarm` and returning it in `Outcome::ReplyAndArm` / `NoReplyAndArm`. When that timer
fires, the op runs **with a timer in the caller slot — there is no client.**

Ops come in two declared kinds, and the substrate already distinguishes them structurally:

| kind | declaration | arm shape | has a client? |
|---|---|---|---|
| **public** (client-facing, on the `:satisfies` surface) | `(poll [s req] …)` | 2-param | yes — a real peer |
| **internal** (leading dash, not on the surface, nobody can dial it) | `(-tick [s] …)` | 1-param | **no, by declaration** |

The `Op` enum's variant for an internal op **retains the leading dash** (`Poll`, `Bump`, `-Tick`) —
`wat/service.wat:876-892`, deliberate and scope-preserved.

**The error fires when the op placed in an alarm is a PUBLIC one.** Such a handler would run believing
it is serving a client, have none, and its reply would go nowhere with nothing reported (measured:
durable state mutated, `Outcome::Reply` returned, no error surfaced anywhere). The rule: **only an op
declared to have no client may be scheduled where there will be no client.**

## Where the name is READ

At a construction site, in a wat program, by the author who wrote the alarm:

```clojure
(start [s req]
  (:wat::service::Outcome::ReplyAndArm s (:probe::Ticker::StartResponse::Ok)
    [(:wat::service::Alarm :after (:wat::time::Millisecond 5)
       :op (:probe::ticker::Op::Poll (:probe::Ticker::PollRequest)))]))
       ;;  ^^^^^^^^^^^^^^^ the offending value; the error's span points HERE
```

It surfaces as an EDN tag the author sees: `#wat.check/<TheName> {…}`.

## The variant's payload (fixed; not part of the cast)

```rust
<TheName> { variant: String, op_type: String }
//          ":probe::ticker::Op::Poll"    ":probe::ticker::Op"
```

## SIBLINGS — the neighbourhood the name must read like

Live `CheckErrorKind` variants, verbatim from `src/check/error.rs`:

```
AmbiguousClauseReturnAtCallSite   ArgTypeMismatch        ArityMismatch        ArityNotOne
CommCallOutOfPosition             DefRedefForbidden      DefRedefTypeChange
DefRestrictedCallerNotAllowed     EnsureFnInvalid        GuardExprNotBoolean
HygieneScopeDivergence            MalformedForm          MalformedSignature
NoMatchingClauseAtCallSite        NotFnForm              ProcessJoinBeforeOutputDrain
ProcessJoinHoldsStdinSender       ReservedPrefix         ReturnTypeMismatch
ReturnTypeNotBool                 TypeMismatch           UnknownCallee        UnnamespacedName
```

And the closest analogue in a *sibling* enum (`TypeErrorKind`), for a structurally identical
"a thing that may not be here, is here" wall:

```
ImpureFieldInPureAggregate    // "pure aggregate X may only hold pure fields — field f has impure type T"
```

## THE CANDIDATES — weigh these, and propose better if none keeps the promise

1. `AlarmArmsPublicOp` — **the shipped placeholder the builder rejected.** Reads as a narration of
   what happened (subject-verb-object), not as a name for an illegal configuration.
2. `PublicOpInAlarm` — mirrors `ImpureFieldInPureAggregate`'s `<offender> In <container-that-forbids-it>`.
3. `ArmedOpNotInternal` — states the violated requirement directly.
4. `AlarmOpNotInternal` — as above, scoped to the field.
5. `NonInternalOpArmed`
6. `ClientOpArmed`
7. `ArmWithoutCaller` — names the *consequence* (a scheduled call with no caller) rather than the shape.
8. `UnarmableOp` — names the offender's status.
9. `OpRequiresCaller` — names why it is refused.

## The questions for the cast

- Which candidate keeps its promise **to the wat author reading `#wat.check/<Name>` at their alarm**,
  with no context beyond the form they just wrote?
- Does the name survive being read WITHOUT the message body? (An error kind is read alone in logs,
  goldens, and greps.)
- Does it read like its siblings above — or does it stand out as a different species?
- Is "public" the right word for the offending kind, given the substrate's own vocabulary is
  *"internal (leading dash)"* vs *"client-facing / on the `:satisfies` surface"*? **The negation may be
  the honest framing** (`NotInternal`) or may be a double-negative that mumbles — say which.
- Is naming the CONTAINER (`Alarm`) load-bearing, or noise given the span already points at the alarm?

Return a single recommendation with reasoning, plus a runner-up and why it lost.
