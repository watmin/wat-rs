# DESIGN — a peer wait can be bounded too

Builder: *"why do we not have symmetry here?... both sides needs ot be able to time out and recover?"*
→ the census → *"draw 1."*

**Drawn 2026-09-15. NOT STRUCK.**

## WHY — 63 of 67 live waiters cannot bound their wait, and one of them is every service's birth

`every-waiter-can-bound-its-wait` (`e7816e543`, report-only) measured it:

```
waiter-sites=679   yes=15  no=646  UNKNOWN=18
live (wat + wat-scripts/service) = 67 · live bounded = 4
recv 0/259 · send 0/215 · readln 0/161 · accept 0/8 · poll all 14 UNKNOWN
```

⛔ **The headline instance:** the generated `child-main` (`wat/service.wat:3510`) awaits its owner's
startup ship with a **bare `(:wat::kernel::recv ~cm-self-sym)`** and writes a `RecvOutcome::TimedOut`
arm that raises *"the peer is alive and silent"*. **That arm cannot fire** — arc 109's NOTE line 43:
*"a bare `(:wat::kernel::recv peer)` blocks forever and can never return `TimedOut`"*. So **a slow owner
hangs every service at birth**, and the arm that appears to handle it is dead code reading as coverage.

⛔⛔ **And a hang here is worse than a crash.** Measured in anger this session: a blocked bare `recv`
**ignored SIGTERM for 125 seconds**; `timeout 30` never returned and `kill -9` was required. An
unbounded waiter is not merely unbounded — **it is unkillable by the ordinary means.**

## ⭐ The mechanism is NOT missing — it exists once and was never generalised

`call-by-deadline` has raced a `Peer` against a timer `Peer` all along:
**`(:wat::kernel::select [peer tmr])`** (`wat/service.wat:4283`), with the timer's tier chosen by
`peer-wire?` (`:4276`). ⚠ **It is the only site in the corpus that does it.** So this is a **usage** gap,
not a substrate gap — and `recv-by-deadline` cannot help, because it **rejects a `Peer` outright**
(*"expected owner handle ((Thread :- [I O]) | (Process :- [I O])), got :wat::kernel::Peer"*, measured).

Arc 109 named both routes and chose the owner-handle one as *"the smaller surface"* — right for the case
in front of it. The symmetric case was never asked about.

## ⛔ THE ONE CONTRACT DECISION

**Widen `:wat::kernel::recv-by-deadline` to accept a `Peer` as well as a `Thread`/`Process` handle,
returning `RecvOutcome` exactly as it does today.**

⭐⭐ **One name, every kind of thing you can wait on — that IS the symmetry the builder asked for.** And
the return type is the reason this is cheap: **a bare `recv` and `recv-by-deadline` both return
`RecvOutcome`**, so converting a site is *adding an argument*. **No arm changes.** The `TimedOut` arm
already written at every bare-`recv` site stops being dead the moment the primitive is swapped.

⚠ **REJECTED — a `:wat::service::` helper** (e.g. `recv-peer-by-deadline`). Smaller, and it leaves **two
names for one idea**, preserving the asymmetry in the vocabulary while removing it from the capability.
`select`/`after` are both `:wat::kernel::`, so the composition belongs in-tier.

### ⛔ AND THE SECOND DECISION: what is the startup deadline, and who picks it?

Converting `child-main` needs a number. ⚠ **It cannot be a `:deadline-ms` clause** — D4 made that a
**macro error** (`the-inert-clause-is-refused`), and reviving the name would re-open a struck ruling.

**Ruling: a single generous constant in the macro, named and justified in the SCORE.** The startup ship
is sent by the parent immediately after spawn; a wait measured in *tens of seconds* is already
pathological. ⭑ **State the number and why** — an unexplained constant here is the `inbox-cap 64`
mistake again (a chosen value that sat unswept for weeks because nobody wrote down where it came from).

⚠ **What happens on timeout is NOT changed by this stone.** The existing arm raises with *"the peer is
alive and silent"* — which becomes **true and reachable**. So the deliverable is **a silent unkillable
hang becoming a named death.** Whether it should instead *recover* is the supervision ruling the builder
has deferred, and it is not this stone.

## Out of scope = REJECTED

- **The other 258 bare `recv` sites.** The census is report-only and the builder rules the sequence.
  This stone converts **one** site — the generated `child-main` — because it is the only one that is in
  **every service** and it is the headline instance.
- **`send` (215), `readln` (161), `accept` (8), `poll` (14 UNKNOWN).** Same invariant, different
  primitives, each owing its own reading.
- **Making the timeout recoverable.** Supervision; deferred.

## Blast radius

```
src/intrinsic/kernel/message.rs   recv-by-deadline accepts a Peer
src/check.rs                      its inference admits the Peer arity/type
src/runtime.rs                    the Peer path: peer-wire? tier + select [peer tmr]
wat/service.wat                   child-main's bare recv gains a deadline (STDLIB — rebuild)
```

## Trap-doors named up front

1. ⛔⛔ **The timer's tier is the PEER'S, via `peer-wire?` — NOT the env's.** `call-by-deadline`'s own
   comment: *"Env/peer-kind … is UNAVAILABLE from a generated client method (no program env on a
   rust-test thread; wrong tier when the caller is thread and the service is process)."* `select`
   refuses a mixed-tier set, so getting this wrong is a runtime refusal, not a wrong answer.
2. ⚠ **The two `@ret` docs disagree on the variant set** — bare `recv` says *"Message / Closed / Lost /
   Shutdown"*, `recv-by-deadline` says *"… / Stopped / TimedOut"*. `RecvOutcome` has six variants.
   **Verify the real set before claiming a swap needs no arm changes**; that claim is the stone's
   economy and it must be measured, not inferred from two doc comments.
3. ⛔ **`wat/service.wat` is stdlib, frozen into the binary.** Read `wat/fix.wat`'s BOOTSTRAP header.
4. **Do not revive `:deadline-ms`** in any clause — D4 refuses it by macro error.
5. **`cargo build --release` does not compile tests.**
