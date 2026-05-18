# Arc 208 — Process I/O returns Result (mirror arc 110/111 at process tier)

**Status:** OPEN 2026-05-17.

**Priority:** **BLOCKING.** Per arc 203 DESIGN § "What arc 203 demands from upstream" demand 2: substrate Process I/O Result slice is required for "the one pattern" to claim parity across thread + process tiers. Arc 203 closure depends on this; arc 170 closure depends on arc 203 closure.

**Pedigree:** Arc 110 made silent kernel-comm illegal at thread tier (send/recv must land inside match-or-expect). Arc 111 flipped thread-tier `send/recv/try-recv/select` signatures to return `Result<_, ThreadDiedError>` (later widened to `Result<_, Vector<ThreadDiedError>>` in arc 113 for chain semantics). Arc 112 attempted process-tier Result for the now-retired `fork-program` API. The arc 170 Stone C2 refactor minted new `Process/readln` + `Process/println` against `ProcessPeer<I,O>` but did NOT carry the Result-bearing discipline forward — that's the regression arc 208 fixes.

**The crack arc 208 closes.** Current substrate state (verified `src/check.rs:13436+13445`, `src/runtime.rs:4782+4785`):

```rust
// :wat::kernel::Process/readln peer -> :I
//   Panics (RuntimeError) on subprocess death — no Result return.

// :wat::kernel::Process/println peer data:O -> :wat::core::nil
//   Panics (RuntimeError) on subprocess death — no Result return.
```

Arc 203 slice 3f SCORE delta lines 32-34 named the consumer pressure: process-tier user wrappers (get-proc, increment-proc, reset-proc, deprovision-proc) can surface `AccessDenied` via Result but transport failure still panics. ServerDied is demonstrated only via a separately-crashed `crash-test-proc` helper + `Process/drain-and-join` (the latter returns `Result<nil, Vector<ProcessDiedError>>` correctly).

The asymmetry: thread tier has `Result<_, Vector<ThreadDiedError>>` for every send/recv site; process tier panics. "The one pattern" cannot claim parity across tiers until process tier mirrors thread tier's discipline.

## Goal — substrate surface (locked 2026-05-17)

Flip the two Process I/O verbs to Result-returning, mirroring arc 110/111's thread-tier pattern:

| Verb | Before (today) | After (arc 208 ships) |
|---|---|---|
| `:wat::kernel::Process/readln` | `[ProcessPeer<I,O>] -> :I` (panics on disconnect) | `[ProcessPeer<I,O>] -> :Result<:I, :Vector<ProcessDiedError>>` |
| `:wat::kernel::Process/println` | `[ProcessPeer<I,O>, :O] -> :nil` (panics on disconnect) | `[ProcessPeer<I,O>, :O] -> :Result<:nil, :Vector<ProcessDiedError>>` |

The exact shape mirrors arc 111's thread-tier outcome — `Sender/send` returns `Result<nil, Vector<ThreadDiedError>>`; `Receiver/recv` returns `Result<Option<T>, Vector<ThreadDiedError>>`. Process tier uses `Vector<ProcessDiedError>` (the substrate's existing process-tier error type per `src/types.rs:632` with structured panic-chain accessors).

**Note on `Process/readln`'s return type.** The thread-tier `Receiver/recv` returns `Result<Option<T>, ...>` — Option discriminates "channel-closed-cleanly" (`None`) from "value-received" (`Some(v)`). Process tier MAY want the same shape if a clean stdin close is distinguishable from subprocess death. Slice 1's audit decides whether Process/readln returns `Result<:I, ...>` or `Result<Option<:I>, ...>` based on substrate semantics. Reasonable default: `Result<:I, ...>` — clean stdin close at process tier IS subprocess death (the subprocess can't read after exit).

## Out of scope (affirmatively, NOT deferral per arc 207 carry-forward discipline)

- **Other Process verbs** (`Process/stdin`, `Process/stdout`, `Process/stderr` accessors; `Process/drain-and-join` already returns Result correctly; `Process/join-result` is `restricted_to :wat::` and returns Result). Arc 208 scope is the two I/O verbs that lie. Other verbs either ALREADY return Result or are not I/O.
- **Walker rule "silent Process I/O illegal"** — arc 110's walker enforces silent kernel-comm illegal at thread tier. Arc 208 will audit whether a parallel walker is needed at process tier. **Default position: NOT needed** because Process I/O currently PANICS (loud) not silently-swallows (which is what arc 110's walker prevented at thread tier where send/recv used to return ()). The discipline gap is "no way to handle the error," not "silent swallowing." Slice 1 audit confirms; if walker needed, opens own slice; if not, affirmatively scoped out.
- **`fork-program` API revival** — retired in arc 170 chain; arc 208 does NOT touch it.
- **Cross-tier abstraction over Sender/send + Process/println** — different transports honestly; abstracting away the difference is wrong shape per `feedback_no_new_types`. The protocols arc (demand 1 of arc 203) is where the abstraction lives at the meta-form level.

## Slicing (sketch — slice 1's audit refines)

Three slices likely sufficient given the precedent shape is settled:

| Slice | Status | What | Notes |
|---|---|---|---|
| **1 — substrate audit + Result flip** | OPEN | Audit current Process I/O code paths; confirm shape decision (`Result<:I, ...>` vs `Result<Option<:I>, ...>` for readln); flip `Process/readln` + `Process/println` signatures + eval handlers; mint tests proving Result roundtrip (Ok path) + Err path (subprocess crash) | Larger first slice because shape is settled by arc 111 precedent; no separate audit slice needed |
| **2 — consumer ripple + (conditional) walker** | BLOCKS on 1 | Arc 203 process-tier demo (`wat-tests/counter-service-process-N3.wat`) gets its honest-Result wrappers — slice 3f's `crash-test-proc` workaround retires because main service wrappers now surface transport failure directly. Other consumers of `Process/readln`/`Process/println` (sonnet greps) flip. IF slice 1 audit identified need for walker rule (silent Process I/O illegal), add here; otherwise affirmatively scoped out | Mechanical ripple; walker rule conditional |
| **3 — closure paperwork** | BLOCKS on 2 | INSCRIPTION (FM 11 grep clean); DESIGN status CLOSED; 058 row; cross-reference arc 110/111 precedent + arc 203 demand 2 closure | Unblocks arc 203 demand 2; one of two demands closed |

## Substrate touchpoints (preliminary; slice 1's audit refines)

- `src/check.rs:13402-13455` — `Process/readln` + `Process/println` type scheme registrations (the BEFORE-AFTER table's substrate site)
- `src/runtime.rs:4782+4785` — dispatch arms for the two verbs
- `src/runtime.rs:17974+18039` — `eval_process_readln` + `eval_process_println` handlers (current implementations that panic on disconnect; rewrite to Result-returning)
- `src/runtime.rs:18093+` — shared peer-struct unwrap helper (both verbs use it; reuse)
- `src/types.rs:632` — `ProcessDiedError` substrate type (already exists; reuse for Vector chain)
- `wat-tests/counter-service-process-N3.wat` — arc 203 process-tier consumer; slice 2 retargets wrapper bodies
- Any other consumer of Process/readln+Process/println surfaced by slice 1's grep

## Connection to broader work

Arc 208 is one of two demands arc 203 has from upstream (arc 203 DESIGN § "What arc 203 demands from upstream" demand 2). The other demand is the protocols arc (defservice meta-form). The two are independent — they can ship in any order; arc 203 closure requires both.

Best-ordering hypothesis per arc 203 DESIGN: arc 208 (substrate honest first), then protocols arc (meta-form on honest substrate). With arc 208 shipping Result-returning Process I/O, the protocols arc's process-tier transport adapter has clean error semantics to wrap; without arc 208, defservice's process-tier wrappers would inherit the same panic-not-Result delta arc 203 slice 3f hit.

After arc 208 closes: arc 203 demand 2 satisfied; arc 203 closure waits on demand 1 (protocols arc).

## Discipline carry-forward

Per arc 207's INSCRIPTION (the discipline lesson load-bearing across arcs):

> Before naming anything "out of scope; no consumer demands it," grep the substrate for arms / errors / panics that name the missing type. If they exist, that IS the consumer pressure; the type belongs in scope.

Arc 208 IS that discipline applied: slice 3f SCORE delta named the consumer pressure (process-tier wrappers can't catch transport failure); the substrate site is concrete (src/runtime.rs:17974+18039 panic-on-disconnect arms); the type that should be there exists (`ProcessDiedError`); the arc opens.

Per `feedback_attack_foundation_cracks` + `feedback_any_defect_catastrophic`: substrate trust is binary. >0 known defects = 0 trust. The process-tier panic-not-Result IS a known defect (named at arc 203 slice 3f); arc 208 closes it.

Per `feedback_simple_is_uniform_composition`: this is a uniform composition with arc 110/111. Thread tier and process tier should have the same error-propagation shape. The substrate's two transports compose into "the one pattern" cleanly only when both tiers vend Result-returning I/O.
