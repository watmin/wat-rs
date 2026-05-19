# Arc 213 stone χ-3 — EXPECTATIONS

## Independent prediction

- **Runtime band:** 25-40 min Mode A. New build.rs file + cargo verification + diagnostic-probe trigger/restore cycle. Pattern is new (no Rust-source-scanner precedent in substrate); sonnet does linear implementation + careful whitelist verification.
- **LOC changed:** ~70-100 (entirely in new build.rs file; possibly +1 line in Cargo.toml if explicit declaration needed)
- **New files:** 2 (build.rs + SCORE doc)
- **Surprises expected:** LOW. Cargo's build.rs auto-detection is well-documented; scanner logic is uniform composition of file-walk + line-grep. The verification-probe step (temp violation → build fail → restore → build clean) is the main hand-execution care.

## Honest-delta watch

### Risk 1 — Cargo build.rs auto-detection

Cargo defaults: a file named `build.rs` next to `Cargo.toml` is auto-detected as the build script — no `[package] build = "build.rs"` declaration needed. Sonnet should verify this by:
1. Adding `build.rs`
2. Running `cargo build --release`
3. Confirming the script runs (look for the script's `cargo:rerun-if-changed=` output in the build log, or temporarily add a `println!("cargo:warning=χ-3 wall scanner active")` and verify it shows up)

If for some reason cargo doesn't auto-detect (older toolchain, workspace edge case): add explicit `build = "build.rs"` to the root crate's `[package]` section in `Cargo.toml`.

### Risk 2 — Whitelist coverage gap

If a legitimate `crossbeam_channel::` reference exists in a non-whitelisted file post-χ-2, the build.rs will catch it and fail the build. Two sub-cases:
- **(a)** χ-2 missed a site — that's a χ-2 gap; STOP, report, do NOT add to whitelist. The wall caught the regression at compile time; this is correct behavior. Orchestrator decides next step (re-spawn χ-2 for the missed site).
- **(b)** A legitimate-by-design reference exists that χ-2 didn't migrate by design — STOP, report what file+line+context, do NOT add whitelist. Orchestrator decides whether to extend whitelist or migrate the site.

### Risk 3 — Scanner false-positive on string literals / comments

The scanner's heuristic flags any line containing `crossbeam_channel::`. String literals like `"crossbeam_channel::Sender"` in error messages would flag. The 4 file-level whitelisted files (check/lexer/parser/types) cover the known cases. If a non-whitelisted file has a string literal mention, the scanner would catch it — possibly correct (might be a misplaced doc reference) or possibly a false positive.

The expected post-χ-2 state: ZERO `crossbeam_channel::` substring in non-whitelisted, non-cascade-primitive lines. If the scanner flags ANYTHING outside this expected zero state, it's a real finding — STOP and report.

### Risk 4 — Cargo caching interferes with diagnostic-probe verification

When sonnet adds the temporary violation to src/freeze.rs, runs cargo build (expect fail), removes the violation, runs cargo build (expect clean) — cargo's incremental cache MAY or MAY NOT replay the build.rs script depending on what changed. The `cargo:rerun-if-changed=src/` directive should force build.rs to re-run when src/ changes. Sonnet may need `cargo clean -p wat` (or just `touch build.rs`) if the second build doesn't re-run the script.

### Risk 5 — Subdirectories under src/

Currently `src/` is flat. If subdirs exist or are added later, the recursive `scan_dir` call handles them. Sonnet should verify by checking `find src -type d` and confirming the scanner handles the actual tree depth.

## Scorecard predictions

| # | Criterion | Expected |
|---|---|---|
| 1 | `build.rs` minted at workspace root (~70-100 LOC) | YES |
| 2 | Cargo auto-detects build.rs (or explicit `build = "build.rs"` in Cargo.toml) | YES |
| 3 | Scanner walks `src/*.rs` recursively | YES |
| 4 | Whitelist: typed_channel.rs / check.rs / lexer.rs / parser.rs / types.rs fully allowed | YES |
| 5 | Whitelist: runtime.rs lines containing `SHUTDOWN_` allowed | YES |
| 6 | `cargo build --release` clean (proves wall holds — χ-2's 35-site migration was complete) | YES |
| 7 | `cargo test --release --test probe_channel_primitive` 3/3 PASS unchanged | YES |
| 8 | `cargo test --release --test probe_pidfd_primitive` 2/2 PASS unchanged | YES |
| 9 | Diagnostic probe: temp violation in src/freeze.rs → `cargo build --release` FAILS with χ-3 diagnostic header | YES |
| 10 | Diagnostic probe: violation removed → `cargo build --release` clean again | YES |
| 11 | SCORE inscribes both diagnostic probe outputs verbatim | YES |
| 12 | Zero modifications outside build.rs + Cargo.toml (if needed) + SCORE doc | YES |
| 13 | Dirty tree intact (src/fork.rs + src/spawn_process.rs untouched) | YES |
| 14 | No workspace test runs (per `feedback_no_hang_vector_in_additive_scorecard`) | YES |
| 15 | Scanner output format: file:line: <code>; clear diagnostic header pointing at INTERSTITIAL doctrine | YES |

## Mode classification

- **Mode A:** all 15 criteria satisfied; wall in place; probe demonstrates catch
- **Mode B (acceptable; honest surface):**
  - Risk 1 fires (explicit Cargo.toml declaration needed): sonnet adds + documents
  - Risk 2 fires (χ-2 gap discovered): sonnet STOPs + reports the missed site; orchestrator re-spawns χ-2
  - Risk 4 fires (caching confusion): sonnet uses `cargo clean` between probe steps + documents
- **Mode C (failure):**
  - Touched any file outside build.rs + Cargo.toml + SCORE doc
  - Touched the dirty tree
  - Extended the whitelist without orchestrator approval
  - Ran workspace tests (wat_arc170)
  - Migrated additional caller sites (that's a follow-up if Risk 2(a) fires; not part of χ-3)

## Calibration metadata

- **Orchestrator confidence:** HIGH on Mode A first-attempt. The build.rs pattern is well-documented in cargo; the scanner logic is mechanical (file-walk + line-contains); the verification-probe is a clean two-step cycle.
- **Risk factors:**
  - Whitelist completeness depends on χ-2 having shipped clean (verified by orchestrator before spawning χ-3)
  - Cargo caching may need explicit `cargo clean` between probe steps
- **Why this matters:** χ-3 establishes the FIRST compile-time Rust-source-scanning wall in the substrate. Pattern precedent for future similar walls: no bare `libc::*` outside designated modules (post-arc 213 ζ), no bare `std::sync::Mutex` (ZERO-MUTEX doctrine), etc. The cascade-completeness gap that produced the 15% hang rate becomes structurally impossible at the Rust import layer.

## Tractability tiebreaker rationale (per `feedback_tractability_tiebreaker`)

χ-3 follows χ-2 in the chain (the wall verifies the migration). No further splitting needed within χ-3 — single concern (compile-time enforcement), single file (build.rs), single verification cycle (probe + restore).

The χ-4 stone (50-trial proof on wat_arc170_program_contracts) follows χ-3 and is the FINAL gate that ships δ-1 atomically with χ-1/2/3.

## Cross-references

- BRIEF-213-CHI-3-COMPILE-TIME-WALL.md — this stone's work-order
- SCORE-213-CHI-1-MINT-CHANNEL-WRAPPER.md — χ-1 (the wrapper the wall protects)
- SCORE-213-CHI-2-MIGRATE-CALLER-SITES.md — χ-2 (the migration the wall enforces)
- INTERSTITIAL § 2026-05-18 (post-δ-1 investigation) "Channel-cascade-completeness wall" — doctrine
- Arc 198 `#[restricted_to]` — wat-level access control precedent; χ-3 establishes the Rust-source compile-time analogue
- `feedback_no_hang_vector_in_additive_scorecard` — why χ-3 verifies via cargo build + probes, not wat_arc170
- `feedback_defect_fix_or_panic_never_revert` — dirty tree stays untouched
