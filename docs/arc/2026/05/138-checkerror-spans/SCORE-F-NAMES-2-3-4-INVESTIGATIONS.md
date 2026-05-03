# Arc 138 F-NAMES-2/3/4 — Investigations

**Written:** 2026-05-03 by orchestrator (read-mostly investigations; no sonnet engagement needed).
**Driver:** NAMES-AUDIT secondary cracks. Determine whether each placeholder is a real user-visible UX leak or honest architecture.

## F-NAMES-2 — `<lambda>` audit

**Sites:** src/runtime.rs (Value type-name display + 3 sites in apply_function: 11187/11193/11225) + src/check.rs:8145 (ReturnTypeMismatch.function field) + src/freeze.rs (2 sites in sigma-fn registration: 148/177).

**Investigation result:**

The Value-type-name display (e.g., `"<lambda>"` for Value::wat__core__lambda) is a TYPE NAME string, same shape as `<Vector dim=N>`, `<WatAST>`, `<Sender>`. NOT a placeholder for a missing identity — it IS the type's display name. NOT a crack.

The runtime/freeze/check sites all use `cur_func.name.clone().unwrap_or_else(|| "<lambda>".into())`. The `Function.name: Option<String>` field is None ONLY at src/runtime.rs:3084 — the `(:wat::core::lambda ...)` primitive itself, when wat code creates an anonymous lambda. All 8 other Function-construction sites (lines 1369, 1396, 1513, 1584, 1611, 1685, etc.) set `name: Some(...)` — define-bound functions, struct constructors, accessors, etc.

**Verdict:** `<lambda>` fires only for genuinely-anonymous `(lambda ...)` expressions where the user wrote no name. Honest. NOT a crack.

## F-NAMES-3 — `<runtime>` invariant check

**Site:** src/span.rs:67 — Span::unknown() default file label.

**Investigation result:**

`is_unknown()` at src/span.rs:80 returns true when `line == 0 && col == 0` — exactly the sentinel state. Every Display path that renders span coordinates (e.g., `span_prefix(span)` in src/types.rs, src/macros.rs, src/check.rs, src/runtime.rs, src/lower.rs, src/edn_shim.rs, src/form_match.rs, src/config.rs) checks `is_unknown()` first and returns empty string. So `<runtime>` is suppressed before reaching user output.

Empirical confirmation: `RUST_BACKTRACE=1 cargo test --release --workspace 2>&1 | grep -c "<runtime>"` returns **0**. Zero user-visible occurrences.

The doc comment at src/span.rs:60-64 explicitly explains the sentinel only surfaces in backtraces if an internal AST node reaches a call site, and notes that `rust_caller_span!` macro is used for runtime-initiated invocations to carry real Rust file:line:col.

**Verdict:** NOT a crack. The sentinel is well-protected by `is_unknown()` checks; never user-visible.

## F-NAMES-4 — `<entry>` investigation

**Site:** src/freeze.rs:421 — `let file_label = base_canonical.unwrap_or("<entry>");`

**Investigation result:**

`base_canonical: Option<&str>` is the canonical path of the entry source. None means in-memory / test source — the doc comment at src/freeze.rs:419-420 names this case explicitly: "use the canonical path when known; fall back to `<entry>` for in-memory / test sources."

Empirical confirmation: `RUST_BACKTRACE=1 cargo test --release --workspace 2>&1 | grep -c "<entry>"` returns **0**. Zero user-visible occurrences.

When `<entry>` would render: bare `wat -c "<source-string>"` invocation with no file path. The label self-identifies as "the entry source provided directly, no canonical disk path."

**Verdict:** NOT a crack. Honest fallback for the genuinely-pathless case.

## Combined verdict

All three NAMES-AUDIT secondary cracks RESOLVE AS NOT CRACKS. Each placeholder correctly identifies a genuine architectural case where NO real identity exists:
- `<lambda>` — anonymous lambdas the user wrote without a name
- `<runtime>` — synthetic AST sentinel (suppressed before render)
- `<entry>` — in-memory entry source with no canonical path

No code changes needed. Drop these from the active cracks queue.

## Optional follow-up (low priority)

F-NAMES-3 invariant test: add a workspace-wide test that asserts `<runtime>` never appears in any test panic output. Currently confirmed empirically; could codify as a CI invariant. Defer to slice 6 if there's appetite, otherwise leave as established discipline.

## Arc 138 status post-investigations

**ALL KNOWN CRACKS CLOSED.** Only slice 6 remains:
- Slice 6: doctrine + INSCRIPTION + USER-GUIDE + 058 row → arc 138 closure.

The arc has shipped:
- 8 error types span-threaded
- 4 substrate cracks closed (F1-F4c)
- 5 F-NAMES sub-slices (1, 1c, 1d-asserthook, 1e + 2/3/4 investigations)

Every panic in the substrate now carries: real thread name, real file path, real line:col coordinates. ZERO `<unnamed>` / `<test>` / `<runtime>` / `<entry>` appearances in workspace test output.
