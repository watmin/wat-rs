# Arc 138 Slice 5 — Sonnet Brief: ConfigError form_index → Span

**Goal:** add `span: Span` to all 8 `ConfigError` variants in src/config.rs. For 2 variants (`SetterAfterNonSetter`, `MalformedSetter`) currently using `form_index: usize`, REPLACE form_index with span (drop the field; the span subsumes its purpose). Update 8 Display arms via local `span_prefix` helper. Thread real spans into ~40 emission sites. Add canary.

**Working directory:** `/home/watmin/work/holon/wat-rs/`.

**Driver:** the original arc 138 slice 5, deferred during cracks campaign. All 4 cracks now closed; resuming the slice 5/6 closure path.

## Read in order

1. `docs/arc/2026/05/138-checkerror-spans/CRACKS-AUDIT.md` — closes status (all 6 cracks F1-F4c shipped).
2. `docs/arc/2026/05/138-checkerror-spans/SCORE-SLICE-4A.md` — same shape: variant restructure + Display + emission threading + canary in one slice.
3. `docs/arc/2026/05/138-checkerror-spans/SCORE-SLICE-2.md` — original worked example (TypeError).
4. `src/config.rs` lines 115-205 (variants + Display).
5. `src/freeze.rs` — confirms ConfigError is wrapped (no field-level destructure) — invisible to restructure.

## Variant restructure plan

8 variants. **Two form_index migrations + 6 plain span additions.** Convert unit/struct variants per pattern.

```rust
SetterAfterNonSetter {
    setter_head: String,  // KEEP
    span: Span,           // NEW (replaces form_index)
    // form_index: usize, ← DROP
}
DuplicateField { field: String, span: Span }
RequiredFieldMissing { field: String, span: Span }
UnknownSetter { head: String, span: Span }
BadArity { head: String, expected: usize, got: usize, span: Span }
BadType { field: String, expected: &'static str, got: &'static str, span: Span }
BadValue { field: String, reason: String, span: Span }
MalformedSetter {
    span: Span,           // NEW (replaces form_index)
    // form_index: usize, ← DROP
}
```

`MalformedSetter` becomes effectively a unit-with-span — could be tuple `MalformedSetter(Span)`. Pick whichever shape reads cleanest with the rest of the file.

## Display arm updates

Add file-local `span_prefix(span: &Span) -> String` helper (mirror src/macros.rs / src/types.rs).

Each Display arm prefixes `{span_prefix(span)}`. **Two arms have intentional Display string content changes:**

- `SetterAfterNonSetter`: drop the `(form index N)` suffix from the message — span coords supersede.
- `MalformedSetter`: drop `at form index N` from the message — same reason.

These two Display changes are INTENTIONAL and on-mission (form_index served as a poor-man's coordinate; spans replace it cleanly). Other 6 arms are pure prefix additions.

## Emission threading

~40 emission sites in src/config.rs, all inside `collect_entry_file` and helpers. The forms slice + per-form iteration already has the AST in scope.

- For sites currently using `form_index` (e.g., `forms[i]`), the span source is `forms[i].span().clone()` — that's the same form_index used to point at, but as a real span.
- For other sites, the span source is the offending form/arg's span (Pattern A or B from prior slices).

DELETE all `form_index: i` from emission constructors after threading the corresponding span.

## Canary

Add a test `config::tests::arc138_config_error_message_carries_span` that triggers a representative variant (e.g., `BadType` via wrong-typed setter, or `MalformedSetter` via empty list setter), asserts `<test>:` substring in rendered Display.

## Constraints

- ONLY src/config.rs modified. NO other files (src/freeze.rs only wraps ConfigError; no destructure changes needed).
- NO trait expansion.
- NO commits, NO pushes.
- All 6 existing arc138 canaries continue to pass.
- Workspace tests pass: `cargo test --release --workspace 2>&1 | grep FAILED | grep -v trading` returns empty.
- The 2 Display content changes (drop form_index suffix) are INTENTIONAL — name them in the report under honest deltas.

## Reporting back

Compact (~300 words):

1. **Diff stat:** 1 file (src/config.rs).
2. **8 variants restructured:** confirm form_index dropped from 2; span added to 8; final shapes per variant.
3. **8 Display arms:** confirm span_prefix; name the 2 with intentional content changes.
4. **Emission distribution:** ~40 sites, pattern split (A/B/E).
5. **Canary:** name + line + what it triggers.
6. **Pre/post Span::unknown() in src/config.rs:** likely 0 → 0 (since variants didn't carry span before; rationale check).
7. **Verification:** all 7 canaries pass (6 existing + 1 new); workspace tests.
8. **Honest deltas.**
9. **Four questions** briefly.

## Why this is small

8 variants + 8 Display + ~40 emissions + 1 canary, single file, no cross-file consumers. F4a was 16 variants + ~50 emissions + cross-file (10 min). Slice 5 is smaller; estimated 8-15 min.
