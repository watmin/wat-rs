# DESIGN — a call outcome cannot lie

**Tighten `call-by-deadline`'s return from a pair to a three-arm enum.**
`wat/service.wat` + `wat-scripts/fanout/circuit.wat`. A pure refactor: every number must be
identical afterwards.

## WHY — one fact carried twice, in a form the tree will copy

`service.wat:3126` returns `(:wat::core::Tuple :- [(:wat::core::Option :- [:O]) :wat::core::i64])`,
with **0 = answer, 1 = peer gone, 2 = deadline**. The pairing is conventional, not enforced:
**`(None, 0)` and `(Some x, 2)` are both writable**, and a caller may consult either half.

**One of the four sites already consults only one half.** `circuit.wat:401`:

```wat
(:wat::core::match (:wat::core::first recv-got)   ;; the code is never read
```

⚠ Not a defect today — that path's fallback redials unconditionally (`:535-542`), so both
non-answer cases reach a correct reconnect. But the discriminator exists *because* collapsing
those cases was wrong (it is what made the executor reject my `Option`-only sketch and pass
STOP-2), and one site in four discards it on day one.

★ **This is stdlib.** Every future service client copies this form. The moment to tighten it is
while there are four call sites, not forty.

## ⛔ THE ONE CONTRACT DECISION

**The reply cannot be obtained without learning why it arrived.**

```wat
(:wat::core::defenum :wat::service::CallOutcome :- [O] :wat::enum::Pure
  :Answered      [reply <- :O]
  :PeerGone      []
  :DeadlineFired [])
```

`(None, 0)` and `(Some x, 2)` then have **no form** — the inconsistent pairs are unrepresentable
rather than merely unwritten. `circuit.wat:401` stops being able to ignore the discriminator,
because there is no longer a half to read.

## PROVEN EXPRESSIBLE — and it is the tree's first

`probe-can-a-defenum-take-a-type-param.wat`, committed first:

```
a=answered=hi;b=peer-lost;c=deadline;parametric-defenum=yes
```

★ `grep "defenum :… :- ["` over `wat/` and `wat-scripts/` returns **nothing**. Every parametric
type in use — `Option`, `RecvOutcome`, `Peer`, `Vector` — is Rust-side. **This is the first
wat-declared parametric enum in the tree**, and the probe exists because "surely defenum takes a
type param" is precisely the assumption that has killed stones this campaign.

## ⚠ ONE COLLAPSE IS DELIBERATELY CARRIED FORWARD, AND NAMED

Code 1 today means **`Lost` OR `Closed`** — two different worlds, already merged by the current
helper. All four callers treat them identically (redial), so the enum keeps the merge rather than
inventing a distinction nobody consumes.

★ **But it is named `PeerGone`, not `PeerLost`.** `PeerLost` would echo `RecvOutcome::Lost`
specifically and quietly claim a precision the value does not have. If a caller ever needs
`Lost` apart from `Closed`, that is a fourth arm — added on purpose, not discovered by someone
trusting the name.

## FILES

`wat/service.wat` — the `defenum` immediately before `call-by-deadline`, **after** the macro so
nothing above `:896` moves — and the helper's return type and four arms.
`wat-scripts/fanout/circuit.wat` — the four call sites.

## OUT OF SCOPE = REJECTED

- **Splitting `Lost` from `Closed`.** Named above; no caller consumes it.
- **Making an undeadlined generated client method unrepresentable** — the standing rung-3 stone,
  still uncut, still needing its own census.
- **All perf work**, the `claim deadline exhausted` crash, and the redelivery fixture that kept
  its name and lost its meaning. All open, none touched here.
