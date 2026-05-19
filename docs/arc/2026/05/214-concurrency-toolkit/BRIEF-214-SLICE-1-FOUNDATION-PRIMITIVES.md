# Arc 214 Slice 1 — Foundation primitives

## Mission

Mint the foundational trait shapes + signatures + error types for arc 214's concurrency toolkit in a NEW file `src/comms/mod.rs`. **NO implementations** — this slice ships ONLY the type-level shape that Slices 2/3 implement and Slices 4-8 consume.

Atomic single-stepping-stone scope. Per the DESIGN (`docs/arc/2026/05/214-concurrency-toolkit/DESIGN.md`): this is Slice 1 of 9 in the comprehensive concurrency arc.

## Substrate context (substrate-truth verified pre-spawn)

- **HolonAST** lives in the external `holon` crate (imported as `use holon::HolonAST`). The wat-rs substrate uses it via `crate::lower::lower` (returns `HolonAST`), `crate::hologram::slot_for_form` (operates on `&HolonAST`), etc.
- **EDN serialization** for HolonAST exists via `crate::edn_shim::read_holon_ast_tagged` / `read_holon_ast_natural` (parsing) and `crate::edn_shim::value_to_edn_with` (for Value, not HolonAST directly — note this asymmetry).
- **`src/lib.rs`** has alphabetically-ordered `pub mod` declarations; `comms` slots between `closure_extract` (line 61) and `compose` (line 62). Add `pub mod comms;` at the alphabetically correct position.
- **No existing `WireError` type** — Slice 1 mints it fresh.
- **No existing `src/comms/` directory** — Slice 1 creates it.

## Concrete deliverables

### 1. Create `src/comms/mod.rs` with the following type-level shapes

**`HolonRepresentable` trait** — anything that can travel ANY substrate concurrency boundary:

```rust
/// Universal wire form for cross-boundary types. Anything that crosses a
/// process or remote tier boundary must roundtrip through HolonAST (substrate's
/// universal "Any" form per arc 057+ project_holon_universal_ast).
///
/// Thread-tier (in-process) channels can also use HolonRepresentable types,
/// but pass T directly via crossbeam (no serialization roundtrip).
///
/// Per project_holon_universal_ast (the strange loop closing 2026-05-19): HolonAST
/// was minted for VSA encoding (arc 057), became universal AST (arc 143 signature
/// reflection, arc 201 type reflection), and is NOW also the universal comms wire
/// form.
pub trait HolonRepresentable: Send + 'static {
    fn to_holon_ast(&self) -> holon::HolonAST;
    fn from_holon_ast(ast: &holon::HolonAST) -> Result<Self, WireError>
    where Self: Sized;
}
```

**Blanket impl decision:** if a clean blanket impl is possible (e.g., `T: Into<HolonAST> + TryFrom<HolonAST>` with appropriate bounds; cloning OK), include it. If trait-bound complexity exceeds value (Into consumes self; HolonRepresentable::to_holon_ast takes &self), OMIT the blanket and document: *"Manual `impl HolonRepresentable for T` per type; blanket impl deferred because Into<HolonAST> consumes self and would require T: Clone overhead at every send."* Sonnet picks the call based on what compiles cleanly.

**`CommSender<T>` and `CommReceiver<T>` traits** — tier-agnostic abstraction; implemented by `wat::comms::thread::Sender<T>` (Slice 2) and `wat::comms::process::Sender<T>` (Slice 3); enables tier-agnostic generic functions for brackets + services:

```rust
pub trait CommSender<T> {
    fn send(&self, value: T) -> Result<(), SendError<T>>;
    fn close(self) -> Result<(), CloseError>;
}

pub trait CommReceiver<T> {
    /// Cascade-aware blocking recv. Wakes on substrate shutdown (returns
    /// Err(RecvError::Shutdown) or equivalent — see RecvError variants).
    fn recv(&self) -> Result<T, RecvError>;
    fn try_recv(&self) -> Result<T, TryRecvError>;
    fn len(&self) -> usize;
    fn close(self) -> Result<(), CloseError>;
}
```

**Error types** — match `crossbeam_channel` ergonomics where possible (callers familiar with crossbeam will recognize the shapes):

```rust
/// Send failed: receiver was dropped or substrate shut down.
#[derive(Debug)]
pub struct SendError<T>(pub T);

/// Recv failed: all senders dropped or substrate shut down.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct RecvError;

/// Non-blocking recv: either empty queue OR disconnected.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TryRecvError {
    Empty,
    Disconnected,
}

/// Close failed (rare; e.g., FD already closed).
#[derive(Debug)]
pub struct CloseError(pub String);

/// HolonAST roundtrip failure during wire serialization/deserialization.
#[derive(Debug)]
pub struct WireError(pub String);
```

**`SelectOutcome<T>` enum** — result of a cascade-aware fan-in select:

```rust
pub enum SelectOutcome<T> {
    /// One of the registered receivers fired; (user-index, result).
    Recv(usize, Result<T, RecvError>),
    /// Substrate shutdown fired; caller should unwind.
    Shutdown,
}
```

**Cascade contract documentation** — module-level doc comment in `src/comms/mod.rs`:

```rust
//! # Comms layer — substrate-internal tier primitives
//!
//! Layer 0a of arc 214's concurrency toolkit. This module defines the
//! tier-agnostic abstractions (HolonRepresentable wire form, CommSender /
//! CommReceiver traits, error types, SelectOutcome) shared by the thread
//! tier (`comms::thread`) and process tier (`comms::process`) implementations.
//!
//! ## Cascade contract (LOAD-BEARING)
//!
//! Every blocking method on tier-specific Receivers + Selects MUST wake on
//! substrate shutdown:
//!
//! - Thread tier: `crossbeam_channel::select! { recv(data), recv(SHUTDOWN_RX) }`
//!   — substrate's shutdown cascade signals via crossbeam channel; tier recv
//!   includes this in its select arm.
//! - Process tier: `io_uring` multi-arm submission on [data_fd, broadcast_fd]
//!   — substrate's broadcast pipe acts as the wake signal; first completion
//!   wins.
//!
//! Callers cannot bypass the cascade because tier wrappers hide the underlying
//! mechanism. Bare `crossbeam_channel::*` and bare `libc::pipe/read/write/
//! poll/epoll/io_uring_*` are unreachable outside the tier wrapper modules
//! (Slice 6 structural wall).
//!
//! ## Audience
//!
//! - **Substrate authors** (building brackets, services, kernel-layer dispatch)
//!   use this module directly via `crate::comms::thread::*` / `crate::comms::
//!   process::*`.
//! - **User code** does NOT touch this layer; uses peer-oriented `:wat::kernel::*`
//!   verbs (Slice 4) that internally dispatch to the right tier.
```

### 2. Add `pub mod comms;` to `src/lib.rs`

Insert at the alphabetically correct position (between `pub mod closure_extract;` line 61 and `pub mod compose;` line 62).

### 3. Create smoke probe `tests/probe_comms_foundation.rs`

```rust
//! Arc 214 Slice 1 smoke probe — verify foundation primitives compile + a
//! sample HolonRepresentable impl roundtrips.

use wat::comms::{HolonRepresentable, WireError};

// Sample impl — verifies the shape is usable
struct ToyType(i64);

impl HolonRepresentable for ToyType {
    fn to_holon_ast(&self) -> holon::HolonAST {
        // Minimal valid HolonAST construction for an i64-bearing wrapper.
        // Sonnet picks the simplest valid HolonAST variant for this; the
        // probe doesn't verify VSA semantics — just shape + roundtrip.
        unimplemented!("sonnet: pick simplest HolonAST variant carrying self.0")
    }
    fn from_holon_ast(ast: &holon::HolonAST) -> Result<Self, WireError> {
        unimplemented!("sonnet: inverse of to_holon_ast")
    }
}

#[test]
fn probe_slice1_holon_representable_compiles() {
    let t = ToyType(42);
    let ast = t.to_holon_ast();
    let t2 = ToyType::from_holon_ast(&ast).expect("roundtrip");
    assert_eq!(t.0, t2.0);
}

#[test]
fn probe_slice1_error_types_construct() {
    // Verify error types are publicly constructible (shape verification only)
    let _s = wat::comms::SendError(42i64);
    let _r = wat::comms::RecvError;
    let _t = wat::comms::TryRecvError::Empty;
    let _c = wat::comms::CloseError("test".into());
    let _w = wat::comms::WireError("test".into());
}

#[test]
fn probe_slice1_select_outcome_constructs() {
    let _ok: wat::comms::SelectOutcome<i64> = wat::comms::SelectOutcome::Recv(0, Ok(42));
    let _err: wat::comms::SelectOutcome<i64> = wat::comms::SelectOutcome::Recv(1, Err(wat::comms::RecvError));
    let _shutdown: wat::comms::SelectOutcome<i64> = wat::comms::SelectOutcome::Shutdown;
}
```

For the `ToyType` HolonAST construction — sonnet picks the simplest valid HolonAST variant carrying an i64. The substrate's `holon` crate has constructors; e.g., `HolonAST::Symbol("toy".into())` or `HolonAST::leaf(...)` — sonnet investigates the holon crate's API briefly + picks the simplest variant. The test verifies SHAPE not semantics.

## Verification

```
cargo build --release                                       # must be clean
cargo test --release --test probe_comms_foundation          # 3/3 PASS
```

Per `feedback_no_hang_vector_in_additive_scorecard`: **DO NOT run `cargo test --release --test wat_arc170_program_contracts`** or any other workspace tests. This slice is purely additive; cargo build clean + 3-test smoke probe is the FULL gate.

## Out of scope (STOP triggers)

- **DO NOT implement `Sender` / `Receiver` / `Select` types.** Those are Slice 2 (thread tier) and Slice 3 (process tier).
- **DO NOT add `wat::comms::thread::*` or `wat::comms::process::*` submodules.** Those are Slice 2/3.
- **DO NOT touch the dirty tree** (`src/fork.rs` + `src/spawn_process.rs` — arc 213 δ-1 replication; precious per `feedback_defect_fix_or_panic_never_revert`).
- **DO NOT touch `src/typed_channel.rs`** (Layer 0 chokepoint we built in arc 213 χ; ships through unaltered).
- **DO NOT run `wat_arc170_program_contracts`** or any workspace tests (per `feedback_no_hang_vector_in_additive_scorecard`).
- **DO NOT modify any file outside the 3-file scope:** `src/comms/mod.rs` (new), `src/lib.rs` (1-line addition), `tests/probe_comms_foundation.rs` (new), and SCORE doc.

If any of these triggers fires: STOP, write the SCORE with honest report, do NOT iterate-to-green.

## Concrete deliverables list

1. New file: `src/comms/mod.rs` (~80-120 LOC: traits + errors + SelectOutcome + cascade-contract module doc; NO impls)
2. Edit: `src/lib.rs` (1 line: `pub mod comms;` alphabetically inserted)
3. New file: `tests/probe_comms_foundation.rs` (~40-60 LOC: 3 smoke tests)
4. SCORE doc: `docs/arc/2026/05/214-concurrency-toolkit/SCORE-214-SLICE-1-FOUNDATION-PRIMITIVES.md`

## Critical constraints

- DO NOT commit. Orchestrator commits after independent SCORE verification.
- Anchor cwd: `/home/watmin/work/holon/wat-rs/` — `pwd` as first action; reject any `.claude/worktrees/` path (harness state; illegal).
- Use `git -C` for any git operations.

## Cross-references

- `docs/arc/2026/05/214-concurrency-toolkit/DESIGN.md` — full arc 214 design; Slice 1 description
- `crates/holon` or external — HolonAST source
- `src/edn_shim.rs:1997` — HolonAST EDN reading precedent
- `src/runtime.rs:45` — `use holon::HolonAST` import pattern
- `project_holon_universal_ast` — the strange-loop closing (HolonAST as universal wire form)
- `feedback_no_hang_vector_in_additive_scorecard` — why no wat_arc170 verification
- `feedback_defect_fix_or_panic_never_revert` — dirty tree preservation
- `feedback_iterative_complexity` — atomic stepping stone discipline
