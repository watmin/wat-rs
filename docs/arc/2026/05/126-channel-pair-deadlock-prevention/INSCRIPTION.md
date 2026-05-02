# Arc 126 — INSCRIPTION

## Status

**Shipped + closed 2026-05-01.** Three slices over the same
day:

- **Slice 1** — the type-check-time rule + 5 unit tests in
  `src/check.rs`. Two sonnet sweeps: first sweep scored 5/6
  hard rows (surfaced arc 128); reland scored 14/14 hard +
  soft. Commit `2b6d053`.
- **Slice 2** — converted 6 deadlock-class tests from
  `:ignore` to `:should-panic("channel-pair-deadlock")` across
  3 wat-test files. First sweep scored 6/8 hard (surfaced
  arc 129); after arc 129 shipped, slice 2 landed cleanly.
  Commit `3ab8700`.
- **Slice 3** — this INSCRIPTION + cross-references. (USER-
  GUIDE / WAT-CHEATSHEET / ZERO-MUTEX additions land
  alongside this commit.)

The arc surfaced two substrate gaps (arc 128, arc 129) along
the way. Both shipped cleanly. The arc 126 chain is the
reference example for the failure-engineering +
artifacts-as-teaching disciplines (see `REALIZATIONS.md`).

## What this arc adds

A type-check-time rule that makes the Pattern B "embedded
reply-tx in payload" discipline structural. Before arc 126,
the discipline was carried by humans: read `ZERO-MUTEX.md`
§ "Routing acks", recognize when caller-bound `(ack-tx,
ack-rx)` from `make-bounded-channel` would deadlock the helper
verb's recv. After arc 126, the substrate enforces it at
freeze time with a self-describing diagnostic.

## The rule

> At every function-call site, walk every argument expression.
> For each argument whose alias-resolved type is `Sender<T>`
> or `Receiver<T>`, trace the value back to its **pair-anchor**
> — the originating `:wat::kernel::make-bounded-channel` /
> `make-unbounded-channel` call — via the binding chain
> `name → (first|second pair-name) → make-bounded-channel ...`.
> If any two arguments to a single call share the SAME
> pair-anchor (one as Sender, one as Receiver), fire
> `CheckError::ChannelPairDeadlock`.

The rule is **structural truth, not type-system proxy**. Same
pair-anchor IS same channel by construction. Two
genuinely-independent channels of identical T have distinct
pair-anchors and pass through. Slice 1's `parse_binding_for_pair_check`
walks the AST chain; arc 117's existing trace machinery is
mirrored at call-site arguments instead of spawn-thread
closure boundaries.

## The diagnostic

```
channel-pair-deadlock at <span>: function call '<callee>'
receives two halves of the same channel pair. Argument
'<sender_arg>' is a Sender<T> and argument '<receiver_arg>' is
a Receiver<T>; both trace back to the make-bounded-channel
allocation at '<pair_anchor>' (let* binding above). Holding
both ends of one channel in one role deadlocks any recv —
the caller's writer keeps the channel alive even when the
receiving thread dies.

Fix options (per ZERO-MUTEX.md § "Routing acks"):
  1. Pair-by-index via HandlePool — each producer pops one
     Handle holding ONE end of EACH of two distinct channels.
  2. Embedded reply-tx in payload — caller does not bind the
     reply-tx; project the Sender directly into the Request.
```

The substring `channel-pair-deadlock` is **load-bearing**: slice
2's `:should-panic` annotations match against it via cargo
libtest's substring search. Arc 129's wrapper fix preserves it
verbatim through the panic chain.

## Detection algorithm

Runs as part of `check_program`'s structural walks, sibling to
arc 117's `validate_scope_deadlock`:

1. **Walk every form.** `walk_for_pair_deadlock` recurses into
   all `WatAST::List` nodes, accumulating let* binding-scope
   as it descends.
2. **Sandbox-boundary respected (arc 128).** When the walker
   hits a `:wat::kernel::run-sandboxed-ast` /
   `run-sandboxed-hermetic-ast` / `fork-program-ast` /
   `spawn-program-ast` call, it skips the first argument (the
   forms-block representing an inner program). Inner forms
   are checked at their own freeze cycle, not at outer freeze.
3. **At each function-call site.** `check_call_for_pair_deadlock`
   walks the call's arguments. For each argument, if its
   resolved type is Sender-kind or Receiver-kind, trace to
   its pair-anchor.
4. **Trace chain.** `trace_to_pair_anchor` walks back through
   the binding-scope: a name binds to `(first <inner>)` or
   `(second <inner>)` projection → recurse on `<inner>`. A
   name binding to `(make-bounded-channel ...)` IS the
   pair-anchor. Returns `(name, span)`.
5. **Group by anchor.** If any pair-anchor has both a Sender
   argument AND a Receiver argument, emit
   `CheckError::ChannelPairDeadlock` with the callee name,
   both argument names, and the pair-anchor's binding name.

## Why this rule, why now

Arc 119 surfaced a substrate-discipline gap during the
HologramCacheService migration: the helper verb signature
took both `(ack-tx, ack-rx)` from a caller-allocated
make-bounded-channel pair, and the caller's let* held both
halves alive across the helper's internal recv. The recv had
two writers (caller + driver clone in Request); driver
panicked → driver clone dropped → caller's clone kept the
channel alive → recv hung forever.

Six tests across two crates exhibited the pattern; pre-arc-124
the tests were silently undiscoverable (proc macro scanner
gap, fixed by arc 124); post-arc-124 they showed up but hung
or timed out. Arc 126 makes the deadlock-class shape a
freeze-time error.

The rule's **structural enforcement** means future authors
writing the same shape get a clean diagnostic at freeze time,
before runtime hang. The substrate teaches the right shape
(pair-by-index via HandlePool, OR embedded reply-tx without
binding); ZERO-MUTEX.md becomes the why-doc rather than the
how-doc.

## Limitations

False-negatives are acceptable; false-positives are not. The
rule errs on the side of staying silent when:

- **Multi-step rx derivations skipped.** A binding like
  `((rx2) (some-helper rx1))` doesn't trace through
  `some-helper`. Future arc widens.
- **Tuple-typealias unpacks skipped.** A user typealias
  hiding Sender + Receiver behind a struct field isn't
  traced past the user-named type.
- **Cross-function tracing skipped.** A pair allocated in
  caller A, passed into helper B, then B passes both halves
  to call C — A's let* has the pair, but the trace from C's
  args back to A's anchor crosses a function boundary.
  Future arc (cross-function symbol-table tracing).
- **Type-only arguments skipped.** If an argument's type is
  not visible to the AST walker, the trace skips it.

## What this arc closes

- The substrate-discipline gap surfaced by arc 119:
  channel-pair-deadlock as compile-time error rather than
  runtime hang.
- Arc 119 step 7's "consumer sweep" — the 6 tests that
  exercised the pattern (across 2 crates + 1 proofs dir)
  are now structurally validated as deadlock-class via
  `:should-panic("channel-pair-deadlock")`.
- The "discipline as prose" gap. ZERO-MUTEX.md § "Routing
  acks" was the discipline carrier; arc 126 makes it
  structural.
- Two latent substrate bugs along the way:
  - Arc 128 — structural check walkers must respect the
    sandbox boundary.
  - Arc 129 — `:time-limit` wrapper must distinguish
    `RecvTimeoutError::Timeout` from `Disconnected`.

## Slice walkthrough

### Slice 1 — the check (substrate)

`src/check.rs` adds:

- `CheckError::ChannelPairDeadlock { callee, sender_arg,
  receiver_arg, pair_anchor, span }` variant + Display arm +
  diagnostic mapping.
- `validate_channel_pair_deadlock` entry walker.
- `walk_for_pair_deadlock` recursive AST walk with binding
  scope accumulation + arc 128's sandbox-boundary guard.
- `check_call_for_pair_deadlock` per-call argument
  classification + group-by-anchor.
- `trace_to_pair_anchor` recursive RHS chain walk.
- `parse_binding_for_pair_check` sibling to arc 117's parser
  returning the RHS for chain tracing.
- `type_is_sender_kind` + `type_is_receiver_kind`
  classifiers using `expand_alias` for one-step peel.
- 5 unit tests: canonical anti-pattern fires; two-different-
  pairs silent; HandlePool-pop-style silent; substring
  assertion; sandboxed-forms skip.

### Slice 2 — the wat-side annotation conversion

The 6 deadlock-class tests convert from
`(:wat::test::ignore "...")` to
`(:wat::test::should-panic "channel-pair-deadlock")`, with
`(:wat::test::time-limit "200ms")` preserved as defense-in-
depth. Sites:

- `crates/wat-lru/wat-tests/lru/CacheService.wat`:
  `test-cache-service-put-then-get-round-trip`.
- `crates/wat-holon-lru/wat-tests/holon/lru/HologramCacheService.wat`:
  `test-step3-put-only`, `test-step4-put-get-roundtrip`,
  `test-step5-multi-client-via-constructor`,
  `test-step6-lru-eviction-via-service`.
- `crates/wat-holon-lru/wat-tests/proofs/arc-119/step-B-single-put.wat`:
  `step_B_single_put`.

### Slice 3 — closure (this slice)

INSCRIPTION (this doc) + USER-GUIDE entry + WAT-CHEATSHEET
section + ZERO-MUTEX cross-reference + 058 changelog row.

## The four questions (final)

**Obvious?** Yes. Same `make-bounded-channel` call → same
channel; both ends in one role → deadlock. The trace walks
data, not heuristics. Arc 117 had the same shape applied at
join-result sites; arc 126 applies it at call-site arguments.

**Simple?** Yes. ~200 LOC slice 1 (560 LOC including
comments + Display + tests). Reuses arc 117's trace
mechanism. Single new CheckError variant. Surface-level
matching after `expand_alias`. Slice 2 is mechanical (~6 site
edits). Slice 3 is doc-only.

**Honest?** Yes. The rule names the load-bearing structural
invariant — same pair-anchor — not a heuristic approximation.
The diagnostic cites `ZERO-MUTEX.md` § "Routing acks" as the
canonical-fix doctrine. The substrate's existing pair-by-
index pattern (Console) is already correct; the rule
enforces it for new code.

**Good UX?** Phenomenal. The 6 deadlock-class tests fail at
type-check time with a clear diagnostic naming the
pair-anchor and the canonical-fix patterns. Future authors
get the same diagnostic at freeze time, before runtime
hang. The substrate teaches the right shape;
ZERO-MUTEX.md becomes the why-doc.

## Failure-engineering record

Arc 126 IS the worked example for the failure-engineering +
artifacts-as-teaching disciplines (see `REALIZATIONS.md`).
The slice progression:

| # | Sweep | Slice | Hard rows | Substrate gap |
|---|---|---|---|---|
| 1 | arc 126 slice 1 | first sweep | 5/6 | arc 128 (boundary guard) |
| 2 | arc 126 slice 1 | reland | 14/14 | none (clean) |
| 3 | arc 126 slice 2 | first sweep | 6/8 | arc 129 (Timeout vs Disconnected) |
| 4 | arc 129 slice 1 | first sweep | 14/14 | none (clean) |
| 5 | arc 126 slice 2 | reland (committed) | green | bundled with arc 129 |

Sweep timings compounded: 13.5 min → 7 min → 5.3 min → 2.5
min as artifacts accumulated. Each non-clean sweep produced
a precisely-diagnosed substrate gap; each follow-on arc
landed cleanly. The discipline is INTACT across structural-
rule arcs (arc 126 — type-check walker), substrate-fix arcs
(arc 128 — walker boundary; arc 129 — proc macro panic
propagation), and across distinct substrate layers
(`src/check.rs`, `crates/wat-macros/src/lib.rs`).

## Cross-references

- `docs/ZERO-MUTEX.md` § "Mini-TCP via paired channels" + §
  "Routing acks" — the doctrine this rule enforces.
- `docs/SERVICE-PROGRAMS.md` § "The lockstep" — the broader
  framework arc 117 + arc 126 jointly support.
- `docs/SUBSTRATE-AS-TEACHER.md` — the migration discipline
  the diagnostic follows.
- `docs/arc/2026/04/117-scope-deadlock-prevention/INSCRIPTION.md`
  — the precedent. Same trace machinery, different rule arm.
- `docs/arc/2026/04/119-holon-lru-put-ack/DESIGN.md` — the
  protocol context. Pattern B's "embedded reply-tx in
  payload" routing demanded the trace; arc 126 makes the
  discipline enforceable.
- `docs/arc/2026/05/125-rpc-deadlock-prevention/DESIGN.md` —
  the WITHDRAWN type-precise sibling. Honest record of why
  type-system-proxy was rejected for structural truth.
- `docs/arc/2026/05/127-thread-process-symmetry/DESIGN.md` —
  the WITHDRAWN architectural rethink. Honest record of why
  the existing substrate is the answer.
- `docs/arc/2026/05/128-check-walker-sandbox-boundary/INSCRIPTION.md`
  — the boundary fix that unblocked slice 1's reland.
- `docs/arc/2026/05/129-time-limit-disconnected-vs-timeout/INSCRIPTION.md`
  — the proc macro fix that unblocked slice 2.
- `docs/arc/2026/05/126-channel-pair-deadlock-prevention/REALIZATIONS.md`
  — the disciplines coined here (failure engineering +
  artifacts-as-teaching).

## Queued follow-ups

- **Multi-step rx/tx derivations.** A future arc widens the
  trace through user-helper functions (cross-function
  symbol-table walk).
- **Tuple-typealias unpack tracing.** A future arc walks
  through user-defined typealiases that hide Senders +
  Receivers behind struct fields.
- **Helper-verb signature redesign.** Arc 119's helper verbs
  `HologramCacheService/put` and `:wat::lru::put` take both
  halves of the ack-channel as arguments — that's the shape
  arc 126 now flags. A future arc could redesign the helper
  signatures to use HandlePool-style pre-allocated pairs OR
  embedded-reply-tx without caller binding, eliminating the
  `:should-panic` annotations on the 6 deadlock-class tests
  (the tests would then PASS without panicking, exercising
  the corrected helper-verb shape). Pending: real consumer
  needs both ends; today the structural enforcement IS
  the resolution per arc 119's closure.
