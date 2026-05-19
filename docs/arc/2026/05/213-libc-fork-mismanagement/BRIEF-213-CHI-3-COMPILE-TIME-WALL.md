# Arc 213 stone χ-3 — Compile-time wall on bare `crossbeam_channel::` outside `src/typed_channel.rs`

## Substrate gap (recap)

After χ-1 (wrapper minted) + χ-2 (35 caller sites migrated), the wat crate's `src/` should have ZERO bare `crossbeam_channel::` references outside the canonical chokepoint locations:

- `src/typed_channel.rs` (the wrapper home; legitimate inner usage)
- `src/check.rs`, `src/lexer.rs`, `src/parser.rs`, `src/types.rs` (wat-visible type-NAME STRINGS in error messages / doc comments / type registry — NOT Rust channel callers)
- `src/runtime.rs` lines containing `SHUTDOWN_` (cascade primitive: SHUTDOWN_RX line 179 / SHUTDOWN_TX_PTR line 185 / init_shutdown_signal factory line 233)

But the discipline is only **convention** until enforced. A future PR could regress: `use crossbeam_channel::Sender;` in a new substrate module + a bare `.recv()` + the cascade-completeness wall is breached + 15% hang rate returns.

The χ doctrine (INTERSTITIAL § 2026-05-18 "Channel-cascade-completeness wall + 'we are our own users'"): make wrong shape structurally impossible. Same shape as arc 198 `#[restricted_to]` (wat-level access), arc 203 struct-restricted (substrate type), arc 212 ζ `WatAST::children()` (newtype wall). χ-3 adds the FIRST compile-time Rust-source wall in the substrate.

## Mission

**Mint a `build.rs` workspace-root scanner** that fails `cargo build` if any bare `crossbeam_channel::` reference appears in `src/*.rs` outside the whitelist.

This is the FIRST Rust-source compile-time wall pattern in wat-rs. χ-3 establishes the pattern; future arcs may add similar walls for `libc::*` (after arc 213 ζ), `std::sync::Mutex` (per ZERO-MUTEX doctrine), etc.

## Mechanical design

### File: `build.rs` at workspace root (`/home/watmin/work/holon/wat-rs/build.rs`)

```rust
//! Arc 213 χ-3 — Compile-time wall on bare `crossbeam_channel::*` in src/*.rs.
//!
//! Per the χ doctrine: bare crossbeam usage bypasses typed_recv's cascade-aware
//! select; any regression reintroduces the 15% hang class. This scanner is
//! the substrate-imposed-not-followed enforcement layer (see INTERSTITIAL
//! § 2026-05-18 "Channel-cascade-completeness wall").
//!
//! Whitelist:
//! - src/typed_channel.rs — wrapper home; legitimate inner crossbeam usage
//! - src/check.rs / lexer.rs / parser.rs / types.rs — wat-visible TYPE-NAME
//!   strings in error messages / doc comments / type registry; NOT Rust callers
//! - src/runtime.rs lines containing `SHUTDOWN_` — cascade primitive itself
//!   (SHUTDOWN_RX / SHUTDOWN_TX_PTR / init_shutdown_signal factory)

use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=src/");
    println!("cargo:rerun-if-changed=build.rs");

    let src_dir = Path::new("src");
    let mut violations = Vec::new();

    scan_dir(src_dir, &mut violations);

    if !violations.is_empty() {
        eprintln!();
        eprintln!("==========================================================");
        eprintln!("arc 213 χ-3 wall violation: bare `crossbeam_channel::` in src/");
        eprintln!("==========================================================");
        eprintln!();
        for v in &violations {
            eprintln!("  {}", v);
        }
        eprintln!();
        eprintln!("Use `wat::typed_channel::{{Sender, Receiver, unbounded, bounded}}` instead.");
        eprintln!("See INTERSTITIAL § 2026-05-18 \"Channel-cascade-completeness wall\".");
        eprintln!();
        std::process::exit(1);
    }
}

fn scan_dir(dir: &Path, violations: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_dir(&path, violations);
        } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            scan_file(&path, violations);
        }
    }
}

fn scan_file(path: &Path, violations: &mut Vec<String>) {
    let path_str = path.to_string_lossy();

    // File-level whitelist
    if path_str.ends_with("src/typed_channel.rs")
        || path_str.ends_with("src/check.rs")
        || path_str.ends_with("src/lexer.rs")
        || path_str.ends_with("src/parser.rs")
        || path_str.ends_with("src/types.rs")
    {
        return;
    }

    let Ok(content) = fs::read_to_string(path) else { return };

    for (lineno, line) in content.lines().enumerate() {
        if !line.contains("crossbeam_channel::") {
            continue;
        }

        // runtime.rs line-level whitelist: cascade primitive
        if path_str.ends_with("src/runtime.rs") && line.contains("SHUTDOWN_") {
            continue;
        }

        violations.push(format!(
            "{}:{}: {}",
            path_str,
            lineno + 1,
            line.trim()
        ));
    }
}
```

### `Cargo.toml` declaration

Verify cargo auto-detects `build.rs` at workspace root. If the root crate's `[package]` section needs an explicit `build = "build.rs"` declaration, add it. Default cargo behavior: a `build.rs` at the same level as `Cargo.toml` is auto-detected — no `build = ...` declaration needed.

### Scope discipline

- Scanner walks `src/*.rs` ONLY (recursively into src/ subdirs if any)
- Does NOT scan `crates/wat-telemetry-sqlite/` (out of scope; sister crate cleanup is a future arc if needed)
- Does NOT scan `crates/wat-edn/tests/` (out of scope; type-name string tests)
- Does NOT scan `tests/` (out of scope; integration tests can use bare crossbeam to exercise substrate edges)

## Verification

```
cargo build --release                                  # must be clean
                                                        #   (proves the wall HOLDS — no violations remain
                                                        #    after χ-2 migrated the 35 caller sites)

cargo test --release --test probe_channel_primitive    # 3/3 PASS (unchanged from χ-1)
cargo test --release --test probe_pidfd_primitive      # 2/2 PASS (unchanged from α)
```

**Mandatory diagnostic verification** (proves the wall CATCHES violations, not just passes by luck):

Temporarily add a violation in a non-whitelisted file (e.g., `src/freeze.rs` is small + already migrated by χ-2):
```rust
// At top of src/freeze.rs, add:
use crossbeam_channel::Sender as TestViolation;  // χ-3 verification probe — REMOVE
```

Run `cargo build --release`. Expected: build FAILS with the χ-3 diagnostic header pointing at `src/freeze.rs:<line>`. Remove the line. Re-run `cargo build --release`. Expected: clean.

The SCORE doc inscribes both: (a) cargo build clean post-migration; (b) the verification-probe trigger output + restored-clean output.

## Out of scope (STOP triggers)

- DO NOT migrate any additional caller sites. χ-2 is the migration; χ-3 is the wall.
- DO NOT scan crates/ or tests/. Single-stone discipline; future arc if expansion is justified.
- DO NOT add additional whitelist entries beyond those named (typed_channel.rs file + 4 type-name-string files + runtime.rs SHUTDOWN_ lines). If a NEW violation surfaces post-χ-2 that's not in the whitelist, the wall correctly catches it — STOP and report; do NOT extend whitelist to make it pass.
- DO NOT touch the dirty tree (src/fork.rs / src/spawn_process.rs). Per `feedback_defect_fix_or_panic_never_revert`.
- DO NOT run wat_arc170_program_contracts or workspace tests. Per `feedback_no_hang_vector_in_additive_scorecard`. The wall is verified by cargo build + probe survival + the diagnostic probe.

## Concrete deliverables

1. New file: `build.rs` at workspace root (~70-100 LOC)
2. Cargo.toml verification (default auto-detect; explicit declaration only if needed)
3. Verification-probe sequence executed + outputs captured in SCORE
4. SCORE doc: `docs/arc/2026/05/213-libc-fork-mismanagement/SCORE-213-CHI-3-COMPILE-TIME-WALL.md`

## Critical constraints

- DO NOT commit. Orchestrator commits after independent SCORE verification.
- DO NOT touch the dirty tree files (src/fork.rs / src/spawn_process.rs).
- Anchor cwd: `/home/watmin/work/holon/wat-rs/` — use `git -C` for git operations; verify `pwd` as first action; any `.claude/worktrees/` path is harness state and illegal to operate on.

## Cross-references

- INTERSTITIAL § 2026-05-18 (post-δ-1 investigation) "Channel-cascade-completeness wall" — doctrine
- SCORE-213-CHI-1-MINT-CHANNEL-WRAPPER.md — χ-1 mint
- SCORE-213-CHI-2-MIGRATE-CALLER-SITES.md — χ-2 migration (pre-condition for χ-3)
- Arc 198 `#[restricted_to]` — sibling pattern (wat-level access control); χ-3 establishes the Rust-source-scan compile-time-wall pattern as a NEW substrate mechanism
- Arc 212 ζ `WatAST::children()` newtype wall — sibling at AST layer
- `feedback_substrate_owns_not_callers_match` — the doctrine the wall enforces structurally
