# Arc 170 Slice 4c-α-i SCORE — migrate readln-echo to Layer 2 (run-hermetic-with-io)

**BRIEF:** `BRIEF-SLICE-4C-ALPHA-I-LAYER2-READLN-ECHO.md`
**EXPECTATIONS:** `EXPECTATIONS-SLICE-4C-ALPHA-I-LAYER2-READLN-ECHO.md`
**Task:** #319
**Branch:** `arc-170-gap-j-v5-deadlock-state`
**Tip at start:** `4dac42b`
**Files touched (1):** `wat-tests/kernel/services/ambient-stdio.wat`

## Scorecard

| Row | What | Result | Evidence |
|-----|------|--------|----------|
| A | `wat-tests/kernel/services/ambient-stdio.wat` no longer contains `:wat::test::run-hermetic-ast` | **YES** | `grep -n "run-hermetic-ast" wat-tests/kernel/services/ambient-stdio.wat` returns 0 lines (file fully cleared — neither live calls nor comment refs remain). |
| B | The readln-echo helper uses `:wat::test::run-hermetic-with-io` with native `"echo me"` input | **YES** | Line 117 invokes `(:wat::test::run-hermetic-with-io :wat::core::String :wat::core::String (:wat::core::Vector :wat::core::String "echo me") <body>)`; helper signature on line 116 is `(:test::run-readln-echo -> :wat::test::RunResultIO<wat::core::String>)`. Native String "echo me" (no EDN-quote-escaping in the wat source). |
| C | The Layer 4 deftest consumer reads `RunResultIO/outputs` (not `stdout`) | **YES** | Line 196: `(:wat::test::RunResultIO/outputs (:test::run-readln-echo))` inside the `assert-eq` call. The `assert-stdout-is` form is gone from the Layer 4 deftest body. |
| D | `cargo build --release --workspace --tests` clean | **YES** | `Finished `release` profile [optimized] target(s) in 5.08s`. Only the pre-existing dead-code / unused-mut warnings remain (`unwrap_bool`, `wat-cli` mut bindings, `probe_sender_receiver_from_pipe` env). Zero compile errors. |
| E | The `test-ambient-stdio-readln-echo` test passes; workspace failure count ≤ variance band (11) | **YES** | `cargo test --release --workspace`: `deftest_wat_rs_test_test_ambient_stdio_readln_echo ... ok` (confirmed across three independent runs). All 5 ambient-stdio deftests reported `ok` in the stable run. Workspace failure count rotated 1–2 across three passes: pass 1 → 1 failed (only `tmp_totally_bogus` — pre-existing should-panic); pass 2 → 2 failed (added `test_ambient_stdio_println_twice` — pre-existing intermittent, NOT touched by this slice); pass 3 → 2 failed (added `lifeline_pipe_zero_orphans_across_100_trials` — pre-existing flake, also untouched). All three failures are well documented pre-existing rotation; the readln-echo test itself was never in the failed set. |

**Verdict: 5/5 PASS.**

## Honest deltas

### No substrate-side surprises

The migration was a pure consumer reshape. Predicted deltas at the typed-channel boundary — EDN encoding behavior, EOF handling, RunResultIO accessor naming, assert-eq on Vector<String> — all worked exactly as the BRIEF predicted, with no substrate intervention needed.

- **EDN encoding at the channel boundary worked symmetrically as predicted.** Parent passes native `"echo me"` → Sender/from-pipe writes EDN-quoted line `"echo me"\n` onto fd 0 → child's `(readln -> :String)` reads + parses → native `"echo me"` → `(println echoed)` writes EDN-quoted line back onto fd 1 → parent's Receiver/from-pipe decodes → native `"echo me"` lands in `RunResultIO/outputs[0]`. The vector equality `(Vector :String "echo me") == outputs` matched on the first try.
- **EOF handled correctly per T18 (bounded I/O).** Single send, single recv, child exits, drain sees EOF, join returns immediately. No deadlock — the substrate's bootstrap-fn pipe-close on child exit propagates cleanly.
- **`RunResultIO/outputs` accessor exists by the predicted name.** Auto-generated via `register_struct_methods` from `src/types.rs:991` (the `RunResultIO` StructDef registration). Direct call form `(:wat::test::RunResultIO/outputs r)` worked without any substrate addition.
- **`assert-eq<T>` handles `Vector<String>` equality via `:wat::core::=`.** Structural equality through generic `=` — no special-case comparator needed.

### Doc-comment thoroughness

Updated four doc-comment regions (the BRIEF named three; I refreshed a fourth for consistency):

1. **File-header lines 12-25** — rewrote "each helper uses run-hermetic-ast which forks via fork-program-ast" into a mixed-layer description that names both Layer 1 (run-hermetic, byte-stream RunResult) and Layer 2 (run-hermetic-with-io, typed channel RunResultIO<O>).
2. **Layer-summary lines 28-32** — replaced all five `run-hermetic-ast` references with the post-migration shapes: Layers 0-3 → `run-hermetic`; Layer 4 → `run-hermetic-with-io`.
3. **Layer 4 helper-comment lines 90-114** — replaced the legacy "Test seeds stdin with one EDN line… TWO elements for trailing newline… Pre-1f-iota readln returned HolonAST" content with a fresh description of the Layer 2 wire format: typed Sender/Receiver from-pipe wrappers, EDN encode/decode symmetry, T18 bounded-I/O exit semantics.
4. **Layer 4 deftest header-comment lines 178-191 (BONUS — beyond the BRIEF's three named regions)** — the BRIEF named three regions, but this fourth one was also stale (described the legacy round-trip with EDN-quoted wire form on both directions). Refreshed to describe the Layer 2 round-trip explicitly: typed Sender → readln → println → typed Receiver → RunResultIO/outputs → assert-eq against native `(Vector :String "echo me")`. Kept consistent voice with the helper-side comment.

### Workspace flake landscape (pre-existing, NOT a regression)

Three runs of `cargo test --release --workspace` show 1–2 failures rotating across a small fixed set:
- `deftest_wat_tests_tmp_totally_bogus` — `should_panic` test, pre-existing across all three runs (not related to this slice; appears intentional from its name).
- `deftest_wat_rs_test_test_ambient_stdio_println_twice` — appeared in pass 2 only. This is a Layer 3 sibling test that this slice does NOT touch (Layer 3 still uses Layer 1's `run-hermetic`). The flake is a pre-existing intermittent in the run-hermetic byte-stream drain path under cargo's concurrent-test scheduler.
- `lifeline_pipe_zero_orphans_across_100_trials` — appeared in pass 3 only. Probe-level lifeline test under hermetic, completely outside the Layer 2 surface this slice migrates.

None of these failures are attributable to this slice's migration. The readln-echo test under migration was `ok` in every run. Per `feedback_tests_not_flaky`, the rotating flakes ARE real substrate races worth investigating eventually — but they're orthogonal to Layer 4 readln-echo + this slice's scope.

## Calibration record

| Metric | Predicted | Actual |
|---|---|---|
| Wall-clock runtime | 15–30 min | ~18 min (within band) |
| Scorecard rows | 5/5 PASS | 5/5 PASS |
| Workspace fail count | ≤ 11 (variance band) | 1–2 across three runs (well below band) |
| readln-echo test in PASSED | yes | yes — every run |
| Substrate surprise surfaced | none expected | none surfaced |
| Mode | A (clean) | A (clean) |

## Post-slice state

Zero wat-level callers of `:wat::test::run-hermetic-ast`. The legacy `:wat::test::run-hermetic-ast` define + `:wat::test::program` wrapper survive in `wat/test.wat` as orphans until 4c-α-iv deletes them atomically with the other 2 wrappers + sandbox.wat + hermetic.wat (pending 4c-α-ii Rust-side migration and 4c-α-iii driver migration).

The five-layer ambient-stdio test file now demonstrates the mixed Layer 1 / Layer 2 surface concretely — Layers 0-3 prove byte-stream stdout/stderr drains via `run-hermetic`; Layer 4 proves typed-channel I/O round-trip via `run-hermetic-with-io`. The progression `byte-stream → typed-channel` mirrors the BRIEF's framing of Layer 2 as the modern wire format.
