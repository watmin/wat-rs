# FINDING — a handler-local dial kills the forked child SILENTLY

**Found 2026-09-13**, while probing `:deadline-ms` for ruling D4. **Builder-directed chase:**
*"you found a process crash?.... and are choosing to use threads?...hrm... sounds like a thing we should chase
hard.... we are going after crashes only in this session.... you induced something we should not find
acceptable..."*

⛔ **He was right and my first move was wrong.** I hit a silent process death and switched the probe to the
thread tier to get past it. That is patching the situation instead of reading the failure, in a session whose
subject is crashes. What follows is the chase.

## The defect, in one sentence

> **A `defservice` handler that `connect`s to a surface NOT declared in its `:peers` is refused by NO
> compile-time check and KILLS the forked child at runtime** — exit **0**, **no stderr**, and the caller
> receives `CallOutcome::Lost` carrying **`LociDiedError::Disconnected []`**, a nullary variant with no cause.

⛔ **CORRECTED 2026-09-13, same day, before any fix was drawn.** My first statement of this said *"a handler
that connects to another service's address"* — with no `:peers` qualifier. **That is overstated, and the
corpus disproves it:** `:fanout::worker` (`circuit.wat:370`) declares
`:peers [:queue::Queue :fanout::Seen]` with both peers in `:ephemeral`, and its `-disrupt` handler
**successfully redials `seen`** — a handler-local dial that works, because the surface is declared and its
forms are therefore spliced into the child.

★ The correction **sharpens the fix rather than weakening the finding**: the missing check is precisely
*"a handler `connect` to a surface absent from `:peers`"*, which the worker satisfies and my probe violated.
So the fix would **not** forbid the legitimate redial shape the corpus relies on.

## The bisection — six variants, one variable at a time

| # | variant | result |
|---|---|---|
| A | handler dials another process's address, `:deadline-ms 300` | ⛔ `outer-Lost cause=Disconnected []` |
| B | **A minus `:deadline-ms`** | ⛔ identical — **the clause is not the killer** |
| C | **handler dials NOTHING**, just replies | ✅ `handler-faced=no-dial` — the child serves fine |
| D | A **plus the capability grant** (`circuit.wat`'s `post-spawn` + `require-granted`) | ⛔ identical — **not a capability gap** |
| E | dial in **`:init`**, peer in `:ephemeral` (the `sqs.wat` shape) | ⛔ **COMPILE-TIME REFUSAL**, with a perfect diagnostic |
| F | A **plus `:peers [:mid::Mid]`** | ⛔ **COMPILE-TIME REFUSAL**, the mirror diagnostic |

⭑ And the handler faced **all four** `ConnectOutcome` arms (`Connected`/`Refused`/`Rejected`/`Failed`) — so this
is **not a missed arm.** The dial does not return at all.

## ⭑⭑ THE WALL EXISTS, IS WELL-DIAGNOSED, AND HAS EXACTLY ONE HOLE

Variants E and F reveal a **bidirectional invariant** the checker already enforces:

```
E  :ephemeral holds a dialed Peer<S::Op,S::Reply>   →  :peers MUST declare S
   "…:ephemeral holds a dialed Peer<mid::Mid::Op,…::Reply> but surface :mid::Mid is not declared
    in :peers — add :peers [… :mid::Mid …] (the explicit s2s dependency DAG)"   wat/service.wat:913

F  :peers declares S                               →  an :ephemeral field MUST hold that Peer
   ":peers declares surface :mid::Mid but no :ephemeral field is typed
    :wat::kernel::Peer<mid::Mid::Op,…::Reply> — add the dialed peer as a root :ephemeral field,
    or drop it from :peers"                                                      wat/service.wat:896
```

★★★ **Those two rules TOGETHER force the init-dial shape** — a service may only dial a surface it declares in
`:peers`, and only by holding the peer in `:ephemeral`, which means dialing in `:init`. That is why
`wat-scripts/queue/sqs.wat` dials the store at `:248`/`:251` **inside `:init`** and threads the peer through
state, and why **the entire corpus does the same and nobody has ever written a handler dial.**

⛔ **A handler-local `connect` participates in NEITHER rule.** It puts nothing in `:ephemeral` and declares
nothing in `:peers`, so neither check has anything to fire on. The declaration is just as missing — and
`:peers [:S]` is what splices `S`'s surface forms into the forked child — so the child is launched without the
types it needs and **dies the moment it dials.**

## Why this is worse than the crashes this campaign already fixed

The raise→value track made failures **matchable**. This one is matchable and **says nothing**:

- `LociDiedError::Disconnected []` is **nullary** — no exit status, no signal, no message, no span.
- A child that raised, a child that was killed, a socket that closed, and *this* all print the identical line.
  ⭑ *Can two different worlds print this line?* Yes — four of them.
- exit **0** and **empty stderr**, so nothing in CI or a floor would notice.
- ⚠ **And I discarded that cause myself** at first (`((CallOutcome::Lost _c) "outer-Lost")`) — the exact defect
  six stones of this campaign removed, reintroduced in my own probe. Printing it is what started the chase.

## The fix shape — a direction, not a design; it reaches `src/`

The checker **already knows how to say the right thing twice.** It simply never looks at a handler body for a
`connect` to a surface-typed `Address`. Extirpare-shaped options, none taken here:

1. ⭑ **Extend the existing pair to a third position:** a `connect` inside an `:impls` body whose `Address` type
   names a surface must require that surface in `:peers` — the same diagnostic, a third call site. **Smallest,
   and it makes the mistake unwritable**, which is where the other two rules already are.
2. **Make the runtime failure informative** — `Disconnected` carries a cause (missing surface / decode failure /
   exit status). Weaker: it converts a silent death into a legible one rather than preventing it. ⚠ But it is
   independently worth doing, because `Disconnected []` is uninformative for **every** cause, not just this one.
3. Reject a surface-typed `Address` as a handler-local `connect` argument outright. Blunt; would forbid shapes
   that may be legitimate.

★ (1) is the one that matches the two rules already in place. (2) is a separate, broader stone and probably
belongs to the raise→value track's tail.

## Reproduce

All six variants are in the session scratchpad; the load-bearing two are trivial to rebuild from this file's
table. The shortest reproduction:

```
a defservice on (:wat::spawn::process), handler calls (:wat::kernel::connect <other service's Address>)
and faces all four ConnectOutcome arms
  → the child dies; the caller sees CallOutcome::Lost (LociDiedError::Disconnected [])
  → exit 0, no stderr
add :peers [:Other]         → compile-time refusal (needs an :ephemeral peer)
add the :ephemeral peer too → compiles, and the dial must then happen in :init
```

⚠ **This is a `docs/` FINDING, not a stone.** The fix reaches `src/check.rs`; **opening it is the builder's
ruling.** Nothing in the tree was changed by this chase.

## Provenance

- Found while probing ruling **D4** (`:deadline-ms`), which remains undrawn.
- Kin: `NOTE-a-callee-deadline-ms-is-inert.md` (same probe session),
  `docs/arc/2026/04/109-kill-std/NOTE-an-outcome-variant-no-primitive-can-construct.md` (a carrier that
  promises more than it delivers — here it is the *payload* that is empty, not the variant that is unreachable).
