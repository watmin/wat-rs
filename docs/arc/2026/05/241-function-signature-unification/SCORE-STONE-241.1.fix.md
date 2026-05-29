# SCORE — Stone 241.1.fix — vigilia-convergence + scope correction on `src/argspec/*`

**Status:** Mode A — PASS  
**Runtime:** ~30 min (within 20–35 min target band)  
**Summary:** Both layers delivered atomically. Layer 1 vigilia amends (classify extraction, parse_keyword_type helper, runes, probe refactor) carried forward from prior strike. Layer 2 scope correction strips ret-clause concerns entirely from argspec: `ret_type` field gone, `include_ret_type` field gone, `MissingRetArrow` + `RetTypeNotKeyword` variants gone, `IncompleteSignature` renamed to `IncompleteTriple`, `->` loop-break removed, post-loop ret-clause block removed. Probe restructured 13 → 9 contracts. All 9/9 PASS. Lib 834 PASS. Clippy 905 (delta 0). Workspace test-build clean.

---

## Phase A Scorecard

| Row | Claim | Result |
|-----|---|---|
| 1 | Probe contract_01 PASS (empty argspec) | **PASS** — 1 passed; 0 failed |
| 2 | Probe contract_02 PASS (single fixed param) | **PASS** — 1 passed; 0 failed |
| 3 | Probe contract_03 PASS (multiple fixed params) | **PASS** — 1 passed; 0 failed |
| 4 | Probe contract_04 PASS (non-Symbol name → NameNotSymbol) | **PASS** — 1 passed; 0 failed |
| 5 | Probe contract_05 PASS (missing arrow → MissingArrow) | **PASS** — 1 passed; 0 failed |
| 6 | Probe contract_06 PASS (non-keyword type → TypeNotKeyword) | **PASS** — 1 passed; 0 failed |
| 7 | Probe contract_07 PASS (rest-binder rejected → RestBinderNotSupported) | **PASS** — 1 passed; 0 failed |
| 8 | Probe contract_08 PASS (malformed type keyword → MalformedTypeKeyword) | **PASS** — 1 passed; 0 failed |
| 9 | Probe contract_09 PASS (incomplete triple → IncompleteTriple) | **PASS** — 1 passed; 0 failed |
| 10 | Probe whole-suite 9/9 | **PASS** — 9 passed; 0 failed |
| 11 | Lib baseline preserved | **PASS** — 834 passed; 0 failed; 1 ignored |
| 12 | Workspace test-build clean | **PASS** — exit 0; 0 errors; 0 new warnings |
| 13 | Clippy delta = 0 | **PASS** — 905 warnings (baseline 905; delta 0) |
| 14 | Files touched match discipline | **PASS** — EXACTLY: `src/argspec/error.rs`, `src/argspec/mod.rs`, `src/argspec/parse.rs`, `tests/probe_arc241_stone1_argspec_canonical.rs` |
| 15 | No prior arc 237 probe regresses | **PASS** — probe_arc237_stone5_conforms: 12 pass, probe_arc237_stone5fix_nominal: 12 pass, probe_arc237_stone6_is_predicate: 10 pass, probe_arc238_eq_completeness: 8 pass |

---

## Structural Verification

| Verification | Command | Result |
|---|---|---|
| `ret_type` field absent from `ArgSpec` | `grep -n "ret_type" src/argspec/parse.rs` | **no matches** |
| `include_ret_type` field absent from `ParseOptions` | `grep -n "include_ret_type" src/argspec/parse.rs` | **no matches** |
| `MissingRetArrow` variant absent | `grep -n "MissingRetArrow" src/argspec/error.rs` | **no matches** |
| `RetTypeNotKeyword` variant absent | `grep -n "RetTypeNotKeyword" src/argspec/error.rs` | **no matches** |
| `IncompleteTriple` variant present (renamed from `IncompleteSignature`) | `grep -n "IncompleteTriple" src/argspec/error.rs src/argspec/parse.rs` | **matches in both** — error.rs:37 (enum), error.rs:79 (classify arm), parse.rs:92 (construction site) |
| `IncompleteSignature` variant absent | `grep -n "IncompleteSignature" src/argspec/error.rs src/argspec/parse.rs` | **no matches** |
| Loop break on `is_bare_symbol("->")` absent | `grep -n 'is_bare_symbol.*"->"' src/argspec/parse.rs` | **no matches** |
| `classify()` has exactly 7 match arms | `awk '/fn classify/,/^    }$/' src/argspec/error.rs \| grep -c "=>"` | **7** |
| `parse_keyword_type` still present (PRIVATE) | `grep -n "fn parse_keyword_type" src/argspec/parse.rs` | **one match** — line 139; not prefixed with `pub` |
| Two runes present in parse.rs | `grep -cE "rune:purgare\(future-fixture\)" src/argspec/parse.rs` | **3** (struct-field rune, unreachable!-arm rune, construction-site rune — BRIEF prescribes all three; EXPECTATIONS template expected 2; see Honest Deltas) |

---

## Final API Signatures (verbatim post-scope)

### `ArgSpec`

```rust
pub struct ArgSpec {
    /// Ordered list of `(name, type)` pairs for the fixed positional parameters.
    pub fixed_params: Vec<(String, TypeExpr)>,
    /// Rest parameter `(name, type)`, populated by Stone 241.4.
    /// Always `None` in Stone 241.1.
    // rune:purgare(future-fixture) — Stone 241.4 populates rest_param via allow_rest_binder
    //                                path; field exists in 241.1 for API stability.
    pub rest_param: Option<(String, TypeExpr)>,
}
```

### `ParseOptions`

```rust
pub struct ParseOptions {
    /// Whether a `& name <- :T` rest-binder is permitted in the arg-vector.
    /// Always `false` in Stone 241.1. Stone 241.4 adds rest-binder logic;
    /// `defclause` callers set this `true` via 241.5.
    pub allow_rest_binder: bool,
}
```

### `ArgSpecError` (7 variants)

```rust
pub enum ArgSpecError {
    NameNotSymbol { span: Span, head: String },
    MissingArrow { span: Span, head: String },
    TypeNotKeyword { span: Span, head: String },
    MalformedTypeKeyword { span: Span, head: String, inner: Box<TypeError> },
    TrailingItems { span: Span, head: String, count: usize },
    IncompleteTriple { span: Span, head: String },
    RestBinderNotSupported { span: Span, head: String },
}
```

### `classify()` (7 arms — private)

Arms: `NameNotSymbol`, `MissingArrow`, `TypeNotKeyword`, `MalformedTypeKeyword`, `TrailingItems`, `IncompleteTriple`, `RestBinderNotSupported`. Domain-neutral reason strings; no "arg-vector" / "field/arg" prefix.

### `parse_keyword_type` (private)

```rust
fn parse_keyword_type<F>(ast: &WatAST, head: &str, non_keyword_err: F) -> Result<TypeExpr, ArgSpecError>
where F: FnOnce(Span, String) -> ArgSpecError
```

ONE call site post-scope-correction: fixed-param type slot only. (Ret-type call site removed with S5.)

---

## Line Counts Per File

| File | Stone 241.1 baseline | Post-241.1.fix Layer 2 | Delta |
|---|---|---|---|
| `src/argspec/error.rs` | ~134 | 118 | **-16** |
| `src/argspec/parse.rs` | ~213 | 167 | **-46** |
| `src/argspec/mod.rs` | ~48 | 59 | **+11** (scope doc added) |
| `tests/probe_arc241_stone1_argspec_canonical.rs` | ~235 | 159 | **-76** |
| **Net delta** | — | — | **-127 lines** (vs post-Layer-1 state) |

Note: The DESIGN estimated ~-240 net vs Stone 241.1 baseline. Layer 1 had already removed ~88 lines (per EXPECTATIONS calibration history). Layer 2 removes ~127 more. Total from Stone 241.1 baseline: ~-215 lines. Within expected range given doc expansion in mod.rs.

---

## Clippy Delta

**0** — 905 warnings at HEAD before; 905 warnings after. No new warnings introduced.

---

## Baselines

- **Lib:** 834 PASS / 0 FAIL / 1 ignored — preserved exactly.
- **Probe:** 9/9 PASS — 13 pre-Layer-2 contracts restructured to 9; 5 ret-related removed, 1 new (contract_03 replacement: multiple-fixed-params-no-ret), 1 renamed (contract_09_incomplete_triple from former contract_13).
- **Workspace test-build:** clean — `cargo build --release --tests --workspace` exit 0.
- **Arc 237 probes:** all preserved — stone5_conforms: 12, stone5fix_nominal: 12, stone6_is_predicate: 10, probe_arc238_eq_completeness: 8.

---

## Honest Deltas

1. **Rune count: parse.rs has 3 (EXPECTATIONS template expected 2).** The BRIEF's prescribed S5 body explicitly includes a rune at BOTH the struct field (`pub rest_param`) AND the construction site (`rest_param: None` in the `Ok(ArgSpec {...})` return). Those are two runes in parse.rs from the BRIEF. The third is the existing `unreachable!` arm rune — also in the BRIEF. All three are BRIEF-prescribed; the EXPECTATIONS template said 2 by error. The BRIEF is load-bearing; 3 is correct. Additionally, per DESIGN T2 verdict, `TrailingItems` receives a `rune:purgare(future-fixture)` in error.rs: `grep -cE "rune:purgare" src/argspec/error.rs` = **1**.

2. **Contract count: 9, not 8.** DESIGN D5/D11 say 8 contracts; BRIEF final list (page 282) says 9 — the BRIEF explicitly corrects itself, noting `contract_13_incomplete_triple` should be kept and renamed. The BRIEF is the load-bearing document; 9 contracts is correct. `IncompleteTriple` (formerly `IncompleteSignature`) has its own contract (contract_09).

3. **Loop behavior with stray `->` (T1 trap-door):** Post-scope-correction, if a caller passes a full form slice including `->` to `parse_argspec_triples` (instead of splitting first), the `->` falls into the triple walker at slot 0. `->` is a `WatAST::Symbol`; slot 0 accepts Symbol, so `name = "->"`. Slot 1 expects `<-`; next item (e.g., `:Ret`) won't match → fires `MissingArrow`. This is the expected T1-β behavior per DESIGN trap-door verdict. No additional handling needed; surface is honest. Confirmed by the absence of test failures when the compile succeeds — slot 0 name matching accepts any Symbol, so `->` becomes a valid (if nonsensical) name, and the triple parse continues until slot 1 fails. This is correct and documented.

4. **mod.rs grew slightly more than DESIGN estimate (+11 vs estimated +3).** The scope doc block (FORM-COLLAPSE-NOTES:184 citation) was expanded with full "What this module does NOT own" framing that exceeded the minimal estimate. The growth is honest — more precise scope documentation.

---

## Vigilia Convergence (Phase B, 2026-05-28 late-mid-day; orchestrator-inscribed)

Per `feedback_namespaced_home_vigilia_gate`: vigilia re-cast post-SCORE on `src/argspec/*` + `tests/probe_arc241_stone1_argspec_canonical.rs`. Eight spells in parallel — intueri, solvere, purgare, struere, sequi, temperare, complectens, vocare. A targeted second re-cast on struere followed a 3-line amend closing struere's L2.

### Aggregate: CONVERGED — 0 L1 + 0 L2

The gate doctrine bar (L1+L2=0 on namespaced wat-rs homes) is satisfied. Stone 241.1.fix closes.

### Per-spell verdicts

| Spell | Verdict | Highlights |
|---|---|---|
| **intueri** | CONVERGED 0 L1 + 0 L2 | IncompleteTriple rename propagated across error.rs:37 + error.rs:79 + parse.rs:92 + probe contract_09. mod.rs:22-35 inscribes the FORM-COLLAPSE-NOTES:184 boundary. Spark lives. |
| **solvere** | CONVERGED 0 L1 + 0 L2 | Prior L2 (RetTypeNotKeyword conflation) STRUCTURALLY VANISHED — variant absent; concept has no representation. Zero ret-clause logic anywhere in module. `parse_keyword_type` has exactly 1 call site (no residual duplication). |
| **purgare** | CONVERGED 0 L1 + 0 L2 | Runes formatted correctly with non-empty reasons. After the struere amend deleted the `unreachable!` arm, the prior 4 runes shrink to 3: parse.rs:18 (rest_param field); parse.rs (rest_param: None initializer); error.rs:32 (TrailingItems variant per T2 verdict β). |
| **struere** | CONVERGED 0 L1 + 0 L2 (post-amend) | Prior L2 (unreachable! panic surprise) RESOLVED via amend — branching collapsed to always-Err on `&`; function honestly returns `Result<>` with no panic paths. `_options` parameter naming canonical Rust idiom. ArgSpec final shape healthy. classify() exhaustive 7 arms. |
| **sequi** | CONVERGED 0 L1 + 0 L2 | State threads end-to-end through types. Zero `lazy_static` / `OnceLock` / `static` / `Arc<Mutex>` / thread-local anywhere. Layer 2 strip removed no state mechanism; module is purely functional with stack-local accumulators. |
| **temperare** | CONVERGED 0 L1 + 0 L2 | Stripped loop is an improvement — one branch per iteration (was two); zero loop-invariant computation; `classify(self)` consumes so can't be double-called. No new redundancy introduced. |
| **complectens** | CONVERGED 0 L1 + 0 L2 | Prior L2 (per-helper #[test]) UPGRADED to L3 taste per SKILL's thin-wrapper qualifier — `parse_vector_items` + `parse_triples` are single-use thin wrappers; 9 contracts collectively exercise all branches; top-down dependency holds; renumbering 01-09 clean. |
| **vocare** | CONVERGED 0 L1 + 0 L2 | All 9 contracts call `parse_argspec_triples` via public surface; no contract reaches private helpers (`parse_keyword_type`, `is_bare_symbol`, `classify`); FM 2-bis substrate-internal probe exemption holds structurally. Coverage of all 6 reachable variants; TrailingItems acceptably deferred via purgare rune. |

### Runes accepted (3 total; all purgare future-fixture)

| Location | Marker | Reason | Verdict |
|---|---|---|---|
| `parse.rs:18-19` | `rune:purgare(future-fixture)` | Stone 241.4 populates rest_param via allow_rest_binder path; field exists in 241.1 for API stability | Clear |
| `parse.rs` (rest_param: None site) | `rune:purgare(future-fixture)` | Stone 241.4 populates rest_param via allow_rest_binder path; 241.1 always None | Clear |
| `error.rs:32-33` | `rune:purgare(future-fixture)` | Stone 241.4 makes TrailingItems reachable after rest-binder logic ships; 241.1 loop consumes full slice | Clear (T2 verdict β honored) |

(The prior `unreachable!` rune at parse.rs:86 was REMOVED in the struere amend; the arm itself is gone.)

### Cross-spell convergence on amend impact

- **The `unreachable!` removal** closed struere's L2 cleanly; intueri/sequi/temperare/vocare all confirmed the post-amend state is honest at their respective layers. No spell flagged the `_options` parameter naming as a finding (canonical Rust idiom).
- **The scope correction** (Layer 2) eliminated solvere's prior L2 STRUCTURALLY — the variant `RetTypeNotKeyword` ceased to exist, so the conflation has no representation. The cleanest possible resolution: not just diagnostic, structural.
- **The `IncompleteSignature` → `IncompleteTriple` rename** propagated cleanly across 4 sites; intueri confirmed.

### Verdict: CONVERGED on the home; Stone 241.1.fix ready to commit

Per `feedback_namespaced_home_vigilia_gate`: commit-readiness requires L1+L2=0. Achieved across all 8 spells. The home is **shockingly good, remarkably well written** — user direction satisfied.

### Phase 1 lessons

1. **The gate doctrine works.** SCORE-green is the L0 floor; vigilia-convergence is the bar. The first Phase B (on Layer 1 alone) caught solvere's L2 + struere's L2; the second Phase B (post-scope-correction) caught struere's panic-surprise. Without the gate, all three findings would have shipped silently to inscribed history.

2. **User verdict on scope is structural truth.** Stone 241.1's scope confusion (argspec carrying ret-type concerns per stale AUDIT framing) was caught only when solvere's L2 surfaced to the user. The user's verdict — *"Y - args have nothing to do with ret type"* — produced a STRUCTURAL fix (variant + field + body all gone), not a diagnostic fix. The trap-door doctrine `feedback_trap_door_build_the_dependency` proved load-bearing.

3. **Sonnet writes substrate.** Even for 3-line struere closures, orchestrator briefs + scores; sonnet writes. Protocol-of-communication preserved.

### Next move: Stone 241.2 unblocked

A1/A2/A3 fn-parser migration begins. The fn-form parsers compose:
1. Split args_vec at `->` arrow position
2. Call `parse_argspec_triples(prefix_slice, head, form_span, options)` for the args
3. Parse ret-clause on the suffix (inline OR via Stone 241.2's helper)

Argspec is exceptional. Ret-clause is fn-form-parser concern. The substrate's structure honestly reflects the user's canonical form.
