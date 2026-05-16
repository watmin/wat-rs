# Arc 170 Slice 4c-α-ii EXPECTATIONS

**BRIEF:** `BRIEF-SLICE-4C-ALPHA-II-RUST-SIDE-CALLER-SWEEP.md`
**Task:** #320

## Independent prediction

**Runtime band:** 30–60 minutes.

Reasoning:
- 16 active call sites across 7 files
- Migration pattern is mechanical for body-AST shape (same as 4a-β's P2a/P3 patterns)
- 2 files have file-header doc-comments to refresh (wat_hermetic_round_trip.rs, wat_run_sandboxed_ast.rs, wat_run_sandboxed.rs — actually 3 files)
- Build + test verification per file: ~5 cycles total
- Largest file (wat_run_sandboxed.rs with 8 sites) is the bulk

**Time-box:** 120 min hard stop.

## SCORE methodology

6 rows YES/NO; per-row evidence patterns:

- **Row A** (zero `run-sandboxed\b` in tests/): `grep -rEn ":wat::kernel::run-sandboxed\b" tests/ | grep -vE ":[0-9]+:\s*//" | wc -l` returns 0.
- **Row B** (zero `run-sandboxed-ast\b` in tests/): similar grep returns 0.
- **Row C** (zero `run-sandboxed-hermetic-ast\b` in tests/): similar grep returns 0.
- **Row D** (canonical macros in migrated sites): per-file grep shows the new verb appears.
- **Row E** (build clean): cargo build Finished, zero errors.
- **Row F** (workspace within baseline): cargo test summed failed ≤ 11.

## Predicted distribution

Educated guess on per-site migration outcomes:

| Pattern | Predicted count | Notes |
|---|---|---|
| Thread destinations (`run-thread`) | 11-13 | Most legacy `run-sandboxed` / `run-sandboxed-ast` sites; bodies don't assert on stdio slots |
| Hermetic destinations (`run-hermetic`) | 3-5 | The 3 `run-sandboxed-hermetic-ast` sites + any thread-destination sites that need stdio capture per three-rule |
| Layer 2 escalations | 0-1 | Rare; only if a test specifically drives stdin via readln |

## Honest deltas to watch for

- **Stdio-capture re-classification** (similar to 4a-β's discovery). Some thread-destination migrations may surface as failing if the body asserts on captured stdout/stderr; rec
lassify to hermetic per FM 7-ter.

- **Source-string parsing.** `:wat::kernel::run-sandboxed` takes a source-string (not forms vector). The body-AST macros take forms. Migration requires parsing the legacy source-string contents and inlining them as the new macro's body. The parsing step is per-site judgment.

- **`(:wat::test::program ...)` wrappers.** Some legacy callers may use `:wat::test::program` to construct the forms vector. The modern macros don't need the program wrapper — the body is directly the user's intent. Unwrap as part of the migration (same pattern as 4a-β P2a).

- **Multi-form bodies.** Wrap in `(:wat::core::do ...)` if a single legacy call's body decomposes into multiple top-level forms.

- **scope :Option<String>**. Always DROP per the BRIEF (legacy substrate plumbing; never functional).

- **wat_run_sandboxed.rs (8 sites)**. Largest file; could surface migration patterns that need per-site judgment. STOP-and-surface threshold of >5 non-trivial sites applies here specifically.

- **Doc-comment refresh.** 3 files have line-1 doc-comments naming the legacy verb. Update to name the canonical macro the file now tests.

## Workspace baseline (commit `ee406b8`)

- `cargo build --release --workspace --tests`: clean
- `cargo test --release --workspace --no-fail-fast`: 2270 passed / 3 failed (lifeline_pipe_zero_orphans, tmp_totally_bogus, startup_error_bubbles_up_as_exit_3 — all pre-existing rotation members)

Post-slice-4c-α-ii target:
- 2270+ passed (no test deletions; migrations may add 1-2 passes if previously rotating)
- ≤ 11 failed (variance band)

## Calibration record (to fill on completion)

| Metric | Predicted | Actual |
|---|---|---|
| Wall-clock runtime | 30–60 min | TBD |
| Scorecard rows | 6/6 PASS | TBD |
| Workspace fail count | ≤ 11 | TBD |
| Thread destinations | 11-13 | TBD |
| Hermetic destinations | 3-5 | TBD |
| Layer 2 escalations | 0-1 | TBD |
| Doc-comment files refreshed | 3 | TBD |
| Mode | A (clean) | TBD |
