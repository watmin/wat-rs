# 296 — Remediation collapse: `collect_hints` IS the retirement/Remedy mechanism (kill `:hint`)

> **Status: STRIKE-READY (2026-07-01). Ratified Option A (four-questions).** A prerequisite structuring strike before the
> `#[derive(ToEdn)]` sweep touches the check family — the hints case proved the derive isn't enough while the DATA still
> smuggles structure into prose.

## The decomplection (why)
The substrate has **one** remediation concept — `Remedy { form, kind: Typo|Retirement, note }` — fed by a `(retired,
replacement, note)` `RETIREMENT_TABLE` (`src/remedy/retirement.rs`), surfaced as `:remedies`. But `check.rs collect_hints`
is a **second, hand-rolled retirement mechanism**: 9 fns (arc_109 `vec`/`list`/`tuple`/`Some`/`None`/`Ok`/`Err`, arc_170
`process-send`/`process-recv`, arc_114 `spawn`/`join`) each `(retired → replacement + why)`, rendered as **prose `:hint`**
and `.join("\n\n")`'d. The smoking gun: **`ReturnTypeMismatch` carries BOTH `:remedies` AND `:hint`** — two remediation
fields, one concept. `TypeMismatch` carries only the prose `:hint`. This is the same fact-kind (a form retirement) living
in two tables, two shapes, two fields — the exact drift the retirement table exists to prevent. (Fossils from arc 109 /
*kill-std*; the table should have absorbed them long ago.)

## The cure (Option A — ratified)
One concept, one type (`Remedy`), one field (`:remedies`), one table. **`:hint` is annihilated.**
- **Name-based hints → `RETIREMENT_TABLE` rows.** Every `collect_hints` fn whose ONLY trigger is a `callee == "…"` match
  (arc_109 `vec`/`list`/`tuple`/`Some`/`None`/`Ok`/`Err`, arc_170 `process-send`/`process-recv`) becomes a
  `RetirementEntry { retired, replacement, note }`. `retirement_lookup` already turns those into `Remedy`s. The fn dies.
- **Shape-triggered hints → a small producer returning `Vec<Remedy>`.** arc_114 (fires on a `ProgramHandle`↔`Thread` type
  mismatch — a name-keyed table cannot express that) becomes part of `fn shape_remedies(callee, expected, got) ->
  Vec<Remedy>`, each `Remedy { form: canonical, kind: Retirement, note: <the multi-step guide> }` (a longer `note`, exactly
  like the table's `struct-restricted` note). Not prose — a `Remedy`. `collect_hints` (the prose joiner) dies.
- **The check family surfaces remediation at serialize time** (remediation is a pure fn of `{callee, expected, got}`; do
  NOT add a stored `remedies` field to `TypeMismatch` — 118 construction sites). ONE shared
  `fn type_error_remedies(callee, expected, got) -> Vec<Remedy>` = `remedies_for(callee, empty())` (retirement + [no typo
  candidates here]) `++ shape_remedies(callee, expected, got)`, deduped by `form`. Both the serializer AND the Display call
  it — one canonical path (no replicate).
  - `TypeMismatch` serializer: `:remedies (type_error_remedies(callee, expected, got))`; drop `:hint`.
  - `ReturnTypeMismatch` serializer: merge its STORED `remedies` with `type_error_remedies(function, expected, got)`
    (dedup by `form`); drop `:hint`.
- **Display** (`src/check/error.rs`): drop the `collect_hints` hint rendering; render the merged remedies via the existing
  `render_remedies` (which already renders `:remedies`). The human "did you mean / migrate" section is preserved — now
  sourced from the same `type_error_remedies`.

## Out of scope (affirmative cuts)
- **The `#[derive(ToEdn)]` application to CheckError** — that follows, AFTER the data is honest (this strike).
- **Stored-vs-lazy uniformity for `ReturnTypeMismatch.remedies`** — leave its stored field; this strike only merges + kills
  `:hint`. (Removing the stored field is a separate nicety, not required to kill the duplicate.)
- **S1/S2/S6/S7** (the other audit findings) — separate strikes.

## Proof
- `grep -rn "collect_hints\|:hint\b\|\"hint\"" src/` → 0 (the prose mechanism + field are gone; only `:hints`-free).
- A probe: a `TypeMismatch` on a retired form (e.g. `:wat::core::vec`) emits `:remedies [#wat.kernel/Remedy {:form
  ":wat::core::Vector" :kind :retirement …}]` (NOT a `:hint` prose blob); a shape-mismatch (`ProgramHandle`↔`Thread`)
  emits its guide as a `Remedy.note`.
- Round-trip + CLI `--check-output` tests updated (`:hint`→`:remedies`). FULL gate `cargo nextest run --release` = 0 failed.
- Weigh: capture a retired-form type error's wire EDN — one `:remedies` Vector of `#wat.kernel/Remedy`, zero prose `:hint`.

## Blast radius
`src/remedy/retirement.rs` (table rows) · `src/check.rs` (`collect_hints`→`shape_remedies` + `type_error_remedies`; delete
the name-based hint fns) · `src/check/error_edn.rs` (serializer `:hint`→`:remedies`) · `src/check/error.rs` (Display) · the
probe · `crates/wat-cli/tests/wat_cli.rs`. NOTHING else.
