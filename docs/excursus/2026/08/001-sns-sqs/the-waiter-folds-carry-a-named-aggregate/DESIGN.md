# DESIGN — the waiter folds carry a named aggregate

**sqs.wat's three waiter-fold accumulators stop being nested Tuples and become one `defstruct` with
named fields.** `wat-scripts/queue/sqs.wat` only.

A **pure refactor**. No behaviour change, no new field, no instrument. Its acceptance test is that
**every number this arc measures is identical**.

## WHY — the language was giving an instruction and the code argued with it

`the queue reports time inside its store` failed to **parse**. Threading one more i64 through the
three waiter folds forced `(Tuple sc ns)` into an already-nested accumulator, and grok reported
*"UnclosedParen with Continue=4; UnexpectedRParen with Continue=5. **No integer works.**"*

The builder's ruling on why:

> *"make a struct or record — **this is on purpose, wide tuples are unwieldy**. structs may hold
> impure bindings, records may only hold pure data bindings (must be edn expressible, a file handle
> or socket is not edn expressable)."*

★ So `Tuple`'s three-element limit is **the push toward a named aggregate**, and `sqs.wat:929` is
the code refusing to take it:

```wat
[st    (:wat::core::first acc)
 inner (:wat::core::second acc)
 keep  (:wat::core::first inner)
 box   (:wat::core::second inner)
 taken (:wat::core::third inner)]
```

`(Tuple :- [Peer (Tuple :- [PersistentVector Vector i64])])` — **the type spelled verbatim twice**
(param and return), destructured by hand, at `:445-453`, `:929-937` and `:1214`.

## ⛔ THE PATTERN IS ALREADY IN THESE FILES, WITH ITS RATIONALE WRITTEN DOWN

**`circuit.wat:1428-1435`** — and read the comment, which is this DESIGN's whole argument:

```wat
;; Named fields so a defservice impl can read them. first/second/third on a
;; Tuple inside :impls typechecks as an unsolved var; a process child also
;; cannot see a defn that is written after the defservice.
(:wat::core::defrecord :fanout::ShareStats
  [calls <- i64  retries <- i64  asleep <- i64  attempts <- i64])
```

★★★ **140 lines later, `circuit.wat:1575` packs the same four numbers** as
`(Tuple (Tuple i64 i64 i64) (Tuple i64 i64))`, type spelled twice. The file contains both the cure
and the disease.

Three more exemplars, all cited:

| exemplar | what it shows |
|---|---|
| `wat/cache.wat:262-272` — `HolographicLru`, a **`defstruct`** | its own comment: *"`defstruct`, not `defrecord`: both fields are live impure handles … a `HolographicLru` can only ever live in a service's `:ephemeral`."* **The precedent for a `Peer`-carrying accumulator.** |
| `wat/query/mem.wat:123` — `MemWrite` (`rows` + `index`) | a **record accumulator threaded through a `foldl`** (`mem.wat:643-650`), rebuilt by name in every branch |
| `wat/grep.wat:161` — `ChildAcc` (`acc` + `idx`) | the same, inside a recursive walk (`grep.wat:230-234`) |

## ⛔ THE ONE CONTRACT DECISION — a `defstruct`, because it carries a `Peer`

```wat
(:wat::core::defstruct :queue::TakeAcc
  [store <- (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])
   keep  <- (:wat::core::PersistentVector :- [:queue::Waiter])
   box   <- (:wat::core::Vector :- [(:wat::service::Directed :- [:queue::Queue::Reply])])
   calls <- :wat::core::i64])
```

**`defstruct`, not `defrecord`** — the accumulator holds a live `Peer`, and a record may hold only
EDN-expressible fields (`ImpureFieldInPureAggregate`, arc 293.W). Probed this session:
`probe-a-fold-accumulator-can-be-a-struct.wat`. A struct never crosses, which is exactly what a
local fold accumulator does.

★★★★ **The point is not the fourth slot. It is that a fifth costs nothing** — a named field changes
no paren depth at any use site, so the edit that killed the last strike becomes ordinary.

## WHAT THIS IS AND IS NOT

⚠ **No behaviour change, and that is the gate.** `store-calls`/pair, `receive-calls`/pair, the
pairs/sec curve at three depths, `distinct`/`dup`, the 38 drop tests, floor 5221 — **every one
identical**. A pure refactor is the rare stone where "nothing moved" is the pass condition.

⚠ **No instrument rides along.** Adding `store-ns` in the same stone would destroy that gate: you
cannot claim every number is identical while adding a field. `store-ns` lands next, and on this
shape it is a one-field edit.

⚠ **`circuit.wat` is untouched.** Its 268-line `-tick` arm and 69 nested-Tuple sites are the larger
problem and its own stone. This one is scoped to the file that actually blocked the strike.

## OUT OF SCOPE — REJECTED

- **`store-ns`.** The next stone; it is what this unblocks.
- **`circuit.wat`'s `-tick` arm** (36 access chains, 33 nested constructions). Own stone, and it has
  a second technique available that this one does not need: `:ephemeral` **function-typed fields**,
  which `sqs.wat:113` and `:158-159` already use — *"process children do not see sibling defns, so
  the body lives here, called via State/take."*
- **Hoisting `none-*` bindings** to shorten the long `Outcome::Continue` tails. Real (circuit 22×,
  sns-fanout 25×, sqs.wat only 6×) and cheap, but a second concern; it would blur the "nothing
  moved" gate.
- **A lint rule for nested Tuples.** Arc 277 owns the linter and is STRIKE-READY. **The builder's
  ruling**, not a side effect of this work.
