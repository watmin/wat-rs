# Arc 111 — intra-process `Result<Option<T>, ThreadDiedError>` — INSCRIPTION

## Status

Shipped 2026-04-30. Type-shape lift complete; substrate + lab swept;
all 737 wat-rs tests + lab tests green. Closes the structural shipment
of arc 111. Rich `Err` payload (cross-thread panic message
propagation) deferred to **arc 113** ("cascading runtime error
messages — the actual cross-thread backtrace") — slice 1's runtime
still ships `Err(ChannelDisconnected)` as a stand-in.

Pushed: wat-rs `76665b0` (substrate sweep, sonnet-driven), `6dbd6b8`
(REALIZATIONS), `31da58f` (validation note); lab `abffae2` (lab
sweep). Closure: this INSCRIPTION + USER-GUIDE update + 058 row.

## What this arc adds

A type-shape change at the kernel-comm boundary that surfaces the
THREE comm states as data:

| Op | Pre-arc-111 return | Arc 111 return |
|---|---|---|
| `:wat::kernel::send` | `:Option<()>` (`:wat::kernel::Sent`) | `:Result<:(), :wat::kernel::ThreadDiedError>` |
| `:wat::kernel::recv` | `:Option<T>` | `:Result<:Option<T>, :wat::kernel::ThreadDiedError>` |
| `:wat::kernel::try-recv` | `:Option<T>` | `:Result<:Option<T>, :wat::kernel::ThreadDiedError>` |
| `:wat::kernel::select` | `:(i64, :Option<T>)` | `:(i64, :Result<:Option<T>, :wat::kernel::ThreadDiedError>)` |

The `:wat::kernel::Sent` typealias retired. New
`:wat::kernel::CommResult<T>` typealias added for symmetry.

### Three states, three arms

For `recv`:

- `Ok(Some v)` — value flowed.
- `Ok(:None)` — channel alive but **terminal** (every sender dropped
  cleanly via scope exit; the protocol's last message).
- `Err(ThreadDiedError)` — sender thread panicked (slice 2 of arc
  113 wires the rich `Panic { message, failure }` payload; arc 111
  ships `ChannelDisconnected` as the placeholder so the type lifts
  honestly without lying about which case fired).

For `send`:

- `Ok(())` — delivered.
- `Err(ThreadDiedError)` — receiver gone. Arc 113 distinguishes
  `ChannelDisconnected` (clean drop) from `Panic { message }`
  (peer-thread panic).

`E` is `:wat::kernel::ThreadDiedError` — the SAME enum
`:wat::kernel::join-result` already returns. Programs that already
matched on `join-result`'s `Err` arms write identical patterns at
recv sites. The comm error type IS the join error type.

### Arc 110's grammar rule extended

`validate_comm_positions` in `src/check.rs` now permits comm calls
inside three parents (was two): `match`, `result::expect`,
`option::expect`. The new `result::expect` slot is the natural
panic-on-`Err` home for the new return type. Comm calls inside
`option::expect` remain valid for callers that have already
unwrapped a Result somewhere upstream.

### Migration hint at every type-mismatch site

`src/check.rs::arc_111_migration_hint` detects type-mismatches
involving `:Result<:Option<T>, :wat::kernel::ThreadDiedError>` and
appends a self-describing migration hint to the error. Every
incorrect call site reads, after the type error:

```
hint: arc 111 — :wat::kernel::send returns :Result<:(), :wat::kernel::ThreadDiedError>
and :wat::kernel::recv / try-recv return :Result<:Option<T>,
:wat::kernel::ThreadDiedError>. Migrate match arms: ((Some v) ...)
→ ((Ok (Some v)) ...); (:None ...) → ((Ok :None) ...) (recv) OR
((Err _) ...) (send); add a third arm ((Err _died) ...) for recv
to handle peer-thread panic.
```

Three audiences read this same diagnostic stream:

1. **Humans** — fix path embedded at every error.
2. **Agents** — brief collapses to "iterate until green."
3. **Orchestrators** — `grep -c "hint: arc 111"` IS the progress bar.

The hint is scaffolding; retires when arc 113 closes (no more
arc-111-shape mismatches in any consumer). See REALIZATIONS.md
for the full insight chain. Task #168 tracks the retirement.

## Why

Arcs 107 and 108 closed proof_004's silent-disconnect-cascade hang
at the call sites that knew to look. Arc 110 closed it at the
grammar — silent comm became a compile error. Arc 111 closes it at
the type — the THREE comm states are now distinct data the receiver
can match on.

User direction (2026-04-30) named the shape:

> we expect a :wat::core::None /as the final message/ - this is
> different from an exception
> [...]
> i don't think this will be rare... we are fully in memory here..
> there's no "remote host" here... if the thing we're glued into
> dies - we die - something catastrophic happened

The terminal-`:None`-as-data realization (Ok(:None)) lives in the
Ok arm; catastrophic peer-death (Err(panic)) lives in the Err arm.
No single signal carries two meanings.

User confirmation that arc 111 was the right next step:

> this is the next arc.. swapping to Result<Option<T>,E>

## What this arc does NOT do

- Does NOT propagate the actual sender-thread panic message
  through the channel. **Arc 113 is the rich-`Err` follow-up** —
  ships the six OnceLock pieces from this arc's DESIGN.md
  (Candidate C). Slice 1's runtime returns
  `Err(ChannelDisconnected)` on every disconnect; arc 113 splits
  out `Err(Panic { message, failure })` for actual sender-thread
  panic.
- Does NOT touch `:wat::kernel::join-result`. Its `Err` arm
  already returns `ThreadDiedError`. Arc 111 reuses the same
  enum at recv sites — programs match the same shape whether
  they're joining a thread or recv'ing from one.
- Does NOT extend the type shape to inter-process comms.
  **Arc 112 is the inter-process arc** — same return shape
  generalized to fork-program subprocess pipes (stdin =
  Sender; stdout = `Ok(Some T)` payload; stderr = `Err(E)`
  payload). Lands after arc 111; arc 113 then propagates rich
  panic info uniformly across in-memory and inter-process.

## Slice walkthrough

### Slice 1 — type-shape change + sweep

`src/check.rs`:
- Schemes for `:wat::kernel::send` / `recv` / `try-recv` / `select`
  updated to the new return shape.
- `validate_comm_positions`: new `CommCtx::ResultExpectValue`
  variant; comm calls now permitted in three slots (match-scrutinee,
  result::expect-value, option::expect-value).
- `arc_111_migration_hint(callee, expected, got)`: detects
  arc-111-shape mismatches and appends a self-describing migration
  hint to `TypeMismatch` and `ReturnTypeMismatch` Display impls.

`src/runtime.rs`:
- `eval_kernel_send`: `Ok(())` on landed; `Err(ChannelDisconnected)`
  on disconnect. `Result<(), ThreadDiedError>`.
- `eval_kernel_recv` / `try_recv`: `Ok(Some v)` on receive; `Ok(:None)`
  on disconnect (slice-1 placeholder; arc 113 distinguishes). The
  outer `Err` arm is unreachable from slice 1's runtime (kept honest
  in the type for arc 113 to wire).
- `eval_kernel_select`: tuple's second element follows the recv shape.

`wat/kernel/queue.wat`:
- `:wat::kernel::Sent` typealias retired.
- `:wat::kernel::CommResult<T>` added: `Result<Option<T>, ThreadDiedError>`.
- `:wat::kernel::Chosen<T>` redefined to use `CommResult<T>` in the
  second tuple slot.

Sweep scope (substrate-side, my edits + sonnet's):
- `wat/std/service/Console.wat` — Console/loop's match grew to 3 arms;
  client helpers migrate `option::expect` to `result::expect`.
- `wat/std/stream.wat` — 6 producer-stage match-on-send arms shift
  to `(Ok _)`/`(Err _)`; recv-loops grow to 3 arms.
- `crates/wat-{lru,holon-lru,telemetry}/wat/` — service implementations
  follow the same shape.
- `crates/*/wat-tests/` — test fixtures updated.
- `wat-tests/std/service-template.wat` — the canonical service
  template; teaches the new 3-arm shape.
- `tests/wat_*.rs` — embedded wat strings updated.

Sweep scope (lab-side, sonnet-driven):
- `wat/services/treasury.wat`
- `wat-tests/cache/L2-spawn.wat`
- `wat-tests-integ/proof/004-cache-telemetry/004-step-{B,C,D,E}.wat`
- `wat-tests-integ/experiment/008-treasury-program/explore-{handles,treasury}.wat`

Tests after sweep:

```
wat-rs:        cargo test --release green; 737 tests, 0 failures
holon-lab:     cargo test --release green; 0 failures
```

### Slice 5 (this slice) — INSCRIPTION + USER-GUIDE + 058 row

This file. USER-GUIDE.md updates the recv/send signatures to the
Result shape and adds the 3-arm match example. 058 FOUNDATION-
CHANGELOG row.

## What this arc closes

- **The two-arm conflation.** Pre-arc-111, `recv`'s `:None` carried
  three meanings under one signal. The match-on-recv "shutdown
  branch" couldn't tell catastrophic peer-panic from clean
  scope-end-of-stream. Now the type makes those distinct, and
  programs match three arms (or two, if they explicitly choose
  via `result::expect` to panic on Err).
- **Result-vs-Option asymmetry between recv and join-result.**
  Both return the same `ThreadDiedError` enum. Same matching
  patterns at every site that talks to a thread.
- **The class of bug arc 110 made impossible at the grammar.**
  Arc 110 made silent comm impossible. Arc 111 makes the comm
  result EXPRESSIVE — three states surface as data, ready for
  arc 113 to populate.

## The four questions (final)

**Obvious?** Yes. Same shape as `join-result`'s return. Programmers
who matched on `Err (ThreadDiedError::Panic msg _)` from `join-result`
write identical code at recv sites. The third arm reads naturally
once the type is known.

**Simple?** Yes. ~5 fns in runtime + ~5 schemes in check. The
grammar walk grew one variant. The migration hint is ~40 LOC in
one helper. No flow analysis. No type-system extension.

**Honest?** Yes. The current substrate's `:Option<T>` collapsed
three states into two arms. Arc 111 lifts the type to surface what
was always internally distinct. Arc 113 will populate the third
state with real data.

**Good UX?** Yes. The migration hint at every type-mismatch site
is the proof — the sweep was mechanical for the agent and the
human, the substrate carrying its own teaching at every
breakage point.

## Cross-references

- `docs/arc/2026/04/111-result-option-recv/DESIGN.md` — full design
  with the six OnceLock pieces (Candidate C) for arc 113's rich `Err`.
- `docs/arc/2026/04/111-result-option-recv/REALIZATIONS.md` —
  substrate-as-teacher / substrate-as-progress-meter /
  program-as-equation / coda.
- `docs/arc/2026/04/110-kernel-comm-expect/INSCRIPTION.md` — the
  grammar rule arc 111 builds on.
- `docs/arc/2026/04/060-join-result/INSCRIPTION.md` — the
  `ThreadDiedError` enum arc 111 reuses as `E`.
- `docs/arc/2026/04/105-spawn-error-as-data/INSCRIPTION.md` —
  `ThreadDiedError::Panic`'s `(message, failure)` shape preserved.

## Queued follow-ups

- **Arc 112** — inter-process compile-time checks + type shape.
  Same `Result<Option<T>, E>` surface generalized to fork-program
  subprocess pipes (stdin = Sender, stdout = `Ok(Some T)`, stderr
  = `Err(E)`). Mechanism: per-pipe EDN framing + the same grammar
  rule from arc 110 applied at the fork boundary.
- **Arc 113** — the actual cross-thread backtrace. Wires the six
  OnceLock pieces from arc 111's DESIGN.md so `Err(Panic msg
  failure)` carries the real panic message from the dying thread
  to every receiver of its channel. Slice 1's
  `Err(ChannelDisconnected)` placeholder retires.
- **Hint retirement (task #168)** — `arc_111_migration_hint`
  removed when no consumer wat code emits arc-111-shape errors
  anywhere. Same retirement pattern as arc 109's redirect arms.

After arcs 112 + 113 close, **arc 109 (kill-std)** resumes from
slice 1c with the FQDN sweep + `wat::std::*` flatten + path-
mirrors-FQDN file moves.
