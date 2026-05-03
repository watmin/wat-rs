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

### Possible cracks (worth checking; less common visibility) — ALL INVESTIGATED, ALL NOT CRACKS

See `SCORE-F-NAMES-2-3-4-INVESTIGATIONS.md` for the combined investigation results.

3. **`<lambda>`** — INVESTIGATED, NOT A CRACK. Fires only for genuinely-anonymous `(lambda ...)` expressions. `Function.name = None` only at the lambda primitive (src/runtime.rs:3084); all 8 other Function constructors set `name: Some(...)`. Honest fallback.

4. **`<runtime>`** — INVESTIGATED, NOT A CRACK. Suppressed by `is_unknown()` checks in every span_prefix render path. Workspace test produces 0 `<runtime>` occurrences. Sentinel never user-visible.

5. **`<entry>`** — INVESTIGATED, NOT A CRACK. Honest fallback for in-memory/test sources with no canonical disk path. Workspace test produces 0 `<entry>` occurrences.

### NOT cracks (intentional shape-describing UI)

- `<source>` / `<path>` (load.rs, runtime.rs) — describe load! input SHAPE, not identity. Correct.
- `<env>` / `<none>` (runtime.rs:529) — describe closed_env presence. Correct.
- `<non-keyword-head>` (runtime.rs:13946) — describes a structural condition. Correct.
- `<unknown>` / `<symbol>` (test_runner.rs) — display fallbacks for nameless test items. Correct.

## Proposed fix plan

### F-NAMES-1: `<test>` and `<unnamed>` (broader scope after investigation)

**Finding (2026-05-03):** `<test>` is hardcoded in `parse_one(src)` and `parse_all(src)` convenience wrappers (src/parser.rs:71/78). Callers that inherit `<test>`:
- src/lib.rs:201 — `pub fn run(src: &str)` USER-FACING public API
- src/runtime.rs lib tests (`eval_expr` helper at line 15138)
- src/macros.rs / config.rs / resolve.rs lib tests
- src/stdlib.rs production stdlib loading (lines 148, 155)
- Any direct `parse_one`/`parse_all` caller in tests

`<unnamed>` is from src/panic_hook.rs:105 + the unnamed thread spawned at crates/wat-macros/src/lib.rs:672 (wat::test! deftest worker).

**Decomposed fix plan:**

#### F-NAMES-1a: deprecate `<test>` convenience wrapper default
- Change src/parser.rs:71/78 — `parse_one(src)` / `parse_all(src)` either:
  - **Option A**: remove the convenience wrappers entirely; force every caller to use `_with_file(src, label)` siblings.
  - **Option B**: change the default from `<test>` to `<unknown-source>` so the placeholder loudly self-identifies as a missing-label gap.
- Either way, audit every caller and provide a real label.
- **Scope:** src/parser.rs + every caller (~10 sites across runtime.rs/macros.rs/config.rs/resolve.rs/stdlib.rs/lib.rs).
- **Estimated runtime:** 15-25 min sonnet.

#### F-NAMES-1b: Rust test helpers pass file!()
- Test helpers like `eval_expr(src)` in src/runtime.rs at line 15135 use `parse_one(src)` directly. After F-NAMES-1a, they need to pass a real label.
- Option: helper macro `eval_expr!(src)` that captures `file!()` / `function_name!()` at call site, OR test helpers take `(src, file_label)` explicitly.
- **Scope:** test helpers across src/.
- Folds into F-NAMES-1a likely.

#### F-NAMES-1c: wat::test! macro deftest thread name
- crates/wat-macros/src/lib.rs:672 — `std::thread::spawn` becomes `Thread::Builder::new().name(format!("wat-test::{}", deftest_name)).spawn(...)`.
- Panic header reads `thread 'wat-test::my_deftest_name' panicked at <real-wat-file>:10:19:` — both pieces become navigable.
- **Scope:** single-file fix in wat-macros/src/lib.rs.
- **Estimated runtime:** 5-10 min sonnet.

#### F-NAMES-1d: lib::run public API source label
- src/lib.rs:201 `pub fn run(src: &str)` should accept a `source_label: &str` parameter (or auto-detect via call-site? unlikely without a macro).
- Public API change — needs versioning consideration.
- **Scope:** src/lib.rs + any callers in tests/examples.
- **Estimated runtime:** 10-15 min sonnet.

**Total F-NAMES-1 estimate:** 30-60 min across 4 sub-slices. Could fold 1a+1b together, 1c standalone, 1d standalone.

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

## Position in queue (updated 2026-05-03 post-investigation)

Per user direction: address AFTER span work wraps. Span work wrapped at slice 5 commit `53ec071`. Now:

1. ✓ F4b FromWat (commit `fbcc1a4`)
2. ✓ F4c ThreadOwnedCell + adjacent (commit `55c21f6`)
3. ✓ Slice 5 ConfigError (commit `53ec071`)
4. ✓ F-NAMES-1 (commit `fc32611`) — `<test>` placeholder eliminated; convenience wrappers deleted; 132 callers swept; production callers explicit.
5. ✓ F-NAMES-1c (commit `76e2b76`) — wat::test! deftest worker named via Thread::Builder.
6. ✓ F-NAMES-1d-asserthook (commit `f803712`) — AssertionPayload.thread_name field captures name at panic site.
7. ✓ F-NAMES-1e (commit `c8a0ed8`) — wat-side spawn workers named (3 sites). ZERO `<unnamed>` panics.
8. ✓ F-NAMES-2/3/4 investigations — all NOT cracks; see SCORE-F-NAMES-2-3-4-INVESTIGATIONS.md.
9. Slice 6 — doctrine + INSCRIPTION + USER-GUIDE + 058 row → arc 138 closure (NEXT and FINAL).

All known cracks closed. Slice 6 is documentation + closure.

## Disk note for context refresh

If this conversation hits compaction: this audit captures the placeholder-label cracks revealed AFTER arc 138's span-threading work landed coordinates everywhere. The coordinates only help if `file` and `thread name` are real — `<test>`/`<unnamed>` defeat the purpose.

Primary fix is single-file (crates/wat-macros/src/lib.rs proc-macro emit). Secondary fixes (`<lambda>`, `<runtime>`, `<entry>`) need investigation passes.

Same no-deferrals doctrine applies: every named placeholder gets resolved or rationaled.
