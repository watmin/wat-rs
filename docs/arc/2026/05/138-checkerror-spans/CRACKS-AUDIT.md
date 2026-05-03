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

### F4. Value-shaped API threading (slice 3b leftover)
- **Where:** ~30 Pattern E sites across spawn.rs, string_ops.rs, edn_shim.rs, marshal.rs in helpers like `expect_string(op, v: Value)`, `expect_i64(op, v: Value)`, `expect_option_string`, etc.
- **Cause:** these helpers receive already-evaluated `Value`, not `WatAST`. The originating AST has been discarded by the time the helper runs.
- **Fix:** add `span: Span` parameter parallel to the Value (e.g., `expect_string(op, v, span)`). Callers thread `args[i].span().clone()` at each call site.
- **Scope:** multi-file across spawn.rs, string_ops.rs, edn_shim.rs, marshal.rs, possibly assertion.rs and others. Caller updates wide.
- **Estimated runtime:** ~60+ min sonnet. Largest of the four.

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
5. **Slice 3b-followup-F4 (sonnet):** Value-shaped API threading across spawn/string_ops/edn_shim/marshal. ~60+ min. Largest.
6. **Slice 5 (sonnet):** ConfigError form_index → Span (the original slice 5; deferred until cracks closed).
7. **Slice 6:** doctrine + INSCRIPTION + USER-GUIDE + 058 row → arc 138 closure.

## Disk note for context refresh

If this conversation hits compaction: this audit captures what we know about arc 138's remaining cracks. The 4 numbered followups (F1–F4) plus pending F5 investigation must be CLOSED before slice 5/6 closure. No deferrals, no "earned-for-follow-up" prose. The user mandated rock-solid foundations for downstream work.
