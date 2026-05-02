# Arc 126 — Channel-pair deadlock prevention

**Status:** **shipped + closed 2026-05-01.** Three slices over
the same day (slice 1: substrate check; slice 2: 6-test
`:ignore` → `:should-panic` conversion; slice 3: this
INSCRIPTION + cross-references). Surfaced two substrate gaps
along the way (arc 128 — sandbox boundary; arc 129 — Timeout
vs Disconnected). See `INSCRIPTION.md` for the close-out
summary + four questions + failure-engineering record. DESIGN
below is the as-drafted record kept verbatim.

## Provenance

Arc 119's "Pattern B Put-ack helper-verb cycle" deadlock surfaced 6
test sites across 2 cache crates after arc 124 made them visible:

- `wat-lru`: `test-cache-service-put-then-get-round-trip`
- `wat-holon-lru`: `HologramCacheService::test-step3..test-step6`
- `wat-holon-lru/proofs/arc-119`: `step-B-single-put` (minimal repro)

All 6 share one structural shape: a let* binding-block where both
halves of one `make-bounded-channel<T>` pair are bound to names AND
both names are passed to a single function call.

```scheme
(:wat::core::let*
  (((ack-pair :PutAckChannel)
    (:wat::kernel::make-bounded-channel :wat::core::unit 1))
   ((ack-tx :PutAckTx) (:wat::core::first ack-pair))
   ((ack-rx :PutAckRx) (:wat::core::second ack-pair))
   ...
   ((_ :wat::core::unit)
    (:wat::holon::lru::HologramCacheService/put req-tx ack-tx ack-rx ...)))
  ...)
```

Both `ack-tx` and `ack-rx` are alive at the moment the helper verb
internally `recv`s on `ack-rx`. The helper's recv has 2 writers: the
driver's clone (sent in via `Request::Put`) and the caller's clone
(still in scope). If the driver dies before sending the ack, the
caller's clone keeps the channel alive; recv never sees EOF;
deadlock.

The substrate compiles the program. The runtime hangs. Arc 117's
existing `ScopeDeadlock` doesn't catch it — that rule fires at
`Thread/join-result` sites, but the deadlocked caller never reaches
join-result because it's stuck on the recv first.

Arc 119 marked the 6 tests `:ignore` with a 200ms `:time-limit`
safety net for visibility. Arc 126 makes the structural shape a
type-check-time error so future authors can't write it.

## The rule

> At every `:wat::core::let*` binding-block, walk every function-call
> in the let*'s extent. For each call, examine each argument that
> resolves to `Sender<T>` or `Receiver<T>` (after `expand_alias`).
> Trace each such argument back to its **pair-anchor** — the
> originating `:wat::kernel::make-bounded-channel` /
> `make-unbounded-channel` call, via the binding chain
> `name → (first|second pair-name) → make-bounded-channel ...`.
> If any two arguments to a single call share the SAME pair-anchor,
> fire `CheckError::ChannelPairDeadlock`.

The rule is structural truth, not type-system proxy. Same pair-anchor
IS same channel by construction. Two genuinely-independent channels
of identical T have distinct pair-anchors and pass through.

## The shape

**Anti-pattern (rejected):**

```scheme
(:wat::core::let*
  (...
   ((pair :Channel<T>) (make-bounded-channel ...))    ;; pair-anchor
   ((tx :Sender<T>) (first pair))                      ;; traces to pair
   ((rx :Receiver<T>) (second pair))                   ;; traces to pair
   ...
   ((_ :unit) (helper-verb tx rx ...)))                ;; ← ERROR: same anchor
  ...)
```

**Canonical (allowed):**

```scheme
;; Two different channels — pre-allocated via HandlePool (the substrate's
;; pair-by-index discipline; see ZERO-MUTEX.md § "Routing acks").
(:wat::core::let*
  (((handle :Service::Handle)                   ;; (ReqTx<Req>, AckRx<unit>)
    (:wat::kernel::HandlePool/pop pool))
   ((req-tx :ReqTx<Req>) (:wat::core::first handle))
   ((ack-rx :AckRx<unit>) (:wat::core::second handle))
   ;; req-tx's anchor: the Request channel created by Service/spawn.
   ;; ack-rx's anchor: the SEPARATE Ack channel created by Service/spawn.
   ;; Distinct anchors → distinct channels → no deadlock.
   ((_ :unit) (helper-verb req-tx ack-rx ...)))
  ...)
```

## The diagnostic

```
channel-pair-deadlock at <span>: function call 'helper-verb' receives
two halves of the same channel pair. Argument 'tx' is a Sender<T>
and argument 'rx' is a Receiver<T>; both trace back to the
make-bounded-channel allocation at 'pair' (let* binding above).
Holding both ends of one channel in a single role guarantees the
recv never sees EOF — the caller's writer keeps the channel alive
even when the receiver should disconnect.

Fix options (per ZERO-MUTEX.md § "Routing acks"):
  1. Pair-by-index via HandlePool — pre-allocate (Tx, AckRx) pairs;
     each producer pops one Handle holding ONE end of EACH of two
     distinct channels.
  2. Embedded reply-tx in payload — caller does NOT bind the
     reply-tx; project the Sender directly into the Request without
     naming it. (Note: this is only safe when the substrate confirms
     no clone of the projected Sender remains in scope; today's
     Arc-clone-on-projection model means this is non-trivial.)

The structural rule the substrate enforces: no single function call
passes both halves of one make-bounded-channel pair.
```

The diagnostic mirrors arc 117's pre/post block style, names the
pair-anchor explicitly, and points to the canonical-fix doctrine.

## Detection algorithm

The check runs after type inference, walking the AST.

1. **Locate every function-call site.** A `:wat::core::let*` binding
   RHS, body form, or any nested call-form whose head is a
   user-defined function. (Skip kernel comm primitives — `send`,
   `recv`, `select`, `try-recv` — those are governed by arc 117.)

2. **For each call, classify arguments.** For each argument
   expression:
   - Resolve the argument's TYPE via the type registry (already
     computed by check_program). After `expand_alias`, the type
     is one of:
     - `Sender<T>` / `Receiver<T>` — eligible for pair-anchor tracing
     - Any other — skip
   - Resolve the argument's VALUE via AST shape:
     - `WatAST::Symbol(name)` — bound name; trace via let* lookup
     - `(:wat::core::first <expr>)` / `(:wat::core::second <expr>)`
       — projection; trace `<expr>` recursively
     - Any other shape — pair-anchor is "unknown" (skip; conservative)

3. **Trace name to pair-anchor.** Given a binding name, find the
   surrounding let* binding for that name. If the binding's RHS is:
   - `(:wat::core::first <pair-name>)` or `(:wat::core::second <pair-name>)` —
     recurse: trace `<pair-name>` to its anchor.
   - `(:wat::kernel::make-bounded-channel ...)` or
     `(:wat::kernel::make-unbounded-channel ...)` — this binding IS
     the pair-anchor. Return its (binding-name, span).
   - Any other shape — pair-anchor unknown; skip.

4. **Compare pair-anchors.** Group call arguments by their resolved
   pair-anchor binding-name. If any pair-anchor has 2+ arguments
   mapped to it, fire `CheckError::ChannelPairDeadlock`.

5. **Issue the diagnostic** with both argument names + the
   pair-anchor's binding name and span.

The trace machinery is the same arc 117 already uses for closure-
captured Receivers — `(captured rx) → (second pair-name) →
(make-bounded-channel ...)`. Arc 126 applies the trace at call-site
arguments instead of spawn-thread closure bodies.

## False-negative caveats (preferred over false-positives)

- **Multi-step rx/tx derivations skipped.** A binding like
  `((rx2) (some-helper rx1))` doesn't trace through `some-helper`.
  Future arc widens.
- **Tuple-typealias unpacks skipped.** A user typealias hiding
  Sender + Receiver behind a struct field isn't traced past the
  user-named type. Future arc.
- **Cross-function tracing skipped.** A pair allocated in caller A,
  passed into helper B, then B passes both halves to call C — A's
  let* has the pair, but the trace from C's args back to A's anchor
  crosses a function boundary. Future arc (would need cross-function
  symbol-table tracing).
- **Type-only arguments skipped.** If an argument's type is not
  visible to the AST walker (inferred-only, not annotated), the
  trace skips it. Annotation discipline already strong; rare.

The four caveats document themselves in the rule's prose. False
negatives are acceptable; false positives are not.

## Diagnostic substring lock

Slice 2 converts the 6 `:ignore`d test sites to `:should-panic
(expected = "<substring>")` annotations. Cargo's libtest matches
the panic message by substring. The Display impl MUST emit a
panic message containing the literal substring:

```
channel-pair-deadlock
```

(lowercase, hyphenated, single identifier — same convention as
arc 117's "scope-deadlock" and arc 110's "kernel-comm-out-of-position").

This substring is the LOAD-BEARING contract between slice 1 and
slice 2. Slice 1's Display MUST emit it; slice 2's
`:should-panic` annotations match against it. Divergent phrasing
breaks the verification chain.

The recommended Display header:

```
channel-pair-deadlock at <span>: function call '<callee>' receives
two halves of the same channel pair. ...
```

Mirrors arc 117's `scope-deadlock at <span>: Thread/join-result on
'<thread_binding>' would block forever. ...` shape.

## CheckError variant

```rust
/// Arc 126 — a function call passes two arguments that trace
/// back to the same `:wat::kernel::make-bounded-channel` /
/// `make-unbounded-channel` pair-anchor. One argument is a
/// Sender<T>; the other is a Receiver<T>; both are halves of one
/// channel. Holding both ends in one role deadlocks any recv on
/// the Receiver — the caller's Sender clone keeps the channel
/// alive even if the receiving thread dies.
ChannelPairDeadlock {
    /// Name of the function being called.
    callee: String,
    /// Name of the Sender<T>-typed argument.
    sender_arg: String,
    /// Name of the Receiver<T>-typed argument.
    receiver_arg: String,
    /// Name of the let* binding that held the pair-anchor
    /// (`(:wat::kernel::make-bounded-channel ...)` RHS).
    pair_anchor: String,
    /// Source location of the function-call site.
    span: Span,
}
```

## Implementation

### Slice 1 — the check

`src/check.rs` adds:

- `CheckError::ChannelPairDeadlock { ... }` — variant + Display
- `validate_channel_pair_deadlock(node, types, errors)` — entry
- `walk_for_pair_deadlock(node, types, errors)` — recursive walker
- `check_call_for_pair_deadlock(call_form, scope, types, errors)` —
  per-call-site check
- `trace_to_pair_anchor(name, scope) -> Option<(anchor_name, span)>` —
  the trace function (mirrors arc 117's binding-chain walk)
- `type_is_sender_or_receiver_kind(ty, types) -> Option<Direction>` —
  classifier returning Sender / Receiver / None

Integrate into `check_program` after `validate_scope_deadlock`.

### Slice 2 — verification + sweep

- Run `cargo test --release --workspace`. The 6 ignored tests should
  fail at type-check time with `ChannelPairDeadlock` diagnostics
  instead of timing out at runtime.
- Convert the 6 sites' annotations from `:ignore` to
  `:should-panic` matching the diagnostic's substring (per arc 122).
  This proves the rule fires structurally — the test BODY trips the
  check, the test panics, cargo test reports it as expected
  (should-panic matched).
- Hand-craft a deliberately-broken probe (the anti-pattern from §
  "The shape"). Confirm `wat --check` rejects with the canonical
  diagnostic.
- Sweep substrate: confirm no substrate-shipped wat file trips the
  check (it shouldn't — Console uses pair-by-index from HandlePool;
  CacheService's helpers should be reshapable to the same).

### Slice 3 — closure

- INSCRIPTION + USER-GUIDE update (new "Common gotcha" entry +
  cross-ref to ZERO-MUTEX.md § "Routing acks").
- WAT-CHEATSHEET adds § "Channel-pair-deadlock rule".
- Cross-reference arc 119 (the protocol context) and arc 117 (the
  sibling rule).

## Why this matches the substrate's discipline

- **Wat is a lisp; data is the source of truth.** The pair-anchor
  is structural — visible in the AST as a `make-bounded-channel`
  call. The trace walks data, not heuristics.
- **Substrate-as-teacher.** The diagnostic names the structural
  truth: "two halves of one channel" and the canonical-fix
  patterns from ZERO-MUTEX.md.
- **Mini-TCP discipline (ZERO-MUTEX § "Routing acks").** Pair-by-
  index via HandlePool is the substrate's pre-allocated channel
  shape; arc 126 makes diverging from it a compile error when the
  divergence is the deadlock-class shape.
- **Zero false positives.** Same pair-anchor IS same channel.
  Different anchors = different channels. The rule cannot fire on
  legitimate two-different-channels-of-same-T patterns.

## The four questions

**Obvious?** Yes. Same `make-bounded-channel` call → same channel;
both ends in one role → deadlock. The trace walks data; the rule
fires on structural truth.

**Simple?** Yes. ~200 LOC. Reuses arc 117's trace mechanism. Single
new CheckError variant. Surface-level matching after `expand_alias`.

**Honest?** Yes. The rule names the load-bearing structural
invariant — same pair-anchor — not a heuristic approximation. The
diagnostic cites ZERO-MUTEX.md § "Routing acks" as the canonical-fix
doctrine. The substrate's existing pair-by-index pattern (Console)
is already correct; the rule enforces it for new code.

**Good UX?** Phenomenal. The 6 ignored tests fail at type-check time
with a clear diagnostic naming the pair-anchor binding and the
canonical-fix patterns. Future authors writing the same shape get
the same diagnostic at freeze time, before runtime hang. The
substrate teaches the right shape; ZERO-MUTEX.md becomes the
why-doc rather than the how-doc.

## Cross-references

- `docs/ZERO-MUTEX.md` § "Mini-TCP via paired channels" + § "Routing
  acks" — the doctrine this rule enforces.
- `docs/SERVICE-PROGRAMS.md` § "The lockstep" — the broader
  framework arc 117 + arc 126 jointly support.
- `docs/SUBSTRATE-AS-TEACHER.md` — the migration discipline the
  diagnostic follows.
- `docs/arc/2026/04/117-scope-deadlock-prevention/INSCRIPTION.md` —
  the precedent. Same trace machinery, different rule arm.
- `docs/arc/2026/04/119-holon-lru-put-ack/DESIGN.md` — the
  protocol context. Pattern B's "embedded reply-tx in payload"
  routing demands the trace; arc 126 makes the discipline
  enforceable.
- `docs/arc/2026/05/125-rpc-deadlock-prevention/DESIGN.md` — the
  withdrawn type-precise sibling. Honest record of why
  type-system-proxy was rejected for structural truth.
- `crates/wat-holon-lru/wat-tests/holon/lru/HologramCacheService.wat`
  — the 4 step3-6 tests; arc 126 makes them fail loud at parse
  time.
- `crates/wat-holon-lru/wat-tests/proofs/arc-119/step-B-single-put.wat`
  — the minimal reproduction.
- `crates/wat-lru/wat-tests/lru/CacheService.wat` — the wat-lru
  cache-service round-trip test.

## Sequencing

After arc 126 ships:

1. The 6 `:ignore`d tests trip the check at freeze time. Convert
   their annotations to `:should-panic` (matching the diagnostic
   substring). The tests then PASS — the panic IS the proof of
   the check working.
2. Arc 119 step 7 (the consumer sweep) becomes the SUBSTRATE
   reshape: rewrite the helper-verb signatures so they use
   pair-by-index from HandlePool, eliminating the need for caller-
   allocated ack channels entirely. Sonnet sweep guided by the
   diagnostic stream.
3. Arc 119 closure: INSCRIPTION + 058 row + INVENTORY § K mark.
4. The deadlock CLASS retires from the substrate.
