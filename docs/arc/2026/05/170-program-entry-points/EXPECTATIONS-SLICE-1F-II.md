# Arc 170 slice 1f-ii — EXPECTATIONS

## Independent prediction

**Predicted runtime: 60-90 minutes opus.**

Pattern inheritance from slice 1f-i (`src/services/mod.rs` rustdoc minted the registration shape) should make this faster than 1f-i was. The novel pieces:
- crossbeam Select dynamic registration (vs libc::poll's static-fd self-pipe in 1f-i)
- Mini-TCP ack discipline (per arc 089 slice 5 + `wat/console.wat`)
- HolonAST → wat_edn::Value bridge for serialization

Comparable to:
- arc 089 slice 5 (Console gains ack channel) — similar
  mini-TCP discipline; ~90 min
- slice 1f-i — pattern-fit was tight (~30 min); 1f-ii has more
  novel pieces but inherits the structure

**Hard cap: 180 minutes** — wakeup scheduled.

## Baseline (post-slice-1f-W)

Slice 1f-ii starts from commit `4278c4d` (slice 1f-W shipped — wire encoding lexical doctrine).

Baseline cargo test (verified):
- **1329 passed / 855 failed** across 127 suites
- Includes the +23 wire_encoding tests from 1f-W

Predicted post-slice-1f-ii count:
- New tests in `tests/services_stdout.rs` add ~10-20 passed
  cases
- Workspace fail count: ~unchanged (±5 band) — slice 1f-ii is
  parallel infrastructure; existing tests don't see the service
  yet

## Scorecard

| Row | What to verify | Pass criterion |
|-----|----------------|----------------|
| A — Module structure | `src/services/stdout.rs` exists; `src/services/mod.rs` re-exports `start_stdout_service` and `StdOutServiceHandle` alongside stdin's exports | ✓ |
| B — `start_stdout_service` idempotent | first call spawns thread; second returns same `&'static StdOutServiceHandle` | ✓ |
| C — Service thread spawns + idles | thread runs without panic; CPU near zero (Select blocks; no busy-wait) | ✓ |
| D — Registration roundtrip | `handle.register(thread_id)` returns crossbeam Sender; `handle.unregister(thread_id)` drops the receiver from the Select set; multi-register supported | ✓ |
| E — Single-thread send + ack | register → send `(Arc<HolonAST>, ack_tx)` → block on ack_rx → receive `()` → assert test-fd bytes match expected EDN line | ✓ |
| F — Wire encoding via slice 1f-W | parametric keyword Atom (e.g., `:HashMap<K,V>`) serializes with underscore form for commas inside `<>` (`HashMap<K_V>` in the output bytes) | ✓ |
| G — Multi-thread send + ack | N=3 threads register; each sends; each receives ack; output bytes contain N intact lines; per-thread ordering preserved within each thread | ✓ |
| H — Shutdown semantics | drain-pending OR immediate-exit (chosen + documented); test verifies the documented behavior | ✓ |
| I — fd ownership convention | service uses output_fd but does NOT close it; caller retains OwnedFd via the pipe pair they allocated | ✓ |
| J — Single-writer guard on fd 1 | producer threads NEVER call libc::write directly; only the service worker writes fd 1; verified by inspection (the API doesn't expose direct fd access) | ✓ |
| K — Rust integration tests green | `cargo test --release --test services_stdout` → all rows green | ✓ |
| L — Workspace fail-count delta within ±5 | `cargo test --release --workspace --no-fail-fast` fail count is 850-860 (post-1f-W was 855) | ✓ |
| M — Honest deltas surfaced | per FM 5; no TODOs in source; no deferral language | ✓ |
| N — Zero new dependencies | `Cargo.toml` unchanged | ✓ |
| O — Foundation + slice 1e + 1f-i + 1f-W files untouched | `git diff 4278c4d..HEAD` shows only `src/services/stdout.rs`, `src/services/mod.rs` (re-export line), `tests/services_stdout.rs` (new) — no other files | ✓ |
| P — Pattern documented for 1f-iii reuse | module-level rustdoc on `src/services/stdout.rs` documents the send-with-ack contract; explicitly notes how 1f-iii (StdErrService) can apply the same pattern with first-panic-wins + libc::exit semantics | ✓ |
| Q — Zero new Mutex/RwLock/CondVar | grep `src/services/stdout.rs` returns 0 hits for these (only doc-comment "Zero Mutex" mentions allowed) | ✓ |

## Honest delta categories

Surface promptly; don't workaround:

- **Dynamic Select rebuild on register/unregister** — crossbeam's Select requires the set built before .ready()/.select() blocks. Adding/removing receivers mid-loop requires breaking out, rebuilding the Select, re-entering. If this creates awkward interleaving (e.g., dropped messages during rebuild), surface for design discussion.
- **Ordering guarantees** — cross-thread ordering NOT guaranteed (Select picks readiness; producers race). Per-thread ordering IS guaranteed. Document in module rustdoc + the relevant test row.
- **Ack channel cardinality** — one-shot bounded(1) per send vs shared per-thread ack stream. Pick one; document. Lean toward one-shot for clarity.
- **HolonAST → wat_edn::Value bridge for write** — slice 1f-i's parse direction has the inverse (`wat_edn::parse → HolonAST`). Slice 1f-ii needs `HolonAST → wat_edn::Value` for `wat_edn::write` to consume. If the bridge function doesn't exist OR has unexpected fidelity issues, surface.
- **Shutdown drain vs immediate-exit** — pending messages on shutdown: drain (friendlier; producers don't lose data) OR drop (faster; test cleanup); design choice. Document.
- **wat/console.wat ack pattern coexistence** — Console (the wat-side crossbeam service) still operational. Slice 3 migrates Console-using tests; slice 1f-ii's StdOutService is the SUBSTRATE version. They co-exist briefly. Surface any conflicts (e.g., shared fd 1 contention if both are active in some test).
- **FM 5 trap** — TODOs verboten. Corner-case scope-bounding belongs in honest deltas, not in code comments.

## Calibration row

Filled at scoring time:

- Actual runtime: ___ min (Mode A clean / B partial / C failed)
- Workspace post-1f-ii: ___ passed / ___ failed
- Fail-count delta from post-1f-W baseline: ___
- Whether delta lands in ±5 band: ___
- Honest deltas surfaced: ___
- Implementation choices: shutdown semantics ___, ack cardinality ___, dynamic Select strategy ___

## What's next (orchestrator-side, post-slice-1f-ii)

When 1f-ii ships:
1. Verify ship criteria locally
2. Author SCORE-SLICE-1F-II.md
3. Atomic commit slice 1f-ii
4. Slice 1f-iii BRIEF + EXPECTATIONS authored — StdErrService applying the registration pattern from 1f-i + the send-with-ack contract from 1f-ii + first-panic-wins + libc::exit cascade semantics
5. Spawn slice 1f-iii

## Sonnet-delegation-protocol pre-flight (recovery doc § 7)

- [x] DESIGN.md current (passes 1-14)
- [x] BRIEF-SLICE-1F-II.md authored + will-be-committed
- [x] EXPECTATIONS-SLICE-1F-II.md (this doc) authored + will-be-committed
- [x] Runtime band: 60-90 min predicted; 180 min hard cap
- [x] Substrate-grep citations in BRIEF point at exact files (slice 1f-i pattern docs, slice 1f-W writer, crossbeam Select precedent, libc::write precedent, arc 089 ack pattern)
- [x] Verified each cited primitive exists (slice 1f-i shipped at `630f621`; slice 1f-W shipped at `4278c4d`; pattern docs in `src/services/mod.rs`)
- [x] No "STOP at first red" + impossible-task constraint
- [x] Baseline established: 1329 passed / 855 failed
- [ ] Will spawn with `model: "opus"` explicitly (substrate work; design choice on Select strategy)
- [ ] Will spawn with `run_in_background: true`
- [ ] Wakeup scheduled at 180 min (3 hours = 10800 s)

## SCORE artifact

Slice 1f-ii is the second of four 1f stepping stones (1f-i + 1f-ii + 1f-iii + 1f-iv). SCORE-SLICE-1F-II.md lands beside this.
