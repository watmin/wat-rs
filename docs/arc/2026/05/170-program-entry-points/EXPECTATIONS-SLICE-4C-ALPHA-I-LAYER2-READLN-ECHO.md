# Arc 170 Slice 4c-α-i EXPECTATIONS

**BRIEF:** `BRIEF-SLICE-4C-ALPHA-I-LAYER2-READLN-ECHO.md`
**Task:** #319

## Independent prediction

**Runtime band:** 15–30 minutes.

Reasoning:
- 1 helper migration (define-form reshape; ~10 lines)
- 1 consumer migration (assertion-form reshape; ~5 lines)
- Doc-comment updates (~20 lines across 3 sections)
- Build + workspace test verification: ~3 min
- Mode A (no architectural surprise expected; the macro shape is documented per the BRIEF's target-shape section)

**Time-box:** 60 min hard stop.

## SCORE methodology

5 rows YES/NO; tight slice with mechanical verification:

- **Row A** (zero `run-hermetic-ast` in this file): `grep -nE ":wat::test::run-hermetic-ast\b" wat-tests/kernel/services/ambient-stdio.wat | grep -v "^[^:]*:[0-9]*:\s*;;"` returns 0 (allow comment-references for archival context).
- **Row B** (new Layer 2 shape): `grep -nE "run-hermetic-with-io|RunResultIO" wat-tests/kernel/services/ambient-stdio.wat` returns matches in the readln-echo area.
- **Row C** (consumer reads outputs slot): `grep "RunResultIO/outputs" wat-tests/kernel/services/ambient-stdio.wat` returns at least 1 match.
- **Row D** (build clean): cargo build clean.
- **Row E** (readln-echo test passes; baseline maintained): cargo test confirms the test in PASSED set; total failed ≤ 11 (variance band).

## Honest deltas to watch for

- **EDN encoding behavior at the typed-channel boundary.** Layer 2's `Sender/from-pipe` EDN-encodes a native String "echo me" as `"echo me"` on the wire (with quotes); the child's `readln -> :String` parses the EDN line back to native "echo me" (no quotes); the child's `println` EDN-encodes the native String back to `"echo me"` on the wire; the parent's `Receiver/from-pipe` decodes back to native "echo me". This SHOULD work cleanly per the macro's design, but if there's a substrate-side detail (e.g., readln expects unquoted form when typed EDN is used), surface as a delta.

- **EOF handling.** The Layer 2 driver sends inputs sequentially, then drains outputs until disconnect. The child reads ONE value (via readln), prints ONE value (via println), then exits. The single-send/single-recv pattern matches the macro's documented T18 case. If the child deadlocks (e.g., waits for EOF that never comes), surface as a delta.

- **RunResultIO struct accessor naming.** The accessor for the outputs slot is presumably `:wat::test::RunResultIO/outputs`. If it's named differently in the substrate (e.g., `RunResultIO::outputs` Rust-mangled), use the actual canonical wat-level accessor name.

- **assert-eq on Vector<String>.** The consumer uses `(:wat::test::assert-eq (outputs ...) (Vector :String "echo me"))`. Verify assert-eq supports Vector<String> equality; if not, use a different comparator (e.g., element-wise check or RunResultIO/outputs's own assertion verb if one exists).

- **Doc-comment refresh thoroughness.** Three doc-comment locations to update: file-header (lines 13-14), layer-summary (line 29), Layer 4 helper-comment (lines 87-107). Don't miss any.

## Calibration record (to fill on completion)

| Metric | Predicted | Actual |
|---|---|---|
| Wall-clock runtime | 15–30 min | TBD |
| Scorecard rows | 5/5 PASS | TBD |
| Workspace fail count | ≤ 11 (variance band) | TBD |
| readln-echo test in PASSED | yes | TBD |
| Substrate surprise surfaced | none expected | TBD |
| Mode | A (clean) | TBD |
