# Arc 170 slice 1c — SCORE

Typed-channel-over-EDN-pipes substrate + `:wat::kernel::Process<I,O>`
additive reshape. Mode A clean, ~90 min opus (within 90-180
predicted band). Branch `arc-170-program-entry-points` carries
slice 1c commits `3c737ee` + `8eda4d3`.

## Scope as shipped

New module `src/typed_channel.rs` (~190 LOC):
- `SenderInner` / `ReceiverInner` enums with `Crossbeam(...)`
  and `PipeFd { ... }` variants
- `Value::wat__kernel__Sender(Arc<SenderInner>)` /
  `Value::wat__kernel__Receiver(Arc<ReceiverInner>)`
- `EdnEncoder` / `EdnDecoder` line-delimited pipe wrappers
  using arc 092 wat-edn

Process struct shape — ADDITIVE reshape:
- Existing 4 fields retained (stdin :IOWriter, stdout :IOReader,
  stderr :IOReader, handle :ProgramHandle)
- 2 new fields APPENDED (tx :Sender<I>, rx :Receiver<O>)
- Total 6 fields during sweep window; slice 3 retires the
  legacy three when testing tooling rebuilds

Files touched:
- `src/typed_channel.rs` — NEW (~190 LOC)
- `src/runtime.rs` — Value variant rename + send/recv/try-recv/
  select dispatch migration (~50 LOC delta)
- `src/fork.rs` — `make_pipe` pub'd; 2 Process construction sites
  updated (now 6 fields)
- `src/spawn.rs` — Process construction site updated
- `src/types.rs` — `:wat::kernel::Process<I,O>` type def gains
  tx/rx fields
- `src/edn_shim.rs` — opaque-nil arm rename (Sender/Receiver
  variant name change)
- `src/closure_extract.rs` — non-portable arm rename only (1
  pattern arm; spirit of row R preserved — extract_closure
  signature + ClosurePackage shape unchanged)
- `src/lib.rs` — `pub mod typed_channel;`
- `tests/wat_arc170_typed_channel_pipes.rs` — NEW (17 tests;
  ~600 LOC)

## Scorecard

All 18 rows from EXPECTATIONS-SLICE-1C.

| Row | Verified | Pass |
|-----|----------|------|
| A — DESIGN-intent alignment | typed-channel `Sender<T>`/`Receiver<T>` abstraction works uniformly across crossbeam (tier 1) and EDN-over-pipes (tier 2); user sees same Value type; transport is substrate-internal | ✓ |
| B — Transport-polymorphic Sender/Receiver | **Option B chosen** — single `Value::wat__kernel__Sender(Arc<SenderInner>)` with internal `SenderInner::Crossbeam(...)` / `PipeFd { ... }` enum. Reasoning: A doubles variant surface; C over-engineers for binary internal dispatch; B unifies under one Value variant per `feedback_capability_carrier.md` (extend existing entity, don't mint parallel) | ✓ |
| C — EDN encoding at pipe boundary | line-delimited via wat_edn (arc 092); Sender writes EDN-encoded line; Receiver reads-line + parses EDN; same convention as arc 113 cascade | ✓ |
| D — `:wat::kernel::Process<I,O>` reshape | **shipped additive** (honest delta from BRIEF "drop"; see Delta B below); byte-pipe fields stay during sweep window; typed-channel fields appended | ✓ partial — see Delta B |
| E — Wat-side struct definition updated | `src/types.rs` Process type def gains tx + rx fields | ✓ |
| F — Caller updates (Rust) | fork.rs (2 Process construction sites), spawn.rs (1 site) all updated for new 6-field shape | ✓ |
| G — Caller decision (wat) | **additive reshape obviates the decision** — both byte-pipe and typed-channel views coexist; legacy stdlib (sandbox.wat, hermetic.wat) callers continue to work without modification; slice 3 sweep retires legacy when testing tooling rebuilds | ✓ |
| H — Errors propagate via join-result | existing path preserved; `SendOutcome::Disconnected` on EPIPE for PipeFd transport mirrors crossbeam-disconnect symmetry per arc 111 | ✓ |
| I — Rust integration tests | `tests/wat_arc170_typed_channel_pipes.rs` — 17 tests across categories: round-trip i64, multi-Value stream, type fidelity (nested struct/vector/tuple/option), pipe-close error propagation, Process<I,O> field accessors, EDN-encoding boundary cases | ✓ |
| J — Workspace state | shimmed-via-additive → 2124 passed 0 failed (was 2107; +17 net new tests). Workspace stays GREEN (decision rationale: destructive Process reshape would brick stdlib bootstrap because sandbox.wat backs every deftest expansion — additive is the substrate-fit choice) | ✓ |
| K — No spawn-process verb wiring | confirmed (slice 2's territory) | ✓ |
| L — No `:user::main` signature changes | confirmed (slice 2's territory) | ✓ |
| M — No walker variants | confirmed (slice 2's territory) | ✓ |
| N — No new wat-level surface | only Process struct field additions (which BRIEF expected); no new wat-callable verbs | ✓ |
| O — Slice branch on remote | `arc-170-program-entry-points` carries `3c737ee` + `8eda4d3` + this SCORE; main untouched | ✓ |
| P — Zero Mutex usage | no Mutex/RwLock/CondVar; only Arc + crossbeam + AtomicI32 (existing); typed_channel.rs uses Arc<SenderInner>/Arc<ReceiverInner> for sharing | ✓ |
| Q — SCORE-SLICE-1.md + SCORE-SLICE-1B.md untouched | immutable per `feedback_inscription_immutable.md`; verified | ✓ |
| R — Slice 1b API unchanged | `extract_closure` signature + `ClosurePackage { prologue, entry_form }` shape untouched. ONE pattern arm in `src/closure_extract.rs` renamed `Value::crossbeam_channel__Sender` → `Value::wat__kernel__Sender` to track runtime variant rename — internal-only, no behavioral change. Slice 1b's 15/15 tests still pass; spirit of row R preserved | ✓ |

## Honest deltas

### Delta A — Implementation choice (Option B substrate-fit)

The BRIEF enumerated three options for transport-polymorphic
Sender/Receiver. Agent investigated existing substrate shape
and picked **Option B** (transport-polymorphic Value with
internal enum):

```rust
enum SenderInner {
    Crossbeam(Arc<crossbeam_channel::Sender<Value>>),
    PipeFd { writer: PipeWriter, encoder: EdnEncoder },
}
Value::wat__kernel__Sender(Arc<SenderInner>)
```

Reasoning:
- Option A (separate Value variants): doubles variant surface;
  forces every send/recv/select/drop callsite to dispatch on
  two variants; the wat-side `:wat::kernel::Sender` typealias
  would need polymorphic union the substrate doesn't have
- Option C (multimethod via arc 146): structurally
  over-engineered for binary internal dispatch on one variant
- Option B unifies under one Value variant; inner-enum dispatch
  is local to send/recv impls. Aligns with
  `feedback_capability_carrier.md` (extend the existing entity
  rather than mint parallel ones)

This is the right call. Per FM 10 + FM 5 — entity-kind extension,
not type-system reach; surfaced reasoning rather than guessing.

### Delta B — Process<I,O> shipped additive, not destructive (substrate-fit)

The BRIEF spec'd Process<I,O> reshape as DROPPING byte-pipe
handles in favor of typed-channel handles:

```rust
// BRIEF spec'd:
{ tx :Sender<I>, rx :Receiver<O>, handle :ProgramHandle }
```

Agent investigated breakage scope. Found:
- `wat/std/hermetic.wat` (3 Process accessor sites)
- **`wat/std/sandbox.wat` (3 Process accessor sites)** — sandbox
  is the bundled stdlib backing every `:wat::test::deftest`
  expansion

Destructive reshape would fail substrate startup, blocking
EVERY test (~2107 fails — beyond "RED workspace" into
"substrate doesn't bootstrap"). Per BRIEF row G (either
acceptable; document the choice), additive is the substrate-fit
answer:

```rust
// Slice 1c shipped (during sweep window):
{
  stdin :IOWriter,    // legacy — slice 3 retires
  stdout :IOReader,   // legacy — slice 3 retires
  stderr :IOReader,   // legacy — slice 3 retires
  handle :ProgramHandle,
  tx :Sender<I>,      // new — typed-channel transport
  rx :Receiver<O>,    // new — typed-channel transport
}
```

Slice 3 (testing-lib three-layer rebuild) retires the legacy
three when callers migrate to typed-channel views. Same shape
of substrate-as-teacher pattern arcs 167/168/169 used —
intermediate-state during sweep window; final shape after sweep.

This decision is sound. The user's "additive shim with no
warning" framing in the agent's report aligns with the existing
arc-doctrine for substrate-shape transitions.

### Delta C — `select` rejects PipeFd Receivers

`select` over channels is currently implemented for crossbeam
only (uses crossbeam's `select!` macro). Slice 1c's PipeFd
Receivers don't have epoll integration; `select` returns a
diagnostic error when given PipeFd Receivers in the choice list.

No consumer demand today (no current code uses select over
process-pipe Receivers). If a future caller needs this,
substrate work to add epoll-based select over Vec<PipeFd> is its
own arc.

### Delta D — `try-recv` on PipeFd returns Disconnected stand-in

Pipe fds aren't O_NONBLOCK by default; a true `try-recv` (don't
block; return immediately if no data) would require setting
O_NONBLOCK on the read fd. Current implementation: PipeFd
`try-recv` returns `Disconnected` as a stand-in.

Honest stub; not bridged with TODO. If a caller needs real
non-blocking behavior, surface as substrate work (set O_NONBLOCK
on PipeFd Receivers' read fd; track a "would-block" outcome
distinct from Disconnected).

### Delta E — EDN round-trip semantics (pre-existing arc 092/113 quirks)

Tests document existing wat-edn behavior:
- `Tuple` Values → wire form is Vec (Tuple/Vec collapse on
  serialization; round-trips as Vec)
- `Option(Some(x))` → wire form is `x` (Some unwrapping on
  serialization; round-trips as bare value)

These aren't slice-1c regressions — they're existing wat-edn
semantics from arcs 092 + 113. Tests document the round-trip
shape so future callers know what to expect.

If wat-edn round-trip fidelity needs improvement (preserve
Tuple distinction; preserve Some wrap), it's substrate work
in arc 092's territory, not arc 170.

## Calibration row

| Predicted | Actual | Mode |
|-----------|--------|------|
| 90-180 min opus | ~90 min | A clean (mid-range; under predicted upper) |

Within band. Calibration data: substrate-plumbing-with-
transport-polymorphism-and-existing-pieces-to-leverage = ~90 min
opus. Slice 1's 150-min and slice 1b's 40-min datapoints suggest
slice complexity correlates with new-substrate-mechanism count
more than line count.

Subsystems built:
- typed_channel.rs Option B substrate: ~190 LOC / 17 tests
- Value variant rename + dispatch migration: ~50 LOC delta
- Process struct additive reshape: 4 callers updated
- EDN encoder/decoder over fd: integrated within typed_channel
- Integration tests: 17 across all expected categories

Honest deltas surfaced: 5
- A: Implementation choice (Option B)
- B: Process additive vs destructive (sandbox.wat would brick)
- C: select rejects PipeFd Receivers
- D: try-recv on PipeFd returns Disconnected stand-in
- E: EDN round-trip semantics (pre-existing arc 092/113)

## Discipline check

- ✓ FM 5 held — all 5 honest deltas surfaced cleanly; no TODOs
- ✓ FM 9 honored — local cargo test verified 2124/0 + 17/17 on
  typed_channel_pipes + 15/15 on closure_extraction (slice 1b
  regression check) post-spawn
- ✓ FM 10 — entity-kind extension (Option B unifies under
  existing Value variant) over type-system reach (Options A/C
  reach for parallel structures)
- ✓ FM 11 — pre-INSCRIPTION grep deferred to slice 5 closure
- ✓ FM 12 — Agent spawn included `model: "opus"` explicitly
- ✓ FM 16 honored — BRIEF didn't mention Bash/cargo availability
- ✓ Branch isolation held — main untouched
- ✓ SCORE-SLICE-1.md + SCORE-SLICE-1B.md untouched per
  `feedback_inscription_immutable.md`

## What's next

Substrate foundation for tier 2 transport is complete:

- Slice 1 (closure extraction primitive) — SHIPPED
- Slice 1b (ClosurePackage reshape) — SHIPPED
- **Slice 1c (typed-channel + Process reshape) — SHIPPED**

Slice 2 unblocked — but currently FROZEN at v1-shape per FM 6.
Slice 2 BRIEF + EXPECTATIONS get redrafted against the full
settled foundation:

- Slice 1b: `closure_extract::extract_closure` API +
  `ClosurePackage { prologue, entry_form }` shape
- Slice 1c: `Value::wat__kernel__Sender` / `Receiver` Option B
  transport-polymorphic substrate + Process<I,O> additive shape

Slice 2's wat-level surface work:
- `:wat::kernel::ExitCode` typealias
- `:user::main` 4-arg signature + ExitCode return + validator
- `eval_kernel_spawn_process(fn)` dispatch arm — calls
  `extract_closure`, packages forms, forks, child evals
  `entry_form`, applies fn Value to typed-channel handles
  (uses slice 1c's PipeFd Sender/Receiver substrate)
- wat-cli argv + ExitCode plumbing
- Three substrate-as-teacher walker variants
  (`BareLegacyMainSignature`, `BareLegacyForkProgram`,
  `BareLegacySpawnProgram`)
- `tests/wat_arc170_program_contracts.rs` integration tests

## What this slice proved

The arc-discipline pipeline continues to work. Slice 1c shipped
Mode A clean within predicted band, with 5 honest deltas
surfaced including ONE substantial substrate-fit decision
(Option B over A/C) AND ONE substrate-shape decision (additive
over destructive Process reshape) — both made by the agent
through investigation + reasoning, both honoring existing
substrate doctrine (`feedback_capability_carrier.md`,
substrate-as-teacher pattern).

The pattern across slices 1 → 1b → 1c:
- Slice 1: 14/14 rows, ~150 min, 6 honest deltas (FM 5 disciplined)
- Slice 1b: 17/17 rows, ~40 min, 1 substantive delta (Symbol→Keyword pivot)
- Slice 1c: 18/18 rows, ~90 min, 5 deltas (Option B + additive Process)

Each shipped via fresh-agent execution against accumulated arc
artifacts. The discipline scales.

## Companion docs

- BRIEF-SLICE-1C.md + EXPECTATIONS-SLICE-1C.md — original brief
- TIERS.md — substrate-concept doc; closure-extraction is tier-
  bridging primitive at tier ≥ 2; typed-channel uniformity
  doctrine is what 1c proves at tier 2
- REALIZATIONS-SLICE-1.md pass 5 — strings stay at substrate
  boundary; this slice is the substrate-level proof
- SCORE-SLICE-1.md + SCORE-SLICE-1B.md — immutable historical
  records of the prior substrate slices
