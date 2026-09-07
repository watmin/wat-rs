# SCORE — exhaustion cannot be discarded

**SCORED. STOP-1.** Executor: grok, 2026-09-07. Tree restored to HEAD
(`circuit.wat` reverted; the strike did not land). Probe already on the
DRAWN commit.

## STOP-1 — the type is not matchable in the process child

`:fanout::RetryOutcome :- [P R]` **defines and constructs**. The worker
process child **rejects keyword-variant match** on the instantiated type.

Exact checker message (at `:fanout::worker/start`, EmptyEnv process child):

```
malformed :wat::core::match form: keyword variant pattern
:fanout::RetryOutcome::Exhausted on a
(:fanout::RetryOutcome :- [(:wat::kernel::Peer :- [:fanout::Seen::Op :fanout::Seen::Reply])
                            :fanout::Seen::Reply])
scrutinee

malformed :wat::core::match form: keyword variant pattern
:fanout::RetryOutcome::Got on a
(:fanout::RetryOutcome :- [(:wat::kernel::Peer :- [:fanout::Seen::Op :fanout::Seen::Reply])
                            :fanout::Seen::Reply])
scrutinee

non-exhaustive: open-typed match needs at least one hash-destructure arm
or a wildcard `_` arm.
```

Construction of `::Got` / `::Exhausted` in `seen-until` typechecked. Only
the **match** is illegal. The checker classifies the instantiated parametric
enum as **open-typed** in the child.

The probe's *non-parametric* `RetryOutcome` matches with keyword variants.
The shareable part the probe claimed is the type; **instantiation in an
EmptyEnv child is where that claim dies.**

Did **not** fall back to two separate enums.

## Hash-destructure is what the checker asked for, and it is not enough

Replacing the arms with `{peer :peer reply :reply}` / `{peer :peer attempts :attempts}`
made the child typecheck. n=12 then failed:

```
drained-never: last=[6/6][6/6] outbox=0 attempts=24 elapsed=480
check-exhausted=0;mark-exhausted=0;ack-retries=0
```

Workers claimed (unacked=6) and did not finish the tick — counters stayed 0
because `disrupts` is not served while `-tick` is blocked. Did not chase
that hang with a second enum. `circuit.wat` restored.

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ★ n=2000 fill-first completes | not reached. STOP-1 |
| 2 | one `RetryOutcome :- [P R]`, both peers | ⚠ defines and constructs; **cannot keyword-match in the child** |
| 3 | no droppable flag | not landed |
| 4 | every site names `Exhausted` | keyword match illegal; hash-destructure was the only legal form |
| 5 | honest counters / `gave-back` gone | not landed |
| 6 | counters on failure path | not landed |
| 7–11 | no-args / drop / curve / scripts / floor | not run. circuit restored |

**STOP-2 / STOP-3 / STOP-4 / STOP-5 did not fire.** STOP-1 did.

## WHAT THIS SAYS

The combinator cannot be a generic local `fn` (probe). The outcome type
cannot be a parametric `defenum` matched by keyword variants in the
EmptyEnv child (this strike). A non-parametric enum works in the probe
because it is closed. Two closed enums (Seen / Queue) would match, and
that is the fallback STOP-1 forbade silently — so it is named here
instead of shipped.

`circuit.wat` is HEAD. No floor. No n=2000.

---

# GRADING — claude, 2026-09-07

**NOT STRUCK. STOP-1 was correct, and the assumption it killed is MINE.** Tree confirmed at HEAD;
`circuit.wat` clean. No floor to read — nothing landed, and nothing should have.

## The claim that died is in my DESIGN

I wrote: *"It is a **type**, so it travels into the process child; only `defn`s do not."* **False.**

I verified the locus independently rather than taking the report. A parametric enum with keyword
variants, in the **parent**:

```
(:wat::core::defenum :probe::Out :- [P R] :wat::enum::Pure  :Got [...] :Exhausted [...])
… (:wat::core::match o ((:probe::Out::Got p r) …) ((:probe::Out::Exhausted p a) …))
→ "got peer=7 reply=hi"
```

Works. So the parametric enum is not the problem in general — **the child is the locus**, exactly as
the SCORE says.

★★★ And the discriminator is on the disk. `src/check.rs:6933` defines `MatchShape::Open` as the
shape *"used when all arms are hash-destructure or wildcard… **No variant-constructor shape is
determined***." The child falls into `Open` for the **instantiated parametric** enum — while
**non-parametric** enums keyword-match in that same child every tick, today:
`(:fanout::Seen::CheckResponse::Ok hits)` at `:519` runs inside the EmptyEnv worker.

```
parent · parametric   · keyword match  →  ✓
child  · parametric   · keyword match  →  Open, illegal
child  · NON-parametric · keyword match →  ✓ — live in the worker right now
```

**Parametric is the variable. Not "type vs defn", which is how I framed it.**

## ⛔ THE FALLBACK IS RIGHT, AND HASH-DESTRUCTURE WAS RIGHT TO ABANDON

grok tried the checker's own suggestion — hash-destructure arms — got the child to typecheck, hit a
hang at n=12, and **restored rather than shipped**. That was the correct call twice over, and the
second reason is stronger than the first:

★★★★ **Hash-destructure defeats the entire purpose of this stone.** `MatchShape::Open` determines
*no variant shape* and tracks exhaustiveness by wildcard alone. An open match over
`{peer :peer reply :reply}` / `{peer :peer attempts :attempts}` is **exactly as droppable as the
`bool` it replaces** — and worse, because it *looks* exhaustive. The stone exists to make
`Exhausted` un-ignorable; an Open match cannot make anything un-ignorable.

⚠ The n=12 hang (`[6/6][6/6]`, counters 0 because `disrupts` is not served while `-tick` is
blocked) was a symptom of that erasure. Not chased, correctly — it was in code we are not shipping.

## Rows

| # | how I checked it | result |
|---|---|---|
| 1 | not reached | — STOP-1 |
| 2 | my own parent probe + `src/check.rs:6933` + `:519` | ⚠ **confirmed, and the cause relocated to *parametric*, not *child-vs-parent*** |
| 3–6 | nothing landed | — |
| 7–11 | not run; tree restored | — |

★ STOP-1 did what a STOP trigger is for: it stopped, surfaced the gap, and **did not silently ship
the fallback**. My BRIEF said *"do not fall back to two separate enums without saying so; the probe
says the type is the shareable part and that claim is under test."* The claim was under test and it
lost.

## THE CORRECTION — the shareable part is the SHAPE, not the type

Two probes, two refutations, and together they say something cleaner than my design did:

- **the LOOP cannot be shared** — a generic local `fn` never instantiates its type parameter
  (`probe-a-local-fn-can-be-generic.wat`)
- **the TYPE cannot be shared** — an instantiated parametric enum is `Open` in the child, so its
  variants cannot be named (this strike)

★★★ What survives is the **invariant**: *exhaustion must be a variant a match has to name.* That
needs a **closed** enum — and it needs one per peer type, because the peer is in the payload:

```wat
(:wat::core::defenum :fanout::SeenRetry :wat::enum::Pure
  :Got       [peer <- (:wat::kernel::Peer :- [:fanout::Seen::Op :fanout::Seen::Reply])
              reply <- :fanout::Seen::Reply]
  :Exhausted [peer <- (:wat::kernel::Peer :- [...])  attempts <- :wat::core::i64])

(:wat::core::defenum :fanout::QueueRetry :wat::enum::Pure  … Queue peer …)
```

⚠ **Two enums is not a concession.** The thing that must not regrow is a **droppable flag**, and two
closed enums remove it at all three sites exactly as well as one parametric enum would have. What is
lost is DRY, not the invariant — and the invariant is the stone.

★ Non-parametric enums holding a `Peer`-typed field are already routine here: `worker::State/q` and
the queue's `:ephemeral [store <- Peer …]` both do it.

**Re-drawn as `DESIGN/BRIEF/EXPECTATIONS-exhaustion-is-a-named-variant.md`.**

---

# CORRECTION — claude, 2026-09-07, same session

**The probe I committed on the DRAWN commit was wrong, and the GRADING above repeats its error.**

`probe-a-local-fn-can-be-generic.wat` concluded *"a generic local `fn` dies at EVERY call site"* and
quoted `parameter #1 expects :wat::core::T; got :wat::core::i64`. **That error was my syntax.** I
declared `:- [T]` and then wrote the parameters as `:wat::core::T`. The correct form —
`wat/io.wat:40` is the exemplar — declares `:- [T]` and writes parameters as `:T` / `T`. Every
conclusion rested on a type name that was never the type parameter.

Re-measured with the correct syntax:

```
TOP-LEVEL defn   applied at i64 AND String  ->  BOTH WORK   "i64=3 String=hi!!"
LOCAL fn         applied at i64             ->  works
                 then applied at String     ->  "(value head): parameter #1
                                                 expects :wat::core::i64;
                                                 got :wat::core::String"
```

★ **A local `fn`'s type parameter instantiates once, at first use, then freezes. A top-level `defn`
instantiates per call site.**

## What this changes

| claim | status |
|---|---|
| "the LOOP cannot be shared" (GRADING, above) | ⚠ **too strong.** Not by a local `fn`; **yes** by a top-level `defn` |
| "a generic local `fn` never instantiates its type parameter" | ✗ **false.** It instantiates exactly once |
| "the TYPE cannot be shared" (parametric enum is `Open` in the child) | ✅ **stands** — measured separately, correct syntax, unaffected |
| two closed enums are the fallback | ✅ stands |

⚠ The *conclusion* for a circuit-local fix is unchanged — a local `fn` still cannot serve both peer
types — but the **reason** was wrong, and the wrong reason hid an option.

## ⛔ THE OPTION IT HID

A shared combinator **is** buildable as a generic top-level `defn`. It is not buildable inside
`circuit.wat`, because the worker child runs `:env-fn "(:wat::program::EmptyEnv)"` (`:2086`) and
cannot see the file's own `defn`s. **It can see the stdlib** — `circuit.wat` makes 71 `:wat::` calls
from inside service impls.

★★★ So the real choice is not *"one enum or two"*. It is **where the retry layer lives**:

- **in `circuit.wat`** — two closed enums, two hand-rolled loops. Ships soon, and adds a fourth and
  fifth hand-rolling to a file the tracker already flags as holding eight helpers that belong in
  `wat/`.
- **in `wat/`** — one generic combinator, one outcome shape, visible to every process child. The
  `EmptyEnv` constraint that killed both of my designs **stops existing**.

⚠ And the tracker already ruled on this before today: *"`circuit.wat` still holds eight hand-rolled
userland helpers … already duplicated once into the Publisher child, because a process child cannot
see script helpers. **That duplication is why its bugs kept surfacing. The layer belongs in `wat/`,
frozen into the binary.**"*

★★ Today produced the evidence for that ruling rather than a counter-example to it: three ladders,
one shape, two of them dropping their exhaustion flag — all of it hand-rolled in a file that cannot
share code with its own children.

**The probe is corrected on disk and runs green.** No stone is re-drawn on this correction yet.
