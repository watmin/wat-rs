# Arc 170 slice 3 — Gap B EXPECTATIONS (sonnet scorecard)

**One spawn.** Substrate addition: `AtomicBool closed` flag on `SenderInner` + `:wat::kernel::Sender/close` runtime primitive + send-side check.

## Independent prediction

**Runtime band:** 60-120 min sonnet. Substrate touches typed_channel.rs + runtime.rs + check.rs + tests.

**Hard cap:** 240 min.

## Scorecard (8 rows)

| Row | What | Pass criterion |
|-----|------|----------------|
| A | `AtomicBool closed` flag added to `SenderInner` (Option A or B) | grep `AtomicBool` near `SenderInner` |
| B | `:wat::kernel::Sender/close` registered as a runtime primitive + dispatched | grep `Sender/close` in src/runtime.rs |
| C | `typed_send` consults the closed flag before transport send | grep + unit test |
| D | Close-then-send returns Err for Crossbeam AND PipeFd transports | unit tests |
| E | PipeFd close triggers reader EOF (shutdown(SHUT_WR)) | unit test |
| F | Close is idempotent | unit test |
| G | Workspace stays at 0 failed | full cargo test |
| H | `cargo check --release` green | clean |

**8 rows.** All must pass.

## Implementation approach

Phase 1 — flag on SenderInner:
- Choose Option A (variant-local AtomicBool) or Option B (wrapper struct); recommend A
- Update `sender_from_crossbeam` and `sender_from_pipe` constructors to initialize the flag to false
- Update existing match arms that destructure `SenderInner` (in typed_send + any other) to ignore the new field — surface count + any cleanup needed

Phase 2 — close primitive:
- `eval_kernel_sender_close` extracts the inner, sets closed=true, and for PipeFd calls `shutdown(fd, SHUT_WR)`
- For shutdown, the `Arc<dyn WatWriter>` may need to surface its underlying fd — sonnet investigates `WatWriter` trait to find the right hook (extract via downcast, or add a method on WatWriter for `shutdown_write`)
- Dispatch arm in runtime.rs near other Sender/* methods OR near :wat::kernel::send
- Type-check scheme in check.rs: `∀T. (Sender<T>) -> :nil`

Phase 3 — typed_send check:
- At the top of `typed_send`, check `closed.load(Ordering::SeqCst)` (or Acquire)
- If true, return `SendOutcome::Disconnected` (or whatever the existing Err variant is for already-disconnected channel) — same shape natural disconnect produces

Phase 4 — tests:
- Unit tests in typed_channel.rs or a new test module covering Crossbeam + PipeFd close-then-send, PipeFd EOF, idempotency
- Wat-level integration test demonstrating end-to-end close → send-returns-Err

## What sonnet should produce

1. **Code changes:**
   - `src/typed_channel.rs` — SenderInner shape update; typed_send flag-check; constructor updates
   - `src/runtime.rs` — eval_kernel_sender_close fn + dispatch arm
   - `src/check.rs` — type-check scheme for `Sender/close`
   - Unit tests in appropriate modules
   - Possibly a wat-level integration test in `tests/wat_arc170_program_contracts.rs` or sibling
2. **SCORE doc:** `docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-3-GAP-B-SENDER-CLOSE.md` following Gap A SCORE structure
3. **Do NOT commit.** Orchestrator atomic-commits after scoring verification.

## What sonnet should NOT do

- Do NOT use Mutex / RwLock / CondVar — AtomicBool only
- Do NOT add Receiver/close in this slice
- Do NOT modify Layer 1/2 macros / drivers
- Do NOT touch deftest / deftest-hermetic
- Do NOT touch BareLegacy* / spawn.rs / Process struct
- Do NOT use deferral language in SCORE
- If PipeFd shutdown mechanism requires non-trivial trait additions to WatWriter, STOP and report — surface as substrate-architectural finding

## Tools required

- Read / Edit / Bash (cargo, git, grep)
- Write for SCORE doc
- No Agent invocations

## Verification commands

```bash
# Baseline
cargo test --release --workspace --no-fail-fast 2>&1 | \
  grep "^test result" | awk -F'[: ;]' '{p+=$5;f+=$8} END {print "passed:" p " failed:" f}'

# Form presence
grep -n "Sender/close" src/runtime.rs src/check.rs
grep -n "AtomicBool\|closed" src/typed_channel.rs | head

# Specific tests
cargo test --release sender_close 2>&1 | tail -10

# Final workspace
cargo test --release --workspace --no-fail-fast 2>&1 | \
  grep "^test result" | awk -F'[: ;]' '{p+=$5;f+=$8} END {print "passed:" p " failed:" f}'
```

## Expected workspace delta

- Baseline: 2193 passed / 0 failed
- Post Gap B: 2193+N passed / 0 failed (N = new unit + integration tests; ~3-6 likely)

## Honest delta categories (anticipated)

1. **Option A vs B for the flag shape**
2. **PipeFd shutdown mechanism through `Arc<dyn WatWriter>`** — whether WatWriter trait needed a method or if downcast worked
3. **eval_kernel_sender_close location** — typed_channel.rs vs runtime.rs; rationale
4. **Receiver/close note** — surface as follow-up suggestion in SCORE
5. **Anything unexpected** — surfaced during authorship
