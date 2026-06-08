# BRIEF — Stone 6.w Strike 3: channel/ convergence (mechanical)

**Executor:** sonnet (background). **Home:** `src/channel/` ONLY (`mod.rs`, `inner.rs`,
`transfer.rs`). **Design substrate:** the CHANNEL/ section of
`docs/arc/2026/05/214-concurrency-toolkit/SCORE-STONE-6.w-VIGILIA-FINDINGS.md`. **Greedy
stance:** fix every solvable finding; the 3 runes below are earned (correct-as-is), not
deferrals. NO structural change — channel/ is the seam; behavior is identical.

**Sequencing:** fire AFTER Strike 1 (process/) lands green. channel/ shares no files with
process/, but both compile the one `wat` crate, so the verify step must see a settled tree.

## Baseline (match on exit)
- `cargo test --release --lib -p wat` → **943 passed; 0 failed; 1 ignored**.
- `cargo clippy --release -p wat 2>&1 | grep -cE "> src/channel/"` → **0** (channel/ is clippy-clean; KEEP it 0).

## Read in order (the rooms)
1. The CHANNEL/ section of the ledger (above).
2. `src/channel/transfer.rs` (460 lines, whole) — holds most findings.
3. `src/channel/mod.rs` (82 lines) + `src/channel/inner.rs` (101 lines).

## FIX (all confirmed against the code by the orchestrator)
1. **Stale "Crossbeam" live-labels → "comms::thread"** (intueri L2). These present Crossbeam
   as the LIVE tier-1 backing; 5.1 HARD-CUT it to `comms::thread`. Sites: `transfer.rs:48`
   ("Tier 1 (Crossbeam): zero-copy enqueue"), `:147` ("blocks on the crossbeam recv"),
   `:263` ("Tier 1 (Crossbeam): checks SHUTDOWN_RX"), `:15-20` (SendOutcome doc "crossbeam:
   queued"), `:31` (RecvOutcome "every sender dropped (crossbeam)"), `mod.rs:41`
   ("pattern-matched on `Value::crossbeam_channel__Sender(_)`" → `Value::wat__kernel__Sender(_)`).
   **DO NOT touch** the `inner.rs:24,53` "the `Crossbeam` variant is deleted" notes — those
   are HISTORICAL retirement records (keep; FM-14 bucket C).
2. **Duplicate EDN-decode body → extract `decode_pipe_line`** (solvere F-CH-1 L2). The
   `read_line → trim_end_matches('\n') → read_edn → Value/DecodeError/Disconnected` block is
   identical at `transfer.rs:241-256` (typed_recv PipeFd) and `:344-354` (typed_try_recv
   PipeFd). Extract one private `fn decode_pipe_line(reader, types, span) -> RecvOutcome`;
   both arms call it. Behavior identical.
3. **Drop the lying underscores** (struere CH-2 L2). `typed_try_recv` params are `_types`
   (`:279`) + `_span` (`:280`) but the PipeFd arm USES them (`read_line(_span)` `:344`,
   `read_edn(trimmed, _types)` `:347`). Rename `_types`→`types`, `_span`→`span`; update the
   two use sites. (If the decode-dedup in #2 moves these into the helper, the underscores
   vanish naturally — do #2 first, then this is moot or trivial.)
4. **SeqCst → Acquire on the broadcast-fd load** (temperare C-1 L3). `transfer.rs:201` and
   `:311` load `SHUTDOWN_BROADCAST_READ_FD` with `Ordering::SeqCst`; it's a once-written fd
   → `Ordering::Acquire` (correctness-equivalent, lighter). Cheap.
5. **exigere deferral-prose → affirmative** (exigere C-1/C-2 L1). `transfer.rs:361-363`
   "select is tier-1-only today; piped channels would need an epoll/poll integration that's
   substrate work for a follow-up arc" → affirmative: select-over-pipe is **Slice 7
   (parallel brackets)**'s named scope; state it as a bounded scope, not a vague future.
   `transfer.rs:393-398` "slice-2 territory if a real consumer demands it" → affirmative:
   `make_pipe_channel_pair` is intentionally a Rust-internal helper (the wat surface comes
   through `spawn-process`); state that as the design, drop the "if a consumer demands it"
   deferral hedge.
6. **sender_close Comms advisory-only — sharpen the contract doc** (sequi F2 / struere CH-1
   L2). `transfer.rs:114-129`: the comment already explains the flag gates sends + Arc-drop
   does the structural disconnect; sharpen it to state the contract honestly (the flag is
   the immediate gate; EOF reaches the peer on last-Sender-clone-drop). Doc-only.
7. **typed_try_recv "non-blocking" can block on a partial line — document** (struere CH-1
   L2). After `poll(0)` says the pipe is readable, `read_line` (`:344`) can block on a
   partial (newline-less) write. Add a doc note: "poll-gated, not read-gated; the write side
   always frames atomically with a trailing newline, so a partial line is not produced by
   the substrate's own senders." Doc-only.

## RUNES (earned — correct-as-is; write `rune:<spell>(<cat>) — <reason>`)
- **sequi F1** — `typed_recv`'s `RecvOutcome::Shutdown` depends on the ambient `SHUTDOWN_RX`
  global, not threaded through the signature. `rune:sequi(ambient-context) — the shutdown
  cascade is the ZERO-MUTEX doctrine's declared ambient channel; threading it through every
  recv signature bloats the surface for no gain; RecvOutcome::Shutdown IS in the return.`
- **perspicere CH-1** — `try_as_comms_receiver`'s `Option<&comms::thread::Receiver<Value>>`
  return. `rune:perspicere(read-once) — read once at the select call site; a typealias adds
  indirection for a single reader.`
- **temperare C-2** — `typed_try_recv` calls `crate::runtime::shutdown_rx()` every
  invocation BEFORE the data `try_recv`. This is REQUIRED: the documented contract
  (`transfer.rs:263-269`) is "shutdown checked first so it overrides any pending Value."
  Moving it after the data check would return a stale Value during shutdown. `rune:temperare(
  correct-as-is) — shutdown-first ordering is load-bearing (shutdown wins ties per Slice B);
  the per-call shutdown_rx() load is the cost of honest shutdown precedence, not redundant work.`
  (If you see a way to cheapen the load WITHOUT reordering — e.g. a relaxed pre-check — weigh
  it; otherwise rune it. Do NOT reorder.)

## NO ACTION
- mora: arc-253 2-state collapse HELD (no regression) — PASS.

## STOP triggers
- If the `decode_pipe_line` extraction (#2) cannot preserve the exact RecvOutcome surface
  (Value / DecodeError / Disconnected for the three read_line outcomes) — STOP, report.
- If dropping the `_types`/`_span` underscores surfaces a borrow/move issue — STOP, report
  (it shouldn't; they're `Option<&_>` + `Span` by value).

## Blast radius
`src/channel/{mod,inner,transfer}.rs` ONLY. No other file. No behavior change. No new public API.

## Verify (run each; report real output)
- `cargo test --release --lib -p wat` → 943/0/1.
- `cargo clippy --release -p wat 2>&1 | grep -E "> src/channel/"` → empty (0 warnings).

## Deliverable
Final message IS the report (consumed programmatically): each finding's disposition
(FIXED / rune(reason)), the verify command outputs, honest deltas, `wc -l` line counts.
Do NOT commit — the orchestrator weighs and commits.
