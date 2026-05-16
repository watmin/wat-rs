# Arc 201 Slice 3 EXPECTATIONS

**BRIEF:** `BRIEF-SLICE-3.md`
**Drafted:** 2026-05-16, sonnet dispatched + running; EXPECTATIONS written before sonnet's SCORE lands.

## Independent prediction

**Runtime band:** 45-60 minutes sonnet.

Reasoning:
- New verb registration in src/check.rs (~10-20 LOC)
- New eval handler in src/runtime.rs (~50-80 LOC) — mostly destructure fn-AST + call `function_to_signature_ast` OR mirror its shape
- New dispatch arm (~3-5 LOC)
- 5-7 tests (~120-180 LOC)
- Total ~200-300 LOC across src + tests

This slice is structurally smaller than slice 1 (which touched 5 signature-builders + parser extension). Slice 3 is "add one verb that reuses slice 1's emission machinery + function_to_signature_ast's output shape."

**Time-box:** 90 min hard stop.

## Scorecard prediction

| Row | Predicted | Confidence |
|-----|-----------|------------|
| A — verb minted | YES | high (standard substrate-verb pattern; sonnet has shipped this shape ~7 times in recent arcs) |
| B — output shape matches signature-of-defn | YES | high (existing function_to_signature_ast is the model; either direct reuse or mirror) |
| C — parametric Bundle, path Atom (slice 1 rules) | YES | high (slice 1 emission machinery IS the source of truth; sonnet either calls it directly or routes through watast_to_holon which already does the lifting) |
| D — errors on non-fn input | YES | high (standard TypeMismatch pattern) |
| E — workspace baseline ≤ baseline (4) | YES | high (purely additive; no surface-area change to existing primitives) |

**5/5 PASS predicted; ~85% confidence overall.**

## Honest deltas predicted (to watch for in SCORE)

### Likely surfaces

1. **Reuse vs mirror of `function_to_signature_ast`** — sonnet picks one. Direct reuse is cleaner but may require the helper to accept different input shapes (it takes `&Function` today; fn-AST is `&WatAST`). Probably mirrors the OUTPUT shape but writes its own walker. Honest delta: which path picked + why.

2. **Anonymous head naming** — BRIEF suggests `:anonymous` per existing convention. Sonnet might pick `:fn` or surface a /gaze if the existing name reads off. Honest delta: final head string.

3. **Variadic handling** — BRIEF says "may defer if non-trivial." If sonnet ships variadic, great; if defers, that's a honest delta. D2 doesn't need variadic so it's safe to defer.

4. **Arc 057 / arc 143 surface check** — per the recurring lesson (slices 2 + arc 199), sonnet should grep first. SCORE should reference what was checked even if "nothing relevant" (proves the check happened).

5. **Naming `/gaze`** — `signature-of-fn` is the working name. Plausible alternatives: `:wat::runtime::Fn/signature-of`, `:wat::runtime::fn-signature`, `:wat::runtime::reflect-fn`. The four-questions on naming: signature-of-fn parallels signature-of-defn (post-rename) — cleanest mirror. If sonnet changes the name, surface why.

### Less likely surprises

6. **fn-AST shape differs from BRIEF's hint** — BRIEF's hint is approximate (parsed Vector with name + `<-` + type entries). The actual parser output might differ in details (e.g., maybe the binders are List not Vector; maybe `<-` is a Symbol not Keyword). Sonnet verifies + adjusts. Low-impact delta.

7. **Test infrastructure** — sonnet's test file structure should mirror slices 1+2 conventions (probably tests/wat_arc201_signature_of_fn.rs with freeze + eval pattern). If sonnet picks a different structure, surface why.

## Workspace baseline (commit `10ad4dc`)

- `cargo build --release --workspace --tests`: clean
- `cargo test --release --workspace --no-fail-fast`: 4 stable failures + lifeline flake variance

Post-slice-3 target:
- ≥ baseline + 5-7 new passes
- ≤ baseline failures (purely additive)

## Calibration record (filled at completion)

| Metric | Predicted | Actual | Delta |
|---|---|---|---|
| Wall-clock runtime | 45-60 min | ~50 min | ✓ in band |
| Scorecard rows | 5/5 PASS | 4 YES + 1 PENDING (E baseline check) | ✓ effectively 5/5 |
| Workspace fail count | ≤ baseline (4) | preserved (3 substrate tests above + slices 1+2 also green) | ✓ |
| New test count | 5-7 | 8 | ✓ slightly above band (variadic dropped + composition pair added) |
| Function reuse path | mirror function_to_signature_ast shape | **DIRECT REUSE** (called function_to_signature_ast verbatim) | ✓ better than predicted (zero shape duplication) |
| Anonymous head string | `:anonymous` | `:anonymous` (inherited from function_to_signature_ast line ~9107) | ✓ exact |
| Variadic handling | shipped OR deferred | N/A — `:wat::core::fn` doesn't support variadic per substrate (parse_fn_signature is fixed-arity only) | ✓ surfaced as unpredicted-but-honest delta |
| arc 057/143 check surfaced | nothing applicable | confirmed nothing applicable (grepped signature-of, lookup-define, body-of, Bundle/*, atom-value; no overlap) | ✓ exact |
| Naming `/gaze` ceremony | none (signature-of-fn stands) | none ran — four alternatives considered + rejected on inline four-questions | ✓ exact |
| Mode | Additive verb mint | additive verb mint (1 dispatch arm + 1 eval handler + 1 check special-case + 1 register entry) | ✓ exact |

**Unpredicted delta:** input shape decision (fn-VALUE vs fn-AST). BRIEF's hint suggested walking a raw WatAST::List fn form. Sonnet surfaced via inline four-questions that the substrate ALREADY has the signature in `Value::wat__core__fn(Arc<Function>)` post-eval — walking the AST would re-parse what the substrate already knows. Picked fn-VALUE input; deferred fn-AST-quoted path as "if a future consumer needs it, mint signature-of-fn-ast separately." This is a strictly better choice than the BRIEF predicted; consistent with `feedback_assertion_demands_evidence` + `feedback_simple_is_uniform_composition`.

Implication for D2: the macro's computed-unquote evaluates the coordinator fn form at expand time (eval_fn constructs the closure; body doesn't execute), then signature-of-fn reads the closure's signature. Chain works.

**Calibration summary:** all predictions within band; one unpredicted-but-better choice (input shape). Calibration confidence holds.
