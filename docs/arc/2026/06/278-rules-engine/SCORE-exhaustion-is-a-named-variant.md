# SCORE — exhaustion is a named variant

**SCORED. STOP-1, with the closed-enum path measured.** Executor: grok,
2026-09-07. `circuit.wat` restored to HEAD. The strike did not land.

Re-draw of `exhaustion-cannot-be-discarded`. Two closed enums were the
corrected mechanism. Three walls sit between DESIGN's `Got [peer reply]`
and a keyword match inside the EmptyEnv child.

## THREE WALLS, IN THE ORDER THEY FIRED

**1. Pure cannot hold a Peer.** `src/check.rs` arc 293.W.2b, parent
typecheck, before any child runs:

```
containment rule (arc 293.W.2b): :wat::enum::Pure enum ":fanout::SeenRetry"
may only hold pure variant fields — variant "Got" field "peer" has impure
type "(:wat::kernel::Peer :- [:fanout::Seen::Op :fanout::Seen::Reply])"
…
Declare the enum :wat::enum::Impure if it must hold a live resource
```

DESIGN's "Peer-typed field in a plain enum is routine" cites records
(`worker::State/q`), not Pure enums. Different form.

**2. Impure, script-level, is invisible in the child.** Constructors
are unknown; the scrutinee is an unsolved var; match is Open:

```
unknown callee: :fanout::SeenRetry::Got
unknown callee: :fanout::SeenRetry::Exhausted
keyword variant pattern :fanout::SeenRetry::Exhausted on a :?8400 scrutinee
non-exhaustive: open-typed match needs at least one hash-destructure arm
or a wildcard `_` arm.
```

`CheckResponse::Ok` at `:519` works because it lives on the Seen
surface, which `:peers` ships. A script-level enum is not in that bundle.

**3. Impure (or Pure-with-Reply) on the Worker surface trips S4c:**

```
surface :fanout::Worker :messages type references :fanout::Seen::Op
which is not declared in this surface's :messages
```

Shipping `Got [peer reply]` on Worker would require declaring the entire
Seen protocol on Worker.

## WHAT DID KEYWORD-MATCH

A **nullary** closed enum on the Worker surface, stdlib payload only:

```wat
(:wat::core::defenum :fanout::SeenRetry :wat::enum::Pure
  :Got []
  :Exhausted [attempts <- :wat::core::i64])
```

Keyword arms `(:fanout::SeenRetry::Got)` / `(:fanout::SeenRetry::Exhausted _att)`
ran in the EmptyEnv child. n=12 completed:

```
n=12;m=2;j=2;total=24;distinct=24;dup=0;…;check-exhausted=0;mark-exhausted=0;ack-retries=0
```

Peer and Reply rode beside the outcome as a Tuple. That is a named
variant the match must name. It is **not** DESIGN's `Got [peer reply]`.

Hash-destructure was not used. Row 3's dishonest path was not taken.

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ★ n=2000 fill-first | not reached |
| 2 | two closed enums, no `:- [` | ⚠ the form that matches cannot carry Peer/Reply |
| 3 | keyword-matched in the child | ✅ for nullary-Got on Worker; ❌ for DESIGN's payload |
| 4–7 | ladders / counters / failure path | not landed |
| 8–12 | no-args / drop / curve / scripts / floor | not run; circuit restored |

**STOP-2 / STOP-3 / STOP-4 / STOP-5 / STOP-6 did not fire.** STOP-1 did:
a non-parametric enum **with DESIGN's payload** cannot be keyword-matched
in the child. `:519` is general for types **the surface already ships**.
It is not general for a new enum that carries a Peer.

## WHAT THIS SAYS

The invariant (exhaustion is a variant a match must name) is expressible
in the child **only** as a closed enum of stdlib fields, declared on a
surface the child re-registers. DESIGN's `Got [peer reply]` has no such
form: Pure forbids Peer, Impure script-level is unknown in the child,
Worker-surface S4c forbids pulling Seen::Op.

The honest next construction is the nullary pair on Worker, with peer
and reply as a side Tuple — or a generic `defn` in `wat/`, which is
out of scope on the builder's ruling.

`circuit.wat` is HEAD.

---

# GRADING — claude, 2026-09-07

**NOT STRUCK. STOP-1 correct. My DESIGN was not merely unsupported — it was incoherent**, and the
substrate said so in a sentence that contains the whole answer. Tree confirmed at HEAD.

## Wall 1, reproduced verbatim on my own probe

```wat
(:wat::core::defenum :probe::Holds :wat::enum::Pure
  :Got [peer <- (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])] …)
```

```
ImpureVariantFieldInPureEnum — containment rule (arc 293.W.2b): :wat::enum::Pure enum
":probe::Holds" may only hold pure variant fields — variant "Got" field "peer" has impure type
"(:wat::kernel::Peer :- […])", which CANNOT BE RECONSTRUCTED FROM EDN BYTES ACROSS AN
ADDRESS-SPACE BOUNDARY. Declare the enum :wat::enum::Impure if it must hold a live resource
(it then STAYS IN SHARED MEMORY AND NEVER CROSSES).
```

★★★ **That is not a language limitation. It is the distributed-systems fact.** A `Peer` is a live
handle. I designed a *value* carrying a *live resource* across a boundary that live resources cannot
cross, and then spent three drafts blaming the type system for refusing it.

## ⛔ MY EXEMPLARS PROVED THE OPPOSITE OF WHAT I CITED THEM FOR

The DESIGN says: *"A `Peer`-typed field in a plain enum is routine here — `worker::State/q` and the
queue's `:ephemeral [store <- Peer …]` both hold one."*

`wat/service.wat:207`:

```
:ephemeral        what I carry        (the body: resources + peer clients; NEVER CROSSES)
```

★★★★ **I cited the field group whose definition is "never crosses" as evidence that a Peer can live
in a value that crosses.** Both memories I hold for exactly this fired and I did not hear them:
*cite an exemplar, do not describe one* — I cited, but of the wrong **form**; and *match the FORM,
not the token* — I matched the token `Peer` and ignored that `:ephemeral` is the substrate's own
word for the part that stays.

⚠ And the same error sits under the previous draft's `:519` claim. `CheckResponse::Ok` keyword-matches
in the child because it lives on the **Seen surface**, which `:peers` ships — not because
"non-parametric enums match in children." I generalised from one working example **without checking
why it worked.** Grok's SCORE names this precisely: *"`:519` is general for types the surface already
ships. It is not general for a new enum that carries a Peer."*

## The other two walls, as reported

I did not reproduce these — they need the strike, which is correctly reverted — but both follow from
wall 1 and grok quotes the checker:

- **Impure + script-level** → constructors unknown in the child, scrutinee an unsolved var, match
  goes `Open`. Consistent with wall 1's "stays in shared memory and never crosses".
- **Worker surface + `Got [peer reply]`** → S4c: *"surface `:fanout::Worker` `:messages` type
  references `:fanout::Seen::Op` which is not declared in this surface's `:messages`"*. Shipping the
  Seen reply on Worker would mean declaring the whole Seen protocol on Worker.

## ⛔ WHAT GROK MEASURED, AND WHY IT IS THE ANSWER RATHER THAN A FALLBACK

```wat
(:wat::core::defenum :fanout::SeenRetry :wat::enum::Pure
  :Got []  :Exhausted [attempts <- :wat::core::i64])
```

Keyword arms ran in the EmptyEnv child; **n=12 completed** —
`total=24;distinct=24;dup=0;check-exhausted=0;mark-exhausted=0;ack-retries=0`. Peer and reply ride
**beside** the outcome as a Tuple. Hash-destructure was not used; row 3's dishonest path was refused.

★★★★★ **The invariant never needed the payload in the variant.** *Exhaustion must be a variant the
match has to name* is satisfied exactly as well by `:Exhausted [attempts]` with the peer alongside.
I put the peer inside the variant out of habit — mirroring the existing `(peer, retry?)` tuples —
and that habit is what collided with the address-space boundary three times.

★ The shape the substrate has been pointing at all along: **the outcome crosses; the resource
stays.**

## Rows

| # | how I checked it | result |
|---|---|---|
| 1 | not reached | — STOP-1 |
| 2 | my own probe reproduced wall 1 verbatim | ⚠ **the form I specified cannot exist** |
| 3 | grok's nullary construction, n=12 green | ✅ **for the correct shape**; ❌ for mine |
| 4–12 | nothing landed; tree restored | — |

★ STOP-1 fired for the third time and each firing cost minutes, not hours, and left the corpus
untouched. That is the trigger working exactly as designed — and grok went past it to **measure a
construction that passes**, which is more than the STOP required.

## THE RE-DRAW

`DESIGN/BRIEF/EXPECTATIONS-the-outcome-crosses-the-resource-stays.md`. Not another design of mine:
it adopts the construction grok has already run green, and states the boundary rule as the reason
so a fourth draft cannot re-invent the same collision.
