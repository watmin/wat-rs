# Arc 110 — silent kernel-comm is now illegal — INSCRIPTION

## Status

Shipped 2026-04-30. Substrate grammar rule + 16-file sweep across
wat-rs + 1-file sweep in holon-lab-trading. All 162 wat-rs tests +
344 lab tests green. Pushed: wat-rs `045112d`, lab `4debcb8`.

Arc 109 (kill-std) was paused for this; resumes at slice 1c.

## What this arc adds

A single grammar rule enforced in `src/check.rs::validate_comm_positions`:

> `:wat::kernel::send` and `:wat::kernel::recv` calls may appear ONLY
> as the **scrutinee** of `:wat::core::match` or as the
> **value-position** of `:wat::core::option::expect`. Any other
> context — let-binding RHS, function-call argument, struct field,
> bare return value of a function whose body terminates with a comm
> call — is a compile-time error: `CommCallOutOfPosition`.

The walk runs **before** the inference phase so a misplaced send/recv
reports as the structural problem it is, not as a downstream type
error. Three pieces:

- `CheckError::CommCallOutOfPosition { callee }` variant — formats
  as: `:wat::kernel::recv may appear only as the scrutinee of
  :wat::core::match or the value-position of
  :wat::core::option::expect; silent disconnect (the :None arm)
  must be handled at every comm call`.
- `CommCtx` enum — `Forbidden | MatchScrutinee | OptionExpectValue`.
  Threads through a recursive walk that knows which slot each
  child sits in.
- `validate_comm_positions(node, ctx, errors)` — at each `WatAST::List`
  with a keyword head, checks (1) is THIS node a comm call (and
  parent forbidden? error). Otherwise dispatches: match's items[1]
  is `MatchScrutinee`; option::expect's items[3] is
  `OptionExpectValue`; everything else descends as Forbidden.

Wired into `check_program` to walk every form and every user-define's
body before inference.

## Why

Two converging realizations:

1. **Arc 107/108's expect tooling was a bridge, not the rule.** Arcs
   107 and 108 added `:wat::core::option::expect` because proof_004's
   silent-disconnect-cascade hang needed an immediate cure. The
   tools work; the cure is real. But arc 108's INSCRIPTION
   explicitly deferred the broader sweep ("Does NOT migrate other
   call sites that COULD use expect (e.g., Service/batch-log,
   Stream's ack loops). Each call site is a separate decision per
   author.") That deferred work was mechanically real, and worse:
   nothing prevented the *next* author from re-introducing the same
   silent shape.

2. **In-memory peer-death is catastrophic.** User direction:

   > we expect a :wat::core::None /as the final message/ - this is
   > different from an exception
   > [...]
   > i don't think this will be rare... i think we want to know
   > when the peer dies and propagate.. we are fully in memory
   > here.. there's no "remote host" here... if the thing we're
   > glued into dies - we die - something catastrophic happened

   `:None` from a recv has two meanings the prior substrate didn't
   distinguish at the type level:
   - Worker recv-loop sees its terminal `:None` because every
     client dropped its sender — clean end-of-work signal.
   - Producer's send returns `:None` because the consumer dropped
     its receiver — protocol violation; this thread cannot
     continue meaningfully.

   The grammar rule forces every author to handle BOTH at every
   call site by (a) writing a `match` with both arms (when
   `:None` is honest data — recv-loops, producer stages), or
   (b) writing `option::expect` (when `:None` is catastrophic).
   The "bind it now and decide later" pathway — the silent ignore
   that caused proof_004 — doesn't exist anymore.

User-direction quote naming the arc shape:

> alright - i think you just found the answer... go update our
> code and we'll fix whatever falls out
>
> new arc - make the entire class of problem illegal and we'll
> resume our name changes once this lands

## What this arc does NOT do

- Does NOT change the type of `:wat::kernel::send` or
  `:wat::kernel::recv`. Both still return `:Option<()>` /
  `:Option<T>` respectively. The grammar restriction is the whole
  change; downstream tooling that consumes those types is
  unchanged.
- Does NOT add cross-thread panic propagation. If a sender thread
  panics, its sender-clone drops; the receiver's recv returns
  `:None`; the receiver's `option::expect` panics with the
  near-side diagnostic — but the FAR side's panic message is
  still lost. **That gap is the next arc** (arc 111: recv/send
  return `Result<Option<T>, E>` so the far-side message rides
  through the channel; the grammar rule from arc 110 stays
  valid, the match arms grow from 2 to 3).
- Does NOT police use of `:Option<T>` from non-comm sources.
  Helper functions returning `:Option<T>` are unchanged; the
  rule is scoped to `:wat::kernel::send` / `:wat::kernel::recv`
  because THOSE are the comm primitives whose `:None` carries
  the peer-death information that hangs CSP programs.
- Does NOT propagate the comm-call check through helper
  functions. A user wrapping a recv in their own helper must
  internally `match` or `expect`; their helper returns `T`, not
  `:Option<T>`. The rule is local — comm calls live where
  they're consumed.

## Slice 1 — substrate enforcement

`src/check.rs`:

- New `CheckError::CommCallOutOfPosition { callee: String }` variant
  with the diagnostic message above.
- New private enum `CommCtx { Forbidden | MatchScrutinee |
  OptionExpectValue }` threaded through a recursive walk.
- New private fn `validate_comm_positions(node, ctx, errors)` —
  ~50 LOC. At each `WatAST::List` with a keyword head:
  - If the head is `:wat::kernel::send` or `:wat::kernel::recv`
    AND ctx is `Forbidden`: push `CommCallOutOfPosition`.
  - If the head is `:wat::core::match` (≥ 4 args): items[1]
    descends as `MatchScrutinee`; items[2..] as `Forbidden`.
  - If the head is `:wat::core::option::expect` (≥ 5 args):
    items[3] descends as `OptionExpectValue`; everything else as
    `Forbidden`.
  - Default: every child descends as `Forbidden`.
- `check_program` invokes the walk over every user-define body
  and every program form BEFORE the inference loop.

Verified by hand-crafted files:

| File | Result |
|---|---|
| `/tmp/comm-bad.wat` (`((_s :wat::kernel::Sent) (send tx 42))`) | 1 `CommCallOutOfPosition` error, exit 3 |
| `/tmp/comm-good.wat` (send inside expect, recv inside match) | passes the check |

## Slice 2 — wat-rs sweep

Substrate's own wat sources brought into compliance:

| File | Sites | Strategy |
|---|---|---|
| `wat/std/service/Console.wat` | 3 | `Console/ack-at`, `Console/out`, `Console/err` flip to expect. The earlier "swallow either way" comment was masking the catastrophe; Console driver dying mid-write must surface, not silently drop the print. |
| `wat/std/stream.wat` | 6 | Producer-stage workers (`map`, `filter`, `inspect`, `take`, `flat-map`, `drain-items`) collapse `let-bind-then-match-on-sent` into `match-of-send-directly`. Same semantics, honest grammar. |
| `crates/wat-holon-lru/wat/holon/lru/HologramCacheService.wat` | 1 | Reply-side send → expect. |
| `crates/wat-lru/wat/lru/CacheService.wat` | 3 | Server-side reply send + client-side `CacheService/get` + `CacheService/put` flip to expect. The "either fall through to :None — caller observes 'miss'" comment was catastrophe in disguise. |
| `crates/wat-telemetry/wat/telemetry/Service.wat` | 1 | `Service/batch-log` → expect. The other (`Service/ack-all`'s match-of-send) stayed legal. |

| File | Sites | Strategy |
|---|---|---|
| `wat-tests/std/service-template.wat` | 10 | The canonical template — every send/recv visibly attached to its consumer. State-recv via `expect` unwraps `Option<State>` into `State` directly; downstream checks read fields without the Some-arm dance. |
| `crates/wat-holon-lru/wat-tests/holon/lru/HologramCacheService.wat` | 16 | Sites 1-13 swept by perl with multi-line balanced-paren regex; sites 14-18 (recv-then-match) inlined to match-at-source manually. |
| `crates/wat-telemetry/wat-tests/telemetry/Service.wat` | 6 | Drained recvs collapsed to match-at-source. |
| `crates/wat-telemetry/wat-tests/telemetry/WorkUnit.wat` | 1 | Same shape. |
| `crates/wat-telemetry/wat-tests/telemetry/WorkUnitLog.wat` | 5 | `extract-level` lambda refactored from `(Option<Event>) -> keyword` to `(Event) -> keyword`; recv hides nowhere. Each `l1`-`l4` site does match-at-source on recv. |
| `tests/wat_stream.rs` | 78 | Bulk-rewritten via single-line perl regex (uniform `((_  :Option<()>) (:wat::kernel::send TX VAL))` shape). |
| `tests/wat_typealias.rs` | 1 | Same. |
| `tests/wat_names_are_values.rs` | 3 | Manual edit. |

Test result after sweep:

```
test result: ok. 737 passed; 0 failed (src/check + runtime tests)
test result: ok. 162 passed; 0 failed (wat-tests)
... and on through each crate's test suite
all green
```

## Slice 3 — doc updates

- `docs/USER-GUIDE.md` § "Send and receive" — adds the arc-110 rule
  prominently next to the type signatures. Anti-pattern + proven-pattern
  examples updated to use `option::expect`.
- `docs/SERVICE-PROGRAMS.md` — Step 3, 4, 6, 7 examples updated. The
  canonical eight-step walkthrough now teaches the `match` and `expect`
  shapes by example. Step 4's bound-then-discarded `(got :Option<i64>)`
  becomes a typed `(got :i64)` via `option::expect` (the doubler-loop
  reply test).
- `docs/arc/2026/04/110-kernel-comm-expect/DESIGN.md` — design captured
  before code; the four questions answered.
- `docs/arc/2026/04/109-kill-std/REALIZATIONS.md` — § "The expect
  tooling is a bridge" anchored why arc 110 had to land before arc
  109's slice 1c could resume.

## Slice 4 — lab sweep

`holon-lab-trading/wat-tests/cache/L2-spawn.wat` — 6 Sent-typed
Pattern-A sends → `option::expect` (perl-driven). 3 recv-then-match
sites collapse to match-at-source. Two L2-spawn deftests green;
total lab cargo test 344 passed.

## What this arc closes

The "silent disconnect → recv hang" class as a substrate-level
guarantee. Arcs 107 and 108 closed it at the call sites that knew to
look. Arc 110 closes it at the grammar — there is no longer a way
for a wat program to omit the `:None` arm. Future authors get the
discipline for free; future regressions are compile errors, not
runtime hangs.

## The four questions (final answers)

**Obvious?** Yes. One grammar rule with two permitted slots.
Reading any wat file: every `recv`/`send` is visibly attached to
its consumer at the same parenthesized form. The error message
names the rule and points at both permitted positions.

**Simple?** Yes. ~80 LOC: one `CheckError` variant + one `CommCtx`
enum + one `validate_comm_positions` walk wired into
`check_program`. No flow analysis. No type-system extension. The
walk is straight recursion.

**Honest?** Yes. The shape of every comm call IS the shape of
every comm-handling decision. The "bind it now and decide later"
pathway doesn't exist; the grammar refuses to compile it. Both
permitted slots map to honest behaviors: match (handle both arms,
including the terminal `:None`) and expect (declare peer-death is
catastrophic; panic with a meaningful message).

**Good UX?** Yes.
- Errors are local — "`:wat::kernel::recv` may appear only inside
  `:wat::core::match` or `:wat::core::option::expect`."
- Reading any wat file, every recv/send is visibly attached to
  its consumer at the same `(...)` form.
- Wrapping helpers stay clean: they internally `match` and return
  `T`, so the obligation gets discharged at the comm call's
  home file.
- Every existing test fixture's restructure improved readability
  — match-at-source is shorter and sharper than bind-then-match;
  expect-unwrap eliminates the `(Some v) v` `(:None default)`
  ceremony at sites where peer-death is catastrophic.

## Cross-references

- `docs/arc/2026/04/110-kernel-comm-expect/DESIGN.md` — design
  document with tier comparison, four-questions evaluation,
  and the follow-up `Result<Option<T>, E>` framing.
- `docs/arc/2026/04/107-option-result-expect/INSCRIPTION.md` —
  the bridge tool's first shape (interim `:wat::std::*` helpers).
- `docs/arc/2026/04/108-typed-expect-special-forms/INSCRIPTION.md`
  — the bridge tool's final shape (`:wat::core::*` special forms
  with `-> :T` at HEAD); explicitly defers the broader sweep
  arc 110 just completed.
- `docs/arc/2026/04/109-kill-std/REALIZATIONS.md` § "The expect
  tooling is a bridge" — why arc 110 had to land mid-arc-109.
- `docs/SERVICE-PROGRAMS.md` § "The lockstep" — why worker
  recv-loops legitimately exit on `:None` (terminal data, not
  exception).
- `holon-lab-trading/wat-tests-integ/proof/004-cache-telemetry/`
  — the deadlock that motivated arcs 107/108/110.

## Queued follow-up

**Arc 111 — `recv`/`send` return `Result<Option<T>, E>`.** User
confirmed (2026-04-30) immediately after arc 110 closed:

> this is the next arc.. swapping to Result<Option<T>,E>

Splits `:None` into "stream alive but terminal (clean shutdown)"
vs "sender thread panicked (catastrophic; e carries the panic
message)". Substrate work: per-thread panic registry or
ProgramHandle back-ref on Sender. Arc 110's grammar rule stays
valid; the match arms grow from 2 to 3. See DESIGN.md § "Follow-up"
for the worked framing.
