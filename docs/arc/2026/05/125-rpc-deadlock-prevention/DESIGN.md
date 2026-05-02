# Arc 125 — RPC deadlock prevention (TYPE-PRECISE) — **WITHDRAWN**

**Status:** **withdrawn 2026-05-01 in favor of arc 126.** This DESIGN
is the honest record of the proposal that was considered and overruled.
The substrate's discipline: rejected proposals stay; sequential numbering;
no v1/v2.

## What this arc proposed

A compile-time check sibling to arc 117's `ScopeDeadlock`. The rule:

> At every `:wat::core::let*` binding-block (or function-call site), if
> a binding (or argument) of type `Sender<T>` is sibling/co-located with
> a binding (or argument) of type `Receiver<T>` for the SAME T after
> alias expansion, fire `CheckError::RpcDeadlock`.

Implementation would have mirrored arc 117's structure:
- `walk_for_rpc_deadlock` recursive walker
- `check_let_star_for_rpc_deadlock` per-let* check
- `type_is_receiver_kind` helper (mirror of `type_is_thread_kind`)
- Reuse `type_contains_sender_kind`
- Match on T-equality after `expand_alias`

Estimated ~150 LOC.

## Why it was considered

Empirically, all 5 of the arc-119-surfaced deadlock cases use the
same shape: caller binds both `(ack-tx :PutAckTx)` and
`(ack-rx :PutAckRx)`, then calls a helper verb passing both. After
alias expansion both resolve to `Sender<unit>` + `Receiver<unit>`
with the same T. A type-precise rule would fire on every case.

The mechanism (mirror of arc 117) is small. It catches what we have.
Ship it.

## Why it was overruled

Following the four-questions discipline (`feedback_four_questions`):

**Obvious?** Partially. "Same T → same channel" is a heuristic, not a
truth. Two genuinely independent `make-bounded-channel<unit>` calls
both produce `Sender<unit>` + `Receiver<unit>` of matching T. A
function legitimately taking one end of channel A and one end of
channel B (both unit-typed) trips the rule falsely. The diagnostic
would say "same channel" when the data says "different channels." The
code would have to lie about the structural truth or ship with a
known-imprecise rule. Neither is obvious; both are smells.

**Simple?** Yes — small mechanism, mirrors existing arc 117. But
"simple AND wrong" is a worse axis than "slightly bigger AND right."

**Honest?** No — and this was the load-bearing failure. Wat is a lisp;
the type registry is data; the AST is data. Same T is the type
system's loose proxy for "same channel pair." The actual structural
truth — same `make-bounded-channel` call as the pair-anchor —
is also data, also walkable, also exact. Choosing the proxy over the
truth when both are accessible is settling for less precision than
the substrate's data permits. The substrate-as-teacher discipline
demands the diagnostic name the structural truth, not a heuristic
approximation.

**Good UX?** Worse than the alternative. False positives erode trust
in the diagnostic. Authors faced with "same T, different channel"
false positives would either disable the check, work around it with
typealiases-to-distinguish-channels-of-same-T (cargo culting), or
chase the diagnostic into a rabbit hole. A rule that's right by
construction has none of those failure modes.

## What overruled it

**Arc 126** — `channel-pair-deadlock-prevention`. Same mechanism class
(structural AST walk + binding lookup; same toolkit arc 117 already
uses), one rule arm different: trace each `Sender<T>` / `Receiver<T>`
back to its **pair-anchor** — the `(make-bounded-channel ...)` call
that originated it via the binding chain `tx ← (first pair) ← (make-bounded-channel ...)`.
Two args (or sibling bindings) with the SAME pair-anchor means same
channel; only THEN does the rule fire.

This is the type system's loose proxy made strict. Arc 117 already
walks `(closure-captured rx) → (second pair) → (make-bounded-channel
...)`. Arc 126 is the same trace, applied at call sites instead of
spawn-thread closure boundaries. The mechanism is proven; the rule
arm is new.

Estimated incremental cost over what arc 125 would have shipped: ~50
LOC (the binding-trace walker is the load-bearing addition; the rest
mirrors arc 117). The honesty gain: structural truth, zero false
positives by construction, diagnostic that names what's actually
wrong.

## What this arc preserves as durable record

- The four questions kill heuristics that have a strict alternative.
- Type-precision via `expand_alias` is a tool worth knowing — it
  passed the obvious + simple bars, failed honest + good-UX. It
  remains in the arc-117 toolbox and may anchor future rules where
  type-level structural matching IS the truth (not a proxy).
- Substrate work prefers structural-truth diagnostics over
  type-system-proxy diagnostics when both are accessible. The data
  is the source of truth. Wat is a lisp.

## Cross-references

- `docs/arc/2026/05/126-channel-pair-deadlock-prevention/DESIGN.md`
  — the arc that ships.
- `docs/arc/2026/04/117-scope-deadlock-prevention/DESIGN.md` — the
  structural-truth precedent; arc 126 reuses its trace machinery.
- `docs/arc/2026/04/119-holon-lru-put-ack/DESIGN.md` — the protocol
  fix that surfaced the deadlock class arc 126 catches.
- Memory: `feedback_four_questions.md` — the discipline that
  killed this arc.
- Memory: `feedback_proposal_process.md` — rejected proposals stay
  as honest record. No v1/v2. Sequential numbers.
