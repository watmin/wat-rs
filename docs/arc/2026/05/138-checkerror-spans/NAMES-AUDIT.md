# Arc 138 — Names Audit (placeholder labels to resolve)

**Written:** 2026-05-03 after F3 ship + user UX observation.
**Status:** plan-only. Execution deferred until F4b/F4c + slice 5 ship; arc 138 closure rolls these in before slice 6.
**Driver:** user invoked the no-placeholders rule. After span threading, every error renders `file:line:col` — but if `file` is `<test>` and the panic header is `<unnamed>`, the coordinates can't be navigated. The crack is real.

## What the user sees

Real panic from a wat::test! snippet:
```
thread '<unnamed>' panicked at <test>:10:19:
assert-eq failed
  actual:   1
  expected: 2
stack backtrace:
   0: :wat::test::assert-eq at <test>:10:19
```

Both `<unnamed>` and `<test>` are unhelpful — neither lets the user open the file or identify the test.

## Inventory — placeholder labels in the substrate

Sweep result from `grep -rn '"<[a-z][a-z-]*>"' src/ crates/`:

### Cracks to resolve (user-visible, harm navigation)

1. **`<test>`** — src/parser.rs:71 + src/parser.rs:78 + src/lexer.rs:461
   - The `parse_one(src)` / `parse_all(src)` convenience wrappers hardcode `<test>` as the source label.
   - The `_with_file` siblings accept a real label.
   - `wat::test!` proc-macro at crates/wat-macros/src/lib.rs uses the convenience wrappers.

2. **`<unnamed>`** — src/panic_hook.rs:105
   - `let thread_name = thread.name().unwrap_or("<unnamed>");`
   - `wat::test!` proc-macro at crates/wat-macros/src/lib.rs:672 spawns the test thread via bare `std::thread::spawn(move || { ... })` — no `.name()` call.
   - Rust's catch_unwind machinery shows `<unnamed>` as the thread label.

### Possible cracks (worth checking; less common visibility)

3. **`<lambda>`** — src/runtime.rs lines 9997, 11187, 11193, 11225 + src/check.rs:8145 + src/freeze.rs:148, 177
   - Anonymous lambda fallback. Visible when error messages mention the callee.
   - May be acceptable when the lambda is genuinely anonymous (literal `(lambda (x) x)` not bound to a name); less acceptable when the lambda IS named via `define` and we lose the name.
   - Investigation needed to distinguish.

4. **`<runtime>`** — src/span.rs:67
   - `Span::unknown()` default file label.
   - SHOULD be invisible after arc 138 — every error path that emits a span uses a real one. If `<runtime>` ever surfaces in user output, it's a Pattern E gap we missed (and the rationale comment somewhere is wrong).
   - Audit task: grep for any user-visible error that renders `<runtime>:` — should find zero.

5. **`<entry>`** — src/freeze.rs:421
   - Entry-file label fallback when base_canonical is None.
   - Probably correct architecture (no entry path = bare `wat <code>`); investigate.

### NOT cracks (intentional shape-describing UI)

- `<source>` / `<path>` (load.rs, runtime.rs) — describe load! input SHAPE, not identity. Correct.
- `<env>` / `<none>` (runtime.rs:529) — describe closed_env presence. Correct.
- `<non-keyword-head>` (runtime.rs:13946) — describes a structural condition. Correct.
- `<unknown>` / `<symbol>` (test_runner.rs) — display fallbacks for nameless test items. Correct.

## Proposed fix plan

### F-NAMES-1: `<test>` and `<unnamed>` (the wat::test! macro emit)

Single-slice fix. Modify the `wat::test!` proc-macro at crates/wat-macros/src/lib.rs:

1. Capture `test_name` (the function name) in the macro's input parsing — already available since the macro generates `fn <name> { ... }`.
2. Compute source label at emit time: `&format!("{}::{}", file!(), test_name)` → renders as `tests/wat_macros.rs::my_test_name` in error messages.
3. Pass that label as the source argument to `parse_*` (use the `_with_file` variants).
4. Spawn the test thread via `Thread::Builder::new().name(format!("wat-test::{}", test_name)).spawn(...)` (the macro at line 672) → panic header reads `thread 'wat-test::my_test_name' panicked at tests/wat_macros.rs::my_test_name:10:19:`.

Optional: also capture `line!()` from the macro invocation site for the source label — gives `tests/wat_macros.rs:42::my_test_name` if that's clearer.

**Scope:** crates/wat-macros/src/lib.rs (one file). Add a canary test that exercises a failing wat::test! and asserts the rendered panic mentions the test function's actual Rust file path.
**Estimated runtime:** 15-25 min sonnet (proc-macro emit changes + canary).

### F-NAMES-2: `<lambda>` audit (decide which sites lose name info)

Audit src/runtime.rs lines 9997, 11187, 11193, 11225 + src/check.rs:8145 + src/freeze.rs:148, 177:
- For each `unwrap_or_else(|| "<lambda>".into())`, check whether the lambda ALWAYS has no name OR whether some callers pass a `define`-bound lambda whose name we're discarding.
- If a name is available upstream, thread it; otherwise leave `<lambda>` (acceptable for genuinely anonymous lambdas).

**Scope:** investigation pass first; fix per-site as needed. May fold into slice 6 closure.

### F-NAMES-3: `<runtime>` invariant check

Add a workspace-wide regex assertion that no user-visible error output renders `<runtime>:`. If any tests render that, we have a missed Pattern E somewhere — fix at root.

**Scope:** test or CI check. Small. Slice 6 candidate.

### F-NAMES-4: `<entry>` investigation

Check src/freeze.rs:421 — confirm `<entry>` only fires when there genuinely is no entry path (bare `wat -c "<code>"`). If so, document and leave. If not, thread the real path.

**Scope:** investigation, probably documentation-only fix.

## Position in queue

Per user direction: address AFTER span work wraps:

1. F4b — FromWat trait expansion
2. F4c — ThreadOwnedCell::with_mut
3. Slice 5 — ConfigError form_index → Span
4. **F-NAMES-1** — wat::test! macro emit (this audit's primary fix)
5. **F-NAMES-2** — `<lambda>` audit
6. **F-NAMES-3** — `<runtime>` invariant check
7. **F-NAMES-4** — `<entry>` investigation
8. Slice 6 — doctrine + INSCRIPTION + USER-GUIDE + 058 row → arc 138 closure

## Disk note for context refresh

If this conversation hits compaction: this audit captures the placeholder-label cracks revealed AFTER arc 138's span-threading work landed coordinates everywhere. The coordinates only help if `file` and `thread name` are real — `<test>`/`<unnamed>` defeat the purpose.

Primary fix is single-file (crates/wat-macros/src/lib.rs proc-macro emit). Secondary fixes (`<lambda>`, `<runtime>`, `<entry>`) need investigation passes.

Same no-deferrals doctrine applies: every named placeholder gets resolved or rationaled.
