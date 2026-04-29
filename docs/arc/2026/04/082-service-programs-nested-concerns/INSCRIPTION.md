# Arc 082 — SERVICE-PROGRAMS.md Step 9 (multi-driver shutdown via function decomposition) — INSCRIPTION

**Status:** shipped 2026-04-29.

`wat-rs/docs/SERVICE-PROGRAMS.md` gained a new ninth step covering
the case where one service's reporter (or any callback) closes over
ANOTHER service's handles. The naive inline triple-nested `let*`
deadlocks the join cascade; the function-decomposition pattern is
the documented fix.

The arc came out of a real deadlock I hit during arc 078 + lab
proof_004 development. The user named it:

> "'triple nest' sounds awful — we need to refactor simpler functions
> and use them"

The lockstep from Step 3 ("outer holds the handle; inner owns the
Senders") still applies — but with two services, each driver needs
its own outer/inner pair. Inline mega-`let*` collapses both into one
scope and the joins block forever. Decomposing into named functions
puts each driver in its own outer scope, with its own inner that
drops its senders before the function returns.

**Design:** [`DESIGN.md`](./DESIGN.md).

---

## What shipped

### `wat-rs/docs/SERVICE-PROGRAMS.md` § Step 9

New ~150-line section added between Step 8 and "The complete pattern."
Covers:

- **The trap** — cache-driver + rundb-driver + cache-req-tx all in
  one scope; trying to join cache-driver from that scope blocks
  because the popped req-tx is still bound.
- **The fix** — each driver-scope becomes a small named function
  with the canonical two-level let*. Three-function example:
  - `drive-requests` (bottom — pure work, no driver)
  - `run-cache-with-rundb-tx` (middle — owns cache-driver, joins
    before returning)
  - deftest body (top — owns rundb-driver, joins after middle returns)
- **The anti-pattern** — fully-worked inline triple-nest with
  "this deadlocks" annotation showing exactly which binding traps
  which join.
- **When function decomposition is required** — whenever a Reporter
  (or any callback) closes over OUTER service handles. The closure
  carries an extra ref to the outer service's senders; the only
  way to ensure they're all gone before joining the outer driver
  is to ensure the closure is gone, which means the inner service
  has FULLY shut down. A small named function that
  joins-before-returning gives that guarantee.
- **Real-world citation** — `holon-lab-trading/wat-tests-integ/proof/004-cache-telemetry/`
  ships this pattern; first attempt deadlocked, the
  function-decomposed version passes (~290ms).

### `wat-rs/docs/CONVENTIONS.md` § Service contract

Added a **"Composing services (the Reporter-closes-over-handles case)"**
subsection under arc 078's service-contract documentation. Cross-references
SERVICE-PROGRAMS.md § Step 9 + the proof_004 citation.

---

## What's documented vs what's substrate

This arc is **docs only** — no new substrate primitives. The
function-decomposition rule is a discipline applied to existing
primitives (kernel spawn / select / HandlePool / send / recv /
join). The substrate already supported this pattern; the docs
weren't yet teaching it.

The ninth step closes the gap that arc 078's service contract
opened: arc 078 codified single-service shape; arc 082 codifies
multi-service composition. Future Step-10+ entries land if
service-program patterns surface that the eight steps + Step 9
don't cover.

## Memory entries created

- `feedback_simple_forms_per_func` — "one outer let* per function;
  large/nested let*'s trip the assistant; offload complexity to
  small named functions."

## Consumer impact

This is the doc layer that any author of a multi-driver test or
multi-service composition reads before writing. The lab's
proof_004 pattern (cache reporter closes over rundb handles) is
the canonical citation; future cross-domain consumers facing the
same shape (any reporter that needs to flush through another
service) follow the same decomposition.

PERSEVERARE.
