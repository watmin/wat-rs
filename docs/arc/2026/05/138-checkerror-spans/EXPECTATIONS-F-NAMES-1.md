# Arc 138 F-NAMES-1 — Pre-handoff expectations

**Brief:** `BRIEF-F-NAMES-1.md`
**Targets:** src/parser.rs (macro defs + wrapper deletion) + ~15 source files containing callers.

## Setup — workspace state pre-spawn

- Baseline: slice 5 commit `53ec071` + NAMES-AUDIT update `ac1824e`. All 4 cracks closed; all 8 error types span-threaded.
- 144 `parse_one(src)` / `parse_all(src)` callers across src/ + tests/ + crates/.
- 7/7 arc138 canaries pass.
- src/parser.rs:71/78 hardcode `<test>` as source label in convenience wrappers.

## Hard scorecard (8 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | Macros added | `parse_one!` and `parse_all!` declarative macros in src/parser.rs; expand to `_with_file(src, concat!(file!(), ":", line!()))`. |
| 2 | Test callers swept | ~140 `parse_one(src)` → `parse_one!(src)` and same for parse_all. |
| 3 | Production callers updated | lib.rs:201 (gains source_label param); load.rs:370 (passes fetched path); stdlib.rs:148/155 (passes stdlib name). |
| 4 | Convenience wrappers deleted | `parse_one(src)` and `parse_all(src)` at src/parser.rs:71/78 REMOVED. |
| 5 | `<test>` placeholder eliminated | `grep -r '"<test>"' src/ crates/` returns empty (or only the lexer test fixture at 461). |
| 6 | Workspace tests pass | All 7 arc138 canaries PASS; `cargo test --release --workspace 2>&1 \| grep FAILED \| grep -v trading` returns empty. |
| 7 | Public API change documented | lib::run signature change called out in honest deltas (downstream callers in tests/examples updated). |
| 8 | No commits | Working tree shows uncommitted modifications only. |

## Soft scorecard (3 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 9 | Test panics show real Rust file paths | spot-check confirms `src/<file>.rs:<line>:` instead of `<test>:`. |
| 10 | Honest report | Per-file caller counts; production-caller real-label choices documented. |
| 11 | Calibration | ≤ 40 min sonnet for 144 sites + macro defs + production updates + deletions. |

## Independent prediction

Largest sweep slice in arc 138 by call-site count, but uniform per-site shape. By the simple-is-uniform-composition principle this is one slice.

- **Most likely (~50%):** 8/8 hard + 3/3 soft. Sonnet ships in 25-40 min. Bulk macro substitution + ~5 production thoughtful updates + 2 wrapper deletions.
- **Public API ripple (~25%):** lib::run signature change touches more callers than expected (examples, integration tests). Sonnet sweeps; possibly adds 5-10 min.
- **wat-edn or test code surprise (~15%):** an unexpected caller pattern. Mechanical fix.
- **Macro hygiene issue (~5%):** `parse_one!` macro export needs `#[macro_export]` + path adjustment to be importable across modules. Mechanical fix.
- **Production-caller judgment call (~5%):** for src/load.rs and src/stdlib.rs, sonnet picks a different label than expected. Acceptable if reasoned.

## Methodology

After sonnet reports back: verify diff stat, grep `"<test>"` count, run all 7 canaries, run workspace tests. Spot-check a test panic to confirm real Rust paths render. Score → commit + push → queue F-NAMES-1c (wat::test! deftest thread name).

## What this slice tells us

- All clean → the `<test>` placeholder is gone; every test panic carries navigable Rust coordinates. F-NAMES-1c (deftest thread name) + F-NAMES-2/3/4 (lambda/runtime/entry audits) + slice 6 closure remaining.
- Public API ripple → calibration data on lib::run downstream impact.
