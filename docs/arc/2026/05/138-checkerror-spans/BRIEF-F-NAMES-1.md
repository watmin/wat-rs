# Arc 138 F-NAMES-1 — Sonnet Brief: kill `<test>` placeholder, real source labels everywhere

**Goal:** eliminate the `<test>` source-label placeholder. Add `parse_one!(src)` / `parse_all!(src)` declarative macros that auto-capture call-site location via `concat!(file!(), ":", line!())`. Sweep ~140 test callers to use the macros. Update ~5 production callers to pass explicit source paths. Delete the `parse_one(src)` and `parse_all(src)` convenience wrappers (no remaining callers).

**Working directory:** `/home/watmin/work/holon/wat-rs/`.

**Driver:** the user invoked the no-placeholders rule (NAMES-AUDIT). After arc 138's span-threading, every error renders `file:line:col` — but if `file` is `<test>` the coordinates are useless. F-NAMES-1 makes them navigable.

## Read in order

1. `docs/arc/2026/05/138-checkerror-spans/NAMES-AUDIT.md` — the F-NAMES-1 charter (note F-NAMES-1a/1b folded; 1c is separate, 1d folded here).
2. `docs/arc/2026/05/138-checkerror-spans/SCORE-SLICE-5.md` — last variant-restructure slice; same protocol.
3. `src/parser.rs` lines 60-90 (parse_one, parse_all, parse_one_with_file, parse_all_with_file).

## What to do

### 1. Add declarative macros (src/parser.rs)

```rust
/// Parse one form, auto-capturing call-site Rust location as the source label.
/// Use in tests; production code calls `parse_one_with_file` with a real path.
#[macro_export]
macro_rules! parse_one {
    ($src:expr) => {
        $crate::parser::parse_one_with_file(
            $src,
            concat!(file!(), ":", line!()),
        )
    };
}

#[macro_export]
macro_rules! parse_all {
    ($src:expr) => {
        $crate::parser::parse_all_with_file(
            $src,
            concat!(file!(), ":", line!()),
        )
    };
}
```

Make the macros importable from anywhere via `crate::parse_one!` / `wat::parse_one!`.

### 2. Sweep test callers (~140 sites)

Replace `parse_one(src)` → `parse_one!(src)` and `parse_all(src)` → `parse_all!(src)` mechanically across:
- src/parser.rs (~49 sites — internal parser tests; `parse_one!(src)` form since macros are file-local)
- src/runtime.rs (~24 sites — `eval_expr` test helper + many tests)
- src/freeze.rs (~21 sites)
- src/lower.rs (~17 sites)
- src/hash.rs (~10 sites)
- src/types.rs / src/stdlib.rs / src/macros.rs / src/config.rs / src/test_runner.rs / src/resolve.rs (smaller)
- crates/wat-edn/* if any (~10 sites in tests + src)

Bulk substitution is fine — every test call site has the same shape.

### 3. Update production callers (~5 sites)

These callers have a REAL source identity available; pass it explicitly via the `_with_file` siblings:

- **src/lib.rs:201** — `pub fn run(src: &str)`. **Add** a `source_label: &str` parameter (public API change). Update internal `parse_one(src)` → `parse_one_with_file(src, source_label)`.
- **src/load.rs:370** — `load!` runtime: the fetched URL/path IS the source label; pass it to `parse_all_with_file(&fetched.source, &fetched.label)` (or whatever field carries the path).
- **src/stdlib.rs:148, 155** — stdlib loading: pass the stdlib file's name as the source label (e.g., the wat-source `name` field).
- **src/load.rs:1049, 1065, 1209** — these are inside `load.rs::tests` — use the `parse_all!(src)` macro instead.

Audit each production caller; pick the right real label.

### 4. Update lib.rs callers of `wat::run`

If `pub fn run` gains a `source_label` parameter, all callers in tests/examples need updating. Investigate and sweep.

### 5. Delete convenience wrappers

After sweep, src/parser.rs:71 (`parse_one`) and src/parser.rs:78 (`parse_all`) should have ZERO callers. DELETE them. Compile failure is the truth signal that no caller was missed.

## Constraints

- Files modified: src/parser.rs + ~15 source files containing test or production callers + crates/wat-edn if applicable. Sonnet can sweep mechanically.
- NO new variants. NO Display string changes. NO trait expansion.
- NO commits, NO pushes.
- All 7 existing arc138 canaries continue to pass.
- Workspace tests pass: `cargo test --release --workspace 2>&1 | grep FAILED | grep -v trading` returns empty.
- After F-NAMES-1, `grep -r '"<test>"' src/ crates/` should return empty (or only the lexer:461 `lex(src, Arc::new("<test>".to_string()))` test — that's separate).

## Pattern check after sweep

Verify a sample test panic now shows real Rust file path:
```
thread '...' panicked at src/freeze.rs:1305:10:19:
```
Instead of:
```
thread '...' panicked at <test>:10:19:
```

## Reporting back

Comprehensive (~400 words):

1. **Diff stat:** files modified.
2. **Macros added:** confirm `parse_one!` and `parse_all!`.
3. **Test caller sweep count:** per-file (target ~140 total).
4. **Production caller updates:** lib.rs (public API change documented), load.rs, stdlib.rs — each with the real source label used.
5. **Convenience wrappers deleted:** confirm `parse_one(src)` and `parse_all(src)` removed.
6. **Verification:** all 7 canaries pass; workspace tests pass.
7. **`grep '"<test>"' src/ crates/`:** should be near-empty.
8. **Honest deltas:** any caller that needed special handling, any wat-edn changes, public API ripple from lib::run signature change.
9. **Four questions** applied.

## Why this is one slice

By the principle "simple is uniform composition" — 140 identical test-caller substitutions PLUS ~5 production thoughtful updates PLUS macro definitions PLUS wrapper deletion is composed simply. No branching, no clever logic. Each piece is mechanical. Estimated 20-40 min sonnet runtime (sweep volume).
