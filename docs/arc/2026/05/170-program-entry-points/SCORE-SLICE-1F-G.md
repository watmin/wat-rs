# Arc 170 slice 1f-γ — SCORE

**Result:** Mode A clean. 14/14 rows pass (Row N — commit message — orchestrator's job, satisfied at commit time).
**Runtime:** ~75 min opus (well within predicted 60-120 band; well under 240 hard cap).
**Files:** 3 modified + 1 new — `src/thread_io.rs` (+625 lines), `src/runtime.rs` (+90 lines), `src/freeze.rs` (+285 lines), `tests/wat_arc170_slice_1f_gamma_orchestrator.rs` (new, 300 lines).

**The runtime is now the orchestrator.** Wat programs that call `(:wat::kernel::println v)` route through the three substrate services end-to-end.

## Calibration

- **Predicted runtime band:** 60-120 min (opus, with 6 locked decisions + 1 design call open in BRIEF)
- **Actual:** ~75 min — mid-band
- **Why mid-band, not faster:** The unforeseen substrate/wat carrier-type mismatch (Pass 18 guaranteed Event shape parity but not Sender<T> carrier parity) required bridge threads — a substantive architectural addition not anticipated in BRIEF. Opus surfaced this cleanly and resolved it with raw `std::thread::spawn` workers that translate Rust↔Value payloads.
- **Calibration lesson:** Cross-substrate-layer integration slices carry hidden friction even when both sides individually look settled. Future slices wiring Rust to wat-side services should budget time for translation layers.

## Scorecard

| Row | What | Result |
|-----|------|--------|
| A | `invoke_user_main` orchestrates: spawn services → register thread-0 → run user::main → cleanup → join services | ✓ delegated to new `invoke_user_main_orchestrated` |
| B | `eval_kernel_spawn_thread` registers user threads; skips service threads via carrier-is-set check | ✓ lazy-registration via `sym.runtime_services()` |
| C | Thread closure epilogue sends Remove + uninstalls ThreadIO | ✓ |
| D | `RuntimeServices` carrier chosen + documented | ✓ **Option B (SymbolTable field)** — see § Carrier choice |
| E | New helpers in `src/thread_io.rs` (register / deregister / RuntimeServices struct + bridges + ambient stdio) | ✓ +625 lines |
| F | Lazy-registration pattern documented (service-thread bootstrap) | ✓ inline comment in `eval_kernel_spawn_thread` |
| G | `cargo check --release` green | ✓ clean (1 pre-existing dead_code warning) |
| H | 5 integration test rows pass | ✓ 5/5 (row A-E in `wat_arc170_slice_1f_gamma_orchestrator.rs`) |
| I | Workspace within ±5 of post-1f-β-iii baseline (1339/869) | ✓ **1344/869 — exactly +5/+0** (the 5 new tests; zero regression) |
| J | Zero new dependencies | ✓ Cargo.toml unchanged |
| K | Zero new Mutex / RwLock / CondVar | ✓ grep clean; OnceLock + AtomicI64 only |
| L | No regression of pre-existing tests | ✓ failure count unchanged at 869 |
| M | Honest deltas surfaced | ✓ 9 categories (BRIEF anticipated 6) |
| N | INSCRIPTION-grade prose in commit message | ✓ orchestrator commits this turn |

**14/14 rows pass.** Mode A clean.

## Carrier choice — locked decision (BRIEF honest-delta resolved)

**Option B: `RuntimeServices` is a field on `SymbolTable`.**

Opus rejected Option A (`OnceLock<RuntimeServices>` static) because OnceLock has no clear-on-exit semantics; sequential `invoke_user_main` calls in one process (the cargo-test shape) would inherit the first set's services, breaking test isolation. Option B propagates naturally via `SymbolTable::clone` (already done in `eval_kernel_spawn_thread`) and drops cleanly on scope exit.

Memory pointer: `feedback_capability_carrier.md` — *"new runtime capabilities attach to SymbolTable next to encoding_ctx."* This slice executes that pattern.

Naming mirrors existing precedent: `set_runtime_services` / `runtime_services()` mirrors `set_encoding_ctx` / `encoding_ctx()`.

## Honest deltas (9 categories, 6 anticipated + 3 unforeseen)

### Anticipated (BRIEF named these)

1. **Carrier choice (A vs B)** — resolved to B; rationale documented in § Carrier choice above.
2. **`spawn_service` helper shape** — new private fn in `src/freeze.rs::invoke_user_main_orchestrated`. Looks up wat-side spawn fn by keyword path; calls `apply_function` with `Value::io__IOReader(...)` or `Value::io__IOWriter(...)`; destructures returned `(Thread, ControlTx)` via new `extract_control_tx` helper. **No friction at the arg site** — IO handles are already pluggable `Arc<dyn WatReader/Writer>`.
3. **`ThreadIO` Clone vs move** — resolved by **moving**. Registration happens in parent BEFORE `std::thread::Builder::spawn`; resulting ThreadIO captured by value in closure and installed in new thread's thread-local on entry. No Clone derivation needed. Add-send failures surface synchronously to parent.
4. **`next_thread_id`** — `AtomicI64::fetch_add` static at `src/thread_io.rs`. Starts at 1 (reserves 0 as future sentinel). Process-scoped; counter survives invocations (fine — service routing tables rebuild per `invoke_user_main`).
5. **`join_service` shape** — new private fn in `src/freeze.rs`. Destructures `Value::Struct(":wat::kernel::Thread")`, extracts `ProgramHandle` from field 2, recvs `SpawnOutcome` directly. Bypasses `Thread/output` Receiver (service's only Output Value is final `:nil`, redundant with ProgramHandle outcome). Panic → `RuntimeError::MalformedForm`; RuntimeErr propagates; channel disconnect → `ChannelDisconnected`.
6. **Test fixture capture** — uses OS pipes (`libc::pipe`) not StringIoReader/Writer. StringIoReader/Writer are ThreadOwnedCell-backed (single-thread-owned); service thread owns the IO handle and test thread inspects from a different thread. OS pipes are cross-thread-safe by construction. Documented in test file header.

### Unforeseen (the substantive ones)

7. **Substrate/wat Event-channel-type mismatch → bridge threads.** Slice 1f-α defined `ThreadIO` with Rust-typed channels (`Sender<StdOutServiceEvent>`); slice 1f-β defined wat-side services with Value-typed channels (`Sender<Value>` for `Sender<wat::kernel::services::*::Event>` at runtime). **Pass 18's "unified Event enum" guaranteed shape parity but not carrier-type parity.** The orchestrator bridges by spawning three tiny `std::thread::spawn` workers per registered thread (one per service) that translate Rust events ↔ Value::Enum payloads. Bridges use raw `std::thread::spawn` (not `:wat::kernel::spawn-thread`) so they don't recursively trigger registration. Documented at length in the `// ─── Slice 1f-γ — runtime-services carrier + bridge protocol ───` header in `src/thread_io.rs`. **Foundation insight:** Pass 18's "unified Event enum" needs companion locking at the Rust↔wat boundary; otherwise translation layers proliferate.

8. **`PipeWriter::from_owned_fd` via `dup(0/1/2)` for default real-fd stdio.** Production paths (wat-cli, `fork.rs:659/1044`) reach `synthesize_real_fd_stdio` which `libc::dup`s inherited fds so the orchestrator's PipeReader/Writer Drop closes the dup'd fds, not the original 0/1/2. Without dup, a subsequent `invoke_user_main` in the same process (cargo-test shape) would face closed real fds. Safe in production (each fork is its own process); important for test isolation.

9. **Wat-side stdin service uses `:wat::edn::read` returning generic `Value`, not `Value::holon__HolonAST`.** The bridge handles this defensively via a local `value_to_holon_ast` coercion (primitives → HolonAST). Notes recorded in bridge inline comment: *"wat-side service does not currently re-wrap via `:wat::holon::leaf`; we coerce here defensively."* **Out of slice 1f-γ's scope.** The wat-side service can be simplified to wrap via `:wat::holon::leaf` at the read site; the bridge coercion can then drop. Tracked for the next slice that touches `wat/kernel/services/stdin.wat` (likely 1f-δ or 1f-ε).

## Implementation choices (locked)

- **Carrier:** `RuntimeServices` field on `SymbolTable`; setter/getter mirror `encoding_ctx`
- **Bridge threads:** raw `std::thread::spawn` per (registered-thread × service); translates Rust↔Value at the channel boundary
- **next_thread_id:** `AtomicI64::fetch_add`; starts at 1
- **Cleanup ordering:** main thread deregisters → drops `RuntimeServices` (Arc<>) → scope-drop cascade kills services → `join_service` per service surfaces any panic
- **Real-fd stdio:** `libc::dup` for prod paths; OS pipes for tests

## Files modified

- `src/thread_io.rs` (+625) — `RuntimeServices` struct, three bridge spawn helpers, `register_thread_with_services` / `deregister_thread_from_services` / `next_thread_id` / `extract_control_tx`, ambient stdio thread-local helpers
- `src/runtime.rs` (+90) — `runtime_services: Option<Arc<RuntimeServices>>` field on `SymbolTable` + setter/getter; modified `eval_kernel_spawn_thread` to lazy-register/install/deregister around spawned thread body
- `src/freeze.rs` (+285) — rewrote `invoke_user_main` to delegate to new `invoke_user_main_orchestrated`: spawns three services, builds RuntimeServices carrier, augments SymbolTable, registers thread-0, runs user::main, cleans up, joins services
- `tests/wat_arc170_slice_1f_gamma_orchestrator.rs` (new, 300 lines) — 5 integration rows (single-thread, multi-thread, panic recovery, scope-drop cascade, readln roundtrip) using OS pipes

## Lessons captured

1. **Pass 18's "unified Event enum" is shape-parity, not carrier-parity.** Bridge threads were the resolution. Future arcs touching tier-boundary translation should anticipate the need for translation layers between Rust-typed and Value-typed channels.

2. **`SymbolTable` carrier pattern works.** OnceLock alternative is unsuitable for test-isolation scenarios. Future runtime capabilities should follow the SymbolTable carrier convention (`feedback_capability_carrier.md`).

3. **Test-shape OS pipes vs ThreadOwnedCell.** When a test fixture spans threads (parent inspects child's stdio), use OS pipes. ThreadOwnedCell-backed in-memory IO is single-thread-owned by design.

4. **Production-vs-test fd handling.** `dup(0/1/2)` is mandatory for in-process re-invocations. wat-cli's fork-based path doesn't need it; cargo-test in-process re-invocation does. Document this distinction clearly.

5. **5/5 integration tests pass at first cargo run.** The 14-row scorecard held against the substrate-as-teacher; no rebound loop. Opus's planning paid off.

## What's next

1. **Atomic-commit slice 1f-γ** (this turn) — 4 files + this SCORE
2. **Slice 1f-δ** — `deftest-hermetic` migrates to `spawn-process`; § Row K closes; the 854 baseline + 15 trio failures resolve. The runtime orchestrator now provides everything `spawn-process` needs (it forks a fresh process; the child runs `invoke_user_main_orchestrated` which boots its own services).
3. **Slice 1f-ε** — Console retirement + consumer sweep
4. Optional small fix in adjacent slice: wat-side stdin service's `value_to_holon_ast` coercion can drop once `:wat::edn::read` wraps via `:wat::holon::leaf` (delta #9)

## Cross-references

- BRIEF: [`BRIEF-SLICE-1F-G.md`](./BRIEF-SLICE-1F-G.md)
- Predecessor: slice 1f-β-iii (`52319ba`) — final wat-side service committed
- Successor: slice 1f-δ — `deftest-hermetic` → `spawn-process`; § Row K closure
- Architecture: REALIZATIONS pass 15-18; TIERS.md § OS-boundary handling
- Memory: `feedback_capability_carrier.md` (the pattern this slice executes); `feedback_zero_mutex.md` (the doctrine this slice respects)
