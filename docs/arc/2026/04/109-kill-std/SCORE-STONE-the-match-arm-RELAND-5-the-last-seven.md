# SCORE — RELAND 5: the last seven, three causes, one of them not ours

No commit. Floor left to the orchestrator (read `^ +Summary`, never a tail). Lands on the match-arm strike + RELAND 1–4 + 296 L (`type-of`). Those were not reverted.

The four probe rows still PASS. `#[ignore]` = 0. Clippy 0 errors.

The AMEND is the sizing: cause 2 is an IR change, and if it is too large, split it. Named multi-field `Pat::Variant` is split. The grid is live without it.

---

## Expectation 1 — the floor (delta per cause, not a total)

**Not run.** Orchestrator. Targeted tests that named the 7:

| cause | tests | this stone |
|---|---|---|
| 1 golden span drift | 4 `peers_bijection` reject goldens | **closed** (recapture) |
| 2 rete still reads the old arm | `grid_axes_run_and_derive_nonvacuously`, `spec_equals_native_on_every_where_family` | **closed** (Vector arm + 0–1 field map + literal 2-elem in core match) |
| 3 wat-scripts loader | `every_wat_scripts_file_loads_on_the_current_runtime` | **not closed** — bisect named, file not patched (STOP-4) |

**7 → 1 remaining.** The remaining 1 is cause 3. Floor will not be `0 failed` until a later stone touches 296 L's `State/seen` mint (or retires the SUPERSEDED probe). That is the finding, not a folded fix.

Also PASS, not in the 7: `peers_bijection_form_spelling_matching_peer_is_accepted` (the positive control, untouched).

## Expectation 2 — cause 1 goldens

**Recaptured.** Diff is the stdlib `:line` ONLY:

| golden | `:line` | `:end :line` |
|---|---|---|
| case1 / case4 missing-ephemeral | 864 → **893** | 871 → **900** |
| case2 / case5 undeclared-peer | 881 → **910** | 889 → **918** |

Same `ProgramBodyEvalFailed`, same `MalformedTemplate`, same `:reason` text, same `:col`. Fixture-file spans (`tests/services/probe_arc278_peers_bijection_case*.wat` `:line` 44 / 70) did not move.

**Class recorded:** `NOTE-a-golden-that-pins-a-stdlib-line.md` (sibling of the Rust-line NOTE). `normalize_rust_source_span_lines` zeros `.rs` Span `:line` only — it does not touch `wat/service.wat`. Recapture, keep pinning (arc 296: a pin discriminates the emitter; do not extend the normaliser to `wat/*.wat` as a side effect of this red). The next stdlib edit will break these four again; that is the class.

## Expectation 3 — rete lowers the bracket/map arm; refusal unchanged

`src/rete/expr_ir.rs`:

- `lower_match` reads a `WatAST::Vector` arm.
- `lower_bracket_arm`: 2-elem `[pat body]` (wildcard / binder / literal / hash); 3-elem namespaced `[<Variant> {:k v} body]`.
- Nested no-body `[Variant {:k v}]` is a **pattern**, in `lower_pat`.
- Map payload, v1: 0 keys → `payload: None` (unit `{}`); 1 key → positional `Some(sub-pat)` of the value; **2+ keys refuse** `"match map-pattern with more than one field is not lowered in v1"`.
- `"malformed match arm"` still fires on anything else. `"match map-destructure is not lowered in v1"` still fires on a bare Map / old List-with-Map head.

`purity.rs:1292` already classified Vector last-as-body. Unchanged.

STOP-3 held: `lower()` is still total or it refuses. The 2+ field refuse is the named v1 scope boundary, not a softened malformed.

### AMEND — the named-payload IR is split

The AMEND is right: `Pat::Variant { name, payload: Option<Box<Pat>> }` is one positional payload. `{:k v}` is named. Teaching the matcher to bind by name, and asking `type-of` for declared field names, is the same position→name move the surface just made, one layer down.

The grid does not need it. `where-record` is `{:level lvl}` / `{:reason reason}` / `{}`; `where-control` Option is `{:value v}` / `{}`. 0–1 field. 2+ stays an honest refuse.

⚠ **Split, as the AMEND allowed.** A later stone: `Pat::Variant` payload becomes named, the matcher binds by name, rete asks `type-of`. A grid axis left dead with a named reason would have been honest; this one is live without widening the refuse.

## Expectation 4 — both grid axes live; spec_equals PASS

```
grid_axes_run_and_derive_nonvacuously          PASS  10.591s
spec_equals_native_on_every_where_family       PASS  20.002s
```

`spec_equals` was native-ahead-of-grammar: rete already had `Pat::Lit` and lowered `[0 false]`; `parse_match_arm` did not. `where-control.wat:205` is the delimiter-flip of `(0 false)` — the same 2-elem shape as `[_ body]` / `[<binder> body]`, head a literal not a binder.

**`MatchArm::Literal`** added in `parse_match_arm` / `eval_parsed_arm` (via `try_match_pattern`) / `infer_match` / `closure_extract`. Int/float/bool/string/rational/bigint/char — the same set `try_match_pattern` already compared. Not a new language; the hole the delimiter-flip fell into.

where-record 3-elem variant maps were already in the grammar; they passed once native lowered them.

## Expectation 5 — cause 3 bisect names the commit

**First-bad: `480f38d05` `SCORE(296 L): reflection answers for EVERY type`.**

Does **not** predate the campaign. Isolated with `./target/release/wat --check wat-scripts/scratch-pad/probe-arc278-surface-registers-service-reads.wat` (not the 689-file loader).

| commit | src vs K | `--check` |
|---|---|---|
| `397100654` SCORE 296 K | — | **PASS** exit 0 |
| `9e7d2b911` (`480f38d05^`, EXPECTATIONS 296 L) | empty diff vs K | (same tree as K) |
| `480f38d05` SCORE 296 L | **the only src-changing commit** in `397100654..c86603825` | FAIL (same src as BRIEF HEAD) |
| `c86603825` BRIEF RELAND-5 | empty diff vs L | **FAIL** exit 1 |

L bundled three things: `type-of` / `TypeInfo` (the stone), **new** `src/match_arm.rs` (191 lines), and `wat/service.wat` +214/−185 (defservice lives there). The probe file itself was **not** rewritten — last commits `ab52b7188` (angle-bracket) and `037ef43ef`. Header already SUPERSEDED 2026-08-05.

Verbatim (current binary, same as BRIEF HEAD):

```
#wat.resolve/UnresolvedReferences {:message "1 unresolved reference" :location nil :causes [] :unresolved [#wat.resolve/UnresolvedReference {:path ":probe::chan-svc::State/seen" :context "call head — not a builtin, not a registered function" :span #wat.core/Span {:file "wat-scripts/scratch-pad/probe-arc278-surface-registers-service-reads.wat" :line 91 :col 35 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 91 :col 63}}}}]}
```

Not patched. Other services still resolve `State/durable` / `State/out` / `State/echo`. This probe is the vector `:durable [seen <- :wat::core::i64]` spelling used as a call head `State/seen`. Mechanism not chased past the named commit — that is 296 L's (and the bundled arm conversion of `service.wat`), not a recapture and not a rete IR.

## Expectation 6 — the 4 probe rows

**PASSED.** `cargo nextest run --release -E 'test(probe_arc109_match_arm)'`:

- `a_map_pattern_arm_binds_its_declared_keys` PASS
- `a_map_pattern_binds_by_name_not_by_position` PASS
- `a_unit_variant_arm_is_an_empty_map` PASS
- `the_retired_positional_clause_is_refused` PASS
- `#[ignore]` = 0 (no `#[ignore]` in the probe file)

## Expectation 7 — clippy

`cargo clippy --release --all-targets --workspace` **0 errors**. 5 dead-code warnings, same five as RELAND 3/4 (`Coverage::Wildcard`, `pattern_coverage`, `ident_span`, `try_match_pattern_ast`, `substitute_many`). Pre-existing.

---

## STOP rows

| STOP | result |
|---|---|
| STOP-1 three causes fixed as one | **held.** Recapture / rete+literal / bisect. Separate files, separate proofs |
| STOP-2 recapture without the class | **held.** NOTE written; verified line-number-only; normaliser not silently extended |
| STOP-3 widen `lower()` refusal | **held.** 2+ field maps still refuse v1; malformed still refuses; grid live by teaching Vector / 0–1 field / literal, not by swallowing |
| STOP-4 cause 3 fixed without a bisect | **held.** Named `480f38d05`. File not patched. Does not predate the campaign |

## Targeted checks

```
cargo nextest run --release -E 'test(probe_arc278_peers_bijection) + test(grid_axes_run_and_derive_nonvacuously) + test(spec_equals_native_on_every_where_family) + test(probe_arc109_match_arm)'
  11 passed, 0 failed

cargo test --release -p wat-doc -p wat-macros -p wat-edn -p wat-reader
  ok (all four crates)

cargo nextest run --release --test lint
  124 passed, 1 failed — every_wat_scripts_file_loads (cause 3, named, not patched)

cargo clippy --release --all-targets --workspace
  0 errors
```
