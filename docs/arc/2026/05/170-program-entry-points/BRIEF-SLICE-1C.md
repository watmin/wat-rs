# Arc 170 slice 1c — typed-channel-over-EDN-pipes substrate + Process<I,O> reshape

## Goal

Mint the substrate plumbing that makes typed-channel
`Sender<T>` / `Receiver<T>` work over EDN-encoded pipes (tier 2
transport), and reshape `:wat::kernel::Process<I,O>` to expose
typed-channel handles instead of byte-pipe handles.

This slice is the substrate proof of EDN-as-transport: same
user-visible `Sender<T>` / `Receiver<T>` abstraction at tier 1
(crossbeam, in-memory typed Values) and tier 2 (EDN-over-pipes,
substrate handles encoding). Per memory `project_pipe_protocol.md`
("one protocol; four transports"), this is the substrate-level
manifestation of that doctrine.

**Zero wat-level surface change in this slice** — pure substrate
plumbing. The wat-level verbs that USE this infrastructure
(`spawn-process`, the `:user::main` 4-arg signature, walkers)
land in slice 2.

## Read first (in order)

1. `docs/arc/2026/05/170-program-entry-points/DESIGN.md` — full
   arc scope; tier framework; the doctrine that user-visible
   IPC stays uniform while transport varies
2. `docs/arc/2026/05/170-program-entry-points/TIERS.md` — load-bearing
   for this slice. Tier 1/2/3 abstraction unification; what
   disappears from user view; closure-extraction is the
   tier-bridging primitive at tier ≥ 2 (slice 1b shipped that)
3. `docs/arc/2026/05/170-program-entry-points/REALIZATIONS-SLICE-1.md` —
   pass 5 (strings at substrate boundary doctrine);
   `feedback_*` cross-references for the load-bearing principles
4. `docs/arc/2026/05/170-program-entry-points/EXPECTATIONS-SLICE-1C.md`
   — your scorecard
5. `docs/COMPACTION-AMNESIA-RECOVERY.md` § 6 (FM 5, FM 9, FM 11,
   FM 12, FM 16) — discipline floor

## Slice 1b context (just shipped)

Slice 1b commit `365343f` + SCORE `84b6ca6`. The closure-extraction
substrate primitive is in its corrected shape:

```rust
pub struct ClosurePackage {
    pub prologue: Vec<WatAST>,
    pub entry_form: WatAST,
}
```

For inline-lambda input: `entry_form` is the fn-form AST
`(:wat::core::fn [name <- :T ...] -> :Ret body)`.
For keyword-path input: `entry_form` is `WatAST::Keyword(path)`
naming a fn defined in `prologue`.

Slice 1c does NOT use `extract_closure` — that's slice 2's job
(spawn-process verb consumes it). Slice 1c lays the typed-channel
transport that slice 2's spawn-process will run extracted closures
ON.

## Branch + commit policy

- **Active branch**: `arc-170-program-entry-points` (carries
  slice 1 + 1b commits + SCOREs + this BRIEF when committed)
- Multiple WIP commits + pushes welcome
- DO NOT push to main; orchestrator merges atomic to main as one
  squash commit after slice 5 closure paperwork ships
- DO NOT edit SCORE-SLICE-1.md or SCORE-SLICE-1B.md (immutable
  per `feedback_inscription_immutable.md`)

## Substrate edits

### 1. Typed-channel-over-EDN-pipes transport

Today's `Value::crossbeam_channel__Sender` / `Receiver` (per
`src/runtime.rs:170-176`) are concrete crossbeam-only. Slice 1c
extends the substrate so `Sender<T>` / `Receiver<T>` can also
carry EDN-encoded values across linux-fd pipes.

Implementation choice — investigate and pick the substrate-fit
answer:

**Option A — separate Value variants per transport.** Add
`Value::pipe_channel__Sender(...)` and `Value::pipe_channel__Receiver(...)`
alongside the crossbeam variants. The wat-level send/recv verbs
dispatch on the Value variant; each variant has its own send/recv
implementation. Pro: explicit; clear which transport a channel uses.
Con: doubles the Value variants for channels; user-visible types
might need polymorphism support to unify them.

**Option B — transport-polymorphic Value with internal enum.**
Refactor `Value::crossbeam_channel__Sender` to carry an enum:

```rust
enum SenderInner {
    Crossbeam(Arc<crossbeam_channel::Sender<Value>>),
    PipeFd { writer: PipeWriter, encoder: EdnEncoder },
}
Value::wat__kernel__Sender(Arc<SenderInner>)
```

The send verb dispatches on the inner variant. Pro: ONE
Value variant; uniform send/recv interface; user code is
transport-agnostic. Con: bigger refactor; touches existing
crossbeam callsites.

**Option C — dispatch via the dispatch primitive (arc 146).**
Mint a Sender/send dispatch that selects implementation based
on the carried transport. Pro: aligned with arc 146's
multimethod entity. Con: more substrate machinery for what may
be a small dispatch problem.

Investigate. Pick the one that fits the substrate's existing
shape best. Surface the choice + reasoning. Per FM 5: don't
bridge; pick + ship. Per FM 10: prefer a substrate-fit answer
(entity kind addition where available) over type-system reach.

### 2. EDN encoding/decoding at the pipe boundary

`Sender<T>::send(value)` must encode the typed Value to EDN
bytes (via existing arc 092 wat-edn) and write to the pipe's fd.
`Receiver<T>::recv()` must read bytes from the pipe's fd, parse
as EDN (line-delimited per `project_pipe_protocol.md`), and
return the typed Value.

Build on existing pieces:
- `wat_edn::write` / `wat_edn::read` — encoding primitives
  (already shipped, arc 092)
- `PipeReader` / `PipeWriter` in `src/io.rs` — fd wrappers
- The fork-program-ast pathway already does EDN-over-stderr for
  arc 113 cascade; same encoding scheme applies here

The wire protocol stays line-delimited EDN (one Value per line).

### 3. `:wat::kernel::Process<I,O>` struct reshape

Today's `:wat::kernel::Process` Struct value (per `src/spawn.rs:220-230`
and `src/fork.rs:867-877`):

```rust
Value::Struct(Arc::new(StructValue {
    type_name: ":wat::kernel::Process".into(),
    fields: vec![
        Value::io__IOWriter(parent_stdin),    // stdin
        Value::io__IOReader(parent_stdout),   // stdout
        Value::io__IOReader(parent_stderr),   // stderr
        Value::wat__kernel__ProgramHandle(...), // join handle
    ],
}))
```

Reshape to:

```rust
Value::Struct(Arc::new(StructValue {
    type_name: ":wat::kernel::Process".into(),
    fields: vec![
        Value::wat__kernel__Sender(...),     // tx — parent feeds child
        Value::wat__kernel__Receiver(...),   // rx — parent reads child output
        Value::wat__kernel__ProgramHandle(...), // handle for join-result
    ],
}))
```

stderr drops as a separate field; errors propagate via
`Process/join-result` returning `Result<(), ProcessDiedError>`
(existing pattern).

The wat-side struct definition for `:wat::kernel::Process<I,O>`
must update to match (in `wat/kernel/...wat` or wherever the
struct is declared).

### 4. Caller updates for new Process shape

Search for callers of `Process/stdin`, `Process/stdout`,
`Process/stderr` field accessors:

- `wat/std/hermetic.wat` (lines 91, 102-103) — these break;
  but hermetic.wat is slice 3's territory (rebuild to typed
  channels). For slice 1c, add legacy field-accessor shims OR
  let hermetic.wat break (slice 3's input). Investigate which
  is honest:
  - If breaking is acceptable for slice 1c (per arc 168
    precedent for substrate breakage in slice 1, sweep in slice
    2), let it break. Slice 1c ships RED workspace; slice 2
    sweep restores green via the wat-level verb update.
  - If breaking is NOT acceptable (arc 109 was-already-progress
    invariant?), add temporary shims that warn-and-proceed.
  - Investigate; pick; surface.

- `src/fork.rs:867-877` — the Process struct construction site
  in `eval_kernel_fork_program`. Update to construct with new
  fields.
- `src/spawn.rs:220-230` — same in `eval_kernel_spawn_program{,_ast}`.
- `src/runtime.rs` Process struct accessors (Process/stdin,
  Process/stdout, Process/stderr) — these field paths break
  for the renamed/dropped fields. Update or retire as
  appropriate.

The Process<I,O> Type definition (in stdlib wat) needs the
new field shape.

### 5. Rust integration tests

New test file (or extend existing) — verify:

- Tier 2 round-trip: parent creates pipe-channel Sender<i64> +
  Receiver<i64>; sends a typed i64 Value through the Sender;
  Receiver yields the same typed Value back. The bytes flowing
  through the pipe are EDN-encoded; user-side code never sees
  them.
- Multi-Value: stream of N typed Values round-trips correctly
  in order
- Type fidelity: complex Values (nested structs, vectors, etc.)
  round-trip without loss
- Error propagation: pipe close on the writer side surfaces as
  Receiver/recv → Result.Err (or whatever the existing
  channel-disconnect signal is)
- Process<I,O> Struct accessor smoke test: a fabricated Process
  Value with typed-channel fields exposes Sender + Receiver
  correctly via field access

Predicted: 5-10 integration tests; place in
`tests/wat_arc170_typed_channel_pipes.rs` or extend the existing
arc170 tests file (your call; pick the cleanest organization).

## What slice 1c does NOT do

- **No spawn-process verb wiring** — that's slice 2. Slice 1c
  ships the substrate plumbing; slice 2 consumes it.
- **No `:user::main` signature changes** — slice 2.
- **No walker variants** — slice 2.
- **No wat-level surface changes** — pure substrate. The wat-side
  Process struct definition updates because its FIELDS change,
  but no new wat-level verbs are minted.
- **No SCORE edits to slice 1 or 1b** — immutable.

## Honest delta categories (if surfaced, report; don't bridge)

- **Implementation choice for Sender/Receiver transport
  polymorphism** — Option A vs B vs C above. Pick + report
  reasoning. If the substrate's existing shape strongly suggests
  one, go with it. Surface the choice.
- **EDN encoding at pipe boundary line-delimited or framed?**
  Today's arc 113 cascade uses `\n` delimiters for stderr-EDN.
  Apply the same convention to the typed-channel pipes. If you
  hit edge cases (Values containing newlines in their string
  representations, etc.), surface — wat-edn likely escapes
  them, but verify.
- **Process<I,O> caller breakage scope** — investigate how many
  sites use `Process/stdin`, `Process/stdout`, `Process/stderr`
  accessors. If small (<5), update inline. If larger, surface;
  may need a slice-1d rather than bundling.
- **Error semantics on pipe close** — when the parent's Sender
  writes to a pipe whose reader has gone away, the syscall
  returns EPIPE. How does this surface to wat code? Per arc 111
  (Result-typed send/recv), this should map to Result.Err on
  the next send. Verify the existing crossbeam → pipe symmetry.
- **Slice 1 honest delta D reprise** — diagnostic type-name
  spelling. If your work touches the NonPortableCapture
  diagnostic, the runtime-vs-source-spelling gap from slice 1
  is still present. Don't fix it in slice 1c (out of scope);
  surface if you trip over it.
- **FM 5 trap** — if you find yourself wanting to leave a TODO
  or skip a hard case, STOP. Surface as honest delta.

## Critical syntax shapes (for any wat-side struct edits)

Per arc 167 + arc 109 + arc 153 doctrines:

- Type definitions stay in their existing FQDN form (struct fields
  use `(name :Type)` shape per `wat/...kernel.wat` precedent —
  NOT the fn-flat-vector shape; struct fields are different from
  fn parameters)
- `:wat::core::nil` (NOT bare `:nil`) for unit type if it appears
- Wat type expressions: `:wat::kernel::Sender<I>` (no inner colon
  before generic — per `feedback_wat_colon_quote.md`); no
  whitespace inside `<>`.

## Predicted runtime

90-180 minutes (opus). Time-box hard cap at 360 minutes.

Comparable to slice 1's prediction (slice 1 = 90-180; actual
~150). Slice 1c involves new substrate primitives (transport-
polymorphic Sender/Receiver) + value type reshape (Process<I,O>)
+ caller updates + tests.

Smaller than a from-scratch slice because the existing
crossbeam Sender/Receiver substrate + arc 092 wat-edn + the
arc 113 cascade pattern + the existing pipe machinery in
fork.rs/spawn.rs all compose into the answer; the work is
extending and integrating, not minting from zero.

## Branch state at slice 1c start

```
$ git log --oneline -5
84b6ca6 (HEAD -> arc-170-program-entry-points)
   arc 170 slice 1b: SCORE — 17/17 rows pass, Mode A clean, ~40 min
365343f arc 170 slice 1b: T1-T15 assertion-shape updates + entry_form Keyword
a23acf3 arc 170 slice 1b: ClosurePackage shape + entry resolution + assembly
15ac2d8 arc 170: syntax fixes (flat-vector + <- arrow + FQDN nil) + Layer 1 macro hides fn ceremony
20968ee arc 170: strings stay at substrate boundary; user works in forms at every tier
```

`cargo test --workspace` baseline at slice 1c start: `passed:
2107 failed: 0`.

Post-slice-1c expected: depends on caller-breakage decision (see
honest delta section). If hermetic.wat breaks intentionally,
expect ~10-30 fail count from hermetic-using tests; that breakage
is slice 2/3's input. If hermetic.wat gets shimmed, workspace
stays green; slice 2 is responsible for the migration.

## SCORE artifact

After slice 1c ships, orchestrator writes SCORE-SLICE-1C.md
(scorecard from EXPECTATIONS-SLICE-1C + honest deltas +
calibration row). You report to chat; orchestrator owns the
SCORE artifact + commit (closure paperwork orchestrator-side
per `feedback_paperwork_orchestrator_side.md`).
