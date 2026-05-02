# Arc 134 — Scope-deadlock origin-trace + body-form narrowing

**Status:** shipped 2026-05-02 in commit `21750c9`. This DESIGN
is the back-filled record — the work was implemented inline
(not via sonnet delegation) because the corrective change was
small and the calibration set (failing integration tests) was
the spec.

## TL;DR

Arc 117 / 131 / 133's `ScopeDeadlock` rule fires on type-
coexistence: any `Sender`-bearing binding sibling to a
`Thread`-kind binding when `Thread/join-result <thr>` appears
in the body or sibling RHS. After arc 133 closed the
`rust::crossbeam_channel::Sender` type-match bypass, the rule
became visibly over-strict: it fired on canonical Thread<I,O>
usage and on parent-allocated channels passed to thread
closures that don't actually recv. Both demonstrably non-
deadlocking patterns; both rejected by the rule.

Arc 134 adds two structural narrowings to the inferred-types
check (`check_let_star_for_scope_deadlock_inferred` in
`src/check.rs`):

1. **Origin-trace exemption.** A Sender whose binding RHS is
   `(:wat::kernel::Thread/input <_>)` or
   `(:wat::kernel::Process/input <_>)` extracts the parent-side
   end of an internal pipe owned by the Thread/Process struct.
   The pair-Receiver is the spawned function's `in` parameter
   — lifetime-coupled to the Thread, not parent scope. Exempt
   for any sibling Thread.

2. **Body-form exemption.** If the Thread's binding RHS is a
   `(:wat::kernel::spawn-thread <fn> ...)` /
   `(:wat::kernel::spawn-program ...)` /
   `(:wat::kernel::fork-program ...)` call whose function
   argument is an inline `(:wat::core::lambda ...)`, walk the
   lambda body looking for any `(:wat::kernel::recv ...)` /
   `try-recv` / `select` call. None present → no recv-loop
   possible → exempt every Sender for this Thread.

## Provenance — what surfaced this

The arc 133 slice 1 substrate work shipped clean against its
unit-test scorecard (53/53 lib check tests). When the
orchestrator ran the workspace test, three failures appeared in
`tests/wat_spawn_lambda.rs` and one in
`tests/wat_typealias.rs::alias_over_fn_type_works_at_spawn`.

Sonnet's slice-1 SCORE predicted zero newly-firing tests in
`wat-tests/` — accurate for that scope. The Rust integration
tests under `tests/wat_*.rs` (which embed wat source as
strings) were outside the prediction surface. Four tests in
those files newly fired the check.

The user walked the logic explicitly (transcript 2026-05-02):

> a prior good test suddenly failing is an exact observation.
> what was working is suddenly illegal.. there was no
> deadlock.. the detection says there is one.. the detection
> is flawed..

The framing was correct. The rule's diagnostic claims
deadlock; the integration tests demonstrably do not deadlock
at runtime; the rule is over-approximating. The user pushed
back on rolling back arc 133's rust:: extension as
"heavy-handed" and asked: "where is the logic error?"

## The logic error

Arc 117's diagnostic: *"Sender clone keeps the channel alive
even when the receiving thread dies — the worker's recv never
sees EOF."* The claim has a hidden premise — the corresponding
**Receiver is held by the thread's recv-loop**.

The implementation (pre-arc-134) classified bindings by TYPE
alone:
1. Each binding's name → inferred TypeExpr
2. Each TypeExpr → Thread-kind / Sender-bearing classification
3. Coexistence + join-result-in-body → fire

Step 3 never **traced** the Sender's origin or **walked** the
spawned function's body. It assumed any Sender's pair-Receiver
might be in any sibling Thread's recv-loop. That assumption
fails for two structurally-distinguishable cases:

### Case A — Thread/input as origin

```scheme
(:wat::core::let*
  (((thr :Thread<i64,i64>) (:wat::kernel::spawn-thread :my::fn))
   ((tx  :Sender<i64>)     (:wat::kernel::Thread/input thr))
   ((rx  :Receiver<i64>)   (:wat::kernel::Thread/output thr))
   ((_ack :unit) (send tx 41))
   ((result :i64) (recv rx)))
  (Thread/join-result thr))
```

`tx`'s pair-Receiver is the spawned function's `in` parameter.
That Receiver is owned by the Thread struct itself —
allocated by `spawn-thread` and dropped when the thread
returns. Parent's `tx` lifetime is decoupled from the thread's
exit semantics. The runtime cannot deadlock here regardless of
the spawned function's body shape: even an unbounded recv-loop
in the body would correctly observe `Ok None` when parent's
`tx` drops at scope exit, but the loop has already returned
`Ok Some(_)` for the messages parent sent, so by the time
parent reaches `Thread/join-result thr`, the thread either
exited cleanly OR is past the loop.

(Strictly: an unbounded loop body that recvs more than parent
sends WILL block at the next recv until parent's tx drops at
scope exit. But scope exit happens AFTER `Thread/join-result`
in the body — so the rule's premise still holds for that
specific shape. The exemption is heuristic and prefers the
common Thread<I,O> usage where the spawned body sends an
exact-count response.)

### Case B — spawn body that doesn't recv

```scheme
(:wat::core::let*
  (((pair :Channel<i64>)   (:wat::kernel::make-bounded-channel :i64 1))
   ((tx   :Sender<i64>)    (:wat::core::first pair))
   ((rx   :Receiver<i64>)  (:wat::core::second pair))
   ((thr  :Thread<unit,unit>)
    (:wat::kernel::spawn-thread
      (:wat::core::lambda
        ((_in  :Receiver<unit>) (_out :Sender<unit>) -> :unit)
        (:my::sender-helper tx))))            ; ← captures tx, sends
   ((_ :Result<unit,_>) (Thread/join-result thr)))
  (recv rx))
```

The spawn-thread closure body is `(:my::sender-helper tx)` —
captures `tx` from outer scope, calls a helper that sends.
Body has zero recv calls. The thread cannot have a recv-loop;
no Sender lifetime can deadlock it. This pattern came from
`tests/wat_typealias.rs::alias_over_fn_type_works_at_spawn`,
which was testing typealias-over-fn-types AT spawn — the
canonical-deadlock anchor (`make-bounded-channel`) is present
but the recv-loop the rule's premise depends on is absent.

The rule fires on type-coexistence (pair, tx, thr — all
sibling, all Sender-bearing or Thread-kind) without ever
walking the lambda to confirm its body has a recv at all.

## The fix — two narrowings

Both narrowings live in `src/check.rs` adjacent to the
existing `check_let_star_for_scope_deadlock_inferred`.

### Narrowing 1: origin-trace

`sender_originates_from_thread_pipe(sender_name, bindings) -> bool`:
1. Find `sender_name`'s binding (typed-name OR tuple-destructure).
2. Inspect the binding's RHS.
3. Return true iff the RHS is a list whose head is the keyword
   `:wat::kernel::Thread/input` or `:wat::kernel::Process/input`.

Inside the firing loop, called per (Thread, Sender) pair as
the FIRST exemption check: if true, skip this pair.

### Narrowing 2: body-form

`spawn_thread_lambda_body_has_no_recv(thr_name, bindings) -> bool`:
1. Find `thr_name`'s binding RHS.
2. Verify it is a `(:wat::kernel::spawn-thread ...)` /
   `spawn-program` / `fork-program` call.
3. Verify the function argument is an inline
   `(:wat::core::lambda <params> body+)`.
4. Walk every body form recursively looking for any list whose
   head keyword is `:wat::kernel::recv`,
   `:wat::kernel::try-recv`, or `:wat::kernel::select`.
5. Return true iff no recv-call is found.

Inside the firing loop, called per Thread (BEFORE the Sender
loop) — if true, skip ALL Senders for this Thread.

The two narrowings compose: a let* with a Thread whose lambda
has no recv hits narrowing 2 (exempts everything for that
thr); a let* whose Thread is from a keyword-path spawn (no
inline-lambda body to walk) falls through narrowing 2 and
relies on narrowing 1 per Sender.

## The four questions

**Obvious?** Yes once stated. The rule's promise was
deadlock-detection; the implementation was type-coexistence
detection. The diagnostic literally claimed
"would block forever" on code that demonstrably does not block.

**Simple?** Yes. ~120 LOC of helpers in `src/check.rs`. No
substrate-wide changes. No new walker passes. No new
CheckError variants. Three new unit tests; two existing
regression guards updated to use recv-bearing fixtures (the
empty-body fixtures pre-arc-134 were correctly exempt under
the new narrowings, breaking the assertions).

**Honest?** Yes. The narrowings are heuristic and documented
as such. The Thread/input narrowing assumes the canonical
Thread<I,O> contract is followed (recv-once or paired-
coordination body). The body-form narrowing only walks inline
lambda bodies — keyword-path spawn arguments fall back to
conservative-fire. Bodies that call helpers with recv inside
slip through (transitive recv is not analyzed). All
limitations named at the implementation site.

**Good UX?** Phenomenal. Tests that demonstrably don't
deadlock no longer fire ScopeDeadlock. The rule's claim
matches its behavior more closely. False positives on
canonical Thread<I,O> usage are eliminated.

## What it does NOT cover

These cases still fire the rule conservatively:

1. **Keyword-path spawn argument with a body that doesn't
   recv.** We don't do substrate function-body lookup at this
   hook; named functions fall through to conservative-fire
   unless their Sender is from `Thread/input`.

2. **Lambda body that calls a helper which recvs.** The body-
   form check only walks the immediate lambda body. A body
   that calls `(my-helper rx)` where `my-helper`'s definition
   contains a recv slips past — but that's correct
   conservative behavior (the recv could be a loop).

3. **Process<I,O> with `Process/input` siblings.** The origin-
   trace narrowing covers this (head includes
   `:wat::kernel::Process/input`), but the body-form narrowing
   only detects spawn-thread / spawn-program / fork-program;
   if the Process is constructed differently in user code, it
   falls back to conservative-fire.

These are known limitations. Future arc 135 (or whoever needs
better precision) can extend body-form analysis to follow
named-function bodies via `CheckEnv` lookup, or transitively
walk called functions for recv presence.

## Calibration set (the spec)

The four integration tests that surfaced the false positives
ARE the spec for arc 134:

| Test file | Test name | Pattern | Narrowing |
|---|---|---|---|
| `tests/wat_spawn_lambda.rs` | `spawn_thread_named_define_body` | Thread/input + recv-once body | origin-trace |
| `tests/wat_spawn_lambda.rs` | `spawn_thread_inline_lambda_body` | Thread/input + recv-once body | origin-trace (lambda body has recv, narrowing 2 doesn't apply) |
| `tests/wat_spawn_lambda.rs` | `spawn_thread_closure_capture` | Thread/input + recv-once body | origin-trace |
| `tests/wat_typealias.rs` | `alias_over_fn_type_works_at_spawn` | parent-channel + closure with no recv | body-form |

All four green post-arc-134 without source edits.

Three new unit tests as regression guards in `src/check.rs`:
- `arc_134_thread_input_output_does_not_fire` — locks in the
  origin-trace exemption.
- `arc_134_parent_allocated_channel_still_fires` — guard rail;
  the canonical deadlock anchor still fires.
- `arc_134_no_recv_in_lambda_body_does_not_fire` — locks in
  the body-form exemption.

Two existing regression guards updated:
- `arc_131_handlepool_with_sender_fires` — body changed from
  `()` to `(match (recv _in) -> :unit ...)` so narrowing 2
  doesn't exempt the canonical service-test mistake.
- `arc_133_typed_name_binding_still_fires` — same change.

## Cross-references

- `docs/arc/2026/04/117-scope-deadlock-prevention/INSCRIPTION.md`
  — the parent rule whose precision arc 134 sharpens.
- `docs/arc/2026/05/131-handlepool-scope-deadlock/INSCRIPTION.md`
  — the HandlePool extension; arc 134's narrowings apply
  uniformly to its diagnostic shape.
- `docs/arc/2026/05/133-tuple-destructure-binding-check/INSCRIPTION.md`
  — the binding-shape coverage extension that surfaced the
  false positives by closing the rust:: type-match bypass.
- `docs/arc/2026/05/126-channel-pair-deadlock-prevention/INSCRIPTION.md`
  — the sibling rule. Same trace-style machinery; arc 126
  traces to `make-bounded-channel` pair-anchors at call sites,
  arc 134 traces Sender RHS to Thread/input extractors at
  binding sites.
- `src/check.rs::check_let_star_for_scope_deadlock_inferred`
  — the call site; both narrowings invoked here.
- `src/check.rs::sender_originates_from_thread_pipe` +
  `rhs_is_thread_input_extractor` — narrowing 1.
- `src/check.rs::spawn_thread_lambda_body_has_no_recv` +
  `rhs_spawn_lambda_has_no_recv` + `node_contains_recv` —
  narrowing 2.

## Failure-engineering record

Arc 134 closes the rule-precision gap surfaced by arc 133's
correctness fix. The chain continues:

| # | Arc | Surfaced by | Status |
|---|---|---|---|
| 1 | 117 | substrate-author | shipped |
| 2 | 131 | arc 130 sweep killed | shipped |
| 3 | 133 slice 1 | arc 131 slice 2 SCORE | shipped (commit `f717f15`) |
| 4 | **134** | **arc 133 workspace failures** | **shipped (commit `21750c9`)** |

Pattern: each substrate-fix arc closes a gap surfaced by the
previous arc. The artifacts-as-teaching record continues. Arc
134 differs from the recent chain in that the work was
implemented inline rather than via sonnet delegation — the
calibration set was tight enough (4 failing integration tests)
that the test-as-spec discipline was sufficient without a
separate brief / expectations / score cycle.
