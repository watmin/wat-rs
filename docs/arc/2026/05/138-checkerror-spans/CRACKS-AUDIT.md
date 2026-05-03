# Arc 138 — Cracks Audit (no deferrals)

**Written:** 2026-05-03 after slices 1–4b shipped.
**Driver:** user invoked the no-deferrals rule. Every crack we know about gets closed before slice 5/6 closure.

## Definition of "crack"

A crack = an emission site where:
- A user-visible error message lacks file:line:col coordinates that a normal Rust error would have
- AND we know how to fix it

NOT a crack = an emission site where there genuinely is no AST to read a span from (parse_program from raw strings, Vec element iteration where collection list_span IS used, empty-list with no head, synthesized dispatchers, proc-macro compile-time emit). These are correct architecture; no Display coordinate would even be honest.

## Audit results — REAL CRACKS

### F1. MacroDef structural span gap (slice 4a leftover)
- **Where:** src/macros.rs `register` and `register_stdlib` methods. 3 Pattern E sites.
- **Cause:** `MacroDef` struct carries no source span; `register(MacroDef)` has nothing to thread.
- **Fix:** add `span: Span` field to `MacroDef`; thread span at every `MacroDef::new(...)` / construction site. Update register to use `def.span.clone()` for emitted errors.
- **Scope:** single file (src/macros.rs). Possibly src/freeze.rs if it constructs MacroDef (check on disk).
- **Estimated runtime:** ~10-15 min sonnet.

### F2. SchemeCtx trait gap (slice 1 leftover)
- **Where:** src/check.rs ~3 leftover Pattern E sites in `push_type_mismatch`, `push_arity_mismatch`, `push_malformed` methods of the SchemeCtx trait.
- **Cause:** SchemeCtx trait methods take no AST; Rust shim implementors call them without span context.
- **Fix:** expand SchemeCtx trait — each `push_*` method gains `span: Span` parameter. Update all implementors.
- **Scope:** src/check.rs (trait def + 3 emission sites) + src/rust_deps/mod.rs (impl of SchemeCtx) + any shim that uses SchemeCtx.
- **Estimated runtime:** ~30-45 min sonnet.

### F3. WatReader/WatWriter trait gap (slice 3b leftover)
- **Where:** src/io.rs ~16 sites in trait method impls (`fn read`, `fn read_all`, `fn read_line`, `fn write`, `fn write_all`, `fn flush`).
- **Cause:** `WatReader` and `WatWriter` traits take `&self` only (or `bytes: &[u8]` for write); no AST in scope.
- **Fix:** expand WatReader/WatWriter traits — add `span: Span` parameter to each method. Update all implementors (RealStdin, RealStdout, PipeReader, PipeWriter, ScopedReader, etc.) AND callers (the wat eval shim that invokes `reader.read(...)` etc.).
- **Scope:** src/io.rs (trait def + impls + callers in eval_io_* functions).
- **Estimated runtime:** ~30-45 min sonnet.

### F4. Value-shaped API threading (slice 3b leftover) — DECOMPOSED into F4a/F4b/F4c

After F3 ship, F4 inventory revealed three distinct sub-cracks sharing the "Value lacks AST context" theme. Each is its own slice:

#### F4a. Value-shaped helpers (spawn.rs / string_ops.rs / io.rs)
- **Where:** ~11 helpers like `expect_string(op, v: Value)`, `expect_i64`, `two_strings`, `expect_reader`, `expect_writer`, etc.
- **Cause:** helpers receive already-evaluated `Value`, not `WatAST`.
- **Fix:** add `span: Span` parameter to each helper. Callers thread span (already in scope as `list_span` or `args[i]`).
- **Scope:** src/spawn.rs (4 helpers), src/string_ops.rs (1 helper, called many times), src/io.rs (5 helpers + leftover from F3). ~11 helpers + ~30 call sites.
- **Estimated runtime:** 15-25 min sonnet.

#### F4b. FromWat trait expansion (marshal.rs)
- **Where:** `pub trait FromWat` at src/rust_deps/marshal.rs:46 with `from_wat(v: &Value, op: &'static str) -> Result<Self, RuntimeError>` method. ~10 impls (i64/f64/bool/String/Unit/Option/Vec/etc.).
- **Cause:** Same shape as WatReader/WatWriter trait gap (closed by F3). FromWat::from_wat takes Value but no span; the 17 Pattern E sites in marshal.rs are inside the impls.
- **Fix:** expand trait surface — `from_wat(v: &Value, op: &'static str, span: Span) -> Result<Self, RuntimeError>`. Update all impls + callers.
- **Scope:** src/rust_deps/marshal.rs only (trait + impls + callers all live here).
- **Estimated runtime:** 15-25 min sonnet (similar shape to F3).

#### F4c. ThreadOwnedCell::with_mut signature broadening
- **Where:** ThreadOwnedCell::with_mut method (mentioned by F3 sonnet); takes `&'static str` for the op name but no span.
- **Cause:** Used inside StringIoWriter::write/write_all (and possibly other places) where owner-check failures emit RuntimeError without span context.
- **Fix:** add `span: Span` parameter to with_mut. Update implementors (1 file probably). Update callers.
- **Scope:** smaller — single helper expansion + few callers.
- **Estimated runtime:** 5-15 min sonnet.

#### Out of F4 scope (intentionally Pattern E)
- **edn_shim.rs Pattern E sites:** the EDN parser layer is correct architecture (per F5 investigation); Pattern E here is intentional, NOT a crack.
- **time.rs / fork.rs / sandbox.rs OS errors:** chrono parsing failures + fork() syscall errors + path validation errors don't have AST context to thread; genuine Pattern E.

### F5. EdnReadError direct Rust callers — INVESTIGATED, NOT A CRACK
- **Investigation result (2026-05-03):** grep across src/ + crates/ found read_edn called only at src/runtime.rs:12686 (inside an eval shim that catches EdnReadError and wraps with span). edn_to_value called only at src/runtime.rs:13149 (also inside a wrapper). NO direct external callers in wat-telemetry-sqlite or anywhere else.
- **Conclusion:** the two-layer architecture (EDN parser layer is span-blind; runtime wrappers add span context) is fully consistent. EdnReadError is correct architecture, not a gap.

## NOT cracks (drop "earned-for-follow-up" language from prior SCOREs)

- parse_program raw strings — pre-AST; fundamentally no AST yet.
- Vec element iteration — collection list_span IS used; per-element AST never existed.
- Empty-list MalformedCall (lower.rs) — no head element to read span from.
- Synthesized dispatchers — no originating user AST.
- proc-macro emit at compile time — runtime span emerges from caller AST via real-spanned wrappers.
- The `eval_edn_read` wrapper path — correct two-layer design; wraps with span at the runtime boundary.
- ClauseGrammarError missing Display impl — already fixed in slice 4a.

## Attack plan

Sequential sonnet engagements. Each is its own slice with BRIEF + EXPECTATIONS + sonnet + verify + SCORE.

1. ~~F5 investigation~~ — DONE; not a crack.
2. **Slice 4a-followup-F1 (sonnet):** MacroDef gains span field. Smallest crack; validates the followup pattern. ~10-15 min.
3. **Slice 1-followup-F2 (sonnet):** SchemeCtx trait expansion + 3 implementors. ~30-45 min.
4. **Slice 3b-followup-F3 (sonnet):** WatReader/WatWriter trait expansion + implementors + eval shim callers. ~30-45 min.
5. **F4a (sonnet):** Value-shaped helpers in spawn/string_ops/io. 15-25 min.
6. **F4b (sonnet):** FromWat trait expansion in marshal.rs. 15-25 min.
7. **F4c (sonnet):** ThreadOwnedCell::with_mut signature broadening. 5-15 min.
8. **Slice 5 (sonnet):** ConfigError form_index → Span (the original slice 5; deferred until cracks closed).
9. **Slice 6:** doctrine + INSCRIPTION + USER-GUIDE + 058 row → arc 138 closure.

## Disk note for context refresh

If this conversation hits compaction: this audit captures what we know about arc 138's remaining cracks. The 4 numbered followups (F1–F4) plus pending F5 investigation must be CLOSED before slice 5/6 closure. No deferrals, no "earned-for-follow-up" prose. The user mandated rock-solid foundations for downstream work.
