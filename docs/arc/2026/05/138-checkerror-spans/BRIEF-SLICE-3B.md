# Arc 138 Slice 3b — Sonnet Brief: thread real spans into external file RuntimeError stubs

**Goal:** replace `Span::unknown()` with the most relevant local span at every emission site marked `// arc 138 slice 3b: span TBD` across 15 external files. The variant fields, Display arms, and helper signatures are settled (slices 3a + 3a-finish). The 156 stub sites are leftover transient stubs added to keep the workspace compiling while slice 3a was in flight.

**Working directory:** `/home/watmin/work/holon/wat-rs/`.

**Scope:** 15 files, 156 stub sites:
- `src/io.rs` (36) — **WARNING: dual-shape — see below**
- `src/time.rs` (32)
- `crates/wat-telemetry-sqlite/src/auto.rs` (18)
- `src/rust_deps/marshal.rs` (17)
- `src/fork.rs` (14)
- `src/string_ops.rs` (13)
- `crates/wat-telemetry-sqlite/src/cursor.rs` (7)
- `src/spawn.rs` (6)
- `src/edn_shim.rs` (4)
- `src/assertion.rs` (4)
- `src/sandbox.rs` (1)
- `src/hologram.rs` (1)
- `src/freeze.rs` (1)
- `crates/wat-telemetry/src/shim.rs` (1)
- `crates/wat-macros/src/codegen.rs` (1) — **WARNING: proc-macro emit site, see below**

NO substrate-design changes. NO new variants. NO trait expansion. NO new helpers beyond minor span-threading. NO commits. ONLY thread spans.

## Read in order — your contract

1. `docs/arc/2026/05/138-checkerror-spans/DESIGN.md` — arc framing.
2. `docs/arc/2026/05/138-checkerror-spans/SCORE-SLICE-3A-FINISH.md` — the predecessor slice's pattern application (especially the Pattern E rationale categories — most apply here too).
3. **Worked examples in src/runtime.rs:** the slice 3a-finish helper-sig broadening pattern. Search for `list_span: &Span` to see how shim helpers were threaded. The shape is uniform.
4. The Span shape: `src/span.rs` — `Span::unknown()` is the sentinel; `WatAST::span() -> &Span`.

## Span path note — external crate vs src/

- Sites in `src/*` use `crate::span::Span::unknown()`. Replace with the appropriate span via the same path.
- Sites in `crates/wat-telemetry-sqlite/`, `crates/wat-telemetry/`, `crates/wat-macros/` use `wat::span::Span::unknown()` (because they're outside the `wat` crate). Replace using the appropriate span; the threading parameter type is `wat::span::Span`.

## Patterns (slice 3a-finish vocabulary, applied here)

**Pattern A — args[i] in scope:** when the function does `if args.len() != N { ... }` then proceeds with `args[0]`, `args[1]`, etc., the offending arg span is `args[i].span().clone()`. For type errors on a specific arg, use that arg's span. For arity errors, see Pattern B.

**Pattern B — list_span: &Span parameter:** when the enclosing function takes `list_span: &Span`, use `list_span.clone()` for whole-form errors. If the function does NOT yet take list_span, broadening is acceptable (Pattern F). For arity errors specifically, the args slice as a whole has no single span — use list_span when broadening is feasible, OR use args[0].span() if any arg exists, OR Pattern E.

**Pattern C — match arm receiving an evaluated value:** when the marker is in a `match eval(&args[i], ...)?` arm, the OFFENDING value came from `eval(&args[i], ...)` — use `args[i].span().clone()`.

**Pattern D — head keyword binding:** if you see `WatAST::Keyword(k, _) if k == "..."` patterns, the `_` is the keyword's span. Rename to `head_span` and use it.

**Pattern E — synthetic / no real span available:** the variant is being constructed inside a trait impl method (e.g., `WatReader::read`, `WatWriter::write`), or a proc-macro emit, or a leaf I/O failure with no AST in scope. Leave `Span::unknown()` AND replace the marker with `// arc 138: no span — <reason>` (e.g., "WatReader trait method — span only at wat call site, not threadable through trait", "proc-macro emit at compile time").

**Pattern F — broaden helper signature with list_span: &Span:** when a shim function or arity helper (e.g., `arity_2(op, args)`, `expect_string(op, v)`) lacks span access, add `list_span: &Span` to the signature and propagate from callers within the same file. Cross-file broadening is OUT OF SCOPE.

## Special handling — `src/io.rs` WatReader/WatWriter trait methods

`src/io.rs` is dual-shape:
- ~half the sites are shim functions with `args: &[WatAST]` (Pattern A/F)
- ~half are inside `impl WatReader for RealStdin` / `impl WatWriter for RealStdout` trait method bodies (`fn read`, `fn read_all`, `fn read_line`, `fn write`, `fn write_all`, `fn flush`)

The trait methods do NOT receive AST context. Threading span through the trait would expand the trait surface (every implementor would need to thread a Span through). **OUT OF SCOPE.** All trait-method sites are Pattern E with rationale: `// arc 138: no span — WatReader/WatWriter trait method, span only at wat call site, threading would expand trait surface`. Document as substrate observation in your report.

This is the SAME shape as the SchemeCtx trait gap from slice 1 finish — earned for follow-up arc, not papered over.

## Special handling — `crates/wat-macros/src/codegen.rs`

This single site is a proc-macro emit — code generated at compile time, not runtime. The Span::unknown() is appropriate because the proc-macro is generating runtime code that will execute against future user inputs. Pattern E with rationale: `// arc 138: no span — proc-macro emit at compile time; runtime span emerges from caller AST`. Substrate observation.

## Constraints

- ONLY the 15 listed files modified. NO other files.
- NO test changes (the canary already passes from slice 3a; no per-file canary needed).
- NO commits, NO pushes.
- NO trait expansion (WatReader, WatWriter, SchemeCtx, etc.). Pattern E with rationale instead.
- `cargo test --release --workspace` exit=0 (excluding lab); existing canaries `runtime::tests::arc138_runtime_error_message_carries_span` + `types::tests::arc138_type_error_message_carries_span` MUST still pass.
- Marker count `// arc 138 slice 3b: span TBD` drops to 0 (every marker resolved — to either real span or Pattern E rationale).

## What success looks like

1. Every shim-function-with-args site uses real span (Pattern A or F).
2. Every trait-method site or proc-macro site has Pattern E rationale.
3. Workspace tests stay green; canaries pass.
4. Diff scoped to the 15 listed files.
5. NO commits.

## Reporting back

Target ~400 words:

1. **Counts**: BEFORE marker count (156) → AFTER count (target 0). Run `grep -rc "arc 138 slice 3b" src/ crates/ | awk -F: '{s+=$2} END {print s}'` and report both.
2. **Pattern distribution**: how many sites used each of patterns A, B, C, D, E, F. Per-file breakdown if helpful.
3. **Files touched**: which of the 15.
4. **Verification**: `cargo test --release --workspace` totals. Both canary results.
5. **`git diff --stat`** — should be 15 files (or fewer if any had 0 changes after Pattern E).
6. **Honest deltas** — Pattern E rationale categories named (especially io.rs trait methods, codegen.rs proc-macro emit); any helper sigs broadened with list_span (Pattern F) named per-file; cross-file broadening (which is forbidden) confirmed not needed.
7. **Four questions applied** to your output.

## What this slice tests (meta)

The hypothesis: with slice 3a + 3a-finish complete, sonnet can sweep the leftover 156 transient stubs across 15 files in one engagement. Patterns are well-understood. Trait-impl boundary is the substrate observation (already named).

If clean → arc 138's RuntimeError sweep is complete; queue slice 4 (MacroError + 3 others).
If Pattern E count is higher than expected (>30%) → real substrate observation; documents which trait surfaces lack span carriers.
If a regression surfaces → diagnose; fix or report honestly.

Begin by reading the worked sites in src/runtime.rs (look at the `list_span: &Span` helper-sig pattern). Plan the per-file pattern split. Sweep file-by-file. Run `cargo test --release --workspace` after every 3-4 files to catch regressions early. Report.
