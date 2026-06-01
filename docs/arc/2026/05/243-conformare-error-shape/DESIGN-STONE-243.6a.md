# DESIGN — Stone 243.6a — `CheckError` → Pattern A (carve to `src/check/error.rs`)

**Status:** STRIKE-READY. Child of arc 243 (conformare). **Split from 243.6 by proactive slicing (2026-06-01):**
- **243.6a (this stone)** — CheckError Pattern A class-elimination + warded carve.
- **243.6b** — `check_program` walker fusion (10×→1×) + `collect_hints` double-compute fold (flat-file structural cleanup on the settled Pattern-A foundation).

The split is the stepping-stone test answering YES: `collect_hints` folds *into* the outer struct (Pattern A is its prerequisite), and the walker fusion touches the same `check_program` the cascade churns (cleaner on a settled shape). Each piece gets a clean "did it work."

## Why this stone

`CheckError` (`src/check.rs:97`) is the last large flat error enum — 33 variants, **every one carrying its own `span` field**. The span-discipline is hand-written convention, not structure; `diagnostic()` (`check.rs:1361`) does N-arm span extraction; no consumer can read span uniformly. Peer retrofit to `TypeError` (243.3, shipped) — the same Pattern A that made the spanless shape **structurally unrepresentable**. The shape `RuntimeError` (243.7a) follows.

Closes the `rune:conformare(deferred-stone-243.6)` at `check.rs:90`.

## What it delivers

- `src/check/error.rs` — the home's **second warded neighbor** (after `env.rs`): `pub struct CheckError { pub span: Span, pub kind: CheckErrorKind }` + `pub enum CheckErrorKind`.
- Every consumer reads `err.span` — one path; `diagnostic()` / `Display` collapse their N-arm span routing.
- `vigilatum` stamp on `src/check/error.rs` (vigilia REMARKABLE bar, L1+L2=0).

## The algorithm

1. **Carve.** Create `src/check/error.rs`; move `CheckError`, `CheckErrors`, and their impls there. `src/check.rs` gains `pub mod error;` + `pub use error::{CheckError, CheckErrorKind, CheckErrors};`. The crate-root re-export (`lib.rs:117 pub use check::{… CheckError, CheckErrors …}`) is **unchanged** — `wat::check::CheckError` stays valid; `CheckErrorKind` joins it. Mirror the `env.rs` carve precedent.
2. **Reshape to Pattern A** (mirror `src/types/error.rs`):
   - `pub struct CheckError { pub span: Span, pub kind: CheckErrorKind }`.
   - `pub enum CheckErrorKind { … 33 variants, span field(s) handled per §contract … }`.
   - **Single-span variants (28):** drop the `span` field; it moves to the outer struct.
   - **Multi-span variants (5):** the most-actionable location → outer `span`; secondary span(s) stay as **domain-named kind fields** (CONFORMARE.md § Multi-span).
   - Preserve `remedies: Vec<Remedy>` fields where present (ReturnTypeMismatch, MalformedForm, …) — they live on the kind variant.
3. **Display.** Split `impl fmt::Display for CheckErrorKind` (span-free, per-variant message) + `impl fmt::Display for CheckError` (delegates to `kind`; prefixes the span when known). Mirror `types/error.rs:152`/`:254`.
4. **diagnostic()** (`check.rs:1361`): the N-arm span extraction collapses to `self.span`; per-variant message routing moves to a `CheckErrorKind` helper. `CheckErrors::diagnostics()` (`1875`) inherits the new shape unchanged.
5. **Cascade** (substrate-as-teacher): **459 `CheckError::` sites — 452 in `check.rs`, 7 cross-file.** Construction `CheckError::Variant { …, span }` → `CheckError { span, kind: CheckErrorKind::Variant { … } }`; match sites destructure `.kind`. Reshape the type, let cargo name every site, iterate to green (fail-count is the meter).

## The error contract (the one surface decision, pinned)

For each of the **5 multi-span variants**, outer `span` = the **most-actionable** location (the site the user edits to fix); the secondary span(s) keep domain-descriptive names on the kind variant:

| Variant | spans | outer `span` = most-actionable | secondary → domain-named kind field |
|---|---|---|---|
| `SandboxScopeLeak` | 3 | the leak site | the two scope-boundary spans |
| `ProcessJoinBeforeOutputDrain` | 2 | the `join-result` call site | the output-channel-accessor span |
| `ProcessJoinHoldsStdinSender` | 2 | the `join-result` call site | the stdin-sender binding span |
| `DefRedefForbidden` | 2 | the redefining `def` site | the original-def span (`original_def_span`) |
| `DefRedefTypeChange` | 2 | the redefining `def` site | the original-def span (`original_def_span`) |

The BRIEF pins this RULE; sonnet reads each variant's existing `Display` message to choose which span is most-actionable and names the secondaries.

## Files touched

- `src/check/error.rs` — NEW (carved).
- `src/check.rs` — CheckError/CheckErrors removed → `pub mod error;` + re-export; 452 in-file construction/match sites reshaped; `diagnostic()`/`Display` collapse.
- `src/runtime.rs` (×2), `src/function/parse.rs` (×2), `src/function/infer.rs` (×2), `src/argspec/mod.rs` (×1) — the 7 cross-file cascade sites.
- `tests/probe_arc243_stone6_checkerror_pattern_a.rs` — the FM 2-bis probe (flips fail-compile → pass).

## Out of scope (REJECTED, not deferred)

- `check_program` walker fusion (10×→1×) → **Stone 243.6b** (`rune:temperare(deferred-stone-243.6)` at `check.rs:1954`).
- `collect_hints` double-compute fold → **Stone 243.6b** (`rune:temperare(deferred-stone-243.6)` at `check.rs:1844`).
- `RuntimeError` boxing → **Stone 243.7a**.
- No error-semantics change (no merging, no recovery) — location-discipline + carve only.

## Probe contracts (`tests/probe_arc243_stone6_checkerror_pattern_a.rs` — committed; disconfirms at HEAD)

1. `checkerror_outer_span_field_required` — outer struct `span` field + universal access.
2. `checkerrorkind_variants_have_no_span_field` — kind variants span-free.
3. `checkerror_span_access_is_single_path` — universal `err.span` (no N-arm match).

**Verified disconfirmation at HEAD:** 6 compile errors, exactly the gap — `E0432` (CheckErrorKind unresolved) + `E0574` ×4 (enum, not struct) + `E0609` (no field `span`). Post-stone: 3/0 pass.

## Trap-doors

| # | Risk | Detection | Resolution |
|---|---|---|---|
| **T3** | `From<E> for CheckError` impls drop span | grep | **EMPTY** — no `From<…> for CheckError` exists; one less risk than 243.3 |
| **T5** | Multi-span variants have no canonical primary | per-variant `Display` review | the §contract table pins most-actionable → outer; secondaries domain-named |
| **TA** | 452 in-file sites — voluminous mechanical churn | cargo fail-count | substrate-as-teacher cascade; the meter waterfalls to 0 |
| **TB** | `remedies: Vec<Remedy>` fields lost during reshape | probe + cargo | preserve on the kind variant; cargo names any drop |

## Calibration

Larger than 243.3 (16 variants): 33 variants + **459-site cascade** (452 in-file mechanical) + 5 multi-span judgments + carve + vigilia REMARKABLE.
- **Phase A (reshape + carve + cascade):** 60–120 min Mode A. STOP at 240 min.
- **Phase B (vigilia REMARKABLE):** expect R1 findings on the new warded home; converge over 2–3 rounds (homes-walk pattern).

## Cross-references

- Template: `src/types/error.rs` (shipped TypeError Pattern A) + `SCORE-STONE-243.3.md` + `tests/probe_arc243_stone3_typeerror_pattern_a.rs`.
- `docs/CONFORMARE.md` § Multi-span; `docs/VIGILATUM.md` (the stamp); `feedback_warded_means_annihilated`.
- arc 243 `DESIGN.md` (stone chain — 243.6 row; this split refines it).
