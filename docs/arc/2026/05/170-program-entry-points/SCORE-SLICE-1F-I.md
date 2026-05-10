# Arc 170 slice 1f-i — SCORE

**Result:** Mode A clean.
**Runtime:** ~30 min opus (way under 90-150 predicted band — pattern-fit was tighter than expected; substrate-grep paid off; no surprises).
**Files:** 3 new + 1 one-line edit.

## Calibration

- **Predicted runtime band:** 90-150 min opus (hard cap 300 min)
- **Actual:** ~30 min — well under band
- **Why faster than predicted:** pattern dropped in cleanly from
  the pre-grep citations; no substrate gaps surfaced; the
  KERNEL_STOPPED static-atomic precedent + libc usage in
  src/fork.rs gave the agent a clear template

## Scorecard

| Row | What | Result |
|-----|------|--------|
| A — Module structure | ✓ — `src/services/mod.rs` (new) + `src/services/stdin.rs` (new); `src/lib.rs` adds `pub mod services;` (1 line) |
| B — `start_stdin_service` idempotent | ✓ — OnceLock-stored `&'static StdInServiceHandle`; second call returns same handle |
| C — Service thread spawns + idles | ✓ — verified by `row_c_service_thread_idles_without_panic` |
| D — Registration roundtrip | ✓ — `register(thread_id) -> Receiver<Option<Arc<HolonAST>>>`, `unregister(thread_id)` |
| E — Single-line EDN parsing | ✓ — bytes "42\n" → consumer receives `Some(HolonAST)` |
| F — Multi-line ordered dispatch | ✓ — bytes "1\n2\n3\n" → ordered Some(1), Some(2), Some(3) |
| G — EOF propagates :None | ✓ — fd 0 close → consumer receives None |
| H — Self-pipe trick verified | ✓ — interleaved data + control messages both wake poll |
| I — Zero Mutex/RwLock/CondVar | ✓ — only references in `src/services/*.rs` are doc comments saying "Zero Mutex"; no actual usage |
| J — libc::poll used; no mio/tokio | ✓ — `libc::poll`, `libc::pipe`, `libc::write`, `libc::STDIN_FILENO` used directly; no async runtime added |
| K — Rust integration tests green | ✓ — `cargo test --release --test services_stdin` → **12 passed / 0 failed** |
| L — Workspace doesn't regress | ✓ — post-1f-i: **1306 passed / 855 failed**; baseline was 1294/855; delta is +12 passed (= new tests) and 0 failed (perfect; within ±5 band) |
| M — Honest deltas surfaced | ✓ — 7 deltas (counted below); none worked-around |
| N — Zero new dependencies | ✓ — Cargo.toml unchanged |
| O — Foundation + slice 1e files untouched | ✓ — `git diff 206bdd1..HEAD --` of foundation files is empty |
| P — Registration API documented for 1f-ii reuse | ✓ — module-level rustdoc on `src/services/mod.rs` documents the singleton + spawn_for_test + register/unregister + self-pipe-poll loop pattern |

**16/16 rows pass.** Mode A clean.

## Honest deltas surfaced

### 1. Dispatch policy = first-registered-wins

BRIEF allowed ONE consumer; with N=1 (typical case), broadcast
and first-wins are equivalent. Agent chose deterministic
first-wins via Vec<Consumer> registration order. Verified by
`row_p_dispatch_first_registered_only`. Multi-consumer routing
remains out of scope per BRIEF; slice 1g will revisit when
multiple threads need to share stdin (rare).

### 2. Malformed lines (UTF-8 + EDN parse errors) drop silently

BRIEF allowed "panic with diagnostic OR cascade through
StdErrService." Without StdErrService (slice 1f-iii), panicking
would tear down the service mid-flight and break tests of
normal flow. Agent chose drop-and-continue — keeps the service
alive; integration with StdErrService cascade lands in slice
1f-iii (where panicking is the right behavior).

### 3. Partial trailing line at EOF dropped

The protocol is line-delimited; a non-newline-terminated tail
is not a complete message. Verified by
`honest_delta_partial_trailing_line_drops_at_eof`. Honest
behavior; users must terminate with newline.

### 4. wat-cli's `spawn_stdin_proxy` survives slice 1e

The parent-side stdin proxy in `crates/wat-cli/src/lib.rs:391`
still owns the parent's real fd 0. Slice 1f-i's StdInService
runs in the CHILD process — its fd 0 is the child-side read
end of a pipe (different fd from parent's real stdin). No
conflict for 1f-i.

**For slice 1f-iv:** the child's fd 0 read path needs to be
single-owner — either the parent's stdin-proxy still pipes
through OR the child's StdInService reads directly. The
existing pipe-based proxy may already work cleanly with the
service reading the child end; slice 1f-iv verifies.

### 5. No "always-on background thread" precedent existed

Pattern minted here is the new substrate idiom:
- OnceLock-stored handle
- Worker thread spawned on first access
- Drop on test handles via Shutdown control message
- `KERNEL_STOPPED` static-atomic + libc-direct-syscall conventions

Composes cleanly with existing precedents; no substrate gap.
Documented in module rustdoc so 1f-ii inherits the shape
mechanically.

### 6. `spawn_for_test(RawFd)` — caller retains fd ownership

The test API uses but does not close the fd; tests own the
OwnedFd via the pipe pair they allocated. Service worker owns
only its self-pipe-read OwnedFd. Documented; not surprising
once named.

### 7. Pre-existing build warning unrelated to slice 1f-i

`unused: parse_fn_signature_for_check` at `src/check.rs:10002`
is pre-existing (predates this slice). Surface for future
arc-163 retirement-leftover audit; not slice 1f-i scope.

## Calibration row

- **Actual runtime:** 30 min (Mode A clean — 60-120 min UNDER
  predicted band; the pattern-fit was very clean)
- **Workspace post-1f-i:** 1306 passed / 855 failed
- **Fail-count delta from post-1e baseline:** 0 (855 → 855;
  inside ±5 band; predicted "small" — perfect)
- **Pass-count delta from post-1e:** +12 (= the new fixture
  tests; expected)
- **Honest deltas surfaced:** 7 (all properly classified —
  scope decisions, behavior choices, or pre-existing unrelated
  noise)
- **Pre-grep paid off:** every BRIEF citation matched substrate
  reality; the pattern was implementable as described

## Lessons captured

1. **Pre-grep + clear pattern citations dramatically reduce
   runtime.** The BRIEF cited KERNEL_STOPPED + libc usage +
   wat/console.wat by exact location; the agent had a clear
   template; ~30 min vs predicted 90-150. Future stepping
   stones (1f-ii, 1f-iii) should benefit from the same effect
   AND from inheriting the pattern this slice mints.

2. **Module-level rustdoc as pattern-spec is load-bearing.**
   The agent documented the registration API + singleton
   pattern + self-pipe trick + Drop semantics in
   `src/services/mod.rs` rustdoc. Slice 1f-ii's BRIEF should
   cite this rustdoc explicitly as the contract to apply.

3. **first-registered-wins dispatch was the right scope choice.**
   N=1 typical case; deterministic; revisitable if N>1 demand
   surfaces. No premature multi-consumer complexity.

4. **Drop-and-continue malformed lines was the right scope
   choice.** Panicking pre-StdErrService would break tests;
   slice 1f-iii inherits the cascade integration cleanly.

## What's next

1. **Atomic-commit slice 1f-i** (this turn) — bundle the 3 new
   files + 1-line lib.rs edit + this SCORE doc
2. **Author BRIEF + EXPECTATIONS for slice 1f-ii (StdOutService)**
   — applies the registration pattern from 1f-i; faster
   (~60-90 min predicted; may run faster if pattern-inheritance
   is as effective as 1f-i's pattern-fit was)
3. **Spawn slice 1f-ii**

## Cross-references

- BRIEF: [`BRIEF-SLICE-1F-I.md`](./BRIEF-SLICE-1F-I.md)
- EXPECTATIONS: [`EXPECTATIONS-SLICE-1F-I.md`](./EXPECTATIONS-SLICE-1F-I.md)
- BUILD-PLAN ref: §3 slice 1f-i
- DESIGN ref: § three substrate services
- REALIZATIONS pass 9 — three substrate services architecture
- Predecessor: slice 1e (`206bdd1`)
- Pattern conceptual ancestor: `wat/console.wat` (one select
  loop, N fan-in via crossbeam — at the wat layer; slice 1f-i
  mirrors at the substrate layer)
