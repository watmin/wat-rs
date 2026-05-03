# Arc 144 Slice 1 — SCORE

**Sweep:** sonnet, agent `a4f2b30280cbc69b2`
**Wall clock:** ~8.4 minutes (504s) — well under both the 25-40 min
Mode A band and the 60-min time-box cap.
**Output verified:** orchestrator stash-tested both surprise failures
(`wat_arc143_manipulation` 5/8 + wat-lru `lru_raw_send_no_recv`)
to confirm they are pre-existing, NOT introduced by sonnet's
refactor. Both verified pre-existing.

**Verdict:** **MODE A — clean ship with FAULTLESS DIAGNOSTIC
DISCIPLINE.** 10/10 hard rows pass; 4/4 soft rows pass. Sonnet ran
the substrate-informed refactor mechanically AND correctly diagnosed
two unexpected failures via git-stash round-trips, surfacing them as
honest deltas instead of either ignoring them or falsely owning
them. The discipline cycle from `feedback_compaction_protocols.md`
held end-to-end on the substrate side.

The two surprises were ORCHESTRATOR DISCIPLINE GAPS surfaced by
sonnet, not sonnet errors:
1. `wat_arc143_manipulation` 3/8 failing was pre-existing drift
   from arc 143's slice 5b deliberate fix (HolonAST::symbol return
   for splice positions, breaking test assertions that expected
   `:x` keyword-prefix forms). Slice 5b's SCORE only verified the
   foldl macro test; never re-ran the manipulation suite.
2. wat-lru `lru_raw_send_no_recv` was caused by the in-flight
   arc 130 `CacheService.wat` modification in the working tree —
   not a slice 1 issue.

## Hard scorecard (10/10 PASS)

| # | Criterion | Result |
|---|---|---|
| 1 | File diff scope | ✅ `src/runtime.rs` modified + new `tests/wat_arc144_lookup_form.rs`. NO other Rust file changes. |
| 2 | `Binding` enum | ✅ 5 variants (UserFunction / Macro / Primitive / SpecialForm / Type) at `src/runtime.rs:6267`. Each carries `name: String` + variant-specific data + `doc_string: Option<String>`. `pub` visibility. `'a` lifetime correctly parameterized for borrowed UserFunction/Macro/Type cases. |
| 3 | `lookup_form` function | ✅ `pub fn lookup_form<'a>(name: &str, sym: &'a SymbolTable) -> Option<Binding<'a>>` at `src/runtime.rs:6315`. Walks 4 registries in dispatch precedence: `sym.functions` → UserFunction; `sym.macro_registry` → Macro; `CheckEnv::with_builtins().get(name)` → Primitive; `sym.types` → Type. SpecialForm path is the 5th branch and returns None (slice 2's territory). |
| 4 | `LookupResult` deleted | ✅ `LookupResult` enum + `lookup_callable` no longer in the codebase. Verified via `git diff` — both removed cleanly. |
| 5 | 3 eval_* primitives refactored | ✅ Each of `eval_lookup_define`, `eval_signature_of`, `eval_body_of` now matches on Binding's 5 variants. UserFunction + Primitive arms preserve existing behavior verbatim (existing helpers reused). Macro + Type arms call new helpers. SpecialForm arm emits sentinel / signature directly. body-of returns :None for Primitive + Type + SpecialForm (honest absence). |
| 6 | 4 NEW helpers | ✅ `macrodef_to_signature_ast` (6111), `macrodef_to_define_ast` (6155), `typedef_to_signature_ast` (6174), `typedef_to_define_ast` (6202). Macro signature emits `(name (p :AST<wat::WatAST>) ... [& (rest :AST<Vec<wat::WatAST>>)] -> :AST<wat::WatAST>)` — honest sentinel since per-param `:AST<T>` isn't tracked in MacroDef. Type signature is single-element `(:Name<T>)` head; type define emits `(:wat::core::struct/enum/etc :Name (:wat::core::__internal/type-decl :Name))` — honest minimal head. |
| 7 | New test file | ✅ `tests/wat_arc144_lookup_form.rs` with 9 tests (exceeds 5+ minimum). All 9 PASS. |
| 8 | **Existing arc 143 tests still green** | ✅ AFTER drift fix (paired commit): `wat_arc143_lookup` 11/11; `wat_arc143_manipulation` 8/8; `wat_arc143_define_alias` 2/3 (slice 6 length canary unchanged — slice 3/4 territory). The manipulation 3/8 fail surfaced by sonnet was pre-existing slice 5b drift; orchestrator fixed in paired commit. |
| 9 | `cargo test --release --workspace` | ✅ Same failure profile as PRE-slice-1 (after baseline correction): only the slice 6 length canary + the in-flight CacheService.wat-induced wat-lru fail. ZERO new regressions from sonnet's refactor. |
| 10 | Honest report | ✅ ~600-word report (longer than 250-350 target due to honest-delta narrative; appropriate for the surprise depth). All required sections present. |

## Soft scorecard (4/4 PASS)

| # | Criterion | Result |
|---|---|---|
| 11 | LOC budget (300-500) | ✅ ~450 LOC (runtime.rs additions + new test file). At top of band; honest scope match. |
| 12 | Style consistency | ✅ Helpers placed adjacent to existing arc 143 slice 1 helpers; arg-validation pattern in eval_*; Span::unknown() in synthesized ASTs (matches arc 143 slice 1's discipline; arc 138's spans don't apply to runtime-synthesized ASTs). |
| 13 | clippy clean | ✅ `cargo clippy --release --all-targets` — no new warnings from sonnet's code. Pre-existing workspace warnings (deref/ref at 6685/6772 — arc 143 slice 3 territory) unchanged. |
| 14 | Sentinel honesty | ✅ Macro signature uses `:AST<wat::WatAST>` sentinel; type define emits `(:wat::core::__internal/type-decl :Name)` body sentinel. Both match the brief's "honest sentinel beats half-rendered." |

## The two honest deltas (sonnet's diagnostic discipline)

### Delta 1 — `wat_arc143_manipulation` 5/8 was PRE-EXISTING (slice 5b drift)

Sonnet's report flagged:
> arc 143 manipulation failures (3) are pre-existing, not introduced
> by this slice. Verified by stashing my refactor and observing
> identical failure profile.

**Orchestrator verified independently:**
```bash
git stash push -- src/runtime.rs
cargo test --release --test wat_arc143_manipulation
# → 5 passed; 3 failed (extract_arg_names_foldl, _stops_before_return_type, rename_then_extract)
git stash pop
```

**Root cause:** Arc 143 slice 5b's SCORE noted that `extract-arg-names`
was changed to return `Value::holon__HolonAST(HolonAST::symbol(name))`
instead of `Value::wat__core__keyword(name)` — bare HolonAST::Symbol
items needed for splice positions in the macro template (variable
references, not literals). The runtime fix was deliberate. The 3
failing test assertions checked for `:x`/`:y`/`:_a*` keyword-prefix
forms in the rendered output (`edn::write` output). After slice 5b,
edn::write renders these as `#wat-edn.holon/Symbol "x"` etc. — no
colon prefix.

**Fix:** Updated 3 test assertions in `tests/wat_arc143_manipulation.rs`
to check for `Symbol "x"` / `Symbol "y"` / `Symbol "_a*` substrings,
plus updated comments to document slice 5b's deliberate change.
Tiny test-only edit; runtime behavior unchanged.

**Discipline gap (orchestrator-side):** Arc 143 slice 5b only
verified the foldl macro test transition (its load-bearing row).
It never re-ran `wat_arc143_manipulation` to catch this drift. The
arc 143 INSCRIPTION shipped a few hours ago claims the workspace
was clean except for the length canary — that claim was incorrect.
Adding a checklist item to `COMPACTION-AMNESIA-RECOVERY.md` to
catch this class of slip in future arcs.

### Delta 2 — wat-lru fail caused by in-flight CacheService.wat

Sonnet's report:
> Caused by the unrelated pre-existing CacheService.wat modification
> already in the working tree. Verified by stashing only that file
> and re-running wat-lru tests (passes).

**Orchestrator verified independently:**
```bash
git stash push -- crates/wat-lru/wat-tests/lru/CacheService.wat
cargo test --release -p wat-lru --no-fail-fast
# → 12 passed; 0 failed
git stash pop
```

CacheService.wat is in-flight arc 130 stepping-stone work in the
working tree from prior session (per the arc 143 closure session's
discovery). Not slice 1's territory; left alone.

## Calibration record

- **Predicted Mode A (~50%) / Mode B-helper (~25%) / Mode B-test
  (~15%) / Mode C (~10%)**: ACTUAL Mode A. Calibration matched
  exactly (the most likely outcome).
- **Predicted runtime (25-40 min)**: ACTUAL ~8.4 min. **MUCH
  faster** than predicted — the substrate-informed brief was tight
  + sonnet executed the helpers + tests fluently. **Calibration
  tightening:** future similar refactor slices should predict
  10-20 min, not 25-40.
- **Time-box (60 min)**: NOT triggered.
- **Predicted LOC (300-500)**: ACTUAL ~450. At top of band; on
  budget.
- **Predicted clippy clean**: HIT.
- **Predicted no-substrate-edits-beyond-runtime.rs**: HIT.

## Discipline lessons forged

1. **Sonnet's git-stash diagnostic discipline.** Sonnet's response
   to "tests failing that the brief said should be green" was NOT
   to ignore + ship, NOT to falsely claim ownership, NOT to ask
   the orchestrator — it was to STASH the refactor and verify the
   failures were pre-existing. THAT is the calibrated diagnostic
   discipline the orchestrator wants from sonnet. Saving as a
   memory hint.
2. **Pre-flight baseline-test discipline (orchestrator).** The
   arc 143 manipulation drift slipped through because slice 5b
   only verified its load-bearing test, not the full suite of
   tests touching the area it changed. Adding a checklist item to
   the recovery doc.

## What this slice unblocks

- **Slice 2** — special-form registry can populate the
  SpecialForm Binding variant; the 5-variant dispatch is in place.
- **Slice 3** — TypeScheme registrations for hardcoded primitives
  (length, get, conj, etc.) become visible via `lookup_form`'s
  CheckEnv walk immediately upon registration.
- **Slice 4** — once slice 3 ships, the slice 6 length canary
  turns green via the SAME dispatch already shipped.
- **Future arc 141 (docstrings)** — populates `doc_string` on
  each Binding variant; no enum refactor needed.

## What this slice surfaces (for arc 144's record)

- The macro signature sentinel (`:AST<wat::WatAST>` for params) is
  the most honest shape today. If MacroDef ever tracks per-param
  `:AST<T>` types separately (slice 5 of arc 144 may add this if
  needed for `(help :wat::test::deftest)` to render properly),
  the helper updates without touching the dispatch.
- The type define sentinel (`(:wat::core::__internal/type-decl :Name)`)
  flags that types' field details are NOT yet exposed via
  `lookup-define`. Future arc may extend `typedef_to_define_ast`
  to render the actual fields/variants/target — but per the four
  questions, the minimal honest sentinel is right for slice 1.
- `extract-arg-names` returns `Vec<HolonAST>` at runtime but the
  type-checker special-case at `check.rs:3201` says
  `Vec<:wat::core::keyword>`. This type-level lie predates slice
  1; flagged here for future cleanup (probably arc 141 + arc 144's
  closure pass since both touch the reflection layer).
