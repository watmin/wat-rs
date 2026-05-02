# Arc 138 Slice 2 — Sonnet Brief: TypeError gains spans

**Goal:** retrofit `Span` onto every `TypeError` variant in `src/types.rs`. Update Display arms to prefix `<file>:<line>:<col>:` on non-unknown spans. Thread real spans through every emission site. Add a canary test that asserts the rendering contains source coordinates. Same project-wide doctrine as arc 138 slice 1 (`CheckError` spans), now applied to the type-registration layer.

**Working directory:** `/home/watmin/work/holon/wat-rs/`.

**Scope:** ONE file modified primarily — `src/types.rs`. Optionally one canary test added in `src/types.rs` or `tests/` (your judgment). NO substrate-design changes outside the variant fields. NO Display string changes beyond the span prefix. NO commits.

## Read in order — your contract

1. `docs/arc/2026/05/138-checkerror-spans/DESIGN.md` — the arc framing. Read "Reference instance" and "Why this matters — agents need to navigate."
2. `docs/arc/2026/05/138-checkerror-spans/SCORE-SLICE-1-FINISH.md` — slice 1 wrapped 8/8+4/4. Substrate observation surfaced: 23 helpers gained `head_span: &Span` parameter. Same threading pattern likely applies here.
3. **The 6 worked-site references in `src/check.rs`** (see slice 1 BRIEF) — these are the canonical examples of `arg.span().clone()`, `body.span().clone()`, `head_span.clone()` patterns. They live in a sibling file but the pattern transfers.
4. **The existing `TypeError::MalformedVariant`** in `src/types.rs` (lines ~963-976) — already carries `span: Span` from arc 130 follow-up. It's the worked example IN THIS FILE for what every other variant should look like after slice 2.
5. The variant defs at lines ~957-1015 in `src/types.rs`. The Display arm at ~1017-end. Current emission sites: `grep -n "Err(TypeError" src/types.rs` returns ~26 sites.

## What to produce

**Step A — Variant defs.** Add `span: Span` to these 10 variants (`MalformedVariant` already has one; SKIP it):

- `DuplicateType`
- `ReservedPrefix`
- `MalformedDecl`
- `MalformedName`
- `MalformedField`
- `MalformedTypeExpr`
- `AnyBanned`
- `CyclicAlias`
- `AliasArityMismatch`
- `InnerColonInCompoundArg`

Each variant's added `span` field gets a 1-2 line doc comment naming the source-coordinate intent (mirror MalformedVariant's `span: Span` doc style).

**Step B — Display arms.** Update each of the 10 Display arms to prefix `<file>:<line>:<col>:` when the span is non-unknown. Use the SAME helper shape as `src/check.rs::span_prefix` (line ~362):

```rust
fn span_prefix(span: &Span) -> String {
    if span.is_unknown() { String::new() } else { format!("{}: ", span) }
}
```

Add this helper at the top of `src/types.rs` (or wherever convenient near the impl Display block). The Display string body stays exactly the same; only the leading `{prefix}` interpolation changes. NO message-text changes beyond the prefix.

`MalformedVariant` already prefixes via its existing `at <span>` substring — leave it AS-IS (don't double-prefix).

**Step C — Emission sites.** Thread real spans through every `Err(TypeError::...)` site (~26 sites). Best-source heuristics:

| Variant | Most relevant span |
|---|---|
| `DuplicateType` | The duplicate's name keyword span (the new decl's name kw). |
| `ReservedPrefix` | The offending name keyword's span. |
| `MalformedDecl` | The whole decl form's span (`form.span().clone()` if `form: &WatAST` or threaded as `decl_span: Span`). |
| `MalformedName` | The bad name keyword's span. |
| `MalformedField` | The field item's span. |
| `MalformedTypeExpr` | The bad type keyword's span. |
| `AnyBanned` | The keyword carrying `:Any`. |
| `CyclicAlias` | The alias name's span (the decl that closes the cycle). |
| `AliasArityMismatch` | The call site's span (this fires from `parse_type_expr`; often called from check.rs context — accept `Span::unknown()` if unreachable from the current call chain, with a `// arc 138: no span — <reason>` comment). |
| `InnerColonInCompoundArg` | The outermost type keyword's span. |

Helpers like `parse_struct(args: Vec<WatAST>)` consume by value — they may need a new `decl_span: Span` parameter (cheap clone; Span is `Arc<String> + i64 + i64`). Same uniform-signature discipline as slice 1's `head_span: &Span`. If you add parameters to helper functions, NAME each one explicitly in your honest deltas.

**Step D — Canary test.** Add ONE unit test that asserts a TypeError surfaced from user source carries `<test>:` (file:line:col) in its rendered Display output. Mirror `check::tests::type_mismatch_message_carries_span` shape. Place it in `src/types.rs::tests` if a `mod tests` exists, or in `tests/` as a standalone integration test. The test should:
- Construct a wat source that triggers a TypeError (e.g., `(:wat::core::struct :my::Bar)` with no fields → `MalformedDecl`; or `(:wat::core::typealias :my::T :Any)` → `AnyBanned`; etc.).
- Run startup_from_source / register_types / similar.
- Assert the rendered StartupError contains `<test>:` substring.

## Constraints

- ONLY `src/types.rs` modified (and optionally the canary test file).
- NO new variants. NO Display string changes beyond the span prefix. NO field renames.
- The existing `MalformedVariant` variant + Display arm stay as-is — they shipped in arc 130; don't touch.
- NO commits, NO pushes.
- `cargo test --release --workspace` exit=0 (excluding lab tests — `grep -E "FAILED" | grep -v trading` should be empty).
- The canary test passes — confirms TypeError Display includes coordinates for at least one variant.
- Pattern E (genuinely no span): each leftover `Span::unknown()` in an emission site MUST carry a `// arc 138: no span — <reason>` rationale comment.

## What success looks like

1. All 10 variants gain `span: Span` field (MalformedVariant unchanged — already shipped).
2. All 10 Display arms prefix the rendering with `<file>:<line>:<col>:` when non-unknown.
3. ~24+ emission sites thread real spans (rough target: 22-25 of 26 fire with real spans; 0-4 leftover Pattern E with comments).
4. Canary test asserts coordinates appear; passes.
5. Workspace test stays green excluding lab.
6. NO commits.

## Reporting back

Target ~400 words:

1. **Counts**: BEFORE `grep -c "Span::unknown()" src/types.rs` → AFTER count.
2. **Variant changes**: 10 variants gained span; list them.
3. **Display arms**: 10 arms updated to prefix; confirm `MalformedVariant` left as-is.
4. **Emission distribution**: how many sites used each pattern (DECL_SPAN / NAME_KW_SPAN / FIELD_SPAN / TYPE_KW_SPAN / ANY_BANNED_KW / etc.).
5. **Helper-fn signatures broadened**: list any helpers (parse_struct / parse_enum / parse_typealias / parse_field / parse_declared_name / parse_type_expr / etc.) that gained a `decl_span: Span` (or similar) parameter — substrate observation per slice 1's discipline.
6. **Canary**: test name + location + verification result.
7. **Verification**: `cargo test --release -p wat --lib` (your canary specifically); `cargo test --release --workspace 2>&1 | grep -E "FAILED" | grep -v trading | head -5` (should be empty).
8. **`git diff --stat src/types.rs`** output.
9. **Honest deltas** — anything beyond the brief: helper signature changes; sites where the span source wasn't obvious; any refactor needed to retain span access through `into_iter()` consumption sites.
10. **Four questions applied** (obvious / simple / honest / good UX).

## What this slice tests (meta)

Slice 1 finish proved: sonnet can sweep emission sites with real spans given (variant + Display + worked-example sites). Slice 2 EXTENDS the test: can sonnet do the FULL retrofit (add variant fields + Display arms + thread emissions + canary) in one engagement?

If clean — arc 138's pattern propagates across files. Slice 3 (RuntimeError, larger) dispatches with high confidence. The substrate's "errors carry coordinates" doctrine is reproducible.

If sonnet ships partial (e.g., variants + Display done but emissions only half-threaded) — we score honestly, write SCORE, decide whether to ship-as-is + follow-up or re-spawn for the gap.

Begin by reading the slice 1 SCORE for calibration, then DESIGN.md, then `src/types.rs`'s MalformedVariant + its Display arm. Plan the variant+Display+helper-signature changes BEFORE editing. Sweep variant defs first, Display arms second, emission sites third, canary fourth. Run cargo test after each batch. Report.
