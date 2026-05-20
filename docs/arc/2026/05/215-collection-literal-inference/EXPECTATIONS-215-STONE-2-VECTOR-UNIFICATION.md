# EXPECTATIONS — Arc 215 Stone 2 — Vector unification

Mode A target: 15/15 PASS.

| # | Row | Expectation |
|---|---|---|
| 1 | `infer_list_constructor` accepts `:wat::type::Infer` for T | Detects `args[0]` is `:wat::type::Infer`; sets `elem_ty = fresh.fresh()`; doesn't error. Mirrors Stone 1's HashSet pattern. |
| 2 | Expression-position `[...]` routes through unified path | `[1 2 3]` at expression position goes through `infer_list_constructor` (directly or via parser-emitted verb-call); produces same `Parametric { head: "wat::core::Vector", args: [i64] }` as before |
| 3 | Binder-position `WatAST::Vector` unchanged | `(:wat::core::let [x 1 y 2] ...)` tuple destructure still works; arc 169 / arc 167 binder semantics intact |
| 4 | Probe 1 — `[1 2 3]` integer Vec preserved | length 3; first element 1; existing behavior unchanged |
| 5 | Probe 2 — `[1.5 2.5]` float Vec preserved | length 2; T inferred f64 |
| 6 | Probe 3 — `["a" "b"]` string Vec preserved | length 2; T inferred String |
| 7 | Probe 4 — `[]` empty Vec preserved | length 0; T is fresh type variable |
| 8 | Probe 5 — `[true false true]` bool Vec preserved | length 3; T inferred bool |
| 9 | Probe 6 — `(:wat::core::Vector :wat::type::Infer 1 2 3)` new path | Produces Vec<i64>; equivalent to `[1 2 3]`; explicit-infer verb form works |
| 10 | Probe 7 — `(:wat::core::Vector :wat::type::Infer)` empty new path | Vec with fresh T; length 0 |
| 11 | Probe 8 — `[1 "two"]` mixed-type rejection | Check fails with TypeMismatch; offending value's span named |
| 12 | Probe 9 — `(:wat::core::Vector :wat::core::i64 1 2 3)` explicit type preserved | Vec<i64>; P1-style explicit form unchanged |
| 13 | Probe 10 — let binder `[x 1 y 2]` preserved | Tuple destructure via Vector binder still works; arc 167 / arc 169 path intact |
| 14 | WAT-CHEATSHEET § 8 updated | `[...]` desugar row added; explicit-infer verb form documented; three-literal unification stated |
| 15 | arc 058 audit row added | Lab repo INDEX.md gets timestamped entry (use `git -C` for cross-repo op) |

## Independent prediction (calibration record)

Recorded before sonnet spawns.

**Target runtime:** 30-50 min Mode A
**Upper bound:** 75 min
**Confidence:** medium-high

**Rationale:**
- Stone 1 calibration: predicted 60-90, actual ~60 (low end of target band)
- Stone 2 is smaller than Stone 1: one inference function extension (mirror of HashSet pattern) + one routing change + ~10 probes + small doc updates
- Established pattern from Stone 1: TypeExpr::Var via fresh.fresh() integrates cleanly with HM walkers; no new variant needed
- Risk factor widening the band: WatAST::Vector topology surprise (STOP-1); if expression-position vs binder-position separation is subtler than the BRIEF anticipates, time goes up
- Risk factor tightening: pattern is well-established; sonnet has the Stone 1 template; mechanical work

**Calibration check (fill in at completion):**
- Actual runtime: [TBD]
- Within prediction band? [TBD]
- If overrun: where? [TBD]

## Out-of-scope rows (deliberately absent)

- `infer_list_constructor` rename (intueri Level-1 finding; logged as honest delta only)
- `'(...)` list literal (needs task #283)
- Match-arm patterns (#402)
- WARD-PASS (out-of-zone)
- INTERSTITIAL (orchestrator-direct)
- Existing `[...]` source-code call site migration (behavior preserved; no migration needed)
- holon-rs (separate crate; out of scope)

## Honesty deltas accepted

Sonnet may surface deltas if encountered:
- WatAST::Vector handler topology subtler than expected — many call sites; some shared between binder + expression position
- Parser-emit-verb-call vs check-internally-route design choice — both honest; pick the cleaner one based on actual code surface
- Intueri Level-1 finding: `infer_list_constructor` name lies (works on Vector); logged for future arc
- Any walker cross-cuts surfaced by the unification refactor
- Test name conflicts with Stone 1's probe file
