# Arc 138 F-NAMES-2/4-coords — Sonnet Brief: lambda + entry get definition coordinates

**Goal:** template names (`<lambda>`, `<entry>`) are fine, but the user mandated they pair with REAL coordinates pointing at where they occur. After this slice:
- `<lambda>` renders as `<lambda@<file>:<line>:<col>>` using `cur_func.body.span()` (the definition site)
- `<entry>` callers pass real `concat!(file!(), ":", line!())` labels (Rust caller location) instead of `None`

**Working directory:** `/home/watmin/work/holon/wat-rs/`.

**Driver:** the user's "the template name is fine as long as we point to where it occurs" principle. F-NAMES-2 and F-NAMES-4 investigations marked these "not cracks" — but the user noted we have the COORDINATES available and should use them.

## Read in order

1. `docs/arc/2026/05/138-checkerror-spans/SCORE-F-NAMES-2-3-4-INVESTIGATIONS.md` — the investigation that called these "not cracks" but missed the coordinate refinement.
2. `docs/arc/2026/05/138-checkerror-spans/NAMES-AUDIT.md` — full names audit context.
3. src/runtime.rs around line 11180-11230 (apply_function) and Function struct at 499.
4. src/freeze.rs around 148 + 177 (sigma fn registration) and 408-425 (startup_from_source).
5. src/check.rs around 8145 (ReturnTypeMismatch).

## What to do — F-NAMES-2-coords (lambda)

For each site that uses `cur_func.name.clone().unwrap_or_else(|| "<lambda>".into())`, replace the fallback with one that includes the definition span:

```rust
let callee_name = match cur_func.name.clone() {
    Some(name) => name,
    None => format!("<lambda@{}>", cur_func.body.span()),
};
```

Sites to update (5 total):
- src/runtime.rs:11191 (frame name in apply_function)
- src/runtime.rs:11197 (ArityMismatch.op in apply_function)
- src/runtime.rs:11225 (tail-call replace_top_frame)
- src/freeze.rs:148 (presence sigma fn path)
- src/freeze.rs:177 (coincident sigma fn path)
- src/check.rs:8145 (ReturnTypeMismatch.function — uses `body` directly; render `<lambda@{}>` with body.span())

The Span Display impl renders as `file:line:col`, so `format!("<lambda@{}>", span)` produces `<lambda@src/freeze.rs:1305:14>`.

## What to do — F-NAMES-4-coords (entry)

Update the 11 callers of `startup_from_source(src, None, loader)` to pass a real label:
- **8 test files** (tests/wat_arc072_letstar_parametric.rs:35, tests/wat_dispatch_e1_vec.rs:51/71/91/106, tests/wat_parametric_enum_typecheck.rs:28, tests/wat_arc104_fork_program.rs:27, tests/wat_core_forms.rs:16): pass `Some(concat!(file!(), ":", line!()))` instead of `None`.
- **src/compose.rs:187**: investigate what context is available; pass either the source path or a meaningful label like `<compose-and-run>` self-identifying.
- **src/fork.rs:961**: the fork-program-ast path; likely has source context — investigate and pass it.
- **2 src/freeze.rs internal test callers** (lines 840, 994): pass `Some(concat!(file!(), ":", line!()))`.

After this sweep, the `<entry>` fallback at src/freeze.rs:421 should rarely fire — only when a caller genuinely passes None despite the change. Document remaining sites if any.

## Constraints

- Files modified: src/runtime.rs + src/freeze.rs + src/check.rs + ~10 test files + src/compose.rs + src/fork.rs.
- All 7 arc138 canaries continue to pass.
- Workspace tests pass: `cargo test --release --workspace 2>&1 | grep FAILED | grep -v trading` returns empty.
- NO commits, NO pushes.
- NO new variants; NO trait expansion.

## Verification spot-check

After fix, run a wat program with an anonymous lambda that panics; confirm panic header / frame includes `<lambda@<file>:<line>:<col>>` rather than bare `<lambda>`.

Run `RUST_BACKTRACE=1 cargo test --release --workspace 2>&1 | grep -c "<entry>"` — should still be 0 (if not, hunt down the leak).

## Reporting back

Compact (~300 words):
1. Diff stat.
2. Lambda site updates (5 confirmed; format chosen).
3. Entry site updates (11 confirmed; per-caller label).
4. Verification: 7/7 canaries; workspace; spot-check.
5. Honest deltas (any caller that didn't fit the pattern).
6. Four questions briefly.

## Why this is small-medium

5 lambda sites (uniform 3-line replacement) + 11 entry sites (uniform argument substitution). Composed simply. ~10-20 min sonnet.
