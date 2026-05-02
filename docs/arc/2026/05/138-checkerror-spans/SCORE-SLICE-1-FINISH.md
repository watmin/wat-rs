# Arc 138 Slice 1 (Finish) — SCORE

**Written:** 2026-05-03 AFTER sonnet's report.
**Agent ID:** `aaa98d49c1f1ffe5d`
**Runtime:** ~30 min (1838 s).

## Independent verification (orchestrator)

| Claim | Sonnet's value | Orchestrator verified |
|---|---|---|
| `Span::unknown()` BEFORE | 206 | 206 (matches the EXPECTATIONS doc baseline) |
| `Span::unknown()` AFTER | 3 | **3** ✓ |
| `// arc 138: no span` comment count | 3 | **3** ✓ (each leftover site has its rationale) |
| `git diff --stat src/check.rs` | 1 file, 304+ / 255- | **1 file, 304+ / 255-** ✓ |
| Canary `type_mismatch_message_carries_span` | PASS | PASS ✓ |
| Workspace `cargo test --release --workspace` | 0 failures (excl lab) | 0 failures (excl lab) ✓ |
| 3 leftover are SchemeCtx trait impl | yes | yes — `push_type_mismatch` / `push_arity_mismatch` / `push_malformed` ✓ |

Diff stat alone exceeds the 85% drop target: **98.5% of `Span::unknown()` instances replaced with real spans**.

## Hard scorecard (8 rows)

| # | Criterion | Result | Evidence |
|---|---|---|---|
| 1 | Single-file diff | **PASS** | Only `src/check.rs` modified. |
| 2 | Span::unknown() count drops | **PASS+** | 206 → 3 (98.5% drop; target was 85%). |
| 3 | Each remaining Span::unknown() has rationale | **PASS** | 3/3 leftover sites carry `// arc 138: no span — <reason>` comments naming the SchemeCtx trait constraint. |
| 4 | Workspace tests pass | **PASS** | `cargo test --release --workspace` exit=0 excluding lab. |
| 5 | Canary passes | **PASS** | `check::tests::type_mismatch_message_carries_span` returns `<test>:line:col` prefixed message. |
| 6 | No new variants / Display / diagnostic changes | **PASS** | Sweep is span-field-only; CheckError variants unchanged; Display arms unchanged; diagnostic() arms unchanged. |
| 7 | No commits | **PASS** | Working tree shows uncommitted modification of `src/check.rs` only. |
| 8 | Honest report | **PASS+** | ~600 words; counts before/after; pattern distribution; touched-line range; verification commands; substrate observations (23 helper signatures + 1 grammar_error helper widened); Pattern E rationale; four questions applied. |

**HARD VERDICT: 8 OF 8 PASS.**

## Soft scorecard (4 rows)

| # | Criterion | Result | Notes |
|---|---|---|---|
| 9 | Pattern distribution | PASS | A dominates at ~143 (~69%); B ~57 (~28%); D ~5 (~2%); E 3 (1.5%). Higher A-skew than predicted (~40%); reflects codebase reality — arg-position errors ARE the most common shape. Sonnet's report counts are honest. |
| 10 | Span source quality | PASS | Spot-checked head_span.clone() in `infer_let_star`, `infer_match`, retired-verb poisons; uses are semantically correct (call-form errors point at head; arg errors point at arg). |
| 11 | Workspace runtime | PASS | Test runtime within baseline. |
| 12 | Honest delta on threading | **PASS+** | Sonnet surfaced TWO substrate observations: (a) 23 helper functions gained `head_span: &Span` parameter — uniform shape now `infer_*(args, head_span, env, locals, fresh, subst, errors)` across the helper family. (b) `SchemeCtx` trait in `rust_deps/mod.rs` doesn't carry AST nodes through `push_*` methods; threading spans there requires expanding the trait surface — explicitly named as substrate observation, NOT papered over. Real architectural data. |

**SOFT VERDICT: 4 OF 4 PASS. Clean ship.**

## Substrate observation — `SchemeCtx` trait gap (Pattern E)

Three leftover `Span::unknown()` sites live in the `SchemeCtx` trait's
`push_type_mismatch`, `push_arity_mismatch`, `push_malformed` methods (`src/check.rs` near the bottom of the rust_deps integration). The trait is implemented for shim-side error pushing — Rust callers (e.g., dispatcher implementations in `src/rust_deps/mod.rs`) call these methods without per-arg AST context.

The honest disposition: this is a real trait-design constraint. Threading a span through the trait would expand its surface (every implementor would need to thread a Span through). Out of scope for slice 1 finish; **earned as a follow-up arc** if downstream demand surfaces — note in arc 138 slice 3 (`RuntimeError` spans) that this trait-impl gap exists at the boundary.

The 3 sites still produce useful CheckErrors with callee/param/got/expected info; only the file:line:col coordinate is missing. Acceptable trade-off; documented.

## Helper signature broadening (Pattern A/B threading)

Sonnet added `head_span: &Span` to 23+ helper functions to thread span access uniformly. The substrate now has a stable convention: every `infer_*` helper has access to the call form's head span via the parameter. Future helpers should keep this shape.

Some helpers carry it as `_head_span: &Span` (don't emit errors directly but maintain caller-uniformity). This is correct — the discipline is uniform-signature-over-conditional-threading.

## Independent prediction calibration

Predicted 70% chance of 8/8 hard + 4/4 soft. Actual: **8/8 hard + 4/4 soft**, with the most-likely path. The pattern was well-defined; the worked examples were sufficient teaching; sonnet's recent calibration stayed strong (8/8 on slices 1-3 of arc 135 → 8/8 on arc 138 slice 1 finish).

Sonnet runtime: 30 min vs predicted 30-50. In band.

## Ship decision

**SHIP.** 8/8 hard + 4/4 soft. The substrate observation about `SchemeCtx` becomes input to slice 3 design (RuntimeError spans). Slice 2 (TypeError) can dispatch with the same playbook.

## Next steps

1. Commit slice 1 finish + this SCORE + the BRIEF + EXPECTATIONS.
2. Push.
3. Write slice 2 BRIEF (TypeError variants in `src/types.rs`) — different file, same pattern. Spawn sonnet in background.
4. Score slice 2 → continue through slices 3-6.

## What this slice tells us

- The arc 138 pattern is **reproducible by sonnet** across substrate sweeps. The two-pattern infrastructure (variant + Display + 6 worked sites) is sufficient teaching.
- Future slices (2-6) dispatch with high confidence.
- The `SchemeCtx` trait surface is a real substrate observation — earned during this sweep, queued for later treatment.
- Helper signatures gaining `head_span: &Span` is a shape that should continue across the substrate. Future helpers maintain the convention.

Sonnet's WORK was clean. Sonnet's REPORT was clean (no fabrication; counts match `git diff --stat`; substrate observations honest). Trust-but-verify confirms both.
