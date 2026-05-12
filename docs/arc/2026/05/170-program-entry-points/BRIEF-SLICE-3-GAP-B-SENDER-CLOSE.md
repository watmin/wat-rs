# Arc 170 slice 3 — Gap B BRIEF (Sender/close — explicit EOF signaling)

**Sonnet.** Closes the substrate gap Phase C Delta 3 / Phase D delta 3 surfaced: no way for a wat-level holder of a `:wat::kernel::Sender<T>` to signal end-of-stream WITHOUT dropping the Value. Currently EOF on a typed channel only happens via Sender drop (scope end). For Layer 2 streaming patterns where the child reads-until-EOF on `rx`, the parent has no way to say "I'm done sending; you can exit your loop" while keeping the rest of the Process Value alive (handle / rx side).

This parallels `:wat::io::IOWriter/close` for byte-stream writers — that primitive exists today. Sender/close is its typed-channel equivalent.

## The form (locked name)

```
(:wat::kernel::Sender/close  (s :wat::kernel::Sender<T>) -> :wat::core::nil)
```

- Idempotent (matching IOWriter/close convention).
- After close, subsequent `(:wat::kernel::send s v)` returns `Result.Err(ChannelDisconnected)` — same shape that crossbeam-disconnect or pipe-EOF surfaces today via the recv side. Send-after-close is graceful, not a panic.
- For tier-1 (Crossbeam) Senders: flipping the closed flag is sufficient; subsequent send checks the flag.
- For tier-2 (PipeFd) Senders: flipping the closed flag AND calling shutdown(2) on the underlying write fd, so the OS-pipe side actually surfaces EOF to the child's reader.

## Required reading IN ORDER

1. **`src/typed_channel.rs:69-130`** — `SenderInner` enum + constructors (`sender_from_crossbeam`, `sender_from_pipe`)
2. **`src/typed_channel.rs:130+`** — `SendOutcome` enum + `typed_send` function — the existing send path that needs the closed-flag check
3. **`src/io.rs:1111+`** — `IOWriter/close` — existing close pattern (byte-stream sibling); look at how it handles idempotency + how it surfaces to the user
4. **`src/runtime.rs:14917`** — `eval_kernel_send` — the wat-level send dispatch arm (needs to consult the closed flag via typed_send)
5. **`src/runtime.rs:3648`** — kernel-verb dispatch table; `Sender/close` lives here as a new arm
6. **`src/runtime.rs:3492`** — `IOWriter/close` dispatch arm for reference
7. **`docs/ZERO-MUTEX.md`** — verify AtomicBool is the right primitive (it is; atomics permitted under zero-Mutex doctrine)
8. **`docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-3-PHASE-C-LAYER1.md`** — Delta 3 documents the gap
9. **`docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-3-PHASE-D-LAYER2.md`** — D3 reaffirms the gap

## Implementation path

### Phase 1 — Add closed flag to Sender; check on send

Wrap or extend `SenderInner` so each Sender carries an `AtomicBool` for closed state. Two viable shapes:

**Option A — flag inside SenderInner:**
```rust
pub enum SenderInner {
    Crossbeam {
        sender: crossbeam_channel::Sender<Value>,
        closed: AtomicBool,
    },
    PipeFd {
        writer: Arc<dyn WatWriter>,
        closed: AtomicBool,
    },
}
```

**Option B — wrapper struct around SenderInner:**
```rust
pub struct SenderState {
    inner: SenderInner,
    closed: AtomicBool,
}
// Value::wat__kernel__Sender(Arc<SenderState>)
```

Recommend Option A (variant-local fields; symmetric per transport; no extra indirection). Surface choice in SCORE.

`typed_send` reads the AtomicBool BEFORE the existing transport send; if closed, returns `SendOutcome::ChannelDisconnected` (or whatever the existing Disconnected variant is). The error reaches the wat-level Result.Err arm naturally — no new error variant needed.

### Phase 2 — `Sender/close` runtime primitive

Add eval function (likely in `src/runtime.rs` or `src/typed_channel.rs`):

```rust
pub fn eval_kernel_sender_close(args: &[WatAST], env: &Environment, sym: &SymbolTable) -> Result<Value, RuntimeError>
```

- Verify arity (1 arg)
- Eval arg; expect `Value::wat__kernel__Sender(inner)`
- For Crossbeam: `inner.closed.store(true, Ordering::SeqCst)`; subsequent send checks
- For PipeFd: set closed flag AND call `shutdown(fd, SHUT_WR)` to close the OS pipe write end (so the child's reader sees EOF)
- Idempotent: closing a closed Sender is a no-op + returns nil
- Return `Value::Unit` (nil)

Register dispatch arm in `runtime.rs::eval_call` table next to other `Sender/*` methods (or with `kernel::send` siblings).

Add type-check scheme in `src/check.rs`:
```
:wat::kernel::Sender/close : ∀T. (Sender<T>) -> :wat::core::nil
```

### Phase 3 — Unit tests

Tests in the typed_channel test module OR a new test file:

1. **Crossbeam close-then-send returns Err**: construct a tier-1 Sender, send one Value (succeeds), close, send another → Err(disconnected)
2. **Crossbeam close-then-recv sees disconnect**: same setup; the paired Receiver sees the Disconnected on the next recv after close
3. **PipeFd close-then-send returns Err**: same shape via pipe-fd transport
4. **PipeFd close triggers reader EOF**: parent closes the Sender; child's Receiver-pipe reads Ok(None) (clean EOF)
5. **Idempotency**: close twice → second call is no-op (no error, returns nil)
6. **Wat-level integration test**: a small program that creates a Sender via existing means, calls `(:wat::kernel::Sender/close s)`, verifies subsequent `(:wat::kernel::send s ...)` returns Err

## Scope (what's IN)

- `AtomicBool closed` flag on Sender (Option A or B — sonnet picks)
- `:wat::kernel::Sender/close` runtime primitive + dispatch arm + type scheme
- `typed_send` checks the flag before transport send
- Rust unit tests for the new primitive + integration test for the wat-level form
- `cargo check --release` green
- Workspace stays at 0 failed
- SCORE doc

## Scope (what's OUT)

- `:wat::kernel::Receiver/close` — symmetric form for the receiving end. Not blocked by anything; just out of THIS slice's scope. Surface as a follow-up suggestion in SCORE if relevant.
- Phase E (deftest consumer sweep) — separate
- Slice 4 (BareLegacy walker + retired-verb eval arms + Process<I,O> legacy fields) — separate
- Future Layer 2 streaming-pattern test that EXERCISES Sender/close end-to-end — could land in this slice as the "wat-level integration test" item, OR as a follow-up if compose-with-Layer-2 is non-trivial. Sonnet decides; surface choice.

## Ship criteria (8 rows)

| Row | What | Pass criterion |
|-----|------|----------------|
| A | `AtomicBool closed` flag on Sender (Option A or B) | grep |
| B | `:wat::kernel::Sender/close` runtime primitive registered + dispatched | grep + cargo check |
| C | `typed_send` checks closed flag before transport send | grep + unit test (close-then-send returns Err) |
| D | Close-then-send returns `Err` for both Crossbeam + PipeFd transports | unit tests |
| E | PipeFd close triggers reader EOF on child side (shutdown(SHUT_WR) called) | unit test (Receiver.recv after Sender close returns Ok(None)) |
| F | Close is idempotent (calling twice is a clean no-op) | unit test |
| G | Workspace stays at 0 failed (count rises by N for new unit tests; existing tests unaffected) | full workspace cargo test |
| H | `cargo check --release` green | clean |

**8 rows.** All must pass.

## Predicted runtime

**60-120 min sonnet.** Substrate work touches typed_channel.rs (Sender shape) + runtime.rs (dispatch + eval fn) + check.rs (type scheme) + tests.

**Hard cap:** 240 min.

## Constraints (hard)

- DO NOT commit. Orchestrator atomic-commits after scoring verification.
- DO NOT use Mutex / RwLock / CondVar — zero-Mutex doctrine. AtomicBool is permitted.
- DO NOT modify `:wat::test::run-hermetic` / `run-hermetic-with-io` macros (they keep working unchanged; Sender/close is a new capability, not a contract change)
- DO NOT touch `deftest` / `deftest-hermetic`
- DO NOT touch BareLegacy* walker / spawn.rs / Process<I,O> struct fields
- DO NOT add Receiver/close in this slice (out of scope)
- DO NOT use deferral language in SCORE — per FM 11
- If shutdown(SHUT_WR) on the PipeFd side surfaces an ownership / fd-lifetime issue with the `Arc<dyn WatWriter>` shape, STOP and report — do not workaround

## Honest delta categories (anticipated)

1. **Option A vs B for the closed flag** — variant-local vs wrapper struct; rationale
2. **PipeFd shutdown mechanism** — how shutdown(SHUT_WR) gets called with the `Arc<dyn WatWriter>` shape; any ownership wrangling
3. **Where eval_kernel_sender_close lives** — typed_channel.rs vs runtime.rs; rationale
4. **Sender Value mutability surface** — Sender Value is currently `Arc<SenderInner>` (immutable). The AtomicBool inside doesn't violate immutable-Arc doctrine (interior mutability via atomics is the standard zero-Mutex pattern); confirm this aligns with ZERO-MUTEX.md
5. **Anything unexpected** — surfaced during authorship

## Cross-references

- Gap A SCORE: [`SCORE-SLICE-3-GAP-A-KEYWORD-REFLECTION.md`](./SCORE-SLICE-3-GAP-A-KEYWORD-REFLECTION.md)
- Phase C / D SCOREs document the gap (Delta 3 in C, D3 in D)
- IOWriter/close precedent: `src/io.rs:1111`
- ZERO-MUTEX doctrine: `docs/ZERO-MUTEX.md`
- Future: Receiver/close as symmetric form (out of this slice's scope)
