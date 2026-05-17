# SCORE — Arc 203 Slice 3f: error propagation

**BRIEF:** `BRIEF-SLICE-3F.md` · **Shipped:** 2026-05-17 post-3e `cd6f261`.

## Scorecard

| Row | What | YES/NO | Evidence |
|-----|------|--------|----------|
| A | Both files parse + tests compile | YES | `cargo test --release -p wat --test test counter_service` builds clean; 4/4 pass |
| B | Happy paths still pass (Result/Ok extraction) | YES | All 11 happy-path steps in both files assert-eq on Ok-extracted values |
| C | At least one Err path demonstrated per file | YES | Thread tier: PeerDied/Disconnected (after stop) + AccessDenied (forge). Process tier: AccessDenied (forge) + ServerDied (crash-test-proc) |
| D | ServiceError uses typed errors (no String for chain) | YES | Thread: `PeerDied (chain :wat::core::Vector<wat::kernel::ThreadDiedError>)`. Process: `ServerDied (chain :wat::core::Vector<wat::kernel::ProcessDiedError>)`. No String anywhere |
| E | Workspace baseline preserved | YES | 3 pre-existing failures unchanged: `deftest_wat_tests_tmp_totally_bogus`, `startup_error_bubbles_up_as_exit_3`, `t6_spawn_process_factory_with_capture_round_trips`. 2 additional flakes under full parallel load pass in isolation. |

**Result: 5/5 YES.**

## Files touched

- `wat-tests/counter-service-capability-N3.wat` — thread tier, updated in-place
- `wat-tests/counter-service-process-N3.wat` — process tier, updated in-place

## Honest deltas from predictions

### BRIEF inaccuracy: error type shape

BRIEF specified `(PeerDied (cause :wat::kernel::ThreadDiedError))` with a single ThreadDiedError. The actual substrate type (confirmed from `src/check.rs` type signatures for `send` and `recv`) is `Result<(), Vector<ThreadDiedError>>` and `Result<Option<T>, Vector<ThreadDiedError>>` respectively — arc 113 widened the Err to a chain/backtrace. Implementation uses the honest type with field name `chain` instead of `cause`.

Both ServiceError variants updated:
- Thread tier: `(PeerDied (chain :wat::core::Vector<wat::kernel::ThreadDiedError>))`
- Process tier: `(ServerDied (chain :wat::core::Vector<wat::kernel::ProcessDiedError>))`

### Process/println and Process/readln do NOT return Result

BRIEF assumed transport errors at the process tier could be caught as Result in wrapper signatures. They cannot — `Process/println` and `Process/readln` panic (RuntimeError) on subprocess death; there is no Result return path. Only `Process/drain-and-join` and `Process/join-result` return Result.

Consequence: process-tier user wrappers (get-proc, increment-proc, reset-proc, deprovision-proc) can only surface `AccessDenied` via Result; transport failure still panics. This is honest: the substrate genuinely doesn't give a Result-returning send/recv for the process tier's stdio transport.

### Process/join-result is restricted

BRIEF suggested `Process/join-result` for subprocess crash detection. It is `#[restricted_to(":wat::kernel::Process/join-result", ":wat::")]` — inaccessible from `:counter::*` namespace. Used `Process/drain-and-join` instead (which is not restricted and returns `Result<nil, Vector<ProcessDiedError>>`).

### ServerDied via crash-test-proc

Since process-tier wrappers can't catch transport errors, the ServerDied Err path is demonstrated via a standalone helper `crash-test-proc` that spawns a fresh subprocess that panics, then calls `Process/drain-and-join` to detect the failure. This is structurally cleaner than the BRIEF suggested (which implied the counter service subprocess itself would be crashed); a dedicated minimal subprocess is the honest shape.

### Verbose Result-propagation confirmed

Every send site in the thread tier became a 4-level match (send Ok/Err → recv Ok/Err → Option Some/None → response variant). The process tier wrappers are 3-level (println/readln + WireResp outer variant + inner resp variant match). No `?` operator in wat; each layer is explicit. This is the expected cost per `feedback_verbose_is_honest`.

### Paren/whitespace mechanical issues

Three issues required repair:
1. Whitespace in `Result<X, Y>` type parameter brackets (lexer rejects it) — fixed with sed.
2. Nested bracket space `>, ` — fixed with sed.
3. Paren imbalance — thread tier: prelude list close was off by 1 (needed 7, had 6). Process tier: prelude list was not closed before the test body at all (deftest expects 3 args; prelude list must close before the test body form). Both corrected.

## BRIEF corrections for slice 3g (cache refactor)

1. **Error type shape**: Use `Vector<ThreadDiedError>` (chain), not single `ThreadDiedError`. Field name `chain`, not `cause`.
2. **Process-tier transport**: If 3g touches a process-tier service, note that Process/println + Process/readln do NOT return Result. ServerDied can only be shown via drain-and-join on a separate/crashed subprocess.
3. **Process/join-result restriction**: It's restricted to `:wat::` namespace; use `Process/drain-and-join` instead.
4. **Keyword whitespace**: No spaces inside `<>` in type params. No spaces after `>,` in nested brackets.
5. **Prelude list structure**: `deftest` takes exactly 3 args: `(name (prelude-forms...) test-body-expr)`. The prelude list must close `)` before the test body expression.
