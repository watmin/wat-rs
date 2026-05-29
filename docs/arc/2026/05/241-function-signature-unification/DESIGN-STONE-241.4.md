# DESIGN — Stone 241.4 — canonical `&` rest-binder extension + defclause opt-in; unblocks 237.8b Gate 1

**Status:** READY (sub-DESIGN). Phase 1 capstone — the canonical parser GAINS rest-binder behavior; defclause opts in; arc 237's blocked Gate 1 flips green. Vigilia-gate doctrine APPLIES (`src/argspec/` is a namespaced home).

## Why this stone

Stone 241.1.fix shipped the canonical `parse_argspec_triples` vigilia-CONVERGED with `allow_rest_binder: bool` as a per-site ParseOption. The flag's `true` path was unwired (the struere amend collapsed both `false` and `true` branches into "always reject `&`") — a permanent guarantee that `Result<>` returns honestly without panic paths until 241.4 ships the actual rest-binder parsing logic. That stone IS THIS ONE.

Three callers downstream need this:
- **A4** `parse_defclause_args` (`runtime.rs:6827`) — defclause sites need `& rest <- :Vector<T>` for variadic clauses (fold patterns, arithmetic recipes, etc.)
- **Probe 237.8b Gate 1** (`tests/probe_arc237_8b_defclause_arithmetic.rs:86`) — currently `#[ignore]`'d with reason *"defclause `&` rest-binder support must be minted FIRST"*; this stone mints it
- **Arc 237.8b** itself — defclause-based arithmetic recipe-lock has been BLOCKED on this; arc 237 stays PAUSED at 237.8b per spawn-block winding until 241.4 lands

Per failure-engineering: this stone CLOSES the future-fixture state of the canonical parser. Three runes:purgare(future-fixture) markers retire (rest_param field; rest_param: None initializer; TrailingItems variant). The canonical parser becomes its complete first-release shape.

## What this stone delivers

Three substrate changes + probe extension + probe-237.8b Gate 1 unignore:

### S1 — Canonical parser extension (`src/argspec/parse.rs`)

Un-prefix `_options: ParseOptions` → `options: ParseOptions`; wire the `&` branch's behavior on `options.allow_rest_binder`:

```rust
while idx < args_vec.len() {
    if is_bare_symbol(&args_vec[idx], "&") {
        if !options.allow_rest_binder {
            return Err(ArgSpecError::RestBinderNotSupported {
                span: args_vec[idx].span().clone(),
                head: head.to_string(),
            });
        }
        // Parse the rest-binder triple after `&`.
        idx += 1;
        if args_vec.len().saturating_sub(idx) < 3 {
            return Err(ArgSpecError::IncompleteTriple {
                span: form_span.clone(),
                head: head.to_string(),
            });
        }
        let (name, ty) = parse_triple(&args_vec[idx..idx + 3], head)?;
        let post_rest = idx + 3;
        if post_rest < args_vec.len() {
            return Err(ArgSpecError::TrailingItems {
                span: form_span.clone(),
                head: head.to_string(),
                count: args_vec.len() - post_rest,
            });
        }
        return Ok(ArgSpec {
            fixed_params,
            rest_param: Some((name, ty)),
        });
    }

    if args_vec.len().saturating_sub(idx) < 3 {
        return Err(ArgSpecError::IncompleteTriple { ... });
    }
    let (name, ty) = parse_triple(&args_vec[idx..idx + 3], head)?;
    fixed_params.push((name, ty));
    idx += 3;
}
```

### S2 — Extract `parse_triple` struere-helper (anticipate vigilia)

The fixed-param walker and the rest-binder branch share the `name <- :T` triple-parse shape. Extract:

```rust
fn parse_triple(
    slice: &[WatAST],
    head: &str,
) -> Result<(String, TypeExpr), ArgSpecError> {
    // Caller ensures slice.len() >= 3 BEFORE calling.
    let name = match &slice[0] {
        WatAST::Symbol(ident, _) => ident.name.clone(),
        other => return Err(ArgSpecError::NameNotSymbol {
            span: other.span().clone(),
            head: head.to_string(),
        }),
    };
    if !is_bare_symbol(&slice[1], "<-") {
        return Err(ArgSpecError::MissingArrow {
            span: slice[1].span().clone(),
            head: head.to_string(),
        });
    }
    let ty = parse_keyword_type(&slice[2], head, |span, head| {
        ArgSpecError::TypeNotKeyword { span, head }
    })?;
    Ok((name, ty))
}
```

Both callers pass `&args_vec[idx..idx+3]`. Single source of truth for `name <- :T` parsing.

### S3 — Rune removals

Three `rune:purgare(future-fixture)` markers retire (they were future-fixture for THIS stone; the future has arrived):

| Site | Pre-state | Post-state |
|---|---|---|
| `parse.rs:18` (rest_param field) | runed; field always None in 241.1 | rune removed; field actively populated when allow_rest_binder fires |
| `parse.rs:127` (rest_param: None initializer at no-rest Ok branch) | runed; future-fixture | rune removed; this branch is the natural no-rest case (still valid; just no longer "future") |
| `error.rs:32` (TrailingItems variant) | runed; unreachable in 241.1 (loop consumed full slice) | rune removed; variant becomes reachable when `[..., & name <- :T, extra]` parsed |

The error.rs rune wording mentioned *"Stone 241.4 makes TrailingItems reachable after rest-binder logic ships"* — that prediction now holds; the rune retires.

### S4 — Defclause caller opts in (`src/runtime.rs:6827` `parse_defclause_args`)

ONE-LINE change:

```rust
// Before (Stone 241.3):
let spec = crate::argspec::parse_argspec_triples(
    args_vec,
    head,
    form_span,
    crate::argspec::ParseOptions { allow_rest_binder: false },
)?;

// After (Stone 241.4):
let spec = crate::argspec::parse_argspec_triples(
    args_vec,
    head,
    form_span,
    crate::argspec::ParseOptions { allow_rest_binder: true },
)?;
```

A4's return signature is `Result<Vec<(String, TypeExpr)>, RuntimeError>` — only fixed_params are returned. But Stone 241.4 also needs to RETURN the rest_param (or signal it). Two options:

- **(α)** A4's signature changes — return `Result<(Vec<...>, Option<(String, TypeExpr)>), RuntimeError>` (fixed_params + rest_param). Caller `parse_defclause_clause` consumes both.
- **(β)** A4 returns the full `ArgSpec` — `Result<ArgSpec, RuntimeError>`. Caller consumes spec.fixed_params + spec.rest_param.

**Verdict (β)**: A4 returns `ArgSpec` directly. Cleaner; the canonical shape stays canonical at the A4 boundary. The unzip-less return shape Stone 241.3 minted gets extended: now A4 returns the whole spec, not just fixed_params.

This is a **public-API change to A4** (signature changes). It's small and confined; the caller (`parse_defclause_clause` at runtime.rs:6947) updates accordingly.

### S5 — Canonical probe extension (`tests/probe_arc241_stone1_argspec_canonical.rs`)

Add ~6 new contracts to the canonical probe (existing 9 stay; new 6 cover rest-binder happy + error paths). New total: 15 contracts.

| New # | Test | Source form | ParseOptions | Expected |
|---|---|---|---|---|
| 10 | rest-only succeeds | `[& rest <- :wat::core::Vector<:wat::core::i64>]` | `allow_rest_binder: true` | Ok; fixed_params: []; rest_param: Some |
| 11 | fixed + rest succeeds | `[x <- :wat::core::i64 & rest <- :wat::core::Vector<:wat::core::i64>]` | `allow_rest_binder: true` | Ok; fixed_params: [(x, i64)]; rest_param: Some |
| 12 | trailing items after rest errors | `[& rest <- :wat::core::Vector<:wat::core::i64> extra]` | `allow_rest_binder: true` | Err(TrailingItems { count: 1 }) — **VERIFIES T2 VERDICT β** |
| 13 | incomplete rest-binder (only `&`) errors | `[&]` | `allow_rest_binder: true` | Err(IncompleteTriple) |
| 14 | rest-binder NameNotSymbol errors | `[& :kw <- :wat::core::i64]` | `allow_rest_binder: true` | Err(NameNotSymbol) |
| 15 | existing contract_07 still works (regression) | `[x <- :i64 & rest <- :Vector<:i64>]` | `allow_rest_binder: false` | Err(RestBinderNotSupported) — preserved |

The existing `contract_07_rest_binder_rejected` STAYS unchanged (allow_rest_binder=false case preserved). The new contracts exercise the `true` path.

**Pre-stone**: contracts 10-14 will FAIL at HEAD (current code rejects `&` regardless of allow_rest_binder). Contract 15 PASSES at HEAD (existing behavior).

### S6 — Unignore probe-237.8b Gate 1

In `tests/probe_arc237_8b_defclause_arithmetic.rs:85`, remove the `#[ignore = "Stone 237.8b-prep: ..."]` annotation from `gate_1_defclause_supports_rest_binder`. The test should now PASS green.

If Gate 1 doesn't pass after the substrate changes, that's a STOP-10 finding (substrate change didn't satisfy the integration expectation; investigate).

## Locked decisions

### D1 — `parse_argspec_triples` PUBLIC SIGNATURE unchanged

Function signature stays `(args_vec: &[WatAST], head: &str, form_span: &Span, options: ParseOptions) -> Result<ArgSpec, ArgSpecError>`. The `options` parameter is now USED (un-prefixed); the ParseOptions struct is unchanged (still one field: `allow_rest_binder`).

### D2 — `parse_triple` is PRIVATE (no pub)

Helper at `src/argspec/parse.rs`, module-internal. Both callers (fixed-param loop body + rest-binder branch) are inside the same module.

### D3 — `ArgSpec` shape UNCHANGED

`fixed_params` + `rest_param` (Option). Both fields stay. The `rest_param` field is no longer future-fixture; rune retires.

### D4 — `ParseOptions` shape UNCHANGED

`allow_rest_binder: bool`. One field. Now actively consulted.

### D5 — `ArgSpecError` variants UNCHANGED

Seven variants (NameNotSymbol, MissingArrow, TypeNotKeyword, MalformedTypeKeyword, TrailingItems, IncompleteTriple, RestBinderNotSupported). TrailingItems becomes REACHABLE; rune retires. No new variants.

### D6 — A4 `parse_defclause_args` returns `ArgSpec`

Per verdict (β) above. Signature changes from `Result<Vec<(String, TypeExpr)>, RuntimeError>` to `Result<ArgSpec, RuntimeError>` (where `ArgSpec` is `crate::argspec::ArgSpec`). Caller `parse_defclause_clause` consumes `spec.fixed_params` + `spec.rest_param`.

This is a small public-API extension. Justified because the caller now needs the rest_param too; returning the whole ArgSpec is cleaner than tuple-returning two fields.

### D7 — `parse_defclause_clause` consumes `spec.rest_param`

The caller integrates rest_param into the clause's runtime structure. The Clause struct (defined elsewhere in runtime.rs) may need a `rest_param: Option<(String, TypeExpr)>` field added — sonnet investigates and integrates.

This is the substrate "make defclause REALLY USE rest-binder" work. If Clause struct doesn't have a rest_param slot, sonnet adds one. If the dispatch logic (defclause clause matching) needs updating to handle rest-binder args at runtime, sonnet does that too.

**Caveat — scope check**: integrating rest_param into Clause's runtime dispatch could be substantial. If sonnet finds the integration is deeper than ~30 lines of mechanical wiring, surface as STOP-6 finding (this stone is the parser + opt-in; deeper integration may belong to a follow-up). Per Gate 1's expectation, basic rest-binder dispatch SHOULD work after this stone; if it doesn't, surface the deeper gap.

### D8 — Vigilia-gate doctrine APPLIES

`src/argspec/parse.rs` + `src/argspec/error.rs` are namespaced home files (per `feedback_namespaced_home_vigilia_gate`). Stone 241.4 commits ONLY after vigilia drives L1+L2 to zero. Phase B vigilia cast on src/argspec/* + the extended canonical probe.

`src/runtime.rs` changes (A4 + Clause integration) are legacy flat substrate — gate does NOT apply there. The vigilia cast is scoped to src/argspec/ only.

### D9 — Per `feedback_stone_briefs_cite_prior_score`: BRIEF cites SCORE-STONE-241.1.fix.md

The canonical parser's shape (classify(), parse_keyword_type helper, ArgSpec/ParseOptions/ArgSpecError surfaces) is established by Stone 241.1.fix. Stone 241.4 EXTENDS that shape; the BRIEF cites the structural foundation as locked.

### D10 — Lib baseline + arc 237 probes + canonical probe

After Stone 241.4:
- `cargo test --release --lib -p wat` ≥ 834 PASS / 0 FAIL
- `cargo test --release --test probe_arc241_stone1_argspec_canonical` = 15/15 PASS (extended)
- `cargo test --release --test probe_arc241_stone2_fn_parser_migration` = 10/10 PASS (preserved)
- `cargo test --release --test probe_arc241_stone3_defclause_parser_migration` = 6/6 PASS (preserved)
- `cargo test --release --test probe_arc237_8b_defclause_arithmetic` — Gate 1 PASSES (was `#[ignore]`'d; now active)
- `cargo build --release --tests --workspace` clean
- `cargo clippy --release` ≤ 905

---

## Trap-door audit

### T1 — Three runes retire in lockstep

The three future-fixture runes were INSCRIBED with Stone 241.4 as the trigger event. This stone IS that event. All three retire in one commit. Sonnet should NOT leave any of them in place "just in case" — the prediction's truth condition (Stone 241.4 ships rest-binder logic) holds; the runes are obsolete.

### T2 — TrailingItems verification (verifies Stone 241.1.fix DESIGN T2 verdict β)

Stone 241.1.fix DESIGN trap-door T2 verdict β kept `TrailingItems` variant + runed it with prediction *"Stone 241.4 makes TrailingItems reachable after rest-binder logic ships."* Contract 12 of the extended canonical probe verifies this — `[& rest <- :Vector<i64> extra]` triggers `TrailingItems { count: 1 }`. The verdict's prediction MUST hold; if contract 12 doesn't trigger TrailingItems, the loop's trailing-items logic is wrong.

### T3 — A4 signature change cascades to its caller

`parse_defclause_clause` at runtime.rs:6947 currently consumes A4's `Vec<(String, TypeExpr)>` return. After Stone 241.4, A4 returns `ArgSpec`. The caller needs to update to consume `spec.fixed_params` AND `spec.rest_param`.

The caller's existing logic uses the fixed_params Vec directly. After the change, it accesses `spec.fixed_params` (same data, different access). Plus a new line to read `spec.rest_param` and integrate.

### T4 — Clause struct may need rest_param field

The Clause struct (defined elsewhere in src/runtime.rs around defclause infrastructure) represents a parsed clause. If it doesn't have a slot for rest_param, sonnet adds one. This is a substrate extension required for the rest-binder to be REPRESENTED in the clause's runtime form.

If the integration is shallow (just add a field), sonnet does it. If it's deep (rest-binder requires new dispatch logic, variadic argument bundling, type-checking changes across many sites), surface as STOP-6 finding — Stone 241.4 was scoped to the PARSER; the runtime dispatch may belong to a follow-up stone.

### T5 — Probe 237.8b Gate 1 unignore is the integration test

Gate 1 runs a wat program that defines a defclause with a rest-binder, then calls it variadically. The full integration (parser → defclause clause → dispatch → fold) must work for the test to pass. If the parser change isn't enough and dispatch needs work too, Gate 1 reveals it. If sonnet's substrate changes are sufficient, Gate 1 flips green; if not, sonnet documents the further gap and Stone 241.5 opens.

### T6 — Stone 241.2 + 241.3 probes preserved

A4's signature change affects only the A4 site and its direct caller. Stone 241.2's (A1/A2/A3) and Stone 241.3's (A4 migration parity) probes test BEHAVIORAL outcomes (does a valid form succeed; does a bad form error). Stone 241.4's changes shouldn't alter those outcomes for the SHAPES the older probes tested. They must stay 10/10 and 6/6 respectively.

### T7 — Vigilia may surface new findings

Stone 241.1.fix's vigilia gate caught 3 findings (drift, conflation, panic-surprise). Stone 241.4's vigilia may surface findings on:
- `parse_triple` helper: is it at the right level? Does it ENFORCE the precondition (slice.len() >= 3) somewhere?
- The rest-binder branch's complexity: is it obvious? Could it decompose further?
- The runes' removal: did sonnet remove ALL three? Any orphan future-fixture markers elsewhere?

The vigilia cast catches what SCORE-green doesn't. Per the gate doctrine: L1+L2=0 before commit; runes load-bearing only.

### T8 — Doc comments on ArgSpec / ParseOptions need updating

The doc on `ArgSpec.rest_param` says *"Always `None` in Stone 241.1"* (line 9-17). After 241.4, that's wrong — it's None unless `allow_rest_binder: true` AND `&` appeared. Doc needs updating to reflect post-241.4 truth.

The doc on `ParseOptions.allow_rest_binder` says *"Stone 241.4 wires this field..."* (line 30-37). Stone 241.4 IS the present; the doc should describe what the field DOES, not future-tense. Update to descriptive language.

The doc on `parse_argspec_triples` (line 40-68) needs an Algorithm step #3 for the rest-binder path.

These are intueri concerns; sonnet updates. Vigilia will flag stale doc text if not updated.

### T9 — Test-assertion cascade may be non-zero (unlike 241.2/241.3)

Stones 241.2 + 241.3 had zero cascade because tests asserted structurally, not on message text. Stone 241.4 might have larger cascade because:
- A4's signature change may break tests that called A4 directly (unlikely — A4 is private)
- Defclause tests that previously expected `RestBinderNotSupported` errors may now succeed with rest-binder support
- Probe 237.8b Gate 1 unignore is the explicit non-zero test update

Cascade tracked in SCORE Honest Deltas.

### T10 — Clippy may flag the parse_triple helper

If `parse_triple` has clippy suggestions (e.g., a too-many-arguments lint if it grows; or a use-iterator lint), sonnet addresses or rune-accepts.

---

## STOP triggers (REJECTION)

1. **STOP-1** — Unexpected compile errors not traced to the migration sites
2. **STOP-2** — Lib baseline regression below 834
3. **STOP-3** — 60 min elapsed (broader scope than 241.3 — substrate + helper extract + defclause opt-in + Clause-struct integration + probe + Gate 1 unignore)
4. **STOP-4** — `holon-rs` touched
5. **STOP-5** — Rust files outside `src/argspec/{parse,error}.rs`, `src/runtime.rs`, `tests/probe_arc241_stone1_argspec_canonical.rs`, `tests/probe_arc237_8b_defclause_arithmetic.rs` touched. `src/lib.rs` MUST stay unchanged; `src/argspec/mod.rs` MUST stay unchanged; `src/check.rs` MUST stay unchanged; Stone 241.2 + 241.3 probes MUST stay at their current PASS counts.
6. **STOP-6** — Scope creep:
   - Clause-struct integration goes deeper than ~30 lines of mechanical wiring (surface as follow-up; STOP)
   - New `ArgSpecError` variants minted
   - New `ParseOptions` fields added
   - Touching fn-form parsers (A1/A2/A3 — they don't opt into rest-binder; Stone 241.4 must NOT change their call to ParseOptions)
   - Unifying type-keyword helpers (T6 from Stone 241.2 DESIGN; still queued for future arc)
7. **STOP-7** — Extended canonical probe doesn't reach 15/15 PASS
8. **STOP-8** — Stone 241.2 / 241.3 / arc 237 / arc 238 probes regress; or Gate 1 doesn't flip green after unignore
9. **STOP-9** — Clippy warnings > 905
10. **STOP-10** — Vigilia Phase B DIVERGES on findings sonnet can't address quickly (re-brief in Stone 241.4.fix if needed)

---

## FM 2-bis evidence

The canonical probe extension (`tests/probe_arc241_stone1_argspec_canonical.rs` with 6 new contracts) IS the disconfirming probe. Contracts 10-14 will FAIL at HEAD (current code rejects `&` regardless of allow_rest_binder). Contract 15 PASSES at HEAD (regression on existing behavior).

The orchestrator commits the probe extension BEFORE the BRIEF (FM 2-bis discipline). The probe at HEAD will be 9/9 → run-with-extension shows X PASS / Y FAIL with X+Y=15; the new contracts disconfirm the missing behavior. Post-stone: 15/15.

The 237.8b Gate 1 IS the integration disconfirming proof — `#[ignore]`'d at HEAD; un-ignored post-stone and PASSING.

---

## SCORE doc spec

`docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.4.md` (NEW). Mirror Stone 241.1.fix's SCORE shape (since vigilia gate applies here):

- Header (Mode A/B; runtime; one-line summary)
- Phase A scorecard ~15 rows
- Structural verification ~10 rows: runes removed; parse_triple extracted; rest-binder branch present; A4 returns ArgSpec; defclause opt-in landed; Gate 1 unignored; canonical probe extended
- Final API signatures (verbatim post-stone)
- Line counts per file
- Clippy delta (should be 0)
- Lib + probe + workspace + arc 237 regression confirmations
- Honest deltas (Clause-struct integration depth; doc updates; etc.)
- **Vigilia Convergence section** — Phase B re-cast by orchestrator; gate doctrine applies here

---

## Calibration

**Target band:** 30–50 min Mode A.
**Upper bound:** 60 min (STOP-3).

**Surface estimate (net delta moderate; mix of additions + rune removals):**

| File | Pre | Post | Delta |
|---|---|---|---|
| `src/argspec/parse.rs` | 167 | ~195 | **+28** (rest-binder branch + parse_triple extract; doc updates; -3 lines from runes) |
| `src/argspec/error.rs` | 118 | 115 | **-3** (TrailingItems rune removed) |
| `src/runtime.rs` (A4 + caller) | (current) | (+~10 lines) | **+10** (opt-in + signature change + Clause integration) |
| `tests/probe_arc241_stone1_argspec_canonical.rs` | 159 | ~250 | **+91** (6 new contracts) |
| `tests/probe_arc237_8b_defclause_arithmetic.rs` | (current) | (-2 lines: `#[ignore]` removed) | **-2** |
| **Net delta** | — | — | **~+124 lines** |

**Confidence: MODERATE-HIGH.** Larger scope than Stones 241.2/241.3 (combined substrate + helper extract + defclause integration + Gate 1 unblock + vigilia gate). The Clause-struct integration depth is the main risk; surface as honest delta or STOP-6 if it explodes.

**Per `feedback_stone_briefs_cite_prior_score`**: BRIEF cites SCORE-STONE-241.1.fix.md for the canonical foundation; cites Stone 241.3 for A4's migration shape (now extended).

---

## What this closes / unblocks

**Phase 1 of arc 241 capstone**: the canonical parser ships with its full first-release shape. All three future-fixture runes retire. The parser-divergence class (closed at Stone 241.3) gets its full BEHAVIORAL surface.

**Arc 237.8b** UNPAUSES — Gate 1 flips green; the rest of 237.8b's recipe-lock work (Gates 2-4 + mint-confirmers) opens. Per spawn-block winding: arc 237.8b had been paused since 2026-05-27 awaiting this stone.

**Phase 2 of arc 241** opens after this: 241.5 minting `:wat::runtime::metadata-of` reflection verb + 241.6 optional `{...}` metadata-map on `def` (defn inherits).

---

## Cross-references

- `SCORE-STONE-241.1.fix.md` § Vigilia Convergence — the canonical foundation; classify() + parse_keyword_type + the three runes Stone 241.4 retires
- `SCORE-STONE-241.3.md` § Phase 1 closure note — Stone 241.4 extends; A4 signature evolves (Vec → ArgSpec)
- `AUDIT.md` § A4 row + Stone 241.5 scope (note: 241.5 in AUDIT was the combined extension; this DESIGN puts the extension at 241.4 per CLIFFNOTES stone-chain refresh; behavior identical)
- `DESIGN.md` § Scope expansion 2026-05-28 — arc 241 stone chain (241.4 unblocks 237.8b)
- `tests/probe_arc237_8b_defclause_arithmetic.rs:86` — Gate 1 `#[ignore]`'d at HEAD; this stone unignores
- `feedback_namespaced_home_vigilia_gate` — vigilia gate APPLIES (src/argspec/ home)
- `feedback_stone_briefs_cite_prior_score` — BRIEF cites Stone 241.1.fix structural foundation
- `feedback_inscription_immutable` — past SCOREs/probes inscribed; this stone extends, doesn't rewrite
- `feedback_sonnet_writes_substrate` — orchestrator briefs + scores; sonnet writes
- `feedback_trap_door_build_the_dependency` — Clause-struct integration: if depth surfaces, BUILD the missing piece OR surface as follow-up (don't declare incoherent)
