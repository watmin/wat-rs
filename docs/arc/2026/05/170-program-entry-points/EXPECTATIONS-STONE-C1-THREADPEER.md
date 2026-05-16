# Arc 170 Stone C1 EXPECTATIONS

**BRIEF:** `BRIEF-STONE-C1-THREADPEER.md`

## Independent prediction

**Runtime band:** 30-45 minutes sonnet.

Reasoning:
- `ThreadPeer<I, O>` struct registration in type system: ~30-50 LOC
- 2 verb registrations (eval handlers + dispatch arms + type signatures): ~50-80 LOC
- Substrate-internal helper for test peer-pair construction: ~20-40 LOC
- 3 tests: ~100-150 LOC
- Reuses existing Sender<T>/Receiver<T> typed-channel substrate (no new infrastructure)

**Time-box:** 60 min hard stop.

## SCORE methodology

5 rows YES/NO per BRIEF:

- **Row A** (type registered): grep `ThreadPeer` in src/ shows registration
- **Row B** (verbs registered): grep shows both Thread/readln + Thread/println
- **Row C** (helper exists): grep shows internal pipe-pair constructor
- **Row D** (3 tests pass): `cargo test --release -p wat --test wat_arc170_stone_c1_threadpeer` green
- **Row E** (workspace baseline maintained): cargo test summed failed ≤ baseline + flake variance

## Honest deltas to watch for

- **ThreadPeer location.** Two candidates: (1) `src/types.rs` adjacent to existing struct registrations (RunResult, RunResultIO from before its deletion, etc.); (2) new file `src/thread_peer.rs` re-exported. Sonnet decides; consistency with arc 198 slice 2 Stone 1 (`src/restriction_entry.rs`) suggests new module if scope warrants.

- **Field composition.** Two natural shapes for the struct:
  - `ThreadPeer<I, O> { rx: Receiver<I>, tx: Sender<O> }` — explicit field types, clean correspondence to type params
  - `ThreadPeer<I, O> { read_end, write_end }` — opaque field types, more substrate-internal flexibility
  - Sonnet picks based on existing struct conventions in src/types.rs

- **Internal pipe-wiring helper.** For tests, sonnet needs a way to construct two peers wired together. Options:
  - Rust-only helper (`fn make_thread_peer_pair<X, Y>() -> (ThreadPeer<Y, X>, ThreadPeer<X, Y>)`) called from test setup
  - Wat-side helper exposed as a primitive (`(make-thread-peer-pair :X :Y)`) — but that exposes more API than necessary
  - Test fixture constructs peers via direct substrate-internal access (most permissive)
  - Probably Rust-only helper — Stone D's bracket macro is what user-facing constructs peers; Stone C1's helper is just for testing.

- **arc 117/133 walker interaction.** If a wat user wat constructs a ThreadPeer with Sender + Receiver fields, the existing sibling-binding walker may fire on the ThreadPeer binding as Sender-bearing. Since Stone C1 doesn't yet expose construction via the bracket, the tests probably construct peers via Rust-side helpers — the walker may not see them. If it does, surface in SCORE.

- **Sender<T>/Receiver<T> substrate API.** Existing typed-channel primitives per arc 109 slice K.kernel-channel + arc 170 slice 1c. Sonnet verifies the API surface (likely `typed_channel::make_pair<T>() -> (Sender<T>, Receiver<T>)` or similar).

- **Walker scope for new verbs.** `Thread/readln` + `Thread/println` need walker entries similar to `recv`/`send` (per arc 110/111). Sender-bearing detection may need to include ThreadPeer in the existing classifier.

## Workspace baseline (commit `1e3cf7a`)

- `cargo build --release --workspace --tests`: clean
- `cargo test --release --workspace --no-fail-fast`: 3 stable failures + lifeline flake variance

Post-Stone-C1 target:
- ≥ baseline + 3 passed (3 new tests)
- ≤ baseline failures (purely additive)

## Calibration record (to fill on completion)

| Metric | Predicted | Actual |
|---|---|---|
| Wall-clock runtime | 30-45 min | TBD |
| Scorecard rows | 5/5 PASS | TBD |
| Workspace fail count | ≤ baseline | TBD |
| New test count | 3 | TBD |
| ThreadPeer location | src/types.rs OR new module | TBD |
| Helper shape | Rust-only OR wat-exposed | TBD |
| Walker interaction surprises | 0-1 | TBD |
| Substrate-discovery surprises | 0-2 | TBD |
| Mode | Additive (new type + new verbs + test helper) | TBD |
