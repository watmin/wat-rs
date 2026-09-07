# SCORE — the match arm is a bracket clause with a map pattern

No commit. Floor and clippy left to the orchestrator. Lands on H-2/H-2b/H-2c/H-3/J/K; those stones were not reverted.

**This stone is closed for the probe.** Rows 1–4 of `probe_arc109_match_arm` all pass. Row 2 printed `-1` (by name, not position). The first strike's freeze death is gone: `wat/service.wat` parses, expands, and the stdlib freezes.

RELAND (this ingest): the first strike's "nested arms in templates" diagnosis was half-right. Nested `match` *inside a converted arm body* already converted on a parseable HEAD file. What actually broke freeze was (1) unit-unquote heads mangled to a bare `~`, (2) splice binders emitting `{:nested ~@init-arg-names}`, (3) serve-op-arms built as a syntax-quoted 2-list that is **not** a child of `match` in the source (spliced later via `~@serve-op-arms`). The loud `#wat.parse/UnexpectedRBracket` at `service.wat:1884` was mixed-delimiter residue from a prior hand-edit, not a walker miss on clean files. RELAND STOP-1 held: no hand-edited arms; HEAD restored, wat-fix extended, corpus reconverted.

---

## Row 1 — bracket + map accepted

**PASSED.** `a_map_pattern_arm_binds_its_declared_keys` exit 0, stdout `-1`.

## Row 2 — binding BY NAME

**PASSED.** `a_map_pattern_binds_by_name_not_by_position` exit 0, stdout `-1`.
Fixture `[a b]` / `(Two 1 2)` / `{:b b :a a}` / `a - b`. By name → `-1`. Position would have been `+1`.

## Row 3 — unit arm is `{}`

**PASSED.** `a_unit_variant_arm_is_an_empty_map` exit 0, stdout `99`.

## Row 4 — ONE grammar, not two

**PASSED.** Planted control `tests/wat_lang/probe_arc109_match_arm__positional_control.wat` still has `((:probe::Pair::Two a b) …)` and was excluded from the codemod. Exit 3 (check refuses the retired clause).

## Row 5 — `#[ignore]`s gone

`grep -c '#\[ignore' tests/wat_lang/probe_arc109_match_arm_is_a_map_pattern.rs` = **0**.

## Row 6 — corpus moved BY CODEMOD

`wat-scripts/fixes/match-arm-to-bracket-map-pattern.wat` exists. Dry-run on `/tmp/match-reland` of HEAD `service.wat` + `edn`/`spawn`/`core`/`outcomes` showed:

```
(~admin-stop-kw body)                  →  [~admin-stop-kw {} body]
((~admin-init-kw ~@init-arg-names) …)  →  [~admin-init-kw ~init-arg-map-ast …]
((~admin-allow-peer-kw pids) …)        →  [~admin-allow-peer-kw {:pids pids} …]
`((~op-variant-kw) body)               →  `[~op-variant-kw {} body]
`((~op-variant-kw ~req-binder) body)   →  `[~op-variant-kw {:req ~req-binder} body]
((:wat::edn::Validation::Invalid ~mpath-sym …) …)
                                       →  [:wat::edn::Validation::Invalid {:path ~mpath-sym :expected ~mexp-sym :got ~mgot-sym} …]
```

Applied to 1869 `.wat` files (everything except the positional-control fixture; wat-fix converted last so the pre-flip binary could still load it). Idempotent on Vector arms.

## Row 7 — `cond` untouched

Cond **clauses** were not rewritten. Remaining `((:` is cond (`((:wat::core::= …)`), `(fn …)` calls, comments, quoted strings, historical wat-fix *source examples* of the old form, and the planted positional control. Same-file match diffs exist where cond and match live together.

## Row 8 — wildcard and binding

`[_ body]` / `[<binder> body]` are 2-element vectors, head-discriminated. Not given a floor run.

## Row 9 — floor (ORCHESTRATOR)

Not run.

## Row 10 — clippy (ORCHESTRATOR)

Not run. `cargo build --release` emits 5 dead-code warnings (`Coverage::Wildcard`, `pattern_coverage`, `ident_span`, `try_match_pattern_ast`, `substitute_many`) — pre-existing from the first strike's rust half.

---

## What the RELAND changed in the wat-fix

1. **Unit unquote.** `(~admin-stop-kw body)` — the pattern IS the unquote list. Treating `:wat::core::unquote` as a constructor mangled the head to a bare `~` (`[~ {:admin-stop-kw admin-stop-kw} body]`, a parse-breaker). Now `[~kw {} body]`.
2. **Splice binders.** `((~admin-init-kw ~@init-arg-names) body)` cannot invent a key-first map. Generator gained `init-arg-map-ast` (a WatAST Map built by `read-string` of `{:name name …}` — not an arm). Wat-fix emits `[~ctor ~init-arg-map-ast body]`. `{ ~@pairs }` still does not parse (one form).
3. **Quasiquote-as-arm.** `` `form `` is a 2-child list `[quasiquote-kw, form]`. A syntax-quoted 2-list whose pattern is tagged (`(~ctor …)`) is a match-arm template even when it is not a child of `match` (serve-op-arms, spliced later via `~@`). `~foo bar` call templates are not converted (pattern head is the unquote keyword, not a tagged list).
4. **`:wat::rete::core::match`** uses the same arm grammar; walker now hits both heads. Four files had leftover paren arms (`probe_then_match_is_refused`, `probe_arc278_55_slice_one_vocabulary`, `where-record`, `where-control`).
5. **Path B client-method AST** (`src/runtime.rs` send_recv_ast). Rust was synthesizing the retired `(pattern body)` List for `StdOut/write` / `StdErr/write`. Rows 1–3 printed the right number then died at shutdown (`stdio.wat:213`, span on the `StdOut/write` call). Converted to Vector + key-first Map. Without this, freeze is green and the probe still exits 1.

## STOP rows

| STOP | result |
|---|---|
| STOP-1 routed through `let` | **held** in the reader. `parse_key_first_pairs` refuses Symbol keys. Hash-destructure of records stays binder-first as a 2-element vector. |
| STOP-2 binding by position | **held.** Row 2 green: `-1`. |
| STOP-3 both grammars | Row 4 passed. Positional control excluded and still old form. |
| STOP-4 arm the codemod cannot rewrite | **reported, then closed.** Splice → `~init-arg-map-ast` (generator helper + wat-fix rewrite, not a hand-edited arm). Unit unquote → `[~kw {}]`. Serve-op-arms → quasiquote-as-arm. |
| STOP-5 `cond` moves | **held** for cond clauses. |
| RELAND STOP-1 hand-edit an arm | **held.** HEAD restored; wat-fix reconverted. The only non-arm generator edit is the `init-arg-map-ast` let-binding next to `init-arg-names`. |
| RELAND STOP-2 convert only 7 stdlib files | **held.** Full 1869-file pass, then rete leftovers, then wat-fix itself. |
| RELAND STOP-3 positional control | **held.** |
| RELAND STOP-4 residual `((:` that is not a match arm | **held.** Cond, `fn` calls, comments, quoted strings, historical wat-fix examples. True leftover match `((:` = the planted positional control (2 hits). |

## Targeted checks

```
cargo test --release --test wat_lang probe_arc109_match_arm -- --nocapture
  4 passed; 0 failed; 0 ignored
  row 1 stdout -1 exit 0
  row 2 stdout -1 exit 0
  row 3 stdout 99 exit 0
  row 4 exit 3 (retired clause refused)
./target/release/wat tests/wat_lang/probe_arc109_match_arm__{declared_order,reversed_keys,unit_empty_map}.wat
  same numbers, exit 0
wat-fix dry-run /tmp/match-reland: unit unquote, splice, nested SendOutcome, serve-op-arms, Validation::Invalid fields
wat-fix full corpus: exit 0, 1869 files
service.wat freeze: green (cargo build --release)
```

## Files (this stone)

```
src/match_arm.rs                                      NEW — arm parser, key-first map
src/runtime.rs                                        eval_match / eval_match_tail / stepper
                                                      + Path B send_recv_ast Vector arms
src/check.rs                                          infer_match / detect_match_shape / cover_variant_arm
src/resolve/{normalize,walk,mod}.rs                   vector arms
src/closure_extract.rs                                vector arms
src/rete/purity.rs                                    vector arms
src/lib.rs                                            mod match_arm
wat-scripts/fixes/match-arm-to-bracket-map-pattern.wat NEW — unit unquote, splice, qq-as-arm, rete match
wat/service.wat                                       init-arg-map-ast helper + corpus via wat-fix
wat/**, tests/**, wat-scripts/**, wat-tests/**        corpus via wat-fix
tests/wat_lang/probe_arc109_match_arm_is_a_map_pattern.rs  un-ignored
```
