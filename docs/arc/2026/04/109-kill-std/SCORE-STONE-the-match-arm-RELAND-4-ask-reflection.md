# SCORE — RELAND 4: the codemod ASKS, it does not guess

No commit. Floor left to the orchestrator (read `^ +Summary`, never a tail). Lands on the match-arm strike + 296 L (`type-of`). Those were not reverted.

The four probe rows still PASS. `#[ignore]` = 0. `mixed_via_macro_runs` PASS (STOP-3).

---

## First act — can declarations register without checking bodies?

**Yes.** Measured:

| door | result |
|---|---|
| `eval-ast!` of `defenum` | **refused** — `is_mutation_head` |
| `load-file!` of a file with bad arms | would splice **bodies** then `check_program` fails |
| freeze `register_types` (step 5) vs `check_program` (step 8) | types register **before** body check — exists in Rust, not a wat call |
| **`eval-with-defs!` of declaration forms only** (no `defn`) | **works.** A lone `defenum :probe::Box` freezes; `type-of` answers Pair's fields `left`, `right` in order. `sift-rules-defsvc` without caller defns freezes; `type-of :usr::my-sift::SiftRulesResponse` answers `err` / `bytes` / `cap` / `cursor` / `path` |

STOP-2 does **not** fire. The codemod is: extract top-level decls → `eval-with-defs!` + `type-of` → rewrite keys. Unresolvable arms are printed `[match-arm] UNRESOLVED …` and left untouched (never guessed).

---

## Expectation 1 — the floor (delta, not a total)

Tree was red at **16**. This stone closes the **13 services** `{:_` failures. **3 remain**, diagnosed under STOP-5, not folded in.

| closed | |
|---|---|
| `probe_arc278_sift_rules::*` (4 tests) | PASS |
| `probe_arc278_sift_rules_arena::*` (4 tests) | PASS |
| `probe_arc209_c1_defservice_op_enum::defservice_emits_op_enum_with_wrapped_request_records` | PASS |

Keys rewritten from `type-of`, e.g. `{:_err _err}` → `{:err _err}`, `{:_cur _cur}` → `{:cursor _cur}`, `{:_bytes _bytes :_cap _cap}` → `{:bytes _bytes :cap _cap}`, `{:_r _r}` → `{:req _r}`. Binders kept; keys are declared fields.

No new failure opened by this rewrite.

## Expectation 2 — the 4 probe rows

**PASSED.** `#[ignore]` = 0.

## Expectation 3 — the hardcoded table is gone

`grep -c '"value"' wat-scripts/fixes/match-arm-to-bracket-map-pattern.wat` → **0**. `seed-builtins` deleted. Option/Result fields come from `type-of :wat::core::Option` / `Result` (`value` / `error`). Serve-op-arms still use the generated `~req-binder` → `req` path (no FQDN at the template site). STOP-4 held.

## Expectation 4 — keys are declared fields

`grep -rc '{:_' --include=*.wat` → **0**.

## Expectation 5 — unresolvable arms REPORTED

On the 3 rewritten files: **empty**. No `[match-arm] UNRESOLVED` line. The list may legitimately be empty; it is.

## Expectation 6 — spliced-arm `defservice`

**PASSED.** `probe_arc170_c2_mixed_macro::mixed_via_macro_runs`. `tagged-template-pattern?` still returns false on `unquote-splicing-form?`. STOP-3 held.

## Expectation 7 — clippy

No Rust this ingest. `cargo clippy --release --all-targets --workspace` remains **0 errors** (5 pre-existing dead-code warnings).

---

## STOP-5 — the 3 non-services, diagnosed, not folded

| test | root | this stone |
|---|---|---|
| `grid_axes_run_and_derive_nonvacuously` | `:wat::rete::lower` rejects a match arm in `where-control.wat:205` (`[0 false]` — a **literal** i64 pattern, delimiter-flipped from `(0 false)`; `parse_match_arm` has no IntLit 2-vector) and `where-record.wat:155` (`:wat::rete::core::match` over `:wr::Status`, `malformed match arm` inside **lower**, not `type-of`) | **not** `{:_` guessing. Not touched |
| `spec_equals_native_on_every_where_family` | same two files, same `rete::lower` message | same |
| `every_wat_scripts_file_loads_on_the_current_runtime` | 1 of 689: `wat-scripts/scratch-pad/probe-arc278-surface-registers-service-reads.wat` UnresolvedReference `:probe::chan-svc::State/seen` line 91. Header SUPERSEDED. Same finding as RELAND 2/3 | not touched |

---

## STOP rows

| STOP | result |
|---|---|
| STOP-1 a key is invented | **held.** Unresolved → report + skip. List empty on the files we rewrote |
| STOP-2 decls cannot register without checking bodies | **does not fire.** `eval-with-defs!` of decls-only is the door |
| STOP-3 discriminator weakened | **held.** `unquote-splicing-form?` still there; threading/serve-op-arms green |
| STOP-4 hardcoded field table survives | **held.** `"value"` count 0 |
| STOP-5 3 non-services folded in | **held.** Diagnosed above; not edited |

## Targeted checks

```
grep -c '"value"' wat-scripts/fixes/match-arm-to-bracket-map-pattern.wat  →  0
grep -rc '{:_' --include='*.wat'  →  0
cargo nextest run --release -E 'test(probe_arc109_match_arm)|test(mixed_via_macro_runs)|test(probe_arc278_sift_rules)|test(probe_arc209_c1_defservice_op_enum)'
  16 passed (probes + services)
```
