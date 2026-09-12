# DESIGN — the owner wait has a deadline

Builder: *"run that probe and draw it"* — after choosing the deadline-bearing recv over the
capability load-order note.

**Drawn 2026-09-12. NOT STRUCK.** Stone 4 of `a-momentary-failure-is-not-fatal`, and the last item on
the "replace a raise with a value the caller can match" track.

## Why

All four owner methods now funnel through **one** `:wat::service::owner-recv-loop`
(`wat/service.wat:3811`), whose bound is a wall clock. Its `TimedOut` arm **cannot fire**:
`RecvOutcome::TimedOut` is never constructed as a value in Rust — only `call-by-deadline` mints one, by
racing a peer against `:wat::kernel::after` in `select`. So a bare `recv` blocks, `GaveUp` is reachable
only while the peer keeps *emitting*, and **a silent service hangs the owner forever.**

★ This is the best possible moment for it: before stone 3 the defect would have needed fixing per
method; now one loop serves `stop` / `hibernate` / `grant` / `revoke`, so one fix closes all four.

Tracked at `docs/arc/2026/04/109-kill-std/NOTE-an-outcome-variant-no-primitive-can-construct.md`.

## ⛔⛔ THE PROBE REFUTED THE STATED BLOCKER — READ THIS BEFORE DESIGNING ANYTHING

The NOTE and the previous SCORE both record the blocker as: *`select` refuses to mix an owner handle
with `after`'s `Peer` — "peers[1] has wrong tier (expected Process)"*. That was **grok's report, and
the orchestrator had not verified it.** A probe was run (2026-09-12,
`/tmp/…/scratchpad/probe-owner-select.wat`, kept out of `wat-scripts/` because it does not type-check —
the load gate would go red).

**What the probe found, in order:**

1. The first error is **not about tiers at all.** It is a type-parameter mismatch:

```
:wat::core::vec: parameter #3
  expects (Peer :- [<S>::Admin (<S>::Status :- [?])])
  got     (Peer :- [Never <S>::Admin])
```

The lineage handle is `Peer<Admin, Status>` — it **sends** `Admin`, **receives** `Status`. `after`
yields `Peer<Never, T>` — it sends nothing. The two cannot unify, and `select`'s vector must be
homogeneous.

2. ⭑ **`call-by-deadline` already solves exactly this, in wat, with no new primitive.** It does not
   pass `after`'s result directly — it `conj`s it into a vector whose **element type it declares**:

```
tmr (first (conj (:wat::core::Vector :- [(:wat::kernel::Peer :- [:I :O])])
                 (:wat::kernel::after kind (Milliseconds ms) inert)))
```

The declared element type is what coerces `Peer<Never,T>` into `Peer<I,O>`.

3. **Applying that same coercion to the owner handle REMOVED the select/timer type error.** The probe
   then stopped on a *different* thing entirely: `peer-wire?` received an unresolved type variable
   (`got :?1383`), because `(<S>::Handle/handle h)` leaves the `Handle`'s transport marker `T`
   (`Shared | Wire`, arc 293.W.2f) un-pinned in a standalone probe.

**So the gap is not "select cannot mix tiers, we need a new primitive."** The gap is **type-parameter
plumbing**: pinning the lineage peer's `S`/`R` and the `Handle`'s transport marker so the same
coercion `call-by-deadline` uses can be written inside the generated owner methods.

⚠ **What the probe did NOT establish:** that the coerced `select` *succeeds at runtime*. The type error
is gone; the runtime tier check grok reported has **not** been re-reached, because the probe stopped
earlier on the marker. **That is STOP-1**, and it must be settled before any `src/` work — the whole
size of this stone depends on it.

## The two candidate shapes, and which the probe favours

**(A) wat-level — `owner-recv-loop` gains a deadline via `select`.** Race the lineage peer against a
matching-tier `after`, using the vector-element coercion. The generated methods already know their
`Admin`/`Status` types (the macro splices `~admin-stop-kw`, `~status-stopped-kw`), so the types the
probe could not pin from outside **are in scope inside the macro.** No new Rust. `TimedOut` becomes
reachable exactly as it is for `call-by-deadline`.

**(B) Rust-level — a `recv-by-deadline` primitive** for owner handles that constructs `TimedOut`
itself, making the variant primitive-reachable and repairing the doctrine in
`NOTE-io-boundary-outcome-enum.md`.

⭑ **The probe favours (A)**, and that inverts the NOTE's guess. (A) is smaller, needs no new
constructor, and reuses a mechanism already proven in this file. **(B) remains the right answer if and
only if STOP-1 shows the runtime tier check genuinely refuses an owner handle** — in which case (A) is
impossible and the NOTE's original framing was right after all.

## The one contract decision

**The deadline is a parameter with a default, not a constant.** `owner-recv-loop` already takes
`budget-ms` (10000 at both call sites). Whatever shape lands must keep the budget a *caller-visible
argument*, because a 10 s wall on `stop` is a policy an owner may need to change, and burying it is how
`vis` came to do two jobs earlier in this campaign.

## Out of scope = REJECTED

- **The capability load-order defect.** Noted at
  `docs/arc/2026/04/109-kill-std/NOTE-a-surface-cannot-see-a-type-declared-after-it.md`; measured at
  **zero userland callers**; the builder sequenced it after this.
- **Making the serve loop's ack retryable** (`the-gate-methods-face-an-outcome`'s named follow-up).
- **The 61 live `Malformed` placeholder arms.** A separate stone; this one touches the owner wait only.
- **Changing `StopOutcome` / `GateOutcome` shapes.** They are correct; only reachability changes.

## Trap-doors named up front

1. ⛔ **The blocker in the NOTE is refuted-but-not-replaced.** STOP-1 exists because the runtime check
   has not been re-reached. Do not design (B) before disproving (A).
2. **`Peer` is `<S,R>` — send-type FIRST.** This session has already read that backwards once and drew
   a wrong conclusion from it.
3. **Two tier predicates disagree by design:** `peer-wire?` (what `call-by-deadline` uses) and
   `peer-process` (what `grant-method-body` uses). Pick deliberately and say which.
4. **`Handle` is `(Handle :- [T])`** with a `Shared | Wire` transport marker — the thing that stopped
   the probe. Inside the macro it is in scope; from outside it is not.
5. **Three carriers of wat source**, and **`cargo build --release` does not compile tests.**
