# Arc 144 Slice 2 — Pre-handoff expectations

**Drafted 2026-05-03.** Special-form registry + ~25-30 form
registrations + lookup_form 5th-branch wire-up + 8+ tests.
Predicted MEDIUM slice (Mode A ~55%; Mode B-form-count ~20%; Mode
B-sketch-format ~15%; Mode C ~10%).

**Brief:** `BRIEF-SLICE-2.md`
**Output:** 2 Rust files modified (`src/runtime.rs` 6-line addition,
`src/lib.rs` 1-line addition) + 2 NEW Rust files
(`src/special_forms.rs`, `tests/wat_arc144_special_forms.rs`).
~250-400 LOC + ~250-word report.

## Setup — workspace state pre-spawn

- Slice 1 closed (commit 42319ef + drift fix 810129f). Binding enum
  + lookup_form + 3 reflection primitives' SpecialForm dispatch
  arms in place. Slice 1's tests (`wat_arc144_lookup_form` 9/9)
  + arc 143 baseline (`wat_arc143_lookup` 11/11,
  `wat_arc143_manipulation` 8/8, `wat_arc143_define_alias` 2/3)
  green per pre-flight verification.
- 1 in-flight uncommitted file (CacheService.wat — arc 130's
  territory, leave alone).
- Workspace baseline failure: only the slice 6 length canary
  (slice 3/4 territory).

## Hard scorecard (10 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | File diff scope | NEW `src/special_forms.rs`, NEW `tests/wat_arc144_special_forms.rs`; MODIFIED `src/lib.rs` (1-line `pub mod`) + `src/runtime.rs` (~6-line registry-consult branch in `lookup_form`). NO other Rust file changes. NO wat files. |
| 2 | `SpecialFormDef` struct | `pub struct SpecialFormDef { pub name: String, pub signature: HolonAST, pub doc_string: Option<String> }`. |
| 3 | `lookup_special_form` API | `pub fn lookup_special_form(name: &str) -> Option<&'static SpecialFormDef>`. Backed by `static REGISTRY: OnceLock<HashMap<String, SpecialFormDef>>` initialized via `build_registry`. |
| 4 | `build_registry` populates ~25-30 forms | Sonnet's audit names the actual count. The 6 categories must all be represented (control / lambdas / type defs / error / quote / spawn / channel). Each registration has a head keyword + a placeholder-style HolonAST sketch. |
| 5 | `lookup_form` 5th branch | The new branch sits between Type and the trailing `None`; consults `lookup_special_form`; emits `Binding::SpecialForm { name, signature: sig.clone(), doc_string: ds.clone() }`. |
| 6 | Sketch format consistent | Each sketch is a `HolonAST::Bundle` with head Keyword + bare-symbol placeholders for slots; consistent format across registrations. |
| 7 | New test file | `tests/wat_arc144_special_forms.rs` with 8+ tests. ALL pass. |
| 8 | **Slice 1 + arc 143 baseline tests still green** | `cargo test --release --test wat_arc144_lookup_form` 9/9; `cargo test --release --test wat_arc143_lookup` 11/11; `cargo test --release --test wat_arc143_manipulation` 8/8; `cargo test --release --test wat_arc143_define_alias` 2/3 (length canary unchanged — slice 3 territory). |
| 9 | **`cargo test --release --workspace`** | Same baseline failure profile: only the slice 6 length canary fails. ZERO new regressions. |
| 10 | Honest report | ~250-word report covers: SpecialFormDef + registry shape, form enumeration with category counts + any deltas from the brief, ONE representative sketch verbatim, lookup_form integration verbatim, test totals, clippy results, honest deltas. |

**Hard verdict:** all 10 must pass. Rows 4 + 6 + 8 are the load-
bearing rows (the audit completeness + sketch consistency + no-
regression).

## Soft scorecard (4 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 11 | LOC budget | Total slice diff: 250-450 LOC. >550 LOC = re-evaluate. |
| 12 | Style consistency | Static OnceLock pattern (per ZERO-MUTEX doctrine); HolonAST construction follows existing arc 143 slice 5b helper patterns; module header docstring describes purpose + arc reference. |
| 13 | clippy clean | `cargo clippy --release --all-targets` no new warnings. |
| 14 | Audit completeness | Sonnet's report names dispatch sites (`check.rs:NNNN` etc.) for each registration deviating from the brief's enumeration. Honest about what was added/removed and WHY. |

## Independent prediction

- **Most likely (~55%) — Mode A clean ship.** Brief is detailed +
  pre-flighted; sonnet executes mechanically. ~15-25 min wall-clock
  (smaller than slice 1 — pure additive; no refactor). Calibrating
  vs slice 1's 8.4 min: this slice has ~25-30 registrations to
  hand-write so it's longer per-LOC than slice 1.
- **Surprise on form count (~20%) — Mode B-count.** Sonnet's audit
  finds either (a) more forms than the brief enumerates (good —
  better coverage; surface them as deltas), OR (b) some forms in
  the brief are NOT actually special forms per the dispatch (e.g.,
  channel ops may be primitives with TypeScheme rather than special
  forms — sonnet should remove them). Either delta surfaces honestly.
- **Surprise on sketch format (~15%) — Mode B-sketch.** Sonnet
  finds the bare-symbol placeholder format is awkward for some
  forms (e.g., variadic vs fixed). Adapts within the spirit of the
  brief; surfaces the deltas.
- **Sweep gets stuck on HolonAST construction API (~10%) — Mode C.**
  Some HolonAST constructors don't exist as the brief assumes (e.g.,
  `HolonAST::keyword(name)` may need `HolonAST::Atom(...)` or
  similar). Sonnet adapts via the available constructors; report
  surfaces the actual shape used.

**Time-box: 50 min cap (2× upper-bound 25 min).**

## Methodology

After sonnet returns:
1. Read this file FIRST.
2. Score each row.
3. Diff via `git diff --stat` — 4 file changes expected (2 new, 2
   modified).
4. Read `src/special_forms.rs` end-to-end — verify the
   `SpecialFormDef` struct + the registry construction + the
   `lookup_special_form` API + the form count.
5. Read `lookup_form` in `src/runtime.rs` — verify the 5th branch
   added correctly.
6. Read 2-3 of the new tests — verify the assertion shape.
7. Run `cargo test --release --test wat_arc144_special_forms` —
   confirm new tests pass.
8. Run slice 1 + arc 143 baseline tests — confirm zero regression.
9. Run `cargo test --release --workspace` — confirm same baseline
   failure profile.
10. Run `cargo clippy --release --all-targets` — confirm no new
    warnings.
11. Score; commit `SCORE-SLICE-2.md`.

## What this slice unblocks

- **Slice 3** — TypeScheme registrations for the 15 hardcoded
  callable primitives (length, get, conj, contains?, the container
  constructors, etc.). Independent of slice 2; can run in parallel.
- **Slice 4** — verification including the slice 6 length canary
  turning green. Blocks on slice 3.
- **Future REPL `(help X)` form** — composes lookup_form +
  signature-of + body-of + doc-string-of. After slice 2, every
  known special form has a queryable signature sketch. After arc
  141, every special form's docstring is also accessible.

The "nothing is special" principle now holds for special forms in
addition to user defines + macros + types + (most) substrate
primitives. Slice 3 closes the hardcoded-primitive gap; slice 4
verifies the full surface.
