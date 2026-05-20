# EXPECTATIONS — Arc 215 Stone 2 — Vector unification + keyword-key lift

Mode A target: 18/18 PASS.

| # | Row | Expectation |
|---|---|---|
| 1 | `infer_list_constructor` accepts `:wat::type::Infer` for T | Detects `args[0]` is `:wat::type::Infer`; sets `elem_ty = fresh.fresh()`; doesn't error. Mirrors Stone 1's HashSet pattern. |
| 2 | Expression-position `[...]` routes through unified path | `[1 2 3]` at expression position goes through `infer_list_constructor` (directly or via parser-emitted verb-call); produces same `Parametric { head: "wat::core::Vector", args: [i64] }` as before |
| 3 | Binder-position `WatAST::Vector` unchanged | `(:wat::core::let [x 1 y 2] ...)` tuple destructure still works; arc 169 / arc 167 binder semantics intact |
| 4 | `{...}` keyword-key parse-time check dropped | `parse_map_literal_body` no longer rejects non-keyword first child of key/value pair with `MalformedBraceLiteral`; alternating k/v rule preserved |
| 5 | `{...}` desugar K changed to `:wat::type::Infer` | Both K and V slots emit `:wat::type::Infer` (symmetric); `infer_hashmap_constructor` accepts via Stone 1's existing extension |
| 6 | Probe 1 — `[1 2 3]` integer Vec preserved | length 3; first element 1; existing behavior unchanged |
| 7 | Probe 2 — `[1.5 2.5]` float Vec preserved | length 2; T inferred f64 |
| 8 | Probe 3 — `["a" "b"]` string Vec preserved | length 2; T inferred String |
| 9 | Probe 4 — `[]` empty Vec preserved | length 0; T is fresh type variable |
| 10 | Probe 5 — `[true false true]` bool Vec preserved | length 3; T inferred bool |
| 11 | Probe 6 — `(:wat::core::Vector :wat::type::Infer 1 2 3)` new path | Produces Vec<i64>; equivalent to `[1 2 3]`; explicit-infer verb form works |
| 12 | Probe 7 — `(:wat::core::Vector :wat::type::Infer)` empty new path | Vec with fresh T; length 0 |
| 13 | Probe 8 — `[1 "two"]` mixed-type rejection | Check fails with TypeMismatch; offending value's span named |
| 14 | Probe 9 — `(:wat::core::Vector :wat::core::i64 1 2 3)` explicit type preserved | Vec<i64>; P1-style explicit form unchanged |
| 15 | Probe 10 — let binder `[x 1 y 2]` preserved | Tuple destructure via Vector binder still works |
| 16 | Probe 11 — `{1 "v" 2 "w"}` int-keyed map | `HashMap<i64, String>`; length 2; get 1 → Some("v") — non-keyword keys accepted |
| 17 | Probe 12 — `{"a" 1 "b" 2}` string-keyed map | `HashMap<String, i64>`; length 2; get "a" → Some(1) |
| 18 | Probe 13 — `{1 "v" "two" "w"}` mixed-K rejection at check | Check fails with TypeMismatch at key #2; diagnostic names i64 expected vs String got (position-named per arc 138) |
| 19 | P2 Probe 6 flipped | `tests/probe_brace_map_literal.rs` — `probe_6_non_keyword_key_rejected_at_parse` renamed (in-function); asserts non-keyword key now ACCEPTED at parse; type-checks as inferred-K map; historical note preserved via doc comment |
| 20 | WAT-CHEATSHEET § 8 updated | `[...]` desugar row added; `{...}` keyword-key lift documented; three-literal unification stated; escape hatch verb form noted |
| 21 | arc 058 audit row added | Lab repo INDEX.md gets timestamped entry (use `git -C` for cross-repo op per `feedback_cross_repo_cwd`) |
| 22 | CONVENTIONS updated | Type-placeholders section extended with two-layer-enforcement note (literal coherence + function-signature unification) |

## Independent prediction (calibration record)

Recorded before sonnet spawns.

**Target runtime:** 45-75 min Mode A
**Upper bound:** 90 min
**Confidence:** medium

**Rationale:**
- Stone 1 calibration: predicted 60-90, actual ~60 (low end of target band)
- Stone 2 (bundled) is comparable to Stone 1: Vector handler refactor + keyword-key parser lift + 18 probes + 3 doc updates + P2 Probe 6 flip
- Pattern is established from Stone 1: TypeExpr::Var via fresh.fresh() integrates cleanly; mechanical work for the inference extension
- Risk factors widening the band:
  - WatAST::Vector topology surprise (STOP-1) — if expression-position vs binder-position separation is subtler than the BRIEF anticipates
  - Keyword-key lift surprise (STOP-4) — Stone 1's K-extension was scoped for K=keyword; lifting to fully-inferred K may surface a special-case
  - P2 Probe 6 flip ripple — if existing tests anywhere assert non-keyword-key rejection-at-parse, they need updating
- Risk factors tightening:
  - Both changes use established Stone 1 patterns (same `:wat::type::Infer` mechanism)
  - No new substrate primitives (just routing changes)
  - Probe matrix mirrors Stone 1's shape

**Calibration check (fill in at completion):**
- Actual runtime: [TBD]
- Within prediction band? [TBD]
- If overrun: where? [TBD]

## Out-of-scope rows (deliberately absent)

- `infer_list_constructor` rename (intueri Level-1 finding; logged as honest delta only)
- `'(...)` list literal — PERMANENTLY DEFERRED per LLM-first analysis
- Match-arm patterns (#402)
- WARD-PASS (out-of-zone)
- INTERSTITIAL (orchestrator-direct)
- Existing `[...]` and `{...}` source-code call site migration (behavior preserved for `[...]`; non-keyword-key cases for `{...}` were previously parse-errors, so no existing source uses them)
- holon-rs (separate crate; out of scope)
- ProgramEnv specifics (Slice 4)
- `:wat::runtime::ProgramEnv/*` accessor verbs (Slice 4)

## Honesty deltas accepted

Sonnet may surface deltas if encountered:
- WatAST::Vector handler topology subtler than expected — many call sites; some shared between binder + expression position
- Parser-emit-verb-call vs check-internally-route design choice — both honest; pick the cleaner one based on actual code surface
- Intueri Level-1 finding: `infer_list_constructor` name lies (works on Vector); logged for future arc
- Any walker cross-cuts surfaced by the unification refactor
- Test name conflicts with Stone 1's probe file
