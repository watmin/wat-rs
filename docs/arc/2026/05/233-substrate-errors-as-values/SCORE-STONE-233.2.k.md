# SCORE — Arc 233 Stone 233.2.k — Value::Tracked variant retirement + Environment stores TrackedValue

**Result: 12/12 PASS**

## Scorecard

| # | Row | Actual |
|---|---|---|
| 1 | Compile clean | `Finished \`release\` profile [optimized] target(s) in 0.10s` — 0 errors |
| 2 | **233.2.k probe FLIPS 0/5 → 5/5** | `test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s` |
| 3 | Lib tests baseline | `test result: ok. 827 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.15s` |
| 4 | Stone 233.2.j probe still passes (exemption mechanism removed) | `test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s` |
| 5 | Stone 233.2.i eval signature probe still passes | `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s` |
| 6 | Stone 233.2.h TrackedValue mint probe still passes | `test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` |
| 7 | Stone 233.2.d substrate-symmetry probe still passes | `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s` |
| 8 | **Stone 233.1 ValueSnapshot diagnostic probes (LOAD-BEARING — probes 6/7/8 stay green via Option A)** | `test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s` |
| 9 | Stone 232.0 dynamic-keyword probes still pass | `test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s` |
| 10 | Clippy no new warnings | `54` (at boundary; matches 233.2.j baseline) |
| 11 | holon-rs untouched | empty output |
| 12 | **probe_value_tracked_transparency.rs DELETED** | `1` (grep -c "No such" returns 1) |

## Cascade summary

### Phase 1 — Environment storage type flip

**`src/runtime.rs`**:
- `EnvCell.bindings: HashMap<String, Value>` → `HashMap<String, TrackedValue>` (1 site)
- `EnvBuilder.bindings: HashMap<String, Value>` → `HashMap<String, TrackedValue>` (1 site)

**2 lines changed.**

### Phase 2 — Environment API signature flips + 6 lookup callers

**`src/runtime.rs`**:
- `Environment::lookup` return type: `Option<Value>` → `Option<TrackedValue>` (1 site)
- `EnvironmentBuilder::bind` parameter: `value: Value` → `tv: TrackedValue` (1 site)
- Symbol arm in `eval_inner`: removed `.into_tracked()` (lookup already returns TrackedValue; direct return) (1 site)
- Symbol head in `eval_list` (bare-symbol callee): changed from `.value_owned()` + `apply_value` to `apply_tracked_callee` to preserve provenance (1 site)
- Tail-call check in `eval_tail` (Symbol head): now uses `tv.value()` pattern match (1 site)
- `closure_extract.rs:196`: `closed_env.lookup(&name)` now returns `TrackedValue`; `.value()` passed to `encode_value_to_ast` (1 site)
- Inline `((fn...) args)` head in `eval_list`: changed from `eval_inner(..)?.value_owned()` + `apply_value` to `eval_inner(..)` + `apply_tracked_callee` (1 site)

**~10 sites across 2 files.**

### Phase 3 — bind_let_binding simplification

**`src/runtime.rs`**:
- `LetBinding::Single` arm: removed the Phase 5 re-wrap (`Value::Tracked { inner: Box::new(tv.value_owned()), provenance }` construction) + `#[probe-3-exempt]` marker. Now stores `tv` (TrackedValue) directly via `scope.child().bind(name, tv).build()`. (~8 lines removed, 2 added)
- `LetBinding::Destructure` arm: each element now wrapped with `TrackedValue::from(elem)` instead of bare `elem` (~2 lines changed)

**Additional: eval_let return type flipped to TrackedValue**

`eval_let` changed from `Result<Value>` → `Result<TrackedValue>`:
- Empty body returns `TrackedValue::from(Value::Unit)` instead of `Ok(Value::Unit)`
- Last body expression: `.map(|tv| tv.value_owned())` removed; `eval_inner` result returned directly
- Added `:wat::core::let` to `dispatch_keyword_head` producers list (bypasses `dispatch_keyword_head_value` boundary which strips TrackedValue)
- `dispatch_keyword_head_value` arm for `let` updated to `.map(|tv| tv.value_owned())` (tail-call path still needs Value)

This was an unplanned addition. The DESIGN assumed `eval_let` returning Value was acceptable because `bind_let_binding` would store TrackedValue. But probes 6/7/8 revealed that when the let body IS the symbol reference (e.g., `(let [k producer-call] k)`), the `k` symbol resolves to TrackedValue, but `eval_let` strips it before returning to `dispatch_keyword_head_value`, which then re-wraps with Unknown. Fix: route let through `dispatch_keyword_head` directly.

**~15 lines across eval_let + dispatch_keyword_head.**

### Phase 4 — All bind() caller sites updated

**`src/runtime.rs`**:
- `register_runtime_defs_form` (line ~3009): `value = eval_inner(...)?.value_owned()` → `tv = eval_inner(...)?`, bind `tv` directly
- Struct destructure in `bind_let_binding` (line ~6307): `bind(fname, elem)` → `bind(fname, TrackedValue::from(elem))`
- `try_match_pattern` Symbol arm (line ~13981): `bind(..., value.clone())` → `bind(..., TrackedValue::from(value.clone()))`
- `apply_function` fixed-arity args (line ~19157): `bind(name, value)` → `bind(name, TrackedValue::from(value))`
- `apply_function` rest-param (line ~19165): `bind(rest_name, Value::Vec(...))` → `bind(rest_name, TrackedValue::from(Value::Vec(...)))`
- `matches?` logic-var binding (line ~12684): `bind(var, value)` → `bind(var, TrackedValue::from(value))`
- Test helpers (3 sites): `bind("program", Value::wat__WatAST(...))`, `bind(name, value)`, `bind("rxs", rxs)` all wrapped with `TrackedValue::from`

**~10 sites.**

### Phase 5 — Variant + helper retirement

**`src/runtime.rs`**:
- `Value::Tracked { inner, provenance }` variant: DELETED (~7 lines)
- Comment block on `Arc 233 Stone 233.2.a` transparent provenance wrapper: DELETED (~7 lines)
- `Value::inner()` helper (~7 lines): DELETED
- `Value::provenance()` helper (~7 lines): DELETED
- `Value::into_tracked()` helper (~9 lines): DELETED
- `Value::Tracked { inner, .. } => inner.type_name()` arm in type_name(): DELETED (1 line)
- Closing `}` moved; replacement comment block added

**~40 lines deleted, 8 comment lines added.**

### Phase 6 — Dead match arm cleanup

**`src/runtime.rs`**:
- `Hash impl`: `Value::Tracked { .. } => unreachable!(...)` arm DELETED (3 lines); also removed `let unwrapped = self.inner()` and updated match from `unwrapped` to `self`; removed discriminant-tagging commentary
- `PartialEq impl`: removed `self.inner()` / `other.inner()` calls from match (2 sites → `self`/`other`)
- `render_value()`: removed `v.inner()` call + `Value::Tracked { .. } => unreachable!(...)` arm; match now on `v` directly
- `ValueSnapshot::of()`: `v.inner().type_name()` → `v.type_name()`; `render_value(v.inner(), 0)` → `render_value(v, 0)`; `v.provenance()` → `Provenance::Unknown` (always bare Values now)
- `ValueSnapshot::of_tracked()`: `tv.value().inner().type_name()` → `tv.value().type_name()`; `render_value(tv.value().inner(), 0)` → `render_value(tv.value(), 0)`

**`src/edn_shim.rs`**:
- `value_to_edn_with`: `Value::Tracked { inner, .. } => value_to_edn_with(inner, types)` arm DELETED (2 lines)

**`src/closure_extract.rs`**:
- `encode_value_with_path`: `Value::Tracked { inner, .. } => encode_value_with_path(inner, ...)` arm DELETED (3 lines)

**~20 lines deleted across 3 files.**

### Phase 7 — .into_tracked() sweep (~26 sites → TrackedValue::from)

**`src/runtime.rs`**:
All `.into_tracked()` calls replaced with `TrackedValue::from(...)`:
- 4 literal arms in eval_inner (IntLit, FloatLit, BoolLit, StringLit)
- Vec literal arm
- Unit returns (`:wat::core::nil`, empty list, empty let body)
- `:None` / unit_variants / runtime_def_values / keyword-fn-lift arms in Keyword branch
- Keyword literal leaf arm
- eval_list: 6 constructor/ctors via `.map(|v| TrackedValue::from(v))`
- dispatch_keyword_head: dispatch_registry arm + final dispatch_keyword_head_value wrap
- Test code: no `.into_tracked()` calls survived (eval_expr/run return bare Value)

**~26 sites.**

### Phase 8 — .inner() sweep (~19 sites → stripped)

**`src/runtime.rs`**:
All `.inner()` calls on `Value` stripped (post-retirement, Value is never wrapped):
- `PartialEq::eq` match (2 calls removed; match directly on `self`/`other`)
- `Hash::hash` (1 call removed; `let unwrapped = self.inner()` → match on `self`)
- `render_value` (1 call removed; match on `v` directly)
- `ValueSnapshot::of` (2 calls removed; `v.inner()` → `v`)
- `ValueSnapshot::of_tracked` (2 calls removed; `tv.value().inner()` → `tv.value()`)
- 9 test sites: `result.inner().clone()` → `result` (match on Value directly)

**~19 sites.**

### Phase 9 — probe-3-exempt mechanism removal

**`tests/probe_stone_233_2_j_producer_migration.rs`**:
- Deleted the `if line.contains("#[probe-3-exempt") { continue; }` block (3 lines)
- Updated assertion message to remove exemption mention (~5 chars changed)

**~4 lines deleted.**

### Phase 10 — probe_value_tracked_transparency.rs deletion

**`tests/probe_value_tracked_transparency.rs`**: FILE DELETED (was the 233.2.a probe for retired surface; probes for retired surface are deleted not refactored per HARD CUT discipline).

### Phase 11 — New helper: apply_tracked_callee

**`src/runtime.rs`** (unplanned; required for probes 6/7/8):
- Added `fn apply_tracked_callee(callee_tv: TrackedValue, ...)` — routes NotCallable errors through `ValueSnapshot::of_tracked` so producer provenance survives to error sites.
- Used by `eval_list` Symbol head and List head paths (both were calling `apply_value(&bare_value, ...)` losing provenance).

**~20 lines added.**

## Honest deltas

- **Destructure slot provenance** — each tuple slot in `LetBinding::Destructure` gets `Provenance::Unknown` via `TrackedValue::from(elem)`. Planned per DESIGN. Arc 233.2.e may revisit.
- **Struct destructure slot provenance** — same: `TrackedValue::from(elem)` for each field. Planned.
- **matches? logic-var provenance** — `TrackedValue::from(value)` for each bound struct field in `?var` patterns. Same discipline as destructure.
- **apply_function arg provenance** — function arguments are bound with `TrackedValue::from(value)`. Arguments are not producers; Unknown is correct.
- **eval_let_tail** — still returns `Result<Value>` (tail-call context; not changed; provenance from let-body in tail position remains Unknown for that path; this is pre-existing and not a regression from 233.2.j).
- **recv/try-recv provenance** — still lost as planned per 233.2.j SCORE honest delta. Arc 233.2.e.

## Unplanned additions (delta from BRIEF prediction)

1. **eval_let TrackedValue return** — BRIEF said "6 lookup callers"; the 7th caller was `eval_let` itself, which stripped provenance at the boundary. Fix: moved `eval_let` to `dispatch_keyword_head` producer list + flipped its return type to `TrackedValue`. (~15 lines)
2. **apply_tracked_callee** — BRIEF didn't anticipate that the eval_list Symbol/List callee path would strip provenance before NotCallable errors. Fix: new helper that preserves TrackedValue through the callee path. (~20 lines)

Both additions were discovered during verification (probe 3 failure → eval_let fix; probes 6/7/8 failure → apply_tracked_callee). Neither is scope creep; both are load-bearing for the correctness of Option A's structural mechanism.

## Time breakdown

- Phase 1-3 (Environment storage flip + API + bind_let_binding): ~10 min
- Phase 4-6 (bind callers + variant/helper delete + match arms): ~20 min
- Phase 7-8 (.into_tracked() + .inner() sweep): ~10 min
- Phase 9-10 (probe-3-exempt removal + file deletion): ~3 min
- First compile + closure_extract.rs fix: ~3 min
- probe 3 failure diagnosis + eval_let fix: ~10 min
- probes 6/7/8 failure diagnosis + apply_tracked_callee: ~15 min
- Test i64 deref fix in lib tests: ~2 min
- Final verification battery + SCORE writing: ~15 min

**Actual total: ~88 min**

## Calibration

Predicted 60–120 min Mode A; actual ~88 min. Within the band.

Two unplanned additions added ~25 min beyond the mechanical sweep. Both were probe-discovered gaps in the DESIGN's scope analysis:
1. `eval_let` was not enumerated as a boundary that strips TrackedValue (DESIGN listed 6 lookup callers but eval_let strips at a different layer).
2. `apply_value` / Symbol callee path was not enumerated as a site that loses provenance before NotCallable construction.

The substrate-as-teacher (FM 15) pattern applied: cargo enumerated errors; probes enumerated behavioral gaps; iterate.

## What this unblocks

- **Stone 233.2.l** — `#[wat_value]` proc-macro structural seal. Value::Tracked is gone; the proc-macro can now apply to Value without encountering a wrapping variant.
- **arc216 stone1 7 probes** (task #496) — auto-resolve. Value::Tracked is structurally absent.
- **Stone 233.2.e** — AST-derived provenance on the fully-sealed substrate (restores recv/try-recv provenance + destructure slot provenance via AST-side mechanism).

## Files modified

- `src/runtime.rs` — Environment storage flip; lookup + bind API flip; all bind callers; bind_let_binding simplification; eval_let return type flip; dispatch_keyword_head let routing; Value::Tracked variant DELETE; Value::inner/provenance/into_tracked DELETE; PartialEq/Hash/render_value/ValueSnapshot dead-arm cleanup; .inner()/.into_tracked() sweep; apply_tracked_callee new helper; test .inner() sites stripped
- `src/edn_shim.rs` — Value::Tracked match arm deleted
- `src/closure_extract.rs` — Value::Tracked match arm deleted; lookup caller updated
- `tests/probe_stone_233_2_j_producer_migration.rs` — probe-3-exempt mechanism removed; assertion message updated
- `tests/probe_value_tracked_transparency.rs` — DELETED

## Cross-references

- `docs/arc/2026/05/233-substrate-errors-as-values/BRIEF-STONE-233.2.k.md` — paired BRIEF
- `docs/arc/2026/05/233-substrate-errors-as-values/EXPECTATIONS-STONE-233.2.k.md` — 12-row scorecard
- `docs/arc/2026/05/233-substrate-errors-as-values/DESIGN-STONE-233.2.k.md` — sub-DESIGN (Option A verdict)
- `docs/arc/2026/05/233-substrate-errors-as-values/SCORE-STONE-233.2.j.md` — establishes the Phase 5 exemption this stone dissolved
- `tests/probe_stone_233_2_k_variant_retired.rs` — FM 2-bis probe (5/5 PASS post-stone)
