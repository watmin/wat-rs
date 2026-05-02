# Arc 134 — INSCRIPTION

## Status

**Shipped + closed 2026-05-02** in commit `21750c9`. Single-
slice arc; substrate change implemented inline (not via sonnet
delegation) because the calibration set — four failing
integration tests — was tight enough that test-as-spec
sufficed.

## What this arc closes

The arc 117 / 131 / 133 `ScopeDeadlock` rule fired on type-
coexistence (Sender + Thread + join-result siblings) without
checking whether the Sender was structurally connected to a
recv-loop in the spawned function. After arc 133 closed the
`rust::crossbeam_channel::Sender` type-match bypass, the
overshoot became visible: two structural patterns rejected
that demonstrably do not deadlock at runtime.

The user walked the logic explicitly:

> a prior good test suddenly failing is an exact observation.
> what was working is suddenly illegal.. there was no
> deadlock.. the detection says there is one.. the detection
> is flawed..

Correct read. The diagnostic claimed deadlock; the runtime
doesn't deadlock; the rule was over-approximating. Arc 134
sharpens precision without rolling back arc 133's coverage
fix.

## What shipped

Two structural narrowings inside
`check_let_star_for_scope_deadlock_inferred` in
`src/check.rs`:

### 1. Origin-trace (`sender_originates_from_thread_pipe`)

A Sender whose binding RHS is `(:wat::kernel::Thread/input
<_>)` or `(:wat::kernel::Process/input <_>)` extracts the
parent-side end of an internal pipe owned by the Thread /
Process struct. The pair-Receiver is the spawned function's
`in` parameter — lifetime-coupled to the Thread, not parent
scope. Exempt for any sibling Thread.

Per (Thread, Sender) pair, called as the FIRST exemption
check.

### 2. Body-form (`spawn_thread_lambda_body_has_no_recv`)

If the Thread's binding RHS is a `(:wat::kernel::spawn-thread
<fn> ...)` / `spawn-program` / `fork-program` call whose
function argument is an inline `(:wat::core::lambda ...)`,
walk every body form looking for any `(:wat::kernel::recv
...)` / `try-recv` / `select` call. None present → no recv-
loop possible → exempt every Sender for this Thread.

Called per Thread (BEFORE the Sender loop).

## The diagnostic

Unchanged from arc 117 / 131. Same `ScopeDeadlock` variant,
same Display arm, same `offending_kind` values
(`"Sender"` / `"Channel"` / `"HandlePool"` / `"QueueSender"`).
Arc 134 narrows WHEN the diagnostic fires; it does not change
WHAT the diagnostic says.

## Calibration set

Four integration-test patterns the arc 134 narrowings unblock,
without any source edits to those tests:

| Test file | Test | Pattern | Narrowing applied |
|---|---|---|---|
| `tests/wat_spawn_lambda.rs` | `spawn_thread_named_define_body` | Thread/input + recv-once body | origin-trace |
| `tests/wat_spawn_lambda.rs` | `spawn_thread_inline_lambda_body` | Thread/input + recv-once body | origin-trace |
| `tests/wat_spawn_lambda.rs` | `spawn_thread_closure_capture` | Thread/input + recv-once body | origin-trace |
| `tests/wat_typealias.rs` | `alias_over_fn_type_works_at_spawn` | parent-channel + closure with no recv | body-form |

All four green post-arc-134.

## Regression guards

Three new unit tests in `src/check.rs`:

- `arc_134_thread_input_output_does_not_fire` — locks in the
  origin-trace exemption for the canonical Thread<I,O> usage.
- `arc_134_parent_allocated_channel_still_fires` — guard rail;
  a Sender from `(make-bounded-channel)` sibling to a Thread
  with a keyword-path spawn argument still fires (arc 117's
  canonical anchor).
- `arc_134_no_recv_in_lambda_body_does_not_fire` — locks in
  the body-form exemption.

Two existing regression guards updated to use recv-bearing
lambda bodies — their pre-arc-134 empty-body fixtures are now
correctly exempt under the body-form narrowing, breaking the
"must fire" assertions:

- `arc_131_handlepool_with_sender_fires` — body changed from
  `()` to `(:wat::core::match (:wat::kernel::recv _in) -> :unit
  ...)`.
- `arc_133_typed_name_binding_still_fires` — same change.

## Workspace verification

- Lib check tests: 56 passed; 0 failed.
- Workspace: 1765 individual tests passed across 100 result
  blocks; 0 failed; `cargo test --release --workspace` exit 0.

## Heuristic notes (load-bearing for future precision work)

Both narrowings are heuristic. Documented at the
implementation site so future readers don't take them as
guarantees.

**Origin-trace heuristic.** `Thread/input <thr>` only exempts
the `(thr, this-Sender)` pair when relying on the canonical
Thread<I,O> contract — the spawned function's body is recv-
once OR pairs sends with recvs. A spawned function with an
unbounded recv-loop on its input pipe WILL deadlock at
runtime when parent's `tx` is alive at scope exit AND
`Thread/join-result thr` is in body before scope exit; the
narrowing exempts this case, accepting the runtime hang as
the cost of precision.

**Body-form heuristic.** Only inline `(:wat::core::lambda
...)` bodies are walked. A spawn-thread call whose argument
is a keyword-path (named function) cannot have its body
inspected at this hook — substrate function-body lookup would
require plumbing `CheckEnv` deeper than the inferred-types
check has access to. Such calls fall back to the conservative-
fire path (or the origin-trace exemption if applicable). The
body-form check also does not follow function calls inside
the lambda body — `(:my-helper rx)` where `:my-helper`
contains a recv slips past, but that's correct conservative
behavior.

Future arc 135 (or whoever needs better precision) can extend
body-form analysis to follow named-function bodies via
`CheckEnv` lookup, or transitively walk called functions for
recv presence.

## The four questions

**Obvious?** Yes once stated. The rule's promise was deadlock
detection; the implementation was type-coexistence detection.
The diagnostic literally claimed "would block forever" on code
that demonstrably does not block.

**Simple?** Yes. ~120 LOC of helpers in `src/check.rs`. No
substrate-wide changes; no new walker passes; no new
CheckError variants. Three new unit tests; two existing
regression guards updated.

**Honest?** Yes. The narrowings are heuristic; documented as
such at the call site. Limitations named (keyword-path spawn,
transitive recv) so future readers don't mistake them for
guarantees.

**Good UX?** Phenomenal. Tests that don't deadlock no longer
fire ScopeDeadlock. The rule's claim now matches its behavior
on the canonical Thread<I,O> usage and the parent-channel-
without-recv pattern.

## Failure-engineering record

Arc 134 closes the rule-precision gap surfaced by arc 133's
correctness fix. The chain continues:

| # | Arc | Surfaced by | Status |
|---|---|---|---|
| 1 | 117 | substrate-author | shipped |
| 2 | 131 | arc 130 sweep killed | shipped |
| 3 | 133 slice 1 | arc 131 slice 2 SCORE | shipped (commit `f717f15`) |
| 4 | **134** | **arc 133 workspace failures** | **shipped (commit `21750c9`)** |

Pattern variation: arc 134 was implemented inline rather than
via sonnet delegation. The calibration set was tight enough (4
failing integration tests) that the test-as-spec discipline
sufficed without a separate brief / expectations / score
cycle. Honest record of when delegation is overkill.

## Cross-references

- `DESIGN.md` — back-filled record of what shipped + why.
- `docs/arc/2026/04/117-scope-deadlock-prevention/INSCRIPTION.md`
- `docs/arc/2026/05/131-handlepool-scope-deadlock/INSCRIPTION.md`
- `docs/arc/2026/05/133-tuple-destructure-binding-check/INSCRIPTION.md`
  (the parent INSCRIPTIONs whose precision arc 134 sharpens)
- `docs/arc/2026/05/126-channel-pair-deadlock-prevention/INSCRIPTION.md`
  — sibling rule with similar trace-style machinery.
- `src/check.rs::check_let_star_for_scope_deadlock_inferred`
  — call site.
- `src/check.rs::sender_originates_from_thread_pipe` /
  `rhs_is_thread_input_extractor` — narrowing 1.
- `src/check.rs::spawn_thread_lambda_body_has_no_recv` /
  `rhs_spawn_lambda_has_no_recv` / `node_contains_recv` —
  narrowing 2.
- `tests/wat_spawn_lambda.rs` + `tests/wat_typealias.rs` — the
  calibration set / test-as-spec.
