# Arc 138 F3 — Sonnet Brief: close WatReader/WatWriter trait gap

**Goal:** expand `WatReader` and `WatWriter` traits in src/io.rs — every method that returns `Result<_, RuntimeError>` gains a `span: Span` parameter. Update all 7 implementors. Update 16 caller sites (13 in src/io.rs eval_io_* shims, 3 in src/runtime.rs spawn-program plumbing).

**Working directory:** `/home/watmin/work/holon/wat-rs/`.

**Driver:** the user invoked the no-deferrals rule (`docs/arc/2026/05/138-checkerror-spans/CRACKS-AUDIT.md`). WatReader/WatWriter is item 3 of 4 cracks; F3 closes it.

## Read in order

1. `docs/arc/2026/05/138-checkerror-spans/CRACKS-AUDIT.md` — F3 charter.
2. `docs/arc/2026/05/138-checkerror-spans/SCORE-SLICE-3B.md` — original observation; specifically the WatReader/WatWriter trait method category.
3. `src/io.rs` lines 36–90 (trait defs).
4. `src/io.rs` lines 107+ (impls).
5. `src/io.rs` lines 850–1150 (eval_io_* shim callers).
6. `src/runtime.rs` lines 12586, 12683, 12697 (spawn plumbing callers).

## What to do

### 1. Trait expansion (src/io.rs)

`WatReader` — 4 methods gain `span: Span`:
```rust
fn read(&self, n: usize, span: Span) -> Result<Option<Vec<u8>>, RuntimeError>;
fn read_all(&self, span: Span) -> Result<Vec<u8>, RuntimeError>;
fn read_line(&self, span: Span) -> Result<Option<String>, RuntimeError>;
fn rewind(&self, span: Span) -> Result<(), RuntimeError>;
```

`WatWriter` — 5 methods. Note `snapshot` returns Option (no error path); `close` has a default impl returning Ok. Methods that emit errors get span:
```rust
fn write(&self, bytes: &[u8], span: Span) -> Result<usize, RuntimeError>;
fn write_all(&self, bytes: &[u8], span: Span) -> Result<(), RuntimeError>;
fn flush(&self, span: Span) -> Result<(), RuntimeError>;
fn snapshot(&self) -> Option<Vec<u8>> { None }  // unchanged — no error path
fn close(&self, span: Span) -> Result<(), RuntimeError> { Ok(()) }
```

(Question for sonnet to decide: does `close` need span? Default impl returns Ok, but specific impls — PipeWriter — emit errors. Add span for consistency.)

Add `use crate::span::Span;` if not present.

### 2. Implementors (src/io.rs)

7 impl blocks need each method body updated to use the span parameter when emitting RuntimeError:
- RealStdin (line 107)
- RealStdout (line 185)
- RealStderr (line 232)
- StringIoReader (line 298)
- StringIoWriter (line 399)
- PipeReader (line 467)
- PipeWriter (line 629)

For each method body that currently emits `RuntimeError::SomeVariant { ..., span: Span::unknown() }` (with the `// arc 138 slice 3b: span TBD` rationale comment from prior slice), replace `Span::unknown()` with the threaded `span` parameter (cloned if needed). DELETE the rationale comments.

For each method body that emits errors from `?` propagation of Rust I/O errors (like `guard.read(&mut buf)?`), the `?` propagation will need to be replaced with explicit `match` that constructs RuntimeError with the span — OR the inner `RuntimeError` already exists from a sub-call and should be returned as-is (no new construction needed). Use judgment per site.

### 3. Caller updates — 16 sites

**src/io.rs (13 sites, all in eval_io_* shim functions ~lines 850-1147):**

Each shim function is a RustDispatch with `args: &[WatAST]` in scope. Use the call-form span. Pattern:
- Find the form's outer span. Many shims already have `list_span: &Span` (from prior slice 3a-finish helper-sig broadening) — use that.
- For shims that don't have list_span yet, `args` slice gives the call form. Use args[0].span() if any arg exists; otherwise add `list_span` parameter.

Example sites:
- line 850: `reader.read(n as usize)?` → `reader.read(n as usize, list_span.clone())?`
- line 863: `reader.read_all()?` → `reader.read_all(list_span.clone())?`
- (etc. for all 13 sites)

**src/runtime.rs (3 sites):**

- line 12586: `stdin_writer.write_all(payload.as_bytes())` — find the enclosing span context and pass it.
- line 12683: `stdout_reader.read_line()?` — same.
- line 12697: `stderr_reader.read_line()?` — same.

These are inside spawn-program plumbing. Find the form's span in scope — likely `args` or `list_span` is available somewhere in the enclosing function.

### 4. Cross-file check

src/harness.rs, src/fork.rs, src/compose.rs only contain type references (`Arc<dyn WatReader>`, `Arc<dyn WatWriter>`) — no method calls. They should NOT need changes.

crates/wat-cli/tests/wat_cli.rs lines 531/672 use `std::io::BufRead::read_line`, NOT WatReader — different trait. SKIP.

## Constraints

- Files modified: src/io.rs (trait def + impls + 13 eval_io_* callers) + src/runtime.rs (3 callers). NO others.
- NO trait method removal. NO Display string changes.
- NO commits, NO pushes.
- All 6 existing arc138 canaries continue to pass.
- Workspace tests pass: `cargo test --release --workspace 2>&1 | grep FAILED | grep -v trading` returns empty.
- The 16 transient `// arc 138 slice 3b: span TBD` markers in src/io.rs should drop to 0 (or near-0; any leftover Pattern E gets `// arc 138: no span — <reason>`).

## Pattern preservation

Use slice 1/2/3a-finish patterns:
- **Pattern A** (args[i].span()) for arg-position errors
- **Pattern B** (list_span/call-form span) for whole-method-call errors — DOMINATES here since trait method failures are about the call site, not specific args
- **Pattern E** (Span::unknown() with rationale) for genuinely unspanned cases (rare)

## Reporting back

Comprehensive (~400 words):

1. **Diff stat:** files modified (target: 2 — src/io.rs and src/runtime.rs).
2. **Trait expansion confirmed:** WatReader 4 methods + WatWriter 5 methods (4 with span, snapshot unchanged).
3. **7 implementors updated:** confirm per-impl that error-emitting bodies use threaded span.
4. **16 caller sites updated:** per-file pattern distribution. List call-site spans used.
5. **Slice 3b markers cleared:** count of `// arc 138 slice 3b: span TBD` in src/io.rs drops to 0 (or near).
6. **Verification:** all 6 canaries pass; workspace tests pass.
7. **Honest deltas:** anything unexpected (helper sigs broadened in eval_io_*, Pattern E sites, signature subtleties on close()).
8. **Four questions** applied.

## Why this is medium-complexity

Trait expansion + 7 implementors + 16 callers in 2 files. Each impl has multiple methods, each method body needs the span parameter wired in. Estimated 30-45 min.
